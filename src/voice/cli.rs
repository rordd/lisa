use crate::Config;
use anyhow::Result;
use tracing::{info, warn};

/// Run the voice command with the provided parameters
pub async fn run_voice_command(
    port: u16,
    host: String,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    config: Config,
) -> Result<()> {
    info!("🎤 Starting Voice Server on {host}:{port}");
    // Create Agent for voice-standalone mode so web chat has full pipeline
    let shared_agent = match crate::agent::Agent::from_config(&config) {
        Ok(agent) => {
            info!("Agent created for voice standalone mode");
            Some(std::sync::Arc::new(tokio::sync::Mutex::new(agent)))
        }
        Err(e) => {
            warn!("Failed to create Agent, web chat will use API proxy: {}", e);
            None
        }
    };
    Box::pin(crate::voice::run_voice_web_with_agent(
        &host,
        port,
        config,
        tls_cert.as_deref(),
        tls_key.as_deref(),
        Vec::new(),
        shared_agent,
    ))
    .await
}
