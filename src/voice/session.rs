//! VoiceSession — orchestrates a Realtime voice conversation with Memory persistence.
//!
//! Connects to the Realtime API via `RealtimeProvider`, collects transcript events,
//! and stores user+assistant turn pairs into the agent's Memory system.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::memory::{Memory, MemoryCategory};
use crate::providers::realtime::RealtimeProvider;
use crate::providers::realtime_types::{AudioChunk, RealtimeConfig, RealtimeEvent, TranscriptTurn};
use crate::providers::ConversationMessage;
use crate::tools::ToolSpec;

/// A voice conversation session that integrates Realtime API with Memory storage.
pub struct VoiceSession {
    /// Realtime API provider (Arc for spawning into background tasks).
    provider: Arc<dyn RealtimeProvider>,
    /// Memory backend for transcript persistence.
    memory: Arc<dyn Memory>,
    /// Session identifier (for memory scoping).
    session_id: String,
    /// Whether to persist transcripts.
    save_transcripts: bool,
    /// Collected transcript turns.
    turns: Vec<TranscriptTurn>,
    /// Chat history to inject into Realtime API at session start (for context sharing).
    prior_history: Vec<ConversationMessage>,
    /// Channel to send transcript turns back to the Agent for history merging.
    transcript_tx: Option<mpsc::Sender<TranscriptTurn>>,
    /// Tool specifications to register with the Realtime API for function calling.
    tool_specs: Vec<ToolSpec>,
}

impl VoiceSession {
    /// Create a new voice session.
    pub fn new(
        provider: Arc<dyn RealtimeProvider>,
        memory: Arc<dyn Memory>,
        session_id: String,
        save_transcripts: bool,
    ) -> Self {
        Self {
            provider,
            memory,
            session_id,
            save_transcripts,
            turns: Vec::new(),
            prior_history: Vec::new(),
            transcript_tx: None,
            tool_specs: Vec::new(),
        }
    }

    /// Set tool specifications for Realtime API function calling.
    pub fn with_tools(mut self, specs: Vec<ToolSpec>) -> Self {
        self.tool_specs = specs;
        self
    }

    /// Convert tool specs into Realtime API session tools format.
    /// Returns a JSON array of tool definitions for the session.update event.
    pub fn tools_as_realtime_json(&self) -> Vec<serde_json::Value> {
        self.tool_specs
            .iter()
            .map(|spec| spec.to_realtime_json())
            .collect()
    }

    /// Set prior chat history to inject into the Realtime API session.
    /// This enables the voice session to be aware of preceding text conversations.
    pub fn with_prior_history(mut self, history: Vec<ConversationMessage>) -> Self {
        self.prior_history = history;
        self
    }

    /// Set a channel to receive transcript turns for merging back into Agent history.
    pub fn with_transcript_sender(mut self, tx: mpsc::Sender<TranscriptTurn>) -> Self {
        self.transcript_tx = Some(tx);
        self
    }

    /// Create from a `RealtimeConfig` and memory backend.
    pub fn from_config(config: RealtimeConfig, memory: Arc<dyn Memory>) -> Self {
        let session_id = format!("voice_{}", uuid::Uuid::new_v4());
        let save = config.save_transcripts;
        let provider = crate::providers::realtime::create_realtime_provider(config);
        Self::new(provider, memory, session_id, save)
    }

    /// Session identifier.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Collected transcript turns so far.
    pub fn turns(&self) -> &[TranscriptTurn] {
        &self.turns
    }

    /// Run the voice session.
    ///
    /// - `audio_rx`: PCM16 audio stream from the microphone
    /// - `audio_out_tx`: channel to forward AI audio output for playback
    ///
    /// Spawns the provider connection in a background task and processes
    /// events (including Memory storage) concurrently.
    /// Returns when the session closes (user disconnect, error, etc.)
    pub async fn run(
        &mut self,
        audio_rx: mpsc::Receiver<AudioChunk>,
        audio_out_tx: Option<mpsc::Sender<AudioChunk>>,
    ) -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::channel::<RealtimeEvent>(1024);

        // Spawn provider connection in background (Arc enables move into task)
        let provider = self.provider.clone();
        let connect_handle = tokio::spawn(async move {
            if let Err(e) = provider.connect(audio_rx, event_tx).await {
                tracing::error!("Realtime provider connection error: {}", e);
            }
        });

        // Event processing loop
        let memory = self.memory.clone();
        let session_id = self.session_id.clone();
        let save_transcripts = self.save_transcripts;
        let mut pending_user_text: Option<String> = None;
        let mut turn_number: u32 = 0;

        while let Some(event) = event_rx.recv().await {
            match event {
                RealtimeEvent::InputTranscript(text) => {
                    tracing::info!("[Voice] User: {}", text);
                    pending_user_text = Some(text);
                }

                RealtimeEvent::OutputTranscript(text) => {
                    tracing::info!("[Voice] Assistant: {}", text);

                    let user_text = pending_user_text.take().unwrap_or_default();
                    turn_number += 1;

                    let now = chrono::Utc::now().to_rfc3339();
                    let turn = TranscriptTurn {
                        turn_number,
                        user_text: user_text.clone(),
                        assistant_text: text.clone(),
                        timestamp: now,
                    };

                    // Store to memory immediately (turn-based strategy)
                    if save_transcripts {
                        let key = format!("voice_turn_{}_{}", session_id, turn_number);
                        let content = format!("User: {}\nAssistant: {}", user_text, text);
                        if let Err(e) = memory
                            .store(
                                &key,
                                &content,
                                MemoryCategory::Custom("voice".to_string()),
                                Some(&session_id),
                            )
                            .await
                        {
                            tracing::warn!("Failed to store voice transcript: {}", e);
                        }
                    }

                    // Send transcript turn to Agent for history merging
                    if let Some(ref tx) = self.transcript_tx {
                        if let Err(e) = tx.send(turn.clone()).await {
                            tracing::warn!("Failed to send voice transcript to agent: {}", e);
                        }
                    }

                    self.turns.push(turn);
                }

                RealtimeEvent::AudioDelta(audio_data) => {
                    // Forward audio to playback channel if provided
                    if let Some(ref tx) = audio_out_tx {
                        let _ = tx.send(audio_data).await;
                    }
                }

                RealtimeEvent::Error(err) => {
                    tracing::error!("[Voice] Error: {}", err);
                }

                RealtimeEvent::Closed => {
                    tracing::info!("[Voice] Session closed after {} turns", turn_number);
                    break;
                }

                // VAD events, ResponseStarted, AudioDone — log only
                _ => {}
            }
        }

        // Wait for provider task to finish cleanly
        if let Err(e) = connect_handle.await {
            tracing::warn!("Voice provider task join error: {e}");
        }

        Ok(())
    }

    /// Convert prior chat history into Realtime API `conversation.item.create` events.
    /// Only user and assistant text messages are included (tool calls are skipped).
    /// Returns a list of JSON strings ready to send over WebSocket.
    pub fn history_as_realtime_items(&self) -> Vec<String> {
        self.history_as_realtime_items_limited(usize::MAX)
    }

    /// Convert prior chat history into Realtime API items, limited to the last `max` entries.
    pub fn history_as_realtime_items_limited(&self, max: usize) -> Vec<String> {
        let history: Vec<_> = if self.prior_history.len() > max {
            self.prior_history[self.prior_history.len() - max..].to_vec()
        } else {
            self.prior_history.clone()
        };
        let mut items = Vec::new();
        for msg in &history {
            match msg {
                ConversationMessage::Chat(chat)
                    if chat.role == "user" || chat.role == "assistant" =>
                {
                    // Realtime API uses "input_text" for user, "text" for assistant
                    let content_type = if chat.role == "user" {
                        "input_text"
                    } else {
                        "text"
                    };
                    let item = serde_json::json!({
                        "type": "conversation.item.create",
                        "item": {
                            "type": "message",
                            "role": chat.role,
                            "content": [{
                                "type": content_type,
                                "text": chat.content
                            }]
                        }
                    });
                    items.push(item.to_string());
                }
                _ => {} // Skip system, tool calls, tool results
            }
        }
        items
    }

    /// Store a session summary to memory (optional, call after session ends).
    ///
    /// This implements session-level summary as an opt-in addition
    /// to the turn-based storage done during the session.
    pub async fn store_session_summary(&self) -> Result<()> {
        if self.turns.is_empty() {
            return Ok(());
        }

        let mut summary = format!(
            "Voice session {} ({} turns):\n",
            self.session_id,
            self.turns.len()
        );

        for turn in &self.turns {
            use std::fmt::Write;
            let _ = writeln!(
                summary,
                "  [{}] User: {} → Assistant: {}",
                turn.turn_number, turn.user_text, turn.assistant_text
            );
        }

        let key = format!("voice_session_{}", self.session_id);
        self.memory
            .store(
                &key,
                &summary,
                MemoryCategory::Custom("voice".to_string()),
                Some(&self.session_id),
            )
            .await?;

        tracing::info!(
            "Stored voice session summary: {} ({} turns)",
            self.session_id,
            self.turns.len()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ChatMessage;

    #[test]
    fn transcript_turn_serializes() {
        let turn = TranscriptTurn {
            turn_number: 1,
            user_text: "안녕하세요".to_string(),
            assistant_text: "안녕하세요! 무엇을 도와드릴까요?".to_string(),
            timestamp: "2026-03-10T09:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&turn).unwrap();
        assert!(json.contains("안녕하세요"));
        assert!(json.contains("turn_number"));
    }

    fn make_chat_msg(role: &str, content: &str) -> ConversationMessage {
        ConversationMessage::Chat(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        })
    }

    /// Minimal Memory implementation for testing (no persistence).
    struct TestMemory;

    #[async_trait::async_trait]
    impl crate::memory::Memory for TestMemory {
        fn name(&self) -> &str {
            "test"
        }
        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        async fn recall(
            &self,
            _query: &str,
            _limit: usize,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<crate::memory::MemoryEntry>> {
            Ok(vec![])
        }
        async fn get(&self, _key: &str) -> anyhow::Result<Option<crate::memory::MemoryEntry>> {
            Ok(None)
        }
        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<crate::memory::MemoryEntry>> {
            Ok(vec![])
        }
        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(false)
        }
        async fn count(&self) -> anyhow::Result<usize> {
            Ok(0)
        }
        async fn health_check(&self) -> bool {
            true
        }
    }

    fn make_session_with_history(history: Vec<ConversationMessage>) -> VoiceSession {
        let memory: Arc<dyn crate::memory::Memory> = Arc::new(TestMemory);
        // Use a dummy provider — we only test history conversion, not connection
        let config = RealtimeConfig {
            api_key: "test".into(),
            model: "test".into(),
            voice: "alloy".into(),
            language: "ko".into(),
            audio_format: "pcm16".into(),
            vad_type: "server_vad".into(),
            vad_threshold: 0.5,
            vad_prefix_padding_ms: 300,
            silence_duration_ms: 500,
            transcription_model: "whisper-1".into(),
            system_prompt: "test".into(),
            provider: crate::providers::realtime_types::RealtimeApiProvider::OpenAI,
            save_transcripts: false,
        };
        let provider = Arc::new(crate::providers::realtime::OpenAiRealtimeProvider::new(
            config,
        ));
        VoiceSession::new(provider, memory, "test-session".into(), false)
            .with_prior_history(history)
    }

    #[test]
    fn history_as_realtime_items_converts_user_and_assistant() {
        let session = make_session_with_history(vec![
            make_chat_msg("user", "안녕"),
            make_chat_msg("assistant", "안녕하세요!"),
        ]);
        let items = session.history_as_realtime_items();
        assert_eq!(items.len(), 2);

        let user_item: serde_json::Value = serde_json::from_str(&items[0]).unwrap();
        assert_eq!(user_item["item"]["role"], "user");
        assert_eq!(user_item["item"]["content"][0]["type"], "input_text");
        assert_eq!(user_item["item"]["content"][0]["text"], "안녕");

        let assistant_item: serde_json::Value = serde_json::from_str(&items[1]).unwrap();
        assert_eq!(assistant_item["item"]["role"], "assistant");
        assert_eq!(assistant_item["item"]["content"][0]["type"], "text");
    }

    #[test]
    fn history_as_realtime_items_skips_system_messages() {
        let session = make_session_with_history(vec![
            make_chat_msg("system", "You are helpful"),
            make_chat_msg("user", "질문"),
            make_chat_msg("assistant", "답변"),
        ]);
        let items = session.history_as_realtime_items();
        assert_eq!(items.len(), 2); // system message skipped
    }

    #[test]
    fn history_as_realtime_items_skips_tool_results() {
        let session = make_session_with_history(vec![
            make_chat_msg("user", "날씨 알려줘"),
            ConversationMessage::ToolResults(vec![crate::providers::ToolResultMessage {
                content: "맑음".into(),
                tool_call_id: "call_1".into(),
            }]),
            make_chat_msg("assistant", "오늘 맑습니다"),
        ]);
        let items = session.history_as_realtime_items();
        assert_eq!(items.len(), 2); // only user + assistant chat messages
    }

    #[test]
    fn history_as_realtime_items_empty_history() {
        let session = make_session_with_history(vec![]);
        let items = session.history_as_realtime_items();
        assert!(items.is_empty());
    }
}
