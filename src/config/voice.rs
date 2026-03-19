use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Voice (Realtime API) ────────────────────────────────────────

fn default_voice_model() -> String {
    "gpt-realtime".into()
}

fn default_voice_name() -> String {
    "alloy".into()
}

fn default_voice_language() -> String {
    "en".into()
}

fn default_vad_threshold() -> f32 {
    0.75
}

fn default_silence_duration_ms() -> u32 {
    500
}

fn default_voice_transcription_model() -> String {
    "whisper-1".into()
}

fn default_voice_vad_type() -> String {
    "server_vad".into()
}

fn default_voice_vad_prefix_padding_ms() -> u32 {
    300
}

fn default_voice_max_history_items() -> usize {
    10
}

fn default_voice_barge_in_delay_ms() -> u32 {
    500
}

fn default_voice_audio_format() -> String {
    "pcm16".into()
}

fn default_voice_system_prompt() -> String {
    "You are a helpful AI voice assistant. Respond concisely and conversationally.".into()
}

fn default_true() -> bool {
    true
}

/// Realtime voice session configuration (`[voice]`).
///
/// Independent from the chat `[model]` settings — allows using different
/// providers and models for text chat vs voice conversations.
///
/// **Configuration split:**
/// - Connection/credential fields (`provider`, `api_key`, `model`, `voice`,
///   `language`, `azure_endpoint`, `azure_deployment`, `system_prompt`,
///   `save_transcripts`) are typically set via env vars (`ZEROCLAW_VOICE_*`)
///   or `.env` since they vary per deployment environment.
/// - Tuning fields (`vad_threshold`, `silence_duration_ms`, `transcription_model`,
///   `vad_type`, `vad_prefix_padding_ms`, `max_history_items`, `barge_in_delay_ms`,
///   `audio_format`) belong in `config.toml [voice]` since they are behavioral
///   parameters with sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VoiceConfig {
    // ── Connection / Credential fields (typically from env vars) ──
    /// Enable Realtime voice sessions.
    #[serde(default)]
    pub enabled: bool,

    /// Realtime API provider: `"azure"` or `"openai"`.
    #[serde(default)]
    pub provider: Option<String>,

    /// API key for the Realtime provider.
    /// Falls back to `AZURE_OPENAI_API_KEY` or `OPENAI_API_KEY` env vars.
    #[serde(default)]
    pub api_key: Option<String>,

    /// Model identifier (e.g. `"gpt-realtime-1.5"`).
    #[serde(default = "default_voice_model")]
    pub model: String,

    /// Voice name (e.g. `"alloy"`, `"echo"`, `"shimmer"`).
    #[serde(default = "default_voice_name")]
    pub voice: String,

    /// Primary language code (ISO-639-1, e.g. `"ko"`, `"en"`).
    #[serde(default = "default_voice_language")]
    pub language: String,

    /// System prompt for the voice assistant.
    #[serde(default = "default_voice_system_prompt")]
    pub system_prompt: String,

    /// Azure OpenAI endpoint (e.g. `"10.182.173.75"`). Required when provider is `"azure"`.
    #[serde(default)]
    pub azure_endpoint: Option<String>,

    /// Azure OpenAI deployment name. Defaults to `model` value if unset.
    #[serde(default)]
    pub azure_deployment: Option<String>,

    /// Whether to save voice transcripts to Memory.
    #[serde(default = "default_true")]
    pub save_transcripts: bool,

    // ── Tuning fields (typically from config.toml [voice]) ──
    /// Voice Activity Detection threshold (0.0–1.0).
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,

    /// Silence duration (ms) before committing user speech.
    #[serde(default = "default_silence_duration_ms")]
    pub silence_duration_ms: u32,

    /// Transcription model for input audio (e.g. `"whisper-1"`).
    #[serde(default = "default_voice_transcription_model")]
    pub transcription_model: String,

    /// Voice Activity Detection type: `"server_vad"` or `"semantic_vad"`.
    #[serde(default = "default_voice_vad_type")]
    pub vad_type: String,

    /// VAD prefix padding in milliseconds.
    #[serde(default = "default_voice_vad_prefix_padding_ms")]
    pub vad_prefix_padding_ms: u32,

    /// Max recent history items to inject into voice session for context.
    #[serde(default = "default_voice_max_history_items")]
    pub max_history_items: usize,

    /// Barge-in delay in milliseconds (client-side debounce before canceling response).
    #[serde(default = "default_voice_barge_in_delay_ms")]
    pub barge_in_delay_ms: u32,

    /// Audio format for input/output (e.g. `"pcm16"`).
    #[serde(default = "default_voice_audio_format")]
    pub audio_format: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            api_key: None,
            model: default_voice_model(),
            voice: default_voice_name(),
            language: default_voice_language(),
            vad_threshold: default_vad_threshold(),
            silence_duration_ms: default_silence_duration_ms(),
            system_prompt: default_voice_system_prompt(),
            azure_endpoint: None,
            azure_deployment: None,
            save_transcripts: true,
            transcription_model: default_voice_transcription_model(),
            vad_type: default_voice_vad_type(),
            vad_prefix_padding_ms: default_voice_vad_prefix_padding_ms(),
            max_history_items: default_voice_max_history_items(),
            barge_in_delay_ms: default_voice_barge_in_delay_ms(),
            audio_format: default_voice_audio_format(),
        }
    }
}

impl VoiceConfig {
    /// Convert to `RealtimeConfig` for use with the Realtime provider.
    ///
    /// Resolves API key from config or environment variables,
    /// and determines the provider type (Azure vs OpenAI).
    pub fn to_realtime_config(
        &self,
    ) -> anyhow::Result<crate::providers::realtime_types::RealtimeConfig> {
        use crate::providers::realtime_types::{RealtimeApiProvider, RealtimeConfig};

        let api_key = self.api_key.clone()
            .or_else(|| std::env::var("AZURE_OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_REALTIME_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!(
                "Voice API key not configured. Set [voice].api_key, AZURE_OPENAI_API_KEY, or OPENAI_API_KEY"
            ))?;

        if api_key.trim().is_empty() {
            anyhow::bail!("Voice API key is empty. Provide a valid key.");
        }

        if !(0.0..=1.0).contains(&self.vad_threshold) {
            anyhow::bail!(
                "vad_threshold must be between 0.0 and 1.0, got {}",
                self.vad_threshold
            );
        }

        if self.silence_duration_ms == 0 {
            anyhow::bail!("silence_duration_ms must be greater than 0");
        }

        let provider_name = self.provider.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "Voice provider not specified. Set [voice].provider to \"azure\" or \"openai\""
            )
        })?;
        let provider = match provider_name {
            "azure" => {
                let endpoint = self.azure_endpoint.clone().ok_or_else(|| {
                    anyhow::anyhow!("Azure voice provider requires [voice].azure_endpoint")
                })?;
                let deployment = self
                    .azure_deployment
                    .clone()
                    .unwrap_or_else(|| self.model.clone());
                RealtimeApiProvider::Azure {
                    endpoint,
                    deployment,
                }
            }
            "openai" => RealtimeApiProvider::OpenAI,
            other => anyhow::bail!("Unknown voice provider: {other}. Use \"azure\" or \"openai\""),
        };

        Ok(RealtimeConfig {
            api_key,
            model: self.model.clone(),
            voice: self.voice.clone(),
            language: self.language.clone(),
            audio_format: self.audio_format.clone(),
            vad_type: self.vad_type.clone(),
            vad_threshold: self.vad_threshold,
            vad_prefix_padding_ms: self.vad_prefix_padding_ms,
            silence_duration_ms: self.silence_duration_ms,
            transcription_model: self.transcription_model.clone(),
            system_prompt: self.system_prompt.clone(),
            provider,
            save_transcripts: self.save_transcripts,
        })
    }

    /// Apply environment variable overrides to voice configuration
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_ENABLED") {
            self.enabled = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_PROVIDER") {
            let val = val.trim();
            if !val.is_empty() {
                self.provider = Some(val.to_string());
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_API_KEY")
            .or_else(|_| std::env::var("AZURE_OPENAI_API_KEY"))
        {
            let val = val.trim();
            if !val.is_empty() {
                self.api_key = Some(val.to_string());
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_MODEL") {
            let val = val.trim();
            if !val.is_empty() {
                self.model = val.to_string();
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_NAME") {
            let val = val.trim();
            if !val.is_empty() {
                self.voice = val.to_string();
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_LANGUAGE") {
            let val = val.trim();
            if !val.is_empty() {
                self.language = val.to_string();
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_VAD_THRESHOLD") {
            if let Ok(v) = val.parse::<f32>() {
                if (0.0..=1.0).contains(&v) {
                    self.vad_threshold = v;
                }
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_SILENCE_MS") {
            if let Ok(v) = val.parse::<u32>() {
                self.silence_duration_ms = v;
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_SYSTEM_PROMPT") {
            let val = val.trim();
            if !val.is_empty() {
                self.system_prompt = val.to_string();
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_AZURE_ENDPOINT")
            .or_else(|_| std::env::var("AZURE_OPENAI_ENDPOINT"))
        {
            let val = val.trim();
            if !val.is_empty() {
                self.azure_endpoint = Some(val.to_string());
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_AZURE_DEPLOYMENT")
            .or_else(|_| std::env::var("AZURE_OPENAI_DEPLOYMENT"))
        {
            let val = val.trim();
            if !val.is_empty() {
                self.azure_deployment = Some(val.to_string());
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_SAVE_TRANSCRIPTS") {
            match val.trim().to_ascii_lowercase().as_str() {
                "0" | "false" | "no" | "off" => self.save_transcripts = false,
                _ => self.save_transcripts = true,
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_TRANSCRIPTION_MODEL") {
            let val = val.trim();
            if !val.is_empty() {
                self.transcription_model = val.to_string();
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_VAD_TYPE") {
            let val = val.trim();
            if !val.is_empty() {
                self.vad_type = val.to_string();
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_VAD_PREFIX_PADDING_MS") {
            if let Ok(v) = val.trim().parse::<u32>() {
                self.vad_prefix_padding_ms = v;
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_MAX_HISTORY_ITEMS") {
            if let Ok(v) = val.trim().parse::<usize>() {
                self.max_history_items = v;
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_BARGE_IN_DELAY_MS") {
            if let Ok(v) = val.trim().parse::<u32>() {
                self.barge_in_delay_ms = v;
            }
        }
        if let Ok(val) = std::env::var("ZEROCLAW_VOICE_AUDIO_FORMAT") {
            let val = val.trim();
            if !val.is_empty() {
                self.audio_format = val.to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_config_to_realtime_config_openai() {
        let config = VoiceConfig {
            api_key: Some("sk-test".into()),
            provider: Some("openai".into()),
            model: "gpt-realtime".into(),
            voice: "echo".into(),
            language: "ko".into(),
            ..VoiceConfig::default()
        };
        let rt = config.to_realtime_config().unwrap();
        assert_eq!(rt.api_key, "sk-test");
        assert_eq!(rt.model, "gpt-realtime");
        assert_eq!(rt.voice, "echo");
        assert_eq!(rt.language, "ko");
        assert!(matches!(
            rt.provider,
            crate::providers::realtime_types::RealtimeApiProvider::OpenAI
        ));
    }

    #[test]
    fn voice_config_to_realtime_config_azure() {
        let config = VoiceConfig {
            api_key: Some("az-key".into()),
            provider: Some("azure".into()),
            azure_endpoint: Some("10.0.0.1".into()),
            azure_deployment: Some("my-deploy".into()),
            ..VoiceConfig::default()
        };
        let rt = config.to_realtime_config().unwrap();
        assert_eq!(rt.api_key, "az-key");
        match &rt.provider {
            crate::providers::realtime_types::RealtimeApiProvider::Azure {
                endpoint,
                deployment,
            } => {
                assert_eq!(endpoint, "10.0.0.1");
                assert_eq!(deployment, "my-deploy");
            }
            _ => panic!("Expected Azure provider"),
        }
    }

    #[test]
    fn voice_config_azure_deployment_defaults_to_model() {
        let config = VoiceConfig {
            api_key: Some("key".into()),
            provider: Some("azure".into()),
            azure_endpoint: Some("host".into()),
            model: "gpt-realtime-1.5".into(),
            // azure_deployment not set
            ..VoiceConfig::default()
        };
        let rt = config.to_realtime_config().unwrap();
        match &rt.provider {
            crate::providers::realtime_types::RealtimeApiProvider::Azure { deployment, .. } => {
                assert_eq!(deployment, "gpt-realtime-1.5");
            }
            _ => panic!("Expected Azure provider"),
        }
    }

    #[test]
    fn voice_config_missing_api_key_errors() {
        // Clear env vars that might interfere
        std::env::remove_var("AZURE_OPENAI_API_KEY");
        std::env::remove_var("OPENAI_REALTIME_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");

        let config = VoiceConfig {
            provider: Some("openai".into()),
            ..VoiceConfig::default()
        };
        assert!(config.to_realtime_config().is_err());
    }

    #[test]
    fn voice_config_azure_missing_endpoint_errors() {
        let config = VoiceConfig {
            api_key: Some("key".into()),
            provider: Some("azure".into()),
            // azure_endpoint not set
            ..VoiceConfig::default()
        };
        assert!(config.to_realtime_config().is_err());
    }

    #[test]
    fn voice_config_unknown_provider_errors() {
        let config = VoiceConfig {
            api_key: Some("key".into()),
            provider: Some("unknown".into()),
            ..VoiceConfig::default()
        };
        assert!(config.to_realtime_config().is_err());
    }
}
