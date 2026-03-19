//! Web server mode for voice: browser connects via WebSocket,
//! server relays audio to/from OpenAI Realtime API.
//!
//! This integrates with the agent's `VoiceSession` and `Memory` system,
//! unlike the standalone PoC which used raw WebSocket relay.

use anyhow::{Context, Result};
use axum::{
    extract::ws::{Message as WsMsg, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as HyperBuilder;
use hyper_util::service::TowerToHyperService;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message as TungMsg};
use tracing::{error, info, warn};

use crate::config::Config;

/// Shared application state for chat and voice API.
#[derive(Clone)]
struct AppState {
    voice_config: Arc<crate::config::VoiceConfig>,
    /// Per-connection chat histories, keyed by session ID.
    chat_histories:
        Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<serde_json::Value>>>>,
    tools: Arc<Vec<serde_json::Value>>,
    memory: Arc<dyn crate::memory::Memory>,
    /// Full config for chat completions (fallback when no shared agent).
    config: Arc<Config>,
    /// Shared Agent instance — when present, chat routes through the full Agent pipeline.
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
    /// Shared HTTP client for chat API proxy (connection pooling).
    http_client: reqwest::Client,
}

#[derive(serde::Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(serde::Serialize)]
struct ChatResponse {
    reply: String,
}

/// Embedded index.html for the voice web client.
const INDEX_HTML: &str = include_str!("static/index.html");

/// Start the voice web server.
///
/// Serves a browser-based voice assistant UI at `/` and a WebSocket
/// relay at `/ws` that bridges browser audio to the OpenAI Realtime API.
///
/// When `tls_cert` and `tls_key` are provided, serves over HTTPS.
pub async fn run_voice_web(
    host: &str,
    port: u16,
    config: Config,
    tls_cert: Option<&str>,
    tls_key: Option<&str>,
) -> Result<()> {
    Box::pin(run_voice_web_with_agent(
        host,
        port,
        config,
        tls_cert,
        tls_key,
        Vec::new(),
        None,
    ))
    .await
}

/// Start the voice web server with a shared Agent for full-pipeline chat.
pub async fn run_voice_web_with_agent(
    host: &str,
    port: u16,
    config: Config,
    tls_cert: Option<&str>,
    tls_key: Option<&str>,
    tool_specs: Vec<crate::tools::ToolSpec>,
    shared_agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
) -> Result<()> {
    let mut voice_config = config.voice.clone();

    if !voice_config.enabled {
        anyhow::bail!("Voice is not enabled. Set [voice] enabled = true in config.toml");
    }

    // Build voice system prompt from workspace identity files (SOUL.md, IDENTITY.md, etc.)
    // This overrides the static system_prompt in config with a workspace-aware prompt.
    let workspace_prompt = build_voice_system_prompt_from_workspace(&config.workspace_dir);
    if !workspace_prompt.is_empty() {
        info!(
            "Voice system prompt built from workspace ({} chars)",
            workspace_prompt.len()
        );
        voice_config.system_prompt = workspace_prompt;
    } else if voice_config.system_prompt.is_empty() {
        info!("No workspace identity files found; using default voice prompt");
        voice_config.system_prompt =
            "You are a helpful AI voice assistant. Respond concisely and conversationally."
                .to_string();
    }

    // Validate that we can build a realtime config (API key, provider, etc.)
    let _realtime_config = voice_config
        .to_realtime_config()
        .context("Failed to build Realtime API config from [voice] settings")?;

    // Initialize memory backend for transcript persistence
    let mem: Arc<dyn crate::memory::Memory> = Arc::from(crate::memory::create_memory_with_storage(
        &config.memory,
        Some(&config.storage.provider.config),
        &config.workspace_dir,
        config.api_key.as_deref(),
    )?);

    let shared_config = Arc::new(voice_config);
    let shared_mem = mem;
    let shared_tools: Arc<Vec<serde_json::Value>> = Arc::new(
        tool_specs
            .iter()
            .map(|spec| spec.to_realtime_json())
            .collect(),
    );
    let has_agent = shared_agent.is_some();
    let state = AppState {
        voice_config: shared_config.clone(),
        chat_histories: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        tools: shared_tools.clone(),
        memory: shared_mem.clone(),
        config: Arc::new(config),
        agent: shared_agent,
        http_client: reqwest::Client::new(),
    };
    if has_agent {
        info!("Web chat will route through shared Agent (full pipeline: tools, memory, skills)");
    } else {
        info!("Web chat will use direct API proxy (no tools)");
    }

    let app = Router::new()
        .route("/", get(index_handler))
        .route(
            "/ws",
            get({
                let cfg = shared_config.clone();
                let mem = shared_mem.clone();
                let tools = shared_tools.clone();
                let agent_for_voice = state.agent.clone();
                move |ws| ws_handler(ws, cfg, mem, tools, agent_for_voice)
            }),
        )
        .route("/api/chat", post(chat_handler))
        .route("/api/config", get(client_config_handler))
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();

    let browser_host = match host {
        "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        _ => host,
    };

    match (tls_cert, tls_key) {
        (Some(cert_path), Some(key_path)) => {
            // HTTPS mode
            let tls_acceptor = build_tls_acceptor(cert_path, key_path)?;

            println!("🎤 Voice Web Server (HTTPS)");
            println!(
                "   Open https://{}:{} in your browser",
                browser_host, actual_port
            );
            println!(
                "   WebSocket endpoint: wss://{}:{}/ws",
                browser_host, actual_port
            );
            println!("   Press Ctrl+C to stop.\n");

            serve_tls(listener, app, tls_acceptor).await
        }
        (None, None) => {
            // Plain HTTP mode
            println!("🎤 Voice Web Server");
            println!(
                "   Open http://{}:{} in your browser",
                browser_host, actual_port
            );
            println!(
                "   WebSocket endpoint: ws://{}:{}/ws",
                browser_host, actual_port
            );
            println!("   Press Ctrl+C to stop.\n");

            axum::serve(listener, app).await?;
            Ok(())
        }
        _ => {
            anyhow::bail!("Both --tls-cert and --tls-key must be provided together");
        }
    }
}

/// Build a TLS acceptor from PEM certificate and key files.
fn build_tls_acceptor(cert_path: &str, key_path: &str) -> Result<tokio_rustls::TlsAcceptor> {
    use rustls::pki_types::PrivateKeyDer;
    use std::io::BufReader;

    let cert_file = std::fs::File::open(cert_path)
        .with_context(|| format!("Failed to open TLS cert: {}", cert_path))?;
    let key_file = std::fs::File::open(key_path)
        .with_context(|| format!("Failed to open TLS key: {}", key_path))?;

    let certs: Vec<_> = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to parse TLS certificate PEM")?;

    if certs.is_empty() {
        anyhow::bail!(
            "TLS certificate file contains no certificates: {}",
            cert_path
        );
    }

    let key: PrivateKeyDer = rustls_pemfile::private_key(&mut BufReader::new(key_file))
        .context("Failed to parse TLS private key PEM")?
        .context("No private key found in PEM file")?;

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("Invalid TLS certificate/key pair")?;

    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(tls_config)))
}

/// Serve the axum app over TLS using manual hyper connection handling.
async fn serve_tls(
    listener: tokio::net::TcpListener,
    app: Router,
    tls_acceptor: tokio_rustls::TlsAcceptor,
) -> Result<()> {
    let accept_loop = async {
        loop {
            let (stream, addr) = listener.accept().await?;
            let acceptor = tls_acceptor.clone();
            let app = app.clone();

            tokio::spawn(async move {
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("TLS handshake failed from {}: {}", addr, e);
                        return;
                    }
                };

                let io = TokioIo::new(tls_stream);
                let svc = TowerToHyperService::new(app);

                if let Err(e) = HyperBuilder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(io, svc)
                    .await
                {
                    // Connection reset / client disconnect — not worth logging as error
                    if !e.to_string().contains("connection reset") {
                        warn!("HTTP connection error from {}: {}", addr, e);
                    }
                }
            });
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        result = accept_loop => result,
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down TLS voice server...");
            Ok(())
        }
    }
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Expose client-side tuning parameters from server config.
async fn client_config_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "bargeInDelayMs": state.voice_config.barge_in_delay_ms,
    }))
}

#[allow(clippy::unused_async)]
async fn ws_handler(
    ws: WebSocketUpgrade,
    config: Arc<crate::config::VoiceConfig>,
    memory: Arc<dyn crate::memory::Memory>,
    tools: Arc<Vec<serde_json::Value>>,
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_browser_session(socket, config, memory, tools, agent))
}

async fn handle_browser_session(
    browser_ws: WebSocket,
    config: Arc<crate::config::VoiceConfig>,
    memory: Arc<dyn crate::memory::Memory>,
    tools: Arc<Vec<serde_json::Value>>,
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
) {
    info!("Browser voice client connected");

    if let Err(e) = relay_session(browser_ws, &config, &tools, agent, memory).await {
        error!("Voice session error: {}", e);
    }

    info!("Browser voice client disconnected");
}

/// Direct WebSocket relay between browser and OpenAI Realtime API.
///
/// This is the simpler relay approach (like the PoC). For full VoiceSession
/// integration with Memory persistence, see the TODO above.
/// Minimum character count for a voice transcript turn pair to be persisted to memory.
/// Matches the chat pipeline's threshold to filter noise (short utterances like "음", "어").
const VOICE_AUTOSAVE_MIN_CHARS: usize = 10;

async fn relay_session(
    browser_ws: WebSocket,
    config: &crate::config::VoiceConfig,
    tools: &[serde_json::Value],
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
    memory: Arc<dyn crate::memory::Memory>,
) -> Result<()> {
    let realtime_config = config.to_realtime_config()?;
    let session_id = uuid::Uuid::new_v4().to_string();

    // Build Realtime API WebSocket URL and auth headers (reuse RealtimeConfig methods)
    let url = realtime_config.ws_url();
    let mut request = url.into_client_request()?;
    for (name, value) in realtime_config.auth_headers() {
        request.headers_mut().insert(
            name.parse::<axum::http::HeaderName>()
                .map_err(|e| anyhow::anyhow!("Invalid header name: {}", e))?,
            value
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid header value: {}", e))?,
        );
    }

    let (openai_ws, _) = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio_tungstenite::connect_async(request),
    )
    .await
    .context("Realtime API connection timed out (30s)")?
    .context("Failed to connect to Realtime API")?;

    info!("Connected to Realtime API");

    let (openai_tx_raw, mut openai_rx) = openai_ws.split();
    let (mut browser_tx, mut browser_rx) = browser_ws.split();

    // Wrap openai_tx in a shared mpsc channel so multiple tasks can send
    let (to_openai_tx, mut to_openai_rx) = tokio::sync::mpsc::channel::<String>(128);
    let mut openai_tx_raw = openai_tx_raw;
    let openai_writer = tokio::spawn(async move {
        while let Some(msg) = to_openai_rx.recv().await {
            if openai_tx_raw.send(TungMsg::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Send session configuration to OpenAI (all fields from RealtimeConfig for consistency)
    let session_config = json!({
        "modalities": ["text", "audio"],
        "voice": realtime_config.voice,
        "instructions": realtime_config.system_prompt,
        "input_audio_format": realtime_config.audio_format,
        "output_audio_format": realtime_config.audio_format,
        "input_audio_transcription": {
            "model": realtime_config.transcription_model,
            "language": realtime_config.language
        },
        "turn_detection": {
            "type": realtime_config.vad_type,
            "threshold": realtime_config.vad_threshold,
            "prefix_padding_ms": realtime_config.vad_prefix_padding_ms,
            "silence_duration_ms": realtime_config.silence_duration_ms
        }
    });

    // Tools are not registered with the Realtime API session because tool execution
    // is not yet implemented in voice mode. Registering tools without execution would
    // cause the API to invoke them and receive placeholder errors, confusing users.
    if !tools.is_empty() {
        warn!(
            "Voice mode: {} tool(s) available but not registered — tool execution in voice mode is not yet implemented",
            tools.len()
        );
    }

    let session_update = json!({
        "type": "session.update",
        "session": session_config
    });

    to_openai_tx
        .send(session_update.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send session config: {}", e))?;

    // Inject Agent's chat history into Realtime API session for context continuity.
    // Clone filtered messages and release the Agent lock before async channel sends
    // to avoid holding the mutex across await points.
    if let Some(ref agent_mutex) = agent {
        let recent_chats: Vec<crate::providers::ChatMessage> = {
            let agent = agent_mutex.lock().await;
            let history = agent.history();
            let max_items = config.max_history_items;
            history
                .iter()
                .filter_map(|msg| {
                    if let crate::providers::ConversationMessage::Chat(chat) = msg {
                        if chat.role == "user" || chat.role == "assistant" {
                            return Some(chat.clone());
                        }
                    }
                    None
                })
                .rev()
                .take(max_items)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        }; // Agent lock released here

        let mut injected = 0;
        for chat in &recent_chats {
            let content_type = if chat.role == "user" {
                "input_text"
            } else {
                "text"
            };
            let item = json!({
                "type": "conversation.item.create",
                "item": {
                    "type": "message",
                    "role": chat.role,
                    "content": [{ "type": content_type, "text": chat.content }]
                }
            });
            if to_openai_tx.send(item.to_string()).await.is_err() {
                break;
            }
            injected += 1;
        }
        if injected > 0 {
            info!(
                "Injected {} Agent history items into voice session",
                injected
            );
        }
    }

    // Relay: Browser → OpenAI (only allow safe event types)
    let browser_to_openai = async {
        while let Some(Ok(msg)) = browser_rx.next().await {
            match msg {
                WsMsg::Text(text) => {
                    // Whitelist: only forward audio and response control events from browser.
                    // Block session.update, conversation.item.create, etc. to prevent
                    // client-side manipulation of system prompt, model, or session config.
                    let allowed = match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(ev) => matches!(
                            ev.get("type").and_then(|t| t.as_str()),
                            Some(
                                "input_audio_buffer.append"
                                    | "input_audio_buffer.commit"
                                    | "input_audio_buffer.clear"
                                    | "response.create"
                                    | "response.cancel"
                            )
                        ),
                        Err(_) => false,
                    };
                    if allowed {
                        if to_openai_tx.send(text.to_string()).await.is_err() {
                            break;
                        }
                    } else {
                        tracing::debug!("[Voice] Blocked non-whitelisted browser event");
                    }
                }
                WsMsg::Binary(data) => {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    let msg = json!({
                        "type": "input_audio_buffer.append",
                        "audio": b64
                    });
                    if to_openai_tx.send(msg.to_string()).await.is_err() {
                        break;
                    }
                }
                WsMsg::Close(_) => break,
                _ => {}
            }
        }
    };

    // Relay: OpenAI → Browser (transcript → Agent sync + memory persistence)
    let agent_for_transcript = agent.clone();
    let memory_for_transcript = memory.clone();
    let session_id_for_transcript = session_id.clone();

    // Buffer for collecting user+assistant transcript pairs before persisting.
    // Stores the pending user utterance and its turn number until the assistant response completes the pair.
    // Using Mutex ensures the turn number and user text are always atomically paired.
    let pending_user_text: Arc<tokio::sync::Mutex<Option<(u32, String)>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    // Start at 1 so saturating_sub(1) in the fallback path never underflows to 0
    // and collides with a valid user turn number.
    let turn_counter: Arc<std::sync::atomic::AtomicU32> =
        Arc::new(std::sync::atomic::AtomicU32::new(1));

    let openai_to_browser = async {
        while let Some(Ok(msg)) = openai_rx.next().await {
            match msg {
                TungMsg::Text(text) => {
                    // Parse event for transcript sync and function call interception
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                        // Sync voice transcripts to Agent.history + persist completed pairs to Memory
                        match event_type {
                            "conversation.item.input_audio_transcription.completed" => {
                                if let Some(transcript) =
                                    event.get("transcript").and_then(|t| t.as_str())
                                {
                                    let trimmed = transcript.trim();
                                    if !trimmed.is_empty() {
                                        let turn_num = turn_counter
                                            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                                        // Sync to Agent history
                                        if let Some(ref agent_mutex) = agent_for_transcript {
                                            let mut agent = agent_mutex.lock().await;
                                            let turn =
                                                crate::providers::realtime_types::TranscriptTurn {
                                                    turn_number: turn_num,
                                                    user_text: trimmed.to_string(),
                                                    assistant_text: String::new(),
                                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                                };
                                            agent.merge_voice_transcripts(&[turn]);
                                        }
                                        // Buffer user text with its turn number for paired persistence
                                        *pending_user_text.lock().await =
                                            Some((turn_num, trimmed.to_string()));
                                    }
                                }
                            }
                            "response.audio_transcript.done" => {
                                if let Some(transcript) =
                                    event.get("transcript").and_then(|t| t.as_str())
                                {
                                    let trimmed = transcript.trim();
                                    if !trimmed.is_empty() {
                                        // Take the paired user turn; fall back to current counter
                                        let pending = pending_user_text.lock().await.take();
                                        let (turn_num, user_part) = match pending {
                                            Some((num, text)) => (num, text),
                                            None => {
                                                let num = turn_counter
                                                    .load(std::sync::atomic::Ordering::Acquire)
                                                    .saturating_sub(1);
                                                (num, String::new())
                                            }
                                        };
                                        // Sync to Agent history
                                        if let Some(ref agent_mutex) = agent_for_transcript {
                                            let mut agent = agent_mutex.lock().await;
                                            let turn =
                                                crate::providers::realtime_types::TranscriptTurn {
                                                    turn_number: turn_num,
                                                    user_text: String::new(),
                                                    assistant_text: trimmed.to_string(),
                                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                                };
                                            agent.merge_voice_transcripts(&[turn]);
                                        }

                                        // Pair complete: persist to memory if meets minimum length
                                        let user_part = user_part.as_str();
                                        let total_chars =
                                            user_part.chars().count() + trimmed.chars().count();

                                        if total_chars >= VOICE_AUTOSAVE_MIN_CHARS {
                                            let user_store = format!("[Voice] {}", user_part);
                                            let assistant_store = format!("[Voice] {}", trimmed);
                                            let mem = memory_for_transcript.clone();
                                            // Use turn_num from the paired buffer (matches Agent history)
                                            // Store user message with session-scoped turn key
                                            if !user_part.is_empty() {
                                                let key = format!(
                                                    "voice_turn_{}_{}_user",
                                                    session_id_for_transcript, turn_num
                                                );
                                                if let Err(e) = mem
                                                    .store(
                                                        &key,
                                                        &user_store,
                                                        crate::memory::MemoryCategory::Custom(
                                                            "voice".to_string(),
                                                        ),
                                                        Some(&session_id_for_transcript),
                                                    )
                                                    .await
                                                {
                                                    warn!("Failed to persist voice user transcript: {}", e);
                                                }
                                            }
                                            // Store assistant response with session-scoped turn key
                                            let key = format!(
                                                "voice_turn_{}_{}_assistant",
                                                session_id_for_transcript, turn_num
                                            );
                                            if let Err(e) = mem
                                                .store(
                                                    &key,
                                                    &assistant_store,
                                                    crate::memory::MemoryCategory::Custom(
                                                        "voice".to_string(),
                                                    ),
                                                    Some(&session_id_for_transcript),
                                                )
                                                .await
                                            {
                                                warn!("Failed to persist voice assistant transcript: {}", e);
                                            }
                                            tracing::debug!(
                                                "[Voice] Persisted transcript pair to memory ({}+{} chars)",
                                                user_part.chars().count(),
                                                trimmed.chars().count()
                                            );
                                        } else {
                                            tracing::debug!(
                                                "[Voice] Skipped short transcript pair ({} chars < {})",
                                                total_chars, VOICE_AUTOSAVE_MIN_CHARS
                                            );
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        // Function call handling is disabled — tools are not registered
                        // with the Realtime API session (see session config above).
                        // When tool execution is implemented, re-enable tool registration
                        // and handle "response.function_call_arguments.done" events here.
                    }

                    // Forward to browser
                    if browser_tx
                        .send(WsMsg::Text(text.to_string().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                TungMsg::Close(_) => break,
                _ => {}
            }
        }
    };

    tokio::select! {
        () = browser_to_openai => info!("Browser → OpenAI relay ended"),
        () = openai_to_browser => info!("OpenAI → Browser relay ended"),
    }

    // Clean up writer task
    drop(to_openai_tx);
    let _ = openai_writer.await;

    Ok(())
}

/// Handle POST /api/chat — routes through shared Agent when available,
/// falls back to direct API proxy otherwise.
///
/// When a shared Agent is present, the full pipeline runs:
/// tools, memory, skills, research, history — exactly like CLI `zeroclaw agent`.
async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> axum::response::Result<Json<ChatResponse>> {
    let message = req.message.trim().to_string();
    if message.is_empty() {
        return Err(axum::response::ErrorResponse::from((
            axum::http::StatusCode::BAD_REQUEST,
            "Empty message",
        )));
    }

    // ── Route through shared Agent (full pipeline) ──
    if let Some(ref agent_mutex) = state.agent {
        let reply = {
            let mut agent = agent_mutex.lock().await;
            agent.turn(&message).await.map_err(|e| {
                error!("Agent turn error: {}", e);
                axum::response::ErrorResponse::from((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Agent processing error",
                ))
            })?
        };
        return Ok(Json(ChatResponse { reply }));
    }

    // ── Fallback: direct API proxy (no tools) ──
    let cfg = &state.config;
    let api_key = cfg
        .api_key
        .as_deref()
        .or(cfg.voice.api_key.as_deref())
        .unwrap_or("");

    if api_key.is_empty() {
        return Err(axum::response::ErrorResponse::from((
            axum::http::StatusCode::BAD_REQUEST,
            "No API key configured. Set api_key in config, [voice].api_key, AZURE_OPENAI_API_KEY, or OPENAI_API_KEY",
        )));
    }

    let model = cfg
        .default_model
        .as_deref()
        .unwrap_or(cfg.voice.model.as_str());

    let (url, auth_header_name, auth_header_value) = if let Some(ref api_url) = cfg.api_url {
        let url = if api_url.contains("/chat/completions") {
            api_url.clone()
        } else {
            format!("{}/chat/completions", api_url.trim_end_matches('/'))
        };
        if api_url.contains("openai.azure.com") || api_url.contains("/openai/deployments/") {
            (url, "api-key".to_string(), api_key.to_string())
        } else {
            (
                url,
                "Authorization".to_string(),
                format!("Bearer {}", api_key),
            )
        }
    } else {
        let url = "https://api.openai.com/v1/chat/completions".to_string();
        (
            url,
            "Authorization".to_string(),
            format!("Bearer {}", api_key),
        )
    };

    let mut messages = Vec::new();
    let sys_prompt = &state.voice_config.system_prompt;
    if !sys_prompt.is_empty() {
        messages.push(json!({ "role": "system", "content": sys_prompt }));
    }
    {
        let histories = state.chat_histories.read().await;
        if let Some(history) = histories.get("default") {
            messages.extend(history.iter().cloned());
        }
    }
    messages.push(json!({ "role": "user", "content": message }));

    let body = json!({
        "model": model,
        "messages": messages,
        "temperature": cfg.default_temperature,
    });

    let resp = state
        .http_client
        .post(&url)
        .header(&auth_header_name, &auth_header_value)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            error!("Chat API request failed: {}", e);
            axum::response::ErrorResponse::from((
                axum::http::StatusCode::BAD_GATEWAY,
                "API request failed",
            ))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        error!("Chat API error {}: {}", status, text);
        return Err(axum::response::ErrorResponse::from((
            axum::http::StatusCode::BAD_GATEWAY,
            "API returned error",
        )));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| {
        error!("Failed to parse API response: {}", e);
        axum::response::ErrorResponse::from((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid API response",
        ))
    })?;

    let reply = data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    {
        let max_history = state.voice_config.max_history_items.max(1) * 2; // pairs of user+assistant
        let mut histories = state.chat_histories.write().await;
        let history = histories
            .entry("default".to_string())
            .or_insert_with(Vec::new);
        history.push(json!({ "role": "user", "content": message }));
        history.push(json!({ "role": "assistant", "content": reply }));
        if history.len() > max_history {
            let drain = history.len() - max_history;
            history.drain(..drain);
        }
    }

    Ok(Json(ChatResponse { reply }))
}

/// Build a voice system prompt by reading workspace identity files.
/// Reads SOUL.md, IDENTITY.md, USER.md, TOOLS.md, MEMORY.md from workspace_dir.
/// Returns an empty string if no files are found.
fn build_voice_system_prompt_from_workspace(workspace_dir: &std::path::Path) -> String {
    let mut prompt = String::from("You are in VOICE mode. Respond concisely and conversationally.\n\n## Identity & Context\n\n");
    let mut found_any = false;

    for filename in &["SOUL.md", "IDENTITY.md", "USER.md", "TOOLS.md", "MEMORY.md"] {
        let path = workspace_dir.join(filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                found_any = true;
                use std::fmt::Write;
                let _ = write!(prompt, "### {filename}\n\n");
                // Truncate large files to keep prompt compact for TTFT
                let max_chars = 8000;
                if trimmed.chars().count() > max_chars {
                    let byte_end = trimmed
                        .char_indices()
                        .nth(max_chars)
                        .map(|(i, _)| i)
                        .unwrap_or(trimmed.len());
                    prompt.push_str(&trimmed[..byte_end]);
                    prompt.push_str("\n\n[...truncated]\n\n");
                } else {
                    prompt.push_str(trimmed);
                    prompt.push_str("\n\n");
                }
            }
        }
    }

    // Add current datetime
    let now = chrono::Local::now();
    {
        use std::fmt::Write;
        let _ = write!(
            prompt,
            "## Current Date & Time\n\n{}\n",
            now.format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    if found_any {
        prompt
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_prompt_from_workspace_with_files() {
        let dir = std::env::temp_dir().join("zeroclaw_voice_prompt_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("SOUL.md"), "I am a helpful assistant").unwrap();
        std::fs::write(dir.join("IDENTITY.md"), "Name: TestBot").unwrap();

        let prompt = build_voice_system_prompt_from_workspace(&dir);
        assert!(prompt.contains("### SOUL.md"));
        assert!(prompt.contains("I am a helpful assistant"));
        assert!(prompt.contains("### IDENTITY.md"));
        assert!(prompt.contains("Name: TestBot"));
        assert!(prompt.contains("Current Date & Time"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn voice_prompt_from_workspace_empty_returns_empty() {
        let dir = std::env::temp_dir().join("zeroclaw_voice_prompt_empty_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let prompt = build_voice_system_prompt_from_workspace(&dir);
        assert!(prompt.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn voice_prompt_truncates_large_files() {
        let dir = std::env::temp_dir().join("zeroclaw_voice_prompt_trunc_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create a file larger than 8000 chars
        let large_content = "x".repeat(10_000);
        std::fs::write(dir.join("SOUL.md"), &large_content).unwrap();

        let prompt = build_voice_system_prompt_from_workspace(&dir);
        assert!(prompt.contains("[...truncated]"));
        // Should not contain the full 10000 chars
        assert!(prompt.len() < 10_000);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn voice_prompt_skips_empty_files() {
        let dir = std::env::temp_dir().join("zeroclaw_voice_prompt_skip_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("SOUL.md"), "Real content").unwrap();
        std::fs::write(dir.join("IDENTITY.md"), "   ").unwrap(); // whitespace only

        let prompt = build_voice_system_prompt_from_workspace(&dir);
        assert!(prompt.contains("### SOUL.md"));
        assert!(!prompt.contains("### IDENTITY.md")); // skipped

        let _ = std::fs::remove_dir_all(&dir);
    }
}
