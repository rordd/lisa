//! Sentinel tool for general-conversation responses under forced tool-use mode.
//!
//! When `tool_choice_required = true` is set globally, the agent loop forces
//! `tool_choice: "required"` on the first turn so the LLM cannot skip tool calls.
//! This tool is the last-resort escape hatch: the LLM should call it ONLY for
//! greetings or casual conversation that has no relation to any registered skill.
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
        description: "Last-resort tool for plain text replies. ONLY use this for greetings \
            or casual conversation that has NO relation to any registered skill or available tool. \
            If the user's request could be handled by ANY other available tool, you MUST call \
            that tool instead of this one. Never use this tool to avoid calling a skill tool."
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
