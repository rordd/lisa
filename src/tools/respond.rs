//! Sentinel tool for general-conversation responses under forced tool-use mode.
//!
//! When `tool_choice_required = true` is set in a skill, the agent loop forces
//! `tool_choice: "required"` on the first turn so the LLM cannot skip tool calls
//! for skill-relevant requests. This tool provides an escape hatch so the LLM can
//! still handle general conversation (e.g. greetings, off-topic questions) without
//! invoking any skill action.
//!
//! The LLM calls `respond(text="...")` instead of a skill tool when no action is
//! needed. The agent loop detects this call and returns the text directly without
//! executing any shell command or side effect.

use crate::tools::traits::ToolSpec;

pub const RESPOND_TOOL_NAME: &str = "respond";

/// Returns the [`ToolSpec`] for the respond sentinel tool.
pub fn respond_tool_spec() -> ToolSpec {
    ToolSpec {
        name: RESPOND_TOOL_NAME.to_string(),
        description: "Use this tool to send a plain text reply when no device action is needed. \
            Call this instead of other tools when the user's request is general conversation, \
            a greeting, or does not require any hardware or system action."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The response text to send to the user."
                }
            },
            "required": ["text"]
        }),
    }
}

/// Extracts the response text from a `respond` tool call's arguments.
pub fn extract_respond_text(args: &serde_json::Value) -> Option<String> {
    args.get("text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
