use crate::agent::Agent;
use crate::providers::traits::{ChatMessage, ConversationMessage};
use anyhow::Result;

impl Agent {
    /// Create a voice session from this agent's configuration.
    /// The voice session shares the agent's memory and workspace context.
    /// Injects recent chat history so voice can continue the conversation.
    /// Returns (VoiceSession, transcript_rx) — caller should drain transcript_rx
    /// and call `merge_voice_transcripts()` to keep chat history in sync.
    pub fn create_voice_session(
        &self,
    ) -> Result<(
        crate::voice::VoiceSession,
        tokio::sync::mpsc::Receiver<crate::providers::realtime_types::TranscriptTurn>,
    )> {
        let rt_provider = self
            .realtime_provider()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Voice not enabled: no realtime provider configured"))?;

        let agent_session = self.session_id().unwrap_or("anon");
        let session_id = format!("voice_{}_{}", agent_session, uuid::Uuid::new_v4());
        let save_transcripts = self
            .voice_config()
            .map(|vc| vc.save_transcripts)
            .unwrap_or(true);

        // Inject recent chat history (last N user/assistant turns)
        let max_history_items = self
            .voice_config()
            .map(|vc| vc.max_history_items)
            .unwrap_or(10);
        let recent_history: Vec<ConversationMessage> = self
            .history()
            .iter()
            .filter(|msg| {
                matches!(msg,
                    ConversationMessage::Chat(chat) if chat.role == "user" || chat.role == "assistant"
                )
            })
            .rev()
            .take(max_history_items)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Create transcript channel for history merging
        let (transcript_tx, transcript_rx) = tokio::sync::mpsc::channel(64);

        let session = crate::voice::VoiceSession::new(
            rt_provider,
            self.memory().clone(),
            session_id,
            save_transcripts,
        )
        .with_prior_history(recent_history)
        .with_transcript_sender(transcript_tx)
        .with_tools(self.tool_specs().to_vec());

        Ok((session, transcript_rx))
    }

    /// Merge voice transcript turns back into chat history.
    /// Call this after a voice session ends to maintain conversation continuity.
    pub fn merge_voice_transcripts(
        &mut self,
        turns: &[crate::providers::realtime_types::TranscriptTurn],
    ) {
        for turn in turns {
            if !turn.user_text.is_empty() {
                self.push_history(ConversationMessage::Chat(ChatMessage::user(format!(
                    "[Voice] {}",
                    turn.user_text
                ))));
            }
            if !turn.assistant_text.is_empty() {
                self.push_history(ConversationMessage::Chat(ChatMessage::assistant(format!(
                    "[Voice] {}",
                    turn.assistant_text
                ))));
            }
        }
    }
}
