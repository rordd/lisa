//! Types for OpenAI Realtime API (WebSocket-based voice/text streaming).

use serde::{Deserialize, Serialize};

/// Azure OpenAI Realtime API version.
pub const AZURE_REALTIME_API_VERSION: &str = "2025-04-01-preview";

/// PCM16 audio chunk (16kHz or 24kHz mono, signed 16-bit little-endian).
pub type AudioChunk = Vec<u8>;

/// Events emitted by a Realtime API session.
#[derive(Debug)]
pub enum RealtimeEvent {
    /// Session created and ready for interaction.
    SessionReady,
    /// AI is generating a response.
    ResponseStarted,
    /// Audio chunk from AI response (PCM16 LE).
    AudioDelta(AudioChunk),
    /// AI response audio stream complete.
    AudioDone,
    /// User's speech transcribed (input_audio_transcription.completed).
    InputTranscript(String),
    /// AI's response transcribed (response.audio_transcript.done).
    OutputTranscript(String),
    /// User started speaking (server VAD).
    SpeechStarted,
    /// User stopped speaking (server VAD).
    SpeechStopped,
    /// Error from the Realtime API.
    Error(String),
    /// WebSocket connection closed.
    Closed,
}

/// API provider type for Realtime connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RealtimeApiProvider {
    /// Azure OpenAI Service.
    Azure {
        endpoint: String,
        deployment: String,
    },
    /// OpenAI direct API.
    OpenAI,
}

/// Configuration for a Realtime voice session.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// API key / credential.
    pub api_key: String,
    /// Model identifier (e.g. "gpt-realtime-1.5").
    pub model: String,
    /// Voice name (e.g. "alloy", "echo", "shimmer").
    pub voice: String,
    /// Primary language code (e.g. "ko", "en").
    pub language: String,
    /// Audio format for input/output (e.g. "pcm16").
    pub audio_format: String,
    /// VAD type (e.g. "server_vad").
    pub vad_type: String,
    /// Voice Activity Detection threshold (0.0–1.0).
    pub vad_threshold: f32,
    /// VAD prefix padding in milliseconds.
    pub vad_prefix_padding_ms: u32,
    /// Silence duration before committing user speech (ms).
    pub silence_duration_ms: u32,
    /// Transcription model (e.g. "whisper-1").
    pub transcription_model: String,
    /// System prompt for the voice assistant.
    pub system_prompt: String,
    /// API provider configuration.
    pub provider: RealtimeApiProvider,
    /// Whether to save transcripts to Memory.
    pub save_transcripts: bool,
}

impl RealtimeConfig {
    /// Build the WebSocket URL for the Realtime API.
    pub fn ws_url(&self) -> String {
        match &self.provider {
            RealtimeApiProvider::Azure {
                endpoint,
                deployment,
            } => {
                format!(
                    "wss://{endpoint}/openai/realtime?api-version={}&deployment={deployment}",
                    AZURE_REALTIME_API_VERSION
                )
            }
            RealtimeApiProvider::OpenAI => {
                format!("wss://api.openai.com/v1/realtime?model={}", self.model)
            }
        }
    }

    /// Build a redacted URL for logging (strips query parameters that may contain credentials).
    pub fn ws_url_redacted(&self) -> String {
        let url = self.ws_url();
        match url.find('?') {
            Some(idx) => format!("{}?<redacted>", &url[..idx]),
            None => url,
        }
    }

    /// Build authentication headers for the WebSocket handshake.
    pub fn auth_headers(&self) -> Vec<(String, String)> {
        match &self.provider {
            RealtimeApiProvider::Azure { .. } => {
                vec![("api-key".to_string(), self.api_key.clone())]
            }
            RealtimeApiProvider::OpenAI => {
                vec![
                    (
                        "Authorization".to_string(),
                        format!("Bearer {}", self.api_key),
                    ),
                    ("OpenAI-Beta".to_string(), "realtime=v1".to_string()),
                ]
            }
        }
    }
}

/// A collected transcript turn (user input + assistant output pair).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTurn {
    /// Sequential turn number within the session.
    pub turn_number: u32,
    /// User's spoken input (transcribed).
    pub user_text: String,
    /// Assistant's spoken response (transcribed).
    pub assistant_text: String,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
}

/// Session-level metadata for a voice conversation.
#[derive(Debug, Clone)]
pub struct RealtimeSessionInfo {
    /// Unique session identifier.
    pub session_id: String,
    /// Model used.
    pub model: String,
    /// Voice used.
    pub voice: String,
    /// Number of completed turns.
    pub turn_count: u32,
    /// Session start time (ISO 8601).
    pub started_at: String,
}
