//! OpenAI Realtime API provider (WebSocket-based voice + text streaming).
//!
//! Supports both Azure OpenAI and direct OpenAI endpoints.
//! This provider does NOT implement the `Provider` trait (HTTP chat),
//! as Realtime uses a fundamentally different communication pattern.

use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

use super::realtime_types::{AudioChunk, RealtimeConfig, RealtimeEvent};

/// Trait for Realtime API providers.
///
/// Separate from the chat `Provider` trait because:
/// - Communication is WebSocket (full-duplex) vs HTTP (request-response)
/// - Session is long-lived and stateful
/// - I/O includes audio streams, not just text
#[async_trait]
pub trait RealtimeProvider: Send + Sync {
    /// Establish a WebSocket connection and run the session event loop.
    ///
    /// - `audio_rx`: incoming PCM16 audio from the microphone
    /// - `event_tx`: outgoing events (transcripts, audio deltas, errors)
    ///
    /// This method runs until the connection closes or an error occurs.
    async fn connect(
        &self,
        audio_rx: mpsc::Receiver<AudioChunk>,
        event_tx: mpsc::Sender<RealtimeEvent>,
    ) -> Result<()>;

    /// Provider display name.
    fn name(&self) -> &str;

    /// Model identifier.
    fn model(&self) -> &str;
}

/// OpenAI Realtime API provider supporting Azure and direct OpenAI.
pub struct OpenAiRealtimeProvider {
    config: RealtimeConfig,
}

impl OpenAiRealtimeProvider {
    /// Create a new provider from configuration.
    pub fn new(config: RealtimeConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl RealtimeProvider for OpenAiRealtimeProvider {
    async fn connect(
        &self,
        mut audio_rx: mpsc::Receiver<AudioChunk>,
        event_tx: mpsc::Sender<RealtimeEvent>,
    ) -> Result<()> {
        let url = self.config.ws_url();
        tracing::info!(
            "Connecting to Realtime API: {}",
            self.config.ws_url_redacted()
        );

        // Build WebSocket request with auth headers
        let mut request = url.into_client_request()?;
        for (name, value) in self.config.auth_headers() {
            request.headers_mut().insert(
                name.parse::<axum::http::HeaderName>()
                    .map_err(|e| anyhow::anyhow!("Invalid header name: {}", e))?,
                value
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid header value: {}", e))?,
            );
        }

        let (ws_stream, _response) = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            tokio_tungstenite::connect_async(request),
        )
        .await
        .context("Realtime API connection timed out (30s)")?
        .context("Failed to connect to Realtime API")?;

        tracing::info!("Connected to Realtime API");

        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // Send session configuration
        let session_update = json!({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "voice": self.config.voice,
                "instructions": self.config.system_prompt,
                "input_audio_format": self.config.audio_format,
                "output_audio_format": self.config.audio_format,
                "input_audio_transcription": {
                    "model": self.config.transcription_model,
                    "language": self.config.language
                },
                "turn_detection": {
                    "type": self.config.vad_type,
                    "threshold": self.config.vad_threshold,
                    "prefix_padding_ms": self.config.vad_prefix_padding_ms,
                    "silence_duration_ms": self.config.silence_duration_ms
                }
            }
        });

        ws_tx
            .send(Message::Text(session_update.to_string().into()))
            .await
            .context("Failed to send session update")?;

        tracing::info!(
            "Realtime session configured (model={}, voice={})",
            self.config.model,
            self.config.voice
        );

        let event_tx_clone = event_tx.clone();

        // Task: forward microphone audio to WebSocket
        let mut send_task = tokio::spawn(async move {
            let b64 = base64::engine::general_purpose::STANDARD;
            while let Some(chunk) = audio_rx.recv().await {
                let encoded = b64.encode(&chunk);
                let msg = json!({
                    "type": "input_audio_buffer.append",
                    "audio": encoded
                });
                if let Err(e) = ws_tx.send(Message::Text(msg.to_string().into())).await {
                    tracing::warn!("Failed to send audio chunk: {e}");
                    break;
                }
            }
        });

        // Task: receive and route API events
        let mut recv_task = tokio::spawn(async move {
            while let Some(msg) = ws_rx.next().await {
                match msg {
                    Ok(Message::Text(text)) => match serde_json::from_str::<Value>(&text) {
                        Ok(event) => dispatch_event(&event, &event_tx_clone).await,
                        Err(e) => tracing::warn!("Failed to parse Realtime API event: {e}"),
                    },
                    Ok(Message::Close(_)) => {
                        let _ = event_tx_clone.send(RealtimeEvent::Closed).await;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        let _ = event_tx_clone
                            .send(RealtimeEvent::Error(e.to_string()))
                            .await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Run until either task ends, then abort the other to prevent resource leak
        tokio::select! {
            _ = &mut send_task => {
                tracing::info!("Audio send task ended");
                recv_task.abort();
            },
            _ = &mut recv_task => {
                tracing::info!("Event receive task ended");
                send_task.abort();
            },
        }

        let _ = event_tx.send(RealtimeEvent::Closed).await;
        Ok(())
    }

    fn name(&self) -> &str {
        match &self.config.provider {
            super::realtime_types::RealtimeApiProvider::Azure { .. } => "azure-realtime",
            super::realtime_types::RealtimeApiProvider::OpenAI => "openai-realtime",
        }
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Route a raw JSON event from the Realtime API to the appropriate `RealtimeEvent`.
async fn dispatch_event(event: &Value, tx: &mpsc::Sender<RealtimeEvent>) {
    let event_type = event["type"].as_str().unwrap_or("");
    let b64 = base64::engine::general_purpose::STANDARD;

    match event_type {
        "session.created" | "session.updated" => {
            tracing::info!("Realtime session ready");
            let _ = tx.send(RealtimeEvent::SessionReady).await;
        }

        "input_audio_buffer.speech_started" => {
            tracing::debug!("Speech started (VAD)");
            let _ = tx.send(RealtimeEvent::SpeechStarted).await;
        }

        "input_audio_buffer.speech_stopped" => {
            tracing::debug!("Speech stopped (VAD)");
            let _ = tx.send(RealtimeEvent::SpeechStopped).await;
        }

        "response.created" => {
            let _ = tx.send(RealtimeEvent::ResponseStarted).await;
        }

        "response.audio.delta" => {
            if let Some(delta) = event["delta"].as_str() {
                if let Ok(audio_bytes) = b64.decode(delta) {
                    let _ = tx.send(RealtimeEvent::AudioDelta(audio_bytes)).await;
                }
            }
        }

        "response.audio.done" => {
            let _ = tx.send(RealtimeEvent::AudioDone).await;
        }

        "response.audio_transcript.done" => {
            if let Some(transcript) = event["transcript"].as_str() {
                let _ = tx
                    .send(RealtimeEvent::OutputTranscript(transcript.to_string()))
                    .await;
            }
        }

        "conversation.item.input_audio_transcription.completed" => {
            if let Some(transcript) = event["transcript"].as_str() {
                tracing::info!("User said: {}", transcript);
                let _ = tx
                    .send(RealtimeEvent::InputTranscript(transcript.to_string()))
                    .await;
            }
        }

        "error" => {
            let error_msg = event["error"]["message"]
                .as_str()
                .unwrap_or("Unknown Realtime API error");
            tracing::error!("Realtime API error: {}", error_msg);
            let _ = tx.send(RealtimeEvent::Error(error_msg.to_string())).await;
        }

        _ => {
            tracing::trace!("Unhandled Realtime event: {}", event_type);
        }
    }
}

/// Factory function to create a `RealtimeProvider` from config.
pub fn create_realtime_provider(config: RealtimeConfig) -> Arc<dyn RealtimeProvider> {
    Arc::new(OpenAiRealtimeProvider::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::realtime_types::{RealtimeApiProvider, RealtimeConfig};

    fn test_config(provider: RealtimeApiProvider) -> RealtimeConfig {
        RealtimeConfig {
            api_key: "test-key".to_string(),
            model: "gpt-realtime-1.5".to_string(),
            voice: "alloy".to_string(),
            language: "ko".to_string(),
            audio_format: "pcm16".to_string(),
            vad_type: "server_vad".to_string(),
            vad_threshold: 0.5,
            vad_prefix_padding_ms: 300,
            silence_duration_ms: 500,
            transcription_model: "whisper-1".to_string(),
            system_prompt: "Test prompt".to_string(),
            provider,
            save_transcripts: true,
        }
    }

    #[test]
    fn azure_ws_url_format() {
        let config = test_config(RealtimeApiProvider::Azure {
            endpoint: "10.182.173.75".to_string(),
            deployment: "gpt-realtime-1.5".to_string(),
        });
        let url = config.ws_url();
        assert!(url.starts_with("wss://10.182.173.75/openai/realtime"));
        assert!(url.contains("deployment=gpt-realtime-1.5"));
    }

    #[test]
    fn openai_ws_url_format() {
        let config = test_config(RealtimeApiProvider::OpenAI);
        let url = config.ws_url();
        assert!(url.starts_with("wss://api.openai.com/v1/realtime"));
        assert!(url.contains("model=gpt-realtime-1.5"));
    }

    #[test]
    fn azure_auth_headers() {
        let config = test_config(RealtimeApiProvider::Azure {
            endpoint: "test.openai.azure.com".to_string(),
            deployment: "deploy".to_string(),
        });
        let headers = config.auth_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "api-key");
        assert_eq!(headers[0].1, "test-key");
    }

    #[test]
    fn openai_auth_headers() {
        let config = test_config(RealtimeApiProvider::OpenAI);
        let headers = config.auth_headers();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, "Authorization");
        assert!(headers[0].1.starts_with("Bearer "));
        assert_eq!(headers[1].0, "OpenAI-Beta");
    }

    #[test]
    fn provider_name_azure() {
        let config = test_config(RealtimeApiProvider::Azure {
            endpoint: "e".into(),
            deployment: "d".into(),
        });
        let provider = OpenAiRealtimeProvider::new(config);
        assert_eq!(provider.name(), "azure-realtime");
    }

    #[test]
    fn provider_name_openai() {
        let config = test_config(RealtimeApiProvider::OpenAI);
        let provider = OpenAiRealtimeProvider::new(config);
        assert_eq!(provider.name(), "openai-realtime");
    }

    #[test]
    fn provider_model() {
        let config = test_config(RealtimeApiProvider::OpenAI);
        let provider = OpenAiRealtimeProvider::new(config);
        assert_eq!(provider.model(), "gpt-realtime-1.5");
    }

    #[test]
    fn ws_url_redacted_strips_query_params() {
        let config = test_config(RealtimeApiProvider::Azure {
            endpoint: "10.182.173.75".to_string(),
            deployment: "gpt-realtime-1.5".to_string(),
        });
        let redacted = config.ws_url_redacted();
        assert!(redacted.starts_with("wss://10.182.173.75/openai/realtime"));
        assert!(redacted.ends_with("?<redacted>"));
        assert!(!redacted.contains("deployment="));
    }
}
