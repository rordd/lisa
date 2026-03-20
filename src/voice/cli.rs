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
    let (shared_agent, tool_specs) = match crate::agent::Agent::from_config(&config) {
        Ok(mut agent) => {
            // Register skill tools so they are available for voice function calling
            let security = std::sync::Arc::new(crate::security::SecurityPolicy::from_config(
                &config.autonomy,
                &config.workspace_dir,
            ));
            let skill_tools = crate::skills::create_skill_tools(agent.skills(), security);
            if !skill_tools.is_empty() {
                info!("{} skill tool(s) registered", skill_tools.len());
                agent.add_tools(skill_tools);
            }

            let specs = agent.tool_specs().to_vec();
            info!(
                "Agent created for voice standalone mode ({} tools)",
                specs.len()
            );
            (
                Some(std::sync::Arc::new(tokio::sync::Mutex::new(agent))),
                specs,
            )
        }
        Err(e) => {
            warn!("Failed to create Agent, web chat will use API proxy: {}", e);
            (None, Vec::new())
        }
    };
    Box::pin(crate::voice::run_voice_web_with_agent(
        &host,
        port,
        config,
        tls_cert.as_deref(),
        tls_key.as_deref(),
        tool_specs,
        shared_agent,
    ))
    .await
}
