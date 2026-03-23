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
    /// Bearer token for authenticating all endpoints.
    auth_token: Arc<String>,
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

    // Resolve or generate auth token (before moving voice_config into Arc)
    let auth_token = voice_config.auth_token.clone().unwrap_or_else(|| {
        let token = uuid::Uuid::new_v4().to_string();
        info!("Generated voice auth token (pass via ZEROCLAW_VOICE_AUTH_TOKEN to persist)");
        token
    });
    let auth_token = Arc::new(auth_token);

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
        auth_token: auth_token.clone(),
    };
    if has_agent {
        info!("Web chat will route through shared Agent (full pipeline: tools, memory, skills)");
    } else {
        info!("Web chat will use direct API proxy (no tools)");
    }

    let app = Router::new()
        .route("/", get({
            let token = auth_token.clone();
            move || index_handler_with_token(token)
        }))
        .route(
            "/ws",
            get({
                let cfg = shared_config.clone();
                let mem = shared_mem.clone();
                let tools = shared_tools.clone();
                let agent_for_voice = state.agent.clone();
                let token = auth_token.clone();
                move |ws: WebSocketUpgrade, headers: axum::http::HeaderMap, query: axum::extract::Query<std::collections::HashMap<String, String>>| {
                    ws_handler_with_auth(ws, headers, query, cfg, mem, tools, agent_for_voice, token)
                }
            }),
        )
        .route("/api/chat", post(chat_handler))
        .route("/api/config", get(client_config_handler))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
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
            println!("   Auth token: {}", auth_token);
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
            println!("   Auth token: {}", auth_token);
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

/// Auth middleware — checks `Authorization: Bearer <token>` on all API routes.
/// The `/` (index) route is excluded because it serves the HTML page that contains the token.
async fn auth_middleware(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = req.uri().path();
    // Allow the index page (embeds the token), /ws (has its own auth via query param),
    // and static resources like favicon.ico
    if path == "/" || path == "/ws" || path == "/favicon.ico" {
        return next.run(req).await;
    }

    let expected = state.auth_token.as_str();
    let authorized = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .is_some_and(|token| token == expected);

    if authorized {
        next.run(req).await
    } else {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Unauthorized. Provide Authorization: Bearer <token> header."}"#,
            ))
            .unwrap_or_else(|_| {
                axum::response::Response::new(axum::body::Body::from("Unauthorized"))
            })
    }
}

/// Serve the index page with the auth token injected for browser auto-auth.
#[allow(clippy::unused_async)]
async fn index_handler_with_token(token: Arc<String>) -> Html<String> {
    // Inject the token as a JS variable so the browser can send it in fetch/WebSocket requests
    let html = INDEX_HTML.replace(
        "// ── Voice state ──",
        &format!(
            "const VOICE_AUTH_TOKEN = '{}';\n    // ── Voice state ──",
            token
        ),
    );
    Html(html)
}

/// Expose client-side tuning parameters from server config.
async fn client_config_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "bargeInDelayMs": state.voice_config.barge_in_delay_ms,
    }))
}

/// WebSocket handler with auth — checks token from query param `?token=` or Authorization header.
/// WebSocket upgrade requests cannot carry custom headers from browser JS, so we accept query params.
#[allow(clippy::unused_async)]
async fn ws_handler_with_auth(
    ws: WebSocketUpgrade,
    headers: axum::http::HeaderMap,
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
    config: Arc<crate::config::VoiceConfig>,
    memory: Arc<dyn crate::memory::Memory>,
    tools: Arc<Vec<serde_json::Value>>,
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
    expected_token: Arc<String>,
) -> axum::response::Response {
    // Check query param first (browser WebSocket), then Authorization header
    let token_from_query = query.get("token").map(|s| s.as_str());
    let token_from_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "));

    let provided_token = token_from_query.or(token_from_header);
    let authorized = provided_token.is_some_and(|t| t == expected_token.as_str());

    if !authorized {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from("Unauthorized"))
            .unwrap_or_else(|_| {
                axum::response::Response::new(axum::body::Body::from("Unauthorized"))
            });
    }

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

/// Minimum character count for a voice transcript turn pair to be persisted to memory.
/// Matches the chat pipeline's threshold to filter noise (short utterances like "음", "어").
const VOICE_AUTOSAVE_MIN_CHARS: usize = 10;

/// Timeout for WebSocket close handshake to prevent hanging on shutdown.
const WS_CLOSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Whitelist of event types allowed from the browser to the OpenAI Realtime API.
/// All other events (especially `session.update`, `conversation.item.create`) are blocked
/// to prevent client-side manipulation of system prompt, model, or session config.
const ALLOWED_BROWSER_EVENTS: &[&str] = &[
    "input_audio_buffer.append",
    "input_audio_buffer.commit",
    "input_audio_buffer.clear",
    "response.create",
    "response.cancel",
];

/// Check if a browser event JSON message is allowed to be forwarded to OpenAI.
fn is_browser_event_allowed(text: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(ev) => {
            let event_type = ev.get("type").and_then(|t| t.as_str()).unwrap_or("");
            ALLOWED_BROWSER_EVENTS.contains(&event_type)
        }
        Err(_) => false,
    }
}

/// Build the `session.update` JSON payload for the Realtime API.
///
/// Includes modalities, voice settings, VAD config, and optionally registers
/// tools for function calling when an Agent is available.
fn build_session_config(
    realtime_config: &crate::providers::realtime_types::RealtimeConfig,
    tools: &[serde_json::Value],
    has_agent: bool,
) -> serde_json::Value {
    let mut session_config = json!({
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

    if !tools.is_empty() && has_agent {
        session_config["tools"] = json!(tools);
        info!(
            "Voice mode: {} tool(s) registered for function calling",
            tools.len()
        );
    }

    json!({
        "type": "session.update",
        "session": session_config
    })
}

/// Inject Agent's chat history into the Realtime API session for context continuity.
///
/// Clones filtered messages and releases the Agent lock before async channel sends
/// to avoid holding the mutex across await points.
async fn inject_agent_history(
    agent_mutex: &Arc<tokio::sync::Mutex<crate::agent::Agent>>,
    max_history_items: usize,
    to_openai_tx: &tokio::sync::mpsc::Sender<String>,
) {
    let recent_chats: Vec<crate::providers::ChatMessage> = {
        let agent = agent_mutex.lock().await;
        let history = agent.history();
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
            .take(max_history_items)
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

/// Relay messages from browser WebSocket to OpenAI Realtime API.
///
/// Only whitelisted event types are forwarded; binary audio data is
/// base64-encoded into `input_audio_buffer.append` events.
async fn handle_browser_to_openai(
    browser_rx: &mut futures_util::stream::SplitStream<WebSocket>,
    to_openai_tx: &tokio::sync::mpsc::Sender<String>,
) {
    while let Some(Ok(msg)) = browser_rx.next().await {
        match msg {
            WsMsg::Text(text) => {
                if is_browser_event_allowed(&text) {
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
}

/// Handle a function call event from the Realtime API.
///
/// Spawns tool execution in the background so the relay loop is not blocked.
/// Sends the tool result back as `function_call_output` and triggers `response.create`
/// to resume the model's response.
fn handle_function_call(
    event: &serde_json::Value,
    agent_mutex: &Arc<tokio::sync::Mutex<crate::agent::Agent>>,
    to_openai_tx: &tokio::sync::mpsc::Sender<String>,
) {
    let call_id = event
        .get("call_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let fn_name = event
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let fn_args_str = event
        .get("arguments")
        .and_then(|v| v.as_str())
        .unwrap_or("{}")
        .to_string();

    if call_id.is_empty() || fn_name.is_empty() {
        warn!(
            "Voice function call event missing call_id or name: call_id={:?}, name={:?}",
            call_id, fn_name
        );
        return;
    }

    let tx = to_openai_tx.clone();
    let agent_clone = agent_mutex.clone();
    tokio::spawn(async move {
        let result = execute_voice_tool(&agent_clone, &fn_name, &fn_args_str).await;

        let output_item = json!({
            "type": "conversation.item.create",
            "item": {
                "type": "function_call_output",
                "call_id": call_id,
                "output": result
            }
        });
        if tx.send(output_item.to_string()).await.is_err() {
            error!("Failed to send tool result to Realtime API — voice session may hang waiting for function_call_output");
            return;
        }

        let response_create = json!({ "type": "response.create" });
        if tx.send(response_create.to_string()).await.is_err() {
            error!("Failed to send response.create after tool execution — voice session may not resume");
        }

        info!("Voice tool '{}' executed, result sent back", fn_name);
    });
}

/// Shared state for the OpenAI→Browser relay, holding transcript pairing buffers
/// and references needed for Agent sync and memory persistence.
struct RelayTranscriptState {
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
    memory: Arc<dyn crate::memory::Memory>,
    session_id: String,
    /// Pending user utterance: (turn_number, text). Paired with the next assistant response.
    pending_user_text: Arc<tokio::sync::Mutex<Option<(u32, String)>>>,
    /// Monotonic turn counter (starts at 1 to avoid 0-collision on saturating_sub fallback).
    turn_counter: Arc<std::sync::atomic::AtomicU32>,
}

/// Relay messages from OpenAI Realtime API to browser WebSocket.
///
/// Handles transcript synchronization to Agent history, memory persistence
/// of completed user+assistant turn pairs, and function call interception.
async fn handle_openai_to_browser(
    openai_rx: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    browser_tx: &mut futures_util::stream::SplitSink<WebSocket, WsMsg>,
    to_openai_tx: &tokio::sync::mpsc::Sender<String>,
    state: &RelayTranscriptState,
) {
    while let Some(Ok(msg)) = openai_rx.next().await {
        match msg {
            TungMsg::Text(text) => {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                    let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                    // Sync voice transcripts to Agent.history + persist completed pairs to Memory
                    match event_type {
                        "conversation.item.input_audio_transcription.completed" => {
                            process_user_transcript(&event, state).await;
                        }
                        "response.audio_transcript.done" => {
                            process_assistant_transcript(&event, state).await;
                        }
                        _ => {}
                    }

                    // Handle function calls from the Realtime API
                    if event_type == "response.function_call_arguments.done" {
                        if let Some(ref agent_mutex) = state.agent {
                            handle_function_call(&event, agent_mutex, to_openai_tx);
                        }
                    }
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
}

/// Process a completed user input audio transcription event.
///
/// Syncs the transcript to Agent history and buffers it for paired memory persistence.
async fn process_user_transcript(event: &serde_json::Value, state: &RelayTranscriptState) {
    let Some(transcript) = event.get("transcript").and_then(|t| t.as_str()) else {
        return;
    };
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        return;
    }

    let turn_num = state
        .turn_counter
        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

    if let Some(ref agent_mutex) = state.agent {
        let mut agent = agent_mutex.lock().await;
        let turn = crate::providers::realtime_types::TranscriptTurn {
            turn_number: turn_num,
            user_text: trimmed.to_string(),
            assistant_text: String::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        agent.merge_voice_transcripts(&[turn]);
    }

    *state.pending_user_text.lock().await = Some((turn_num, trimmed.to_string()));
}

/// Process a completed assistant audio transcript event.
///
/// Syncs to Agent history and persists the user+assistant turn pair to memory
/// if it meets the minimum character threshold.
async fn process_assistant_transcript(event: &serde_json::Value, state: &RelayTranscriptState) {
    let Some(transcript) = event.get("transcript").and_then(|t| t.as_str()) else {
        return;
    };
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        return;
    }

    // Take the paired user turn; fall back to current counter
    let pending = state.pending_user_text.lock().await.take();
    let (turn_num, user_part) = match pending {
        Some((num, text)) => (num, text),
        None => {
            let num = state
                .turn_counter
                .load(std::sync::atomic::Ordering::Acquire)
                .saturating_sub(1);
            (num, String::new())
        }
    };

    // Sync to Agent history
    if let Some(ref agent_mutex) = state.agent {
        let mut agent = agent_mutex.lock().await;
        let turn = crate::providers::realtime_types::TranscriptTurn {
            turn_number: turn_num,
            user_text: String::new(),
            assistant_text: trimmed.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        agent.merge_voice_transcripts(&[turn]);
    }

    // Persist to memory if the pair meets minimum length
    let total_chars = user_part.chars().count() + trimmed.chars().count();
    if total_chars < VOICE_AUTOSAVE_MIN_CHARS {
        tracing::debug!(
            "[Voice] Skipped short transcript pair ({} chars < {})",
            total_chars,
            VOICE_AUTOSAVE_MIN_CHARS
        );
        return;
    }

    let mem = state.memory.clone();
    let session_id = &state.session_id;

    if !user_part.is_empty() {
        let key = format!("voice_turn_{}_{}_user", session_id, turn_num);
        let user_store = format!("[Voice] {}", user_part);
        if let Err(e) = mem
            .store(
                &key,
                &user_store,
                crate::memory::MemoryCategory::Custom("voice".to_string()),
                Some(session_id),
            )
            .await
        {
            warn!("Failed to persist voice user transcript: {}", e);
        }
    }

    let key = format!("voice_turn_{}_{}_assistant", session_id, turn_num);
    let assistant_store = format!("[Voice] {}", trimmed);
    if let Err(e) = mem
        .store(
            &key,
            &assistant_store,
            crate::memory::MemoryCategory::Custom("voice".to_string()),
            Some(session_id),
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
}

/// Direct WebSocket relay between browser and OpenAI Realtime API.
///
/// Orchestrates the session lifecycle:
/// 1. Connect to OpenAI Realtime API
/// 2. Send session configuration and inject chat history
/// 3. Run bidirectional relay (browser ↔ OpenAI)
/// 4. Graceful shutdown with close frames on both connections
async fn relay_session(
    browser_ws: WebSocket,
    config: &crate::config::VoiceConfig,
    tools: &[serde_json::Value],
    agent: Option<Arc<tokio::sync::Mutex<crate::agent::Agent>>>,
    memory: Arc<dyn crate::memory::Memory>,
) -> Result<()> {
    let realtime_config = config.to_realtime_config()?;
    let session_id = uuid::Uuid::new_v4().to_string();

    // Build Realtime API WebSocket URL and auth headers
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
        // Return the sink so we can send a close frame during shutdown
        openai_tx_raw
    });

    // Send session configuration
    let session_update = build_session_config(&realtime_config, tools, agent.is_some());
    to_openai_tx
        .send(session_update.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send session config: {}", e))?;

    // Inject Agent's chat history for context continuity
    if let Some(ref agent_mutex) = agent {
        inject_agent_history(agent_mutex, config.max_history_items, &to_openai_tx).await;
    }

    // Set up transcript state for the OpenAI→Browser relay
    let transcript_state = RelayTranscriptState {
        agent: agent.clone(),
        memory: memory.clone(),
        session_id: session_id.clone(),
        pending_user_text: Arc::new(tokio::sync::Mutex::new(None)),
        turn_counter: Arc::new(std::sync::atomic::AtomicU32::new(1)),
    };

    // Run bidirectional relay
    tokio::select! {
        () = handle_browser_to_openai(&mut browser_rx, &to_openai_tx) => {
            info!("Browser → OpenAI relay ended");
        }
        () = handle_openai_to_browser(&mut openai_rx, &mut browser_tx, &to_openai_tx, &transcript_state) => {
            info!("OpenAI → Browser relay ended");
        }
    }

    // ── Graceful shutdown: send close frames to both connections ──
    info!("Initiating graceful WebSocket shutdown for session {}", session_id);

    // Send close frame to browser
    let browser_close = browser_tx.send(WsMsg::Close(Some(axum::extract::ws::CloseFrame {
        code: axum::extract::ws::close_code::NORMAL,
        reason: "Session ended".into(),
    })));
    if let Err(e) = tokio::time::timeout(WS_CLOSE_TIMEOUT, browser_close).await {
        warn!("Browser close frame timed out: {}", e);
    }

    // Send close frame to OpenAI via the writer task's sink
    // First, drop the sender to signal the writer task to exit and return the sink
    drop(to_openai_tx);
    match tokio::time::timeout(WS_CLOSE_TIMEOUT, openai_writer).await {
        Ok(Ok(mut openai_sink)) => {
            let close_msg = TungMsg::Close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                reason: "Session ended".into(),
            }));
            if let Err(e) = tokio::time::timeout(
                WS_CLOSE_TIMEOUT,
                openai_sink.send(close_msg),
            )
            .await
            {
                warn!("OpenAI close frame timed out: {}", e);
            }
        }
        Ok(Err(e)) => warn!("OpenAI writer task panicked: {}", e),
        Err(_) => warn!("OpenAI writer task shutdown timed out"),
    }

    info!("Voice session {} shut down gracefully", session_id);
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

/// Maximum characters in a tool output returned to the Realtime API.
/// Voice context is limited; large outputs degrade latency and comprehension.
const VOICE_TOOL_MAX_OUTPUT_CHARS: usize = 4000;

/// Truncate tool output for voice context if it exceeds [`VOICE_TOOL_MAX_OUTPUT_CHARS`].
fn truncate_voice_output(output: &str) -> String {
    if output.chars().count() > VOICE_TOOL_MAX_OUTPUT_CHARS {
        let truncated: String = output.chars().take(VOICE_TOOL_MAX_OUTPUT_CHARS).collect();
        format!("{}... (truncated)", truncated)
    } else {
        output.to_string()
    }
}

/// Execute a tool via the Agent's tool dispatcher for voice function calling.
/// Returns the tool result as a string (JSON or plain text).
///
/// NOTE: The Agent mutex is held for the duration of tool execution because
/// `Agent.tools` stores `Box<dyn Tool>` which cannot be cloned or extracted
/// without the lock. Long-running tools will block other agent operations
/// (transcript saves, chat messages). Migrating `Agent.tools` to
/// `Arc<dyn Tool>` would allow lock-free execution but requires a cross-cutting
/// refactor — tracked as a follow-up.
async fn execute_voice_tool(
    agent_mutex: &Arc<tokio::sync::Mutex<crate::agent::Agent>>,
    name: &str,
    arguments_json: &str,
) -> String {
    let args: serde_json::Value = serde_json::from_str(arguments_json).unwrap_or(json!({}));

    // Hold lock only for tool lookup + execution; truncation happens after release.
    let result = {
        let agent = agent_mutex.lock().await;
        agent.execute_tool(name, args).await
    };

    match result {
        Ok(result) => truncate_voice_output(&result.output),
        Err(e) => {
            warn!("Voice tool '{}' execution failed: {}", name, e);
            format!("Error executing tool '{}': {}", name, e)
        }
    }
}

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

    // ── Event whitelist tests ──

    #[test]
    fn whitelist_allows_audio_buffer_append() {
        let msg = r#"{"type":"input_audio_buffer.append","audio":"AAAA"}"#;
        assert!(is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_allows_audio_buffer_commit() {
        let msg = r#"{"type":"input_audio_buffer.commit"}"#;
        assert!(is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_allows_audio_buffer_clear() {
        let msg = r#"{"type":"input_audio_buffer.clear"}"#;
        assert!(is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_allows_response_create() {
        let msg = r#"{"type":"response.create"}"#;
        assert!(is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_allows_response_cancel() {
        let msg = r#"{"type":"response.cancel"}"#;
        assert!(is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_blocks_session_update() {
        let msg = r#"{"type":"session.update","session":{"instructions":"evil prompt"}}"#;
        assert!(!is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_blocks_conversation_item_create() {
        let msg = r#"{"type":"conversation.item.create","item":{"type":"message","role":"user","content":[{"type":"input_text","text":"injected"}]}}"#;
        assert!(!is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_blocks_unknown_event() {
        let msg = r#"{"type":"response.function_call_arguments.done"}"#;
        assert!(!is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_blocks_missing_type_field() {
        let msg = r#"{"data":"no type field"}"#;
        assert!(!is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_blocks_invalid_json() {
        assert!(!is_browser_event_allowed("not json at all"));
    }

    #[test]
    fn whitelist_blocks_empty_string() {
        assert!(!is_browser_event_allowed(""));
    }

    #[test]
    fn whitelist_blocks_null_type() {
        let msg = r#"{"type":null}"#;
        assert!(!is_browser_event_allowed(msg));
    }

    #[test]
    fn whitelist_blocks_numeric_type() {
        let msg = r#"{"type":42}"#;
        assert!(!is_browser_event_allowed(msg));
    }

    // ── Transcript autosave threshold tests ──

    #[test]
    fn autosave_threshold_filters_short_pairs() {
        // "ok" (2) + "Got it!" (7) = 9 chars < 10 minimum
        let user = "ok";
        let assistant = "Got it!";
        let total = user.chars().count() + assistant.chars().count();
        assert!(total < VOICE_AUTOSAVE_MIN_CHARS);
    }

    #[test]
    fn autosave_threshold_accepts_sufficient_pairs() {
        // "안녕하세요" (5) + "네, 안녕하세요!" (7) = 12 chars >= 10
        let user = "안녕하세요";
        let assistant = "네, 안녕하세요!";
        let total = user.chars().count() + assistant.chars().count();
        assert!(total >= VOICE_AUTOSAVE_MIN_CHARS);
    }

    // ── Auth middleware tests ──

    #[test]
    fn auth_token_generation_produces_valid_uuid() {
        let token = uuid::Uuid::new_v4().to_string();
        assert!(!token.is_empty());
        assert!(uuid::Uuid::parse_str(&token).is_ok());
    }

    // ── Workspace prompt tests ──

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

    // ── Tool output truncation tests ──

    #[test]
    fn truncate_voice_output_short_passthrough() {
        let input = "Hello, world!";
        assert_eq!(truncate_voice_output(input), input);
    }

    #[test]
    fn truncate_voice_output_exact_boundary() {
        let input: String = "a".repeat(VOICE_TOOL_MAX_OUTPUT_CHARS);
        assert_eq!(truncate_voice_output(&input), input);
    }

    #[test]
    fn truncate_voice_output_over_limit() {
        let input: String = "b".repeat(VOICE_TOOL_MAX_OUTPUT_CHARS + 100);
        let result = truncate_voice_output(&input);
        assert!(result.ends_with("... (truncated)"));
        // Truncated content should be exactly max_chars + suffix
        let expected_len = VOICE_TOOL_MAX_OUTPUT_CHARS + "... (truncated)".len();
        assert_eq!(result.len(), expected_len);
    }

    #[test]
    fn truncate_voice_output_multibyte_chars() {
        // Korean characters are 3 bytes each in UTF-8
        let input: String = "가".repeat(VOICE_TOOL_MAX_OUTPUT_CHARS + 10);
        let result = truncate_voice_output(&input);
        assert!(result.ends_with("... (truncated)"));
        // Should truncate by char count, not byte count
        let content_part = result.strip_suffix("... (truncated)").unwrap();
        assert_eq!(content_part.chars().count(), VOICE_TOOL_MAX_OUTPUT_CHARS);
    }

    #[test]
    fn truncate_voice_output_empty_string() {
        assert_eq!(truncate_voice_output(""), "");
    }

    // ── Function call event parsing tests ──

    #[test]
    fn parse_function_call_event_extracts_fields() {
        let event = serde_json::json!({
            "type": "response.function_call_arguments.done",
            "call_id": "call_abc123",
            "name": "get_weather",
            "arguments": r#"{"city":"Seoul"}"#
        });
        let call_id = event.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
        let fn_name = event.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let fn_args = event
            .get("arguments")
            .and_then(|v| v.as_str())
            .unwrap_or("{}");

        assert_eq!(call_id, "call_abc123");
        assert_eq!(fn_name, "get_weather");
        assert_eq!(fn_args, r#"{"city":"Seoul"}"#);
    }

    #[test]
    fn parse_function_call_event_missing_call_id() {
        let event = serde_json::json!({
            "type": "response.function_call_arguments.done",
            "name": "get_weather",
            "arguments": "{}"
        });
        let call_id = event.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
        assert!(call_id.is_empty());
    }

    #[test]
    fn parse_function_call_event_missing_name() {
        let event = serde_json::json!({
            "type": "response.function_call_arguments.done",
            "call_id": "call_abc123",
            "arguments": "{}"
        });
        let fn_name = event.get("name").and_then(|v| v.as_str()).unwrap_or("");
        assert!(fn_name.is_empty());
    }

    #[test]
    fn parse_function_call_event_missing_arguments_defaults() {
        let event = serde_json::json!({
            "type": "response.function_call_arguments.done",
            "call_id": "call_abc123",
            "name": "get_weather"
        });
        let fn_args = event
            .get("arguments")
            .and_then(|v| v.as_str())
            .unwrap_or("{}");
        assert_eq!(fn_args, "{}");
    }

    #[test]
    fn parse_function_call_arguments_invalid_json_defaults() {
        let bad_json = "not valid json {{{";
        let args: serde_json::Value = serde_json::from_str(bad_json).unwrap_or(json!({}));
        assert_eq!(args, json!({}));
    }

    // ── Tool registration in session config tests ──

    #[test]
    fn session_config_includes_tools_when_present() {
        let mut session_config = json!({"model": "gpt-4o-realtime"});
        let tools = vec![json!({"type": "function", "name": "test_tool"})];
        let has_agent = true;

        if !tools.is_empty() && has_agent {
            session_config["tools"] = json!(tools);
        }

        assert!(session_config.get("tools").is_some());
        let registered = session_config["tools"].as_array().unwrap();
        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0]["name"], "test_tool");
    }

    #[test]
    fn session_config_omits_tools_without_agent() {
        let mut session_config = json!({"model": "gpt-4o-realtime"});
        let tools = vec![json!({"type": "function", "name": "test_tool"})];
        let has_agent = false;

        if !tools.is_empty() && has_agent {
            session_config["tools"] = json!(tools);
        }

        assert!(session_config.get("tools").is_none());
    }

    #[test]
    fn session_config_omits_tools_when_empty() {
        let mut session_config = json!({"model": "gpt-4o-realtime"});
        let tools: Vec<serde_json::Value> = vec![];
        let has_agent = true;

        if !tools.is_empty() && has_agent {
            session_config["tools"] = json!(tools);
        }

        assert!(session_config.get("tools").is_none());
    }
}
