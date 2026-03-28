use crate::providers::traits::{
    ChatMessage, ChatRequest as ProviderChatRequest, ChatResponse as ProviderChatResponse,
    Provider, ProviderCapabilities, TokenUsage, ToolCall as ProviderToolCall,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use base64::Engine as _;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    credential: Option<String>,
    base_url: String,
    thinking_mode: Option<String>, // "adaptive", "enabled", "disabled", or None (default=off)
    thinking_budget: Option<u32>,  // for type="enabled" only
    effort: Option<String>,        // "low", "medium", "high", "max"
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
    temperature: f64,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct NativeChatRequest<'a> {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<SystemPrompt>,
    messages: Vec<NativeMessage>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<NativeToolDef<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    thinking_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    budget_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct OutputConfig {
    effort: String,
}

#[derive(Debug, Serialize)]
struct NativeMessage {
    role: String,
    content: Vec<NativeContentOut>,
}

#[derive(Debug, Serialize)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

/// Content block allowed inside a tool_result (text or image only).
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ToolResultBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

/// `content` field of a tool_result block: plain string or array of blocks.
/// The array form is required when the result contains images.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ToolResultContent {
    Text(String),
    Blocks(Vec<ToolResultBlock>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum NativeContentOut {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

#[derive(Debug, Serialize)]
struct NativeToolSpec<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
}

/// Anthropic Computer Use tool spec (computer_20251124)
#[derive(Debug, Serialize)]
struct ComputerUseToolSpec {
    #[serde(rename = "type")]
    tool_type: String,
    name: String,
    display_width_px: u32,
    display_height_px: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
}

/// Tool definition: regular or Computer Use
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum NativeToolDef<'a> {
    Regular(NativeToolSpec<'a>),
    ComputerUse(ComputerUseToolSpec),
}

#[derive(Debug, Clone, Serialize)]
struct CacheControl {
    #[serde(rename = "type")]
    cache_type: String,
}

impl CacheControl {
    fn ephemeral() -> Self {
        Self {
            cache_type: "ephemeral".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum SystemPrompt {
    String(String),
    Blocks(Vec<SystemBlock>),
}

#[derive(Debug, Serialize)]
struct SystemBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize)]
struct NativeChatResponse {
    #[serde(default)]
    content: Vec<NativeContentIn>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
    #[serde(default)]
    cache_creation_input_tokens: Option<u64>,
    #[serde(default)]
    cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct NativeContentIn {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

impl AnthropicProvider {
    pub fn new(credential: Option<&str>) -> Self {
        Self::with_base_url(credential, None)
    }

    pub fn with_base_url(credential: Option<&str>, base_url: Option<&str>) -> Self {
        let base_url = base_url
            .map(|u| u.trim_end_matches('/'))
            .unwrap_or("https://api.anthropic.com")
            .to_string();

        // Read thinking config from env:
        //   ANTHROPIC_THINKING_MODE = adaptive | enabled | disabled
        //   ANTHROPIC_THINKING_BUDGET = 10000  (for enabled mode)
        //   ANTHROPIC_EFFORT = low | medium | high | max
        let thinking_mode = std::env::var("ANTHROPIC_THINKING_MODE")
            .ok()
            .filter(|s| !s.is_empty());
        let thinking_budget = std::env::var("ANTHROPIC_THINKING_BUDGET")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        let effort = std::env::var("ANTHROPIC_EFFORT")
            .ok()
            .filter(|s| !s.is_empty());

        Self {
            credential: credential
                .map(str::trim)
                .filter(|k| !k.is_empty())
                .map(ToString::to_string),
            base_url,
            thinking_mode,
            thinking_budget,
            effort,
        }
    }

    fn is_setup_token(token: &str) -> bool {
        token.starts_with("sk-ant-oat01-")
    }

    /// Claude Code CLI version to impersonate for setup-token auth.
    const CLAUDE_CODE_VERSION: &'static str = "2.1.62";

    fn apply_auth(
        &self,
        request: reqwest::RequestBuilder,
        credential: &str,
        has_computer_tool: bool,
    ) -> reqwest::RequestBuilder {
        if Self::is_setup_token(credential) {
            let mut beta = "claude-code-20250219,oauth-2025-04-20,fine-grained-tool-streaming-2025-05-14,token-efficient-tools-2025-02-19".to_string();
            if has_computer_tool {
                beta.push_str(",computer-use-2025-11-24");
            }
            request
                .header("Authorization", format!("Bearer {credential}"))
                .header("anthropic-beta", beta)
                .header(
                    "user-agent",
                    format!("claude-cli/{}", Self::CLAUDE_CODE_VERSION),
                )
                .header("x-app", "cli")
        } else {
            let mut beta = "token-efficient-tools-2025-02-19".to_string();
            if has_computer_tool {
                beta.push_str(",computer-use-2025-11-24");
            }
            request
                .header("x-api-key", credential)
                .header("anthropic-beta", beta)
        }
    }

    /// Cache system prompts larger than ~1024 tokens (3KB of text)
    fn should_cache_system(text: &str) -> bool {
        text.len() > 3072
    }

    /// Cache conversations with more than 4 messages (excluding system)
    fn should_cache_conversation(messages: &[ChatMessage]) -> bool {
        messages.iter().filter(|m| m.role != "system").count() > 4
    }

    /// Apply cache control to the last message content block
    fn apply_cache_to_last_message(messages: &mut [NativeMessage]) {
        if let Some(last_msg) = messages.last_mut() {
            if let Some(last_content) = last_msg.content.last_mut() {
                match last_content {
                    NativeContentOut::Text { cache_control, .. }
                    | NativeContentOut::ToolResult { cache_control, .. } => {
                        *cache_control = Some(CacheControl::ephemeral());
                    }
                    NativeContentOut::ToolUse { .. } | NativeContentOut::Image { .. } => {}
                }
            }
        }
    }

    fn convert_tools<'a>(tools: Option<&'a [ToolSpec]>) -> (Option<Vec<NativeToolDef<'a>>>, bool) {
        let items = match tools {
            Some(t) if !t.is_empty() => t,
            _ => return (None, false),
        };
        let mut has_computer_tool = false;
        let mut native_tools: Vec<NativeToolDef<'a>> = items
            .iter()
            .map(|tool| {
                if tool.name == "computer" {
                    has_computer_tool = true;
                    let w = tool.parameters.get("__display_width_px")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(1024) as u32;
                    let h = tool.parameters.get("__display_height_px")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(768) as u32;
                    NativeToolDef::ComputerUse(ComputerUseToolSpec {
                        tool_type: "computer_20251124".into(),
                        name: "computer".into(),
                        display_width_px: w,
                        display_height_px: h,
                        cache_control: None,
                    })
                } else {
                    NativeToolDef::Regular(NativeToolSpec {
                        name: &tool.name,
                        description: &tool.description,
                        input_schema: &tool.parameters,
                        cache_control: None,
                    })
                }
            })
            .collect();

        // Cache the last tool definition (caches all tools)
        if let Some(last_tool) = native_tools.last_mut() {
            match last_tool {
                NativeToolDef::Regular(ref mut t) => {
                    t.cache_control = Some(CacheControl::ephemeral());
                }
                NativeToolDef::ComputerUse(ref mut t) => {
                    t.cache_control = Some(CacheControl::ephemeral());
                }
            }
        }

        (Some(native_tools), has_computer_tool)
    }

    fn parse_assistant_tool_call_message(content: &str) -> Option<Vec<NativeContentOut>> {
        let value = serde_json::from_str::<serde_json::Value>(content).ok()?;
        let tool_calls = value
            .get("tool_calls")
            .and_then(|v| serde_json::from_value::<Vec<ProviderToolCall>>(v.clone()).ok())?;

        let mut blocks = Vec::new();
        if let Some(text) = value
            .get("content")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            blocks.push(NativeContentOut::Text {
                text: text.to_string(),
                cache_control: None,
            });
        }
        for call in tool_calls {
            let input = serde_json::from_str::<serde_json::Value>(&call.arguments)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
            blocks.push(NativeContentOut::ToolUse {
                id: call.id,
                name: call.name,
                input,
                cache_control: None,
            });
        }
        Some(blocks)
    }

    fn parse_tool_result_message(content: &str) -> Option<NativeMessage> {
        let value = serde_json::from_str::<serde_json::Value>(content).ok()?;
        let tool_use_id = value
            .get("tool_call_id")
            .and_then(serde_json::Value::as_str)?
            .to_string();
        let result = value
            .get("content")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let content = Self::parse_tool_result_content(&result);
        Some(NativeMessage {
            role: "user".to_string(),
            content: vec![NativeContentOut::ToolResult {
                tool_use_id,
                content,
                cache_control: None,
            }],
        })
    }

    /// Split a tool result string into a `ToolResultContent`.
    ///
    /// Lines that start with `data:image/` are extracted as `Image` blocks;
    /// the remaining text (if any) becomes a `Text` block.  When no image
    /// lines are present the original string is returned as-is via the plain
    /// `Text` variant so that the serialised payload is unchanged.
    fn parse_tool_result_content(result: &str) -> ToolResultContent {
        if !result.lines().any(|l| l.trim_start().starts_with("data:image/")) {
            return ToolResultContent::Text(result.to_string());
        }

        let mut blocks: Vec<ToolResultBlock> = Vec::new();
        let mut text_lines: Vec<&str> = Vec::new();

        for line in result.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("data:image/") {
                // Flush accumulated text before this image.
                if !text_lines.is_empty() {
                    let text = text_lines.join("\n");
                    if !text.trim().is_empty() {
                        blocks.push(ToolResultBlock::Text { text });
                    }
                    text_lines.clear();
                }
                // Parse the data URI.
                if let Some(comma) = trimmed.find(',') {
                    let header = &trimmed[5..comma]; // strip "data:"
                    let mime = header.split(';').next().unwrap_or("image/jpeg").to_string();
                    let data = trimmed[comma + 1..].trim().to_string();
                    blocks.push(ToolResultBlock::Image {
                        source: ImageSource {
                            source_type: "base64".to_string(),
                            media_type: mime,
                            data,
                        },
                    });
                }
            } else {
                text_lines.push(line);
            }
        }

        // Flush any trailing text.
        if !text_lines.is_empty() {
            let text = text_lines.join("\n");
            if !text.trim().is_empty() {
                blocks.push(ToolResultBlock::Text { text });
            }
        }

        if blocks.is_empty() {
            ToolResultContent::Text(result.to_string())
        } else {
            ToolResultContent::Blocks(blocks)
        }
    }

    /// Claude Code identity prefix required for setup-token auth.
    const CLAUDE_CODE_IDENTITY: &'static str =
        "You are Claude Code, Anthropic's official CLI for Claude.";

    fn convert_messages(
        messages: &[ChatMessage],
        is_setup_token: bool,
    ) -> (Option<SystemPrompt>, Vec<NativeMessage>) {
        let mut system_text = None;
        let mut native_messages = Vec::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    if system_text.is_none() {
                        system_text = Some(msg.content.clone());
                    }
                }
                "assistant" => {
                    if let Some(blocks) = Self::parse_assistant_tool_call_message(&msg.content) {
                        native_messages.push(NativeMessage {
                            role: "assistant".to_string(),
                            content: blocks,
                        });
                    } else if !msg.content.trim().is_empty() {
                        native_messages.push(NativeMessage {
                            role: "assistant".to_string(),
                            content: vec![NativeContentOut::Text {
                                text: msg.content.clone(),
                                cache_control: None,
                            }],
                        });
                    }
                }
                "tool" => {
                    let tool_msg = if let Some(tr) = Self::parse_tool_result_message(&msg.content) {
                        tr
                    } else if !msg.content.trim().is_empty() {
                        NativeMessage {
                            role: "user".to_string(),
                            content: vec![NativeContentOut::Text {
                                text: msg.content.clone(),
                                cache_control: None,
                            }],
                        }
                    } else {
                        continue;
                    };
                    // Tool results map to role "user"; merge consecutive ones
                    // into a single message so Anthropic doesn't reject the
                    // request for having adjacent same-role messages.
                    if native_messages
                        .last()
                        .is_some_and(|m| m.role == tool_msg.role)
                    {
                        native_messages
                            .last_mut()
                            .unwrap()
                            .content
                            .extend(tool_msg.content);
                    } else {
                        native_messages.push(tool_msg);
                    }
                }
                _ => {
                    // Parse image markers from user message content
                    let (text, image_refs) = crate::multimodal::parse_image_markers(&msg.content);
                    let mut content_blocks: Vec<NativeContentOut> = Vec::new();

                    // Add image content blocks for each image reference
                    for img_ref in &image_refs {
                        let (media_type, data) = if img_ref.starts_with("data:") {
                            // Data URI format: data:image/jpeg;base64,/9j/4AAQ...
                            if let Some(comma) = img_ref.find(',') {
                                let header = &img_ref[5..comma];
                                let mime =
                                    header.split(';').next().unwrap_or("image/jpeg").to_string();
                                let b64 = img_ref[comma + 1..].trim().to_string();
                                (mime, b64)
                            } else {
                                continue;
                            }
                        } else if std::path::Path::new(img_ref.trim()).exists() {
                            // Local file path
                            match std::fs::read(img_ref.trim()) {
                                Ok(bytes) => {
                                    let b64 =
                                        base64::engine::general_purpose::STANDARD.encode(&bytes);
                                    let ext = std::path::Path::new(img_ref.trim())
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or("jpg");
                                    let mime = match ext {
                                        "png" => "image/png",
                                        "gif" => "image/gif",
                                        "webp" => "image/webp",
                                        _ => "image/jpeg",
                                    }
                                    .to_string();
                                    (mime, b64)
                                }
                                Err(_) => continue,
                            }
                        } else {
                            continue;
                        };

                        content_blocks.push(NativeContentOut::Image {
                            source: ImageSource {
                                source_type: "base64".to_string(),
                                media_type,
                                data,
                            },
                        });
                    }

                    // Add text content block (skip empty text when images are present)
                    if text.is_empty() && !image_refs.is_empty() {
                        content_blocks.push(NativeContentOut::Text {
                            text: "[image]".to_string(),
                            cache_control: None,
                        });
                    } else if !text.trim().is_empty() {
                        content_blocks.push(NativeContentOut::Text {
                            text,
                            cache_control: None,
                        });
                    }

                    // Merge into previous user message if present (e.g.
                    // when a user message immediately follows tool results
                    // which are also role "user" in Anthropic's format).
                    if native_messages.last().is_some_and(|m| m.role == "user") {
                        native_messages
                            .last_mut()
                            .unwrap()
                            .content
                            .extend(content_blocks);
                    } else {
                        native_messages.push(NativeMessage {
                            role: "user".to_string(),
                            content: content_blocks,
                        });
                    }
                }
            }
        }

        // Convert system text to SystemPrompt with cache control if large.
        // For setup-token auth, prepend Claude Code identity block (required
        // by Anthropic to access premium models via OAuth setup tokens).
        let system_prompt = if is_setup_token {
            let mut blocks = vec![SystemBlock {
                block_type: "text".to_string(),
                text: Self::CLAUDE_CODE_IDENTITY.to_string(),
                cache_control: None,
            }];
            if let Some(text) = system_text {
                blocks.push(SystemBlock {
                    block_type: "text".to_string(),
                    text,
                    cache_control: if Self::should_cache_system(
                        &blocks.iter().map(|b| b.text.as_str()).collect::<String>(),
                    ) {
                        Some(CacheControl::ephemeral())
                    } else {
                        None
                    },
                });
            }
            Some(SystemPrompt::Blocks(blocks))
        } else {
            system_text.map(|text| {
                if Self::should_cache_system(&text) {
                    SystemPrompt::Blocks(vec![SystemBlock {
                        block_type: "text".to_string(),
                        text,
                        cache_control: Some(CacheControl::ephemeral()),
                    }])
                } else {
                    SystemPrompt::String(text)
                }
            })
        };

        (system_prompt, native_messages)
    }

    fn parse_text_response(response: ChatResponse) -> anyhow::Result<String> {
        response
            .content
            .into_iter()
            .find(|c| c.kind == "text")
            .and_then(|c| c.text)
            .ok_or_else(|| anyhow::anyhow!("No response from Anthropic"))
    }

    fn parse_native_response(response: NativeChatResponse) -> ProviderChatResponse {
        let mut text_parts = Vec::new();
        let mut reasoning_parts = Vec::new();
        let mut tool_calls = Vec::new();

        let usage = response.usage.map(|u| {
            tracing::info!(
                input_tokens = ?u.input_tokens,
                output_tokens = ?u.output_tokens,
                cache_creation = ?u.cache_creation_input_tokens,
                cache_read = ?u.cache_read_input_tokens,
                "Anthropic usage"
            );
            TokenUsage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
                cached_input_tokens: u.cache_read_input_tokens,
            }
        });

        for block in response.content {
            match block.kind.as_str() {
                "text" => {
                    if let Some(text) = block.text.map(|t| t.trim().to_string()) {
                        if !text.is_empty() {
                            text_parts.push(text);
                        }
                    }
                }
                "tool_use" => {
                    let name = block.name.unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    let arguments = block
                        .input
                        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
                    tool_calls.push(ProviderToolCall {
                        id: block.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                        name,
                        arguments: arguments.to_string(),
                    });
                }
                "thinking" => {
                    if let Some(thinking_text) = block.text {
                        tracing::debug!("Thinking block: {} chars", thinking_text.len());
                        reasoning_parts.push(thinking_text);
                    }
                }
                _ => {}
            }
        }

        ProviderChatResponse {
            text: if text_parts.is_empty() {
                None
            } else {
                Some(text_parts.join("\n"))
            },
            tool_calls,
            usage,
            reasoning_content: if reasoning_parts.is_empty() {
                None
            } else {
                Some(reasoning_parts.join("\n"))
            },
        }
    }

    fn http_client(&self) -> Client {
        crate::config::build_runtime_proxy_client_with_timeouts("provider.anthropic", 120, 10)
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        let credential = self.credential.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Anthropic credentials not set. Set ANTHROPIC_API_KEY or ANTHROPIC_OAUTH_TOKEN (setup-token)."
            )
        })?;

        let is_setup = Self::is_setup_token(credential);

        // setup-token auth: system must be a blocks array (plain string causes 400)
        // Regular API key: plain string is fine
        let system_value: Option<serde_json::Value> = if is_setup {
            let identity = Self::CLAUDE_CODE_IDENTITY;
            let mut blocks = vec![serde_json::json!({
                "type": "text",
                "text": identity
            })];
            if let Some(sp) = system_prompt {
                if !sp.trim().is_empty() {
                    blocks.push(serde_json::json!({
                        "type": "text",
                        "text": sp
                    }));
                }
            }
            Some(serde_json::Value::Array(blocks))
        } else {
            system_prompt.map(|s| serde_json::Value::String(s.to_string()))
        };

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": message}],
            "temperature": temperature,
        });
        if let Some(sys) = system_value {
            body["system"] = sys;
        }

        let mut request = self
            .http_client()
            .post(format!("{}/v1/messages", self.base_url))
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body);

        request = self.apply_auth(request, credential, false);

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(super::api_error("Anthropic", response).await);
        }

        let chat_response: ChatResponse = response.json().await?;
        Self::parse_text_response(chat_response)
    }

    async fn chat(
        &self,
        request: ProviderChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ProviderChatResponse> {
        let credential = self.credential.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Anthropic credentials not set. Set ANTHROPIC_API_KEY or ANTHROPIC_OAUTH_TOKEN (setup-token)."
            )
        })?;

        let is_setup = Self::is_setup_token(credential);
        let (system_prompt, mut messages) = Self::convert_messages(request.messages, is_setup);

        // Auto-cache last message if conversation is long
        if Self::should_cache_conversation(request.messages) {
            Self::apply_cache_to_last_message(&mut messages);
        }

        let thinking = self.thinking_mode.as_deref().and_then(|mode| match mode {
            "adaptive" | "enabled" => {
                let budget = self.thinking_budget.unwrap_or(10000);
                Some(ThinkingConfig {
                    thinking_type: mode.to_string(),
                    budget_tokens: Some(budget),
                })
            }
            "disabled" => None, // default behavior, no need to send
            other => {
                tracing::warn!("Unknown ANTHROPIC_THINKING_MODE '{}', ignoring", other);
                None
            }
        });
        let output_config = self.effort.as_deref().and_then(|e| match e {
            "low" | "medium" | "high" | "max" => Some(OutputConfig {
                effort: e.to_string(),
            }),
            other => {
                tracing::warn!("Unknown ANTHROPIC_EFFORT '{}', ignoring", other);
                None
            }
        });

        // Thinking requires temperature=1 and max_tokens > budget_tokens
        let (temp, max_tok) = if let Some(ref t) = thinking {
            let budget = t.budget_tokens.unwrap_or(10000);
            (1.0, std::cmp::max(16384, budget + 1024))
        } else {
            (temperature, 4096)
        };

        let (converted_tools, has_computer_tool) = Self::convert_tools(request.tools);
        let native_request = NativeChatRequest {
            model: model.to_string(),
            max_tokens: max_tok,
            system: system_prompt,
            messages,
            temperature: temp,
            tool_choice: if request.tool_choice == Some("required") && converted_tools.is_some() {
                Some(serde_json::json!({"type": "any"}))
            } else {
                None
            },
            tools: converted_tools,
            thinking,
            output_config,
        };

        let req = self
            .http_client()
            .post(format!("{}/v1/messages", self.base_url))
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&native_request);

        let response = self.apply_auth(req, credential, has_computer_tool).send().await?;
        if !response.status().is_success() {
            return Err(super::api_error("Anthropic", response).await);
        }

        let native_response: NativeChatResponse = response.json().await?;
        Ok(Self::parse_native_response(native_response))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            native_tool_calling: true,
            vision: true,
            prompt_caching: true,
        }
    }

    fn supports_native_tools(&self) -> bool {
        true
    }

    async fn chat_with_tools(
        &self,
        messages: &[ChatMessage],
        tools: &[serde_json::Value],
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ProviderChatResponse> {
        // Convert OpenAI-format tool JSON to ToolSpec so we can reuse the
        // existing `chat()` method which handles full message history,
        // system prompt extraction, caching, and Anthropic native formatting.
        let tool_specs: Vec<ToolSpec> = tools
            .iter()
            .filter_map(|t| {
                let func = t.get("function").or_else(|| {
                    tracing::warn!("Skipping malformed tool definition (missing 'function' key)");
                    None
                })?;
                let name = func.get("name").and_then(|n| n.as_str()).or_else(|| {
                    tracing::warn!("Skipping tool with missing or non-string 'name'");
                    None
                })?;
                Some(ToolSpec {
                    name: name.to_string(),
                    description: func
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    parameters: func
                        .get("parameters")
                        .cloned()
                        .unwrap_or(serde_json::json!({"type": "object"})),
                })
            })
            .collect();

        let request = ProviderChatRequest {
            messages,
            tools: if tool_specs.is_empty() {
                None
            } else {
                Some(&tool_specs)
            },
            tool_choice: None,
        };
        self.chat(request, model, temperature).await
    }

    async fn warmup(&self) -> anyhow::Result<()> {
        if let Some(credential) = self.credential.as_ref() {
            let mut request = self
                .http_client()
                .post(format!("{}/v1/messages", self.base_url))
                .header("anthropic-version", "2023-06-01");
            request = self.apply_auth(request, credential, false);
            // Send a minimal request; the goal is TLS + HTTP/2 setup, not a valid response.
            // Anthropic has no lightweight GET endpoint, so we accept any non-network error.
            let _ = request.send().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::anthropic_token::{detect_auth_kind, AnthropicAuthKind};

    #[test]
    fn creates_with_key() {
        let p = AnthropicProvider::new(Some("anthropic-test-credential"));
        assert!(p.credential.is_some());
        assert_eq!(p.credential.as_deref(), Some("anthropic-test-credential"));
        assert_eq!(p.base_url, "https://api.anthropic.com");
    }

    #[test]
    fn creates_without_key() {
        let p = AnthropicProvider::new(None);
        assert!(p.credential.is_none());
        assert_eq!(p.base_url, "https://api.anthropic.com");
    }

    #[test]
    fn creates_with_empty_key() {
        let p = AnthropicProvider::new(Some(""));
        assert!(p.credential.is_none());
    }

    #[test]
    fn creates_with_whitespace_key() {
        let p = AnthropicProvider::new(Some("  anthropic-test-credential  "));
        assert!(p.credential.is_some());
        assert_eq!(p.credential.as_deref(), Some("anthropic-test-credential"));
    }

    #[test]
    fn creates_with_custom_base_url() {
        let p = AnthropicProvider::with_base_url(
            Some("anthropic-credential"),
            Some("https://api.example.com"),
        );
        assert_eq!(p.base_url, "https://api.example.com");
        assert_eq!(p.credential.as_deref(), Some("anthropic-credential"));
    }

    #[test]
    fn custom_base_url_trims_trailing_slash() {
        let p = AnthropicProvider::with_base_url(None, Some("https://api.example.com/"));
        assert_eq!(p.base_url, "https://api.example.com");
    }

    #[test]
    fn default_base_url_when_none_provided() {
        let p = AnthropicProvider::with_base_url(None, None);
        assert_eq!(p.base_url, "https://api.anthropic.com");
    }

    #[tokio::test]
    async fn chat_fails_without_key() {
        let p = AnthropicProvider::new(None);
        let result = p
            .chat_with_system(None, "hello", "claude-3-opus", 0.7)
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("credentials not set"),
            "Expected key error, got: {err}"
        );
    }

    #[test]
    fn setup_token_detection_works() {
        assert!(AnthropicProvider::is_setup_token("sk-ant-oat01-abcdef"));
        assert!(!AnthropicProvider::is_setup_token("sk-ant-api-key"));
    }

    #[test]
    fn apply_auth_uses_bearer_and_beta_for_setup_tokens() {
        let provider = AnthropicProvider::new(None);
        let request = provider
            .apply_auth(
                provider
                    .http_client()
                    .get("https://api.anthropic.com/v1/models"),
                "sk-ant-oat01-test-token",
                false,
            )
            .build()
            .expect("request should build");

        assert_eq!(
            request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some("Bearer sk-ant-oat01-test-token")
        );
        assert_eq!(
            request
                .headers()
                .get("anthropic-beta")
                .and_then(|v| v.to_str().ok()),
            Some("claude-code-20250219,oauth-2025-04-20,fine-grained-tool-streaming-2025-05-14")
        );
        assert_eq!(
            request
                .headers()
                .get("user-agent")
                .and_then(|v| v.to_str().ok()),
            Some(format!("claude-cli/{}", AnthropicProvider::CLAUDE_CODE_VERSION).as_str())
        );
        assert_eq!(
            request.headers().get("x-app").and_then(|v| v.to_str().ok()),
            Some("cli")
        );
        assert!(request.headers().get("x-api-key").is_none());
    }

    #[test]
    fn apply_auth_uses_x_api_key_for_regular_tokens() {
        let provider = AnthropicProvider::new(None);
        let request = provider
            .apply_auth(
                provider
                    .http_client()
                    .get("https://api.anthropic.com/v1/models"),
                "sk-ant-api-key",
                false,
            )
            .build()
            .expect("request should build");

        assert_eq!(
            request
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok()),
            Some("sk-ant-api-key")
        );
        assert!(request.headers().get("authorization").is_none());
        assert!(request.headers().get("anthropic-beta").is_none());
    }

    #[tokio::test]
    async fn chat_with_system_fails_without_key() {
        let p = AnthropicProvider::new(None);
        let result = p
            .chat_with_system(Some("You are ZeroClaw"), "hello", "claude-3-opus", 0.7)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn chat_request_serializes_without_system() {
        let req = ChatRequest {
            model: "claude-3-opus".to_string(),
            max_tokens: 4096,
            system: None,
            messages: vec![Message {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            temperature: 0.7,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(
            !json.contains("system"),
            "system field should be skipped when None"
        );
        assert!(json.contains("claude-3-opus"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn chat_request_serializes_with_system() {
        let req = ChatRequest {
            model: "claude-3-opus".to_string(),
            max_tokens: 4096,
            system: Some("You are ZeroClaw".to_string()),
            messages: vec![Message {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            temperature: 0.7,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"system\":\"You are ZeroClaw\""));
    }

    #[test]
    fn chat_response_deserializes() {
        let json = r#"{"content":[{"type":"text","text":"Hello there!"}]}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 1);
        assert_eq!(resp.content[0].kind, "text");
        assert_eq!(resp.content[0].text.as_deref(), Some("Hello there!"));
    }

    #[test]
    fn chat_response_empty_content() {
        let json = r#"{"content":[]}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(resp.content.is_empty());
    }

    #[test]
    fn chat_response_multiple_blocks() {
        let json =
            r#"{"content":[{"type":"text","text":"First"},{"type":"text","text":"Second"}]}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 2);
        assert_eq!(resp.content[0].text.as_deref(), Some("First"));
        assert_eq!(resp.content[1].text.as_deref(), Some("Second"));
    }

    #[test]
    fn temperature_range_serializes() {
        for temp in [0.0, 0.5, 1.0, 2.0] {
            let req = ChatRequest {
                model: "claude-3-opus".to_string(),
                max_tokens: 4096,
                system: None,
                messages: vec![],
                temperature: temp,
            };
            let json = serde_json::to_string(&req).unwrap();
            assert!(json.contains(&format!("{temp}")));
        }
    }

    #[test]
    fn detects_auth_from_jwt_shape() {
        let kind = detect_auth_kind("a.b.c", None);
        assert_eq!(kind, AnthropicAuthKind::Authorization);
    }

    #[test]
    fn cache_control_serializes_correctly() {
        let cache = CacheControl::ephemeral();
        let json = serde_json::to_string(&cache).unwrap();
        assert_eq!(json, r#"{"type":"ephemeral"}"#);
    }

    #[test]
    fn system_prompt_string_variant_serializes() {
        let prompt = SystemPrompt::String("You are a helpful assistant".to_string());
        let json = serde_json::to_string(&prompt).unwrap();
        assert_eq!(json, r#""You are a helpful assistant""#);
    }

    #[test]
    fn system_prompt_blocks_variant_serializes() {
        let prompt = SystemPrompt::Blocks(vec![SystemBlock {
            block_type: "text".to_string(),
            text: "You are a helpful assistant".to_string(),
            cache_control: Some(CacheControl::ephemeral()),
        }]);
        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains("You are a helpful assistant"));
        assert!(json.contains(r#""type":"ephemeral""#));
    }

    #[test]
    fn system_prompt_blocks_without_cache_control() {
        let prompt = SystemPrompt::Blocks(vec![SystemBlock {
            block_type: "text".to_string(),
            text: "Short prompt".to_string(),
            cache_control: None,
        }]);
        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains("Short prompt"));
        assert!(!json.contains("cache_control"));
    }

    #[test]
    fn native_content_text_without_cache_control() {
        let content = NativeContentOut::Text {
            text: "Hello".to_string(),
            cache_control: None,
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains("Hello"));
        assert!(!json.contains("cache_control"));
    }

    #[test]
    fn native_content_text_with_cache_control() {
        let content = NativeContentOut::Text {
            text: "Hello".to_string(),
            cache_control: Some(CacheControl::ephemeral()),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains("Hello"));
        assert!(json.contains(r#""cache_control":{"type":"ephemeral"}"#));
    }

    #[test]
    fn native_content_tool_use_without_cache_control() {
        let content = NativeContentOut::ToolUse {
            id: "tool_123".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::json!({"location": "San Francisco"}),
            cache_control: None,
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"tool_use""#));
        assert!(json.contains("tool_123"));
        assert!(json.contains("get_weather"));
        assert!(!json.contains("cache_control"));
    }

    #[test]
    fn native_content_tool_result_with_cache_control() {
        let content = NativeContentOut::ToolResult {
            tool_use_id: "tool_123".to_string(),
            content: ToolResultContent::Text("Result data".to_string()),
            cache_control: Some(CacheControl::ephemeral()),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"tool_result""#));
        assert!(json.contains("tool_123"));
        assert!(json.contains("Result data"));
        assert!(json.contains(r#""cache_control":{"type":"ephemeral"}"#));
    }

    #[test]
    fn native_tool_spec_without_cache_control() {
        let schema = serde_json::json!({"type": "object"});
        let tool = NativeToolSpec {
            name: "get_weather",
            description: "Get weather info",
            input_schema: &schema,
            cache_control: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("get_weather"));
        assert!(!json.contains("cache_control"));
    }

    #[test]
    fn native_tool_spec_with_cache_control() {
        let schema = serde_json::json!({"type": "object"});
        let tool = NativeToolSpec {
            name: "get_weather",
            description: "Get weather info",
            input_schema: &schema,
            cache_control: Some(CacheControl::ephemeral()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("get_weather"));
        assert!(json.contains(r#""cache_control":{"type":"ephemeral"}"#));
    }

    #[test]
    fn should_cache_system_small_prompt() {
        let small_prompt = "You are a helpful assistant.";
        assert!(!AnthropicProvider::should_cache_system(small_prompt));
    }

    #[test]
    fn should_cache_system_large_prompt() {
        let large_prompt = "a".repeat(3073); // Just over 3072 bytes
        assert!(AnthropicProvider::should_cache_system(&large_prompt));
    }

    #[test]
    fn should_cache_system_boundary() {
        let boundary_prompt = "a".repeat(3072); // Exactly 3072 bytes
        assert!(!AnthropicProvider::should_cache_system(&boundary_prompt));

        let over_boundary = "a".repeat(3073);
        assert!(AnthropicProvider::should_cache_system(&over_boundary));
    }

    #[test]
    fn should_cache_conversation_short() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "System prompt".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi".to_string(),
            },
        ];
        // Only 2 non-system messages
        assert!(!AnthropicProvider::should_cache_conversation(&messages));
    }

    #[test]
    fn should_cache_conversation_long() {
        let mut messages = vec![ChatMessage {
            role: "system".to_string(),
            content: "System prompt".to_string(),
        }];
        // Add 5 non-system messages
        for i in 0..5 {
            messages.push(ChatMessage {
                role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                content: format!("Message {i}"),
            });
        }
        assert!(AnthropicProvider::should_cache_conversation(&messages));
    }

    #[test]
    fn should_cache_conversation_boundary() {
        let mut messages = vec![];
        // Add exactly 4 non-system messages
        for i in 0..4 {
            messages.push(ChatMessage {
                role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                content: format!("Message {i}"),
            });
        }
        assert!(!AnthropicProvider::should_cache_conversation(&messages));

        // Add one more to cross boundary
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: "One more".to_string(),
        });
        assert!(AnthropicProvider::should_cache_conversation(&messages));
    }

    #[test]
    fn apply_cache_to_last_message_text() {
        let mut messages = vec![NativeMessage {
            role: "user".to_string(),
            content: vec![NativeContentOut::Text {
                text: "Hello".to_string(),
                cache_control: None,
            }],
        }];

        AnthropicProvider::apply_cache_to_last_message(&mut messages);

        match &messages[0].content[0] {
            NativeContentOut::Text { cache_control, .. } => {
                assert!(cache_control.is_some());
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn apply_cache_to_last_message_tool_result() {
        let mut messages = vec![NativeMessage {
            role: "user".to_string(),
            content: vec![NativeContentOut::ToolResult {
                tool_use_id: "tool_123".to_string(),
                content: ToolResultContent::Text("Result".to_string()),
                cache_control: None,
            }],
        }];

        AnthropicProvider::apply_cache_to_last_message(&mut messages);

        match &messages[0].content[0] {
            NativeContentOut::ToolResult { cache_control, .. } => {
                assert!(cache_control.is_some());
            }
            _ => panic!("Expected ToolResult variant"),
        }
    }

    #[test]
    fn apply_cache_to_last_message_does_not_affect_tool_use() {
        let mut messages = vec![NativeMessage {
            role: "assistant".to_string(),
            content: vec![NativeContentOut::ToolUse {
                id: "tool_123".to_string(),
                name: "get_weather".to_string(),
                input: serde_json::json!({}),
                cache_control: None,
            }],
        }];

        AnthropicProvider::apply_cache_to_last_message(&mut messages);

        // ToolUse should not be affected
        match &messages[0].content[0] {
            NativeContentOut::ToolUse { cache_control, .. } => {
                assert!(cache_control.is_none());
            }
            _ => panic!("Expected ToolUse variant"),
        }
    }

    #[test]
    fn apply_cache_empty_messages() {
        let mut messages = vec![];
        AnthropicProvider::apply_cache_to_last_message(&mut messages);
        // Should not panic
        assert!(messages.is_empty());
    }

    #[test]
    fn convert_tools_adds_cache_to_last_tool() {
        let tools = vec![
            ToolSpec {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
                parameters: serde_json::json!({"type": "object"}),
            },
            ToolSpec {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
                parameters: serde_json::json!({"type": "object"}),
            },
        ];

        let (native_tools, has_computer) = AnthropicProvider::convert_tools(Some(&tools));
        let native_tools = native_tools.unwrap();
        assert!(!has_computer);

        assert_eq!(native_tools.len(), 2);
        match &native_tools[0] {
            NativeToolDef::Regular(t) => assert!(t.cache_control.is_none()),
            _ => panic!("expected Regular"),
        }
        match &native_tools[1] {
            NativeToolDef::Regular(t) => assert!(t.cache_control.is_some()),
            _ => panic!("expected Regular"),
        }
    }

    #[test]
    fn convert_tools_single_tool_gets_cache() {
        let tools = vec![ToolSpec {
            name: "tool1".to_string(),
            description: "Only tool".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        }];

        let (native_tools, _) = AnthropicProvider::convert_tools(Some(&tools));
        let native_tools = native_tools.unwrap();

        assert_eq!(native_tools.len(), 1);
        match &native_tools[0] {
            NativeToolDef::Regular(t) => assert!(t.cache_control.is_some()),
            _ => panic!("expected Regular"),
        }
    }

    #[test]
    fn convert_tools_computer_tool_uses_special_format() {
        let tools = vec![
            ToolSpec {
                name: "shell".to_string(),
                description: "Run shell".to_string(),
                parameters: serde_json::json!({"type": "object"}),
            },
            ToolSpec {
                name: "computer".to_string(),
                description: "Computer Use".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "__display_width_px": 1024,
                    "__display_height_px": 640
                }),
            },
        ];

        let (native_tools, has_computer) = AnthropicProvider::convert_tools(Some(&tools));
        assert!(has_computer);
        let native_tools = native_tools.unwrap();
        assert_eq!(native_tools.len(), 2);

        match &native_tools[0] {
            NativeToolDef::Regular(t) => assert_eq!(t.name, "shell"),
            _ => panic!("expected Regular"),
        }
        match &native_tools[1] {
            NativeToolDef::ComputerUse(t) => {
                assert_eq!(t.tool_type, "computer_20251124");
                assert_eq!(t.display_width_px, 1024);
                assert_eq!(t.display_height_px, 640);
                assert!(t.cache_control.is_some());
            }
            _ => panic!("expected ComputerUse"),
        }
    }

    #[test]
    fn convert_messages_small_system_prompt() {
        let messages = vec![ChatMessage {
            role: "system".to_string(),
            content: "Short system prompt".to_string(),
        }];

        let (system_prompt, _) = AnthropicProvider::convert_messages(&messages, false);

        match system_prompt.unwrap() {
            SystemPrompt::String(s) => {
                assert_eq!(s, "Short system prompt");
            }
            SystemPrompt::Blocks(_) => panic!("Expected String variant for small prompt"),
        }
    }

    #[test]
    fn convert_messages_large_system_prompt() {
        let large_content = "a".repeat(3073);
        let messages = vec![ChatMessage {
            role: "system".to_string(),
            content: large_content.clone(),
        }];

        let (system_prompt, _) = AnthropicProvider::convert_messages(&messages, false);

        match system_prompt.unwrap() {
            SystemPrompt::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0].text, large_content);
                assert!(blocks[0].cache_control.is_some());
            }
            SystemPrompt::String(_) => panic!("Expected Blocks variant for large prompt"),
        }
    }

    #[test]
    fn backward_compatibility_native_chat_request() {
        // Test that requests without cache_control serialize identically to old format
        let req = NativeChatRequest {
            model: "claude-3-opus".to_string(),
            max_tokens: 4096,
            system: Some(SystemPrompt::String("System".to_string())),
            messages: vec![NativeMessage {
                role: "user".to_string(),
                content: vec![NativeContentOut::Text {
                    text: "Hello".to_string(),
                    cache_control: None,
                }],
            }],
            temperature: 0.7,
            tools: None,
            tool_choice: None,
            thinking: None,
            output_config: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("cache_control"));
        assert!(json.contains(r#""system":"System""#));
    }

    #[tokio::test]
    async fn warmup_without_key_is_noop() {
        let provider = AnthropicProvider::new(None);
        let result = provider.warmup().await;
        assert!(result.is_ok());
    }

    #[test]
    fn convert_messages_preserves_multi_turn_history() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "gen a 2 sum in golang".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "```go\nfunc twoSum(nums []int) {}\n```".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "what's meaning of make here?".to_string(),
            },
        ];

        let (system, native_msgs) = AnthropicProvider::convert_messages(&messages, false);

        // System prompt extracted
        assert!(system.is_some());
        // All 3 non-system messages preserved in order
        assert_eq!(native_msgs.len(), 3);
        assert_eq!(native_msgs[0].role, "user");
        assert_eq!(native_msgs[1].role, "assistant");
        assert_eq!(native_msgs[2].role, "user");
    }

    /// Integration test: spin up a mock Anthropic API server, call chat_with_tools
    /// with a multi-turn conversation + tools, and verify the request body contains
    /// ALL conversation turns and native tool definitions.
    #[tokio::test]
    async fn chat_with_tools_sends_full_history_and_native_tools() {
        use axum::{routing::post, Json, Router};
        use std::sync::{Arc, Mutex};
        use tokio::net::TcpListener;

        // Captured request body for assertion
        let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
        let captured_clone = captured.clone();

        let app = Router::new().route(
            "/v1/messages",
            post(move |Json(body): Json<serde_json::Value>| {
                let cap = captured_clone.clone();
                async move {
                    *cap.lock().unwrap() = Some(body);
                    // Return a minimal valid Anthropic response
                    Json(serde_json::json!({
                        "id": "msg_test",
                        "type": "message",
                        "role": "assistant",
                        "content": [{"type": "text", "text": "The make function creates a map."}],
                        "model": "claude-opus-4-6",
                        "stop_reason": "end_turn",
                        "usage": {"input_tokens": 100, "output_tokens": 20}
                    }))
                }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Create provider pointing at mock server
        let provider = AnthropicProvider {
            credential: Some("test-key".to_string()),
            base_url: format!("http://{addr}"),
            thinking_mode: None,
            thinking_budget: None,
            effort: None,
        };

        // Multi-turn conversation: system → user (Go code) → assistant (code response) → user (follow-up)
        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("gen a 2 sum in golang"),
            ChatMessage::assistant("```go\nfunc twoSum(nums []int, target int) []int {\n    m := make(map[int]int)\n    for i, n := range nums {\n        if j, ok := m[target-n]; ok {\n            return []int{j, i}\n        }\n        m[n] = i\n    }\n    return nil\n}\n```"),
            ChatMessage::user("what's meaning of make here?"),
        ];

        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "shell",
                "description": "Run a shell command",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    },
                    "required": ["command"]
                }
            }
        })];

        let result = provider
            .chat_with_tools(&messages, &tools, "claude-opus-4-6", 0.7)
            .await;
        assert!(result.is_ok(), "chat_with_tools failed: {:?}", result.err());

        let body = captured
            .lock()
            .unwrap()
            .take()
            .expect("No request captured");

        // Verify system prompt extracted to top-level field
        let system = &body["system"];
        assert!(
            system.to_string().contains("helpful assistant"),
            "System prompt missing: {system}"
        );

        // Verify ALL conversation turns present in messages array
        let msgs = body["messages"].as_array().expect("messages not an array");
        assert_eq!(
            msgs.len(),
            3,
            "Expected 3 messages (2 user + 1 assistant), got {}",
            msgs.len()
        );

        // Turn 1: user with Go request
        assert_eq!(msgs[0]["role"], "user");
        let turn1_text = msgs[0]["content"].to_string();
        assert!(
            turn1_text.contains("2 sum"),
            "Turn 1 missing Go request: {turn1_text}"
        );

        // Turn 2: assistant with Go code
        assert_eq!(msgs[1]["role"], "assistant");
        let turn2_text = msgs[1]["content"].to_string();
        assert!(
            turn2_text.contains("make(map[int]int)"),
            "Turn 2 missing Go code: {turn2_text}"
        );

        // Turn 3: user follow-up
        assert_eq!(msgs[2]["role"], "user");
        let turn3_text = msgs[2]["content"].to_string();
        assert!(
            turn3_text.contains("meaning of make"),
            "Turn 3 missing follow-up: {turn3_text}"
        );

        // Verify native tools are present
        let api_tools = body["tools"].as_array().expect("tools not an array");
        assert_eq!(api_tools.len(), 1);
        assert_eq!(api_tools[0]["name"], "shell");
        assert!(
            api_tools[0]["input_schema"].is_object(),
            "Missing input_schema"
        );

        server_handle.abort();
    }

    #[test]
    fn native_response_parses_usage() {
        let json = r#"{
            "content": [{"type": "text", "text": "Hello"}],
            "usage": {"input_tokens": 300, "output_tokens": 75}
        }"#;
        let resp: NativeChatResponse = serde_json::from_str(json).unwrap();
        let result = AnthropicProvider::parse_native_response(resp);
        let usage = result.usage.unwrap();
        assert_eq!(usage.input_tokens, Some(300));
        assert_eq!(usage.output_tokens, Some(75));
    }

    #[test]
    fn native_response_parses_without_usage() {
        let json = r#"{"content": [{"type": "text", "text": "Hello"}]}"#;
        let resp: NativeChatResponse = serde_json::from_str(json).unwrap();
        let result = AnthropicProvider::parse_native_response(resp);
        assert!(result.usage.is_none());
    }

    #[test]
    fn capabilities_returns_vision_and_native_tools() {
        let provider = AnthropicProvider::new(Some("test-key"));
        let caps = provider.capabilities();
        assert!(
            caps.native_tool_calling,
            "Anthropic should support native tool calling"
        );
        assert!(caps.vision, "Anthropic should support vision");
    }

    #[test]
    fn convert_messages_with_image_marker_data_uri() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Check this image: [IMAGE:data:image/jpeg;base64,/9j/4AAQ] What do you see?"
                .to_string(),
        }];

        let (_, native_msgs) = AnthropicProvider::convert_messages(&messages, false);

        assert_eq!(native_msgs.len(), 1);
        assert_eq!(native_msgs[0].role, "user");
        // Should have 2 content blocks: image + text
        assert_eq!(native_msgs[0].content.len(), 2);

        // First block should be image
        match &native_msgs[0].content[0] {
            NativeContentOut::Image { source } => {
                assert_eq!(source.source_type, "base64");
                assert_eq!(source.media_type, "image/jpeg");
                assert_eq!(source.data, "/9j/4AAQ");
            }
            _ => panic!("Expected Image content block"),
        }

        // Second block should be text (parse_image_markers may leave extra spaces)
        match &native_msgs[0].content[1] {
            NativeContentOut::Text { text, .. } => {
                // The text may have extra spaces where the marker was removed
                assert!(
                    text.contains("Check this image:") && text.contains("What do you see?"),
                    "Expected text to contain 'Check this image:' and 'What do you see?', got: {}",
                    text
                );
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn convert_messages_with_only_image_marker() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "[IMAGE:data:image/png;base64,iVBORw0KGgo]".to_string(),
        }];

        let (_, native_msgs) = AnthropicProvider::convert_messages(&messages, false);

        assert_eq!(native_msgs.len(), 1);
        assert_eq!(native_msgs[0].content.len(), 2);

        // First block should be image
        match &native_msgs[0].content[0] {
            NativeContentOut::Image { source } => {
                assert_eq!(source.media_type, "image/png");
            }
            _ => panic!("Expected Image content block"),
        }

        // Second block should be placeholder text
        match &native_msgs[0].content[1] {
            NativeContentOut::Text { text, .. } => {
                assert_eq!(text, "[image]");
            }
            _ => panic!("Expected Text content block with [image] placeholder"),
        }
    }

    #[test]
    fn convert_messages_without_image_marker() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
        }];

        let (_, native_msgs) = AnthropicProvider::convert_messages(&messages, false);

        assert_eq!(native_msgs.len(), 1);
        assert_eq!(native_msgs[0].content.len(), 1);

        match &native_msgs[0].content[0] {
            NativeContentOut::Text { text, .. } => {
                assert_eq!(text, "Hello, how are you?");
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn image_content_serializes_correctly() {
        let content = NativeContentOut::Image {
            source: ImageSource {
                source_type: "base64".to_string(),
                media_type: "image/jpeg".to_string(),
                data: "testdata".to_string(),
            },
        };
        let json = serde_json::to_string(&content).unwrap();
        // The outer "type" is the enum tag, inner "type" (source_type) is renamed
        assert!(json.contains(r#""type":"image""#), "JSON: {}", json);
        assert!(json.contains(r#""type":"base64""#), "JSON: {}", json); // source_type is serialized as "type"
        assert!(
            json.contains(r#""media_type":"image/jpeg""#),
            "JSON: {}",
            json
        );
        assert!(json.contains(r#""data":"testdata""#), "JSON: {}", json);
    }

    #[test]
    fn convert_messages_merges_consecutive_tool_results() {
        // Simulate a multi-tool-call turn: assistant with two tool_use blocks
        // followed by two separate tool result messages.
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Do two things.".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: serde_json::json!({
                    "content": "",
                    "tool_calls": [
                        {"id": "call_1", "name": "shell", "arguments": "{\"command\":\"ls\"}"},
                        {"id": "call_2", "name": "shell", "arguments": "{\"command\":\"pwd\"}"}
                    ]
                })
                .to_string(),
            },
            ChatMessage {
                role: "tool".to_string(),
                content: serde_json::json!({
                    "tool_call_id": "call_1",
                    "content": "file1.txt\nfile2.txt"
                })
                .to_string(),
            },
            ChatMessage {
                role: "tool".to_string(),
                content: serde_json::json!({
                    "tool_call_id": "call_2",
                    "content": "/home/user"
                })
                .to_string(),
            },
        ];

        let (system, native_msgs) = AnthropicProvider::convert_messages(&messages, false);

        assert!(system.is_some());
        // Should be: user, assistant, user (merged tool results)
        // NOT: user, assistant, user, user (which Anthropic rejects)
        assert_eq!(
            native_msgs.len(),
            3,
            "Expected 3 messages (user, assistant, merged tool results), got {}.\nRoles: {:?}",
            native_msgs.len(),
            native_msgs.iter().map(|m| &m.role).collect::<Vec<_>>()
        );
        assert_eq!(native_msgs[0].role, "user");
        assert_eq!(native_msgs[1].role, "assistant");
        assert_eq!(native_msgs[2].role, "user");
        // The merged user message should contain both tool results
        assert_eq!(
            native_msgs[2].content.len(),
            2,
            "Expected 2 tool_result blocks in merged message"
        );
    }

    #[test]
    fn convert_messages_no_adjacent_same_role() {
        // Verify that convert_messages never produces adjacent messages with the
        // same role, regardless of input ordering.
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: serde_json::json!({
                    "content": "I'll run a command",
                    "tool_calls": [
                        {"id": "tc1", "name": "shell", "arguments": "{\"command\":\"echo hi\"}"}
                    ]
                })
                .to_string(),
            },
            ChatMessage {
                role: "tool".to_string(),
                content: serde_json::json!({
                    "tool_call_id": "tc1",
                    "content": "hi"
                })
                .to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Thanks!".to_string(),
            },
        ];

        let (_system, native_msgs) = AnthropicProvider::convert_messages(&messages, false);

        for window in native_msgs.windows(2) {
            assert_ne!(
                window[0].role, window[1].role,
                "Adjacent messages must not share the same role: found two '{}' messages in a row",
                window[0].role
            );
        }
    }

    // --- parse_tool_result_content / vision tests ---

    #[test]
    fn tool_result_content_plain_text_unchanged() {
        let result = AnthropicProvider::parse_tool_result_content("hello world");
        let json = serde_json::to_string(&result).unwrap();
        // Plain string serialisation — no array wrapper.
        assert_eq!(json, r#""hello world""#);
    }

    #[test]
    fn tool_result_content_with_image_produces_blocks() {
        let input = "before\ndata:image/png;base64,abc123\nafter";
        let result = AnthropicProvider::parse_tool_result_content(input);
        let json = serde_json::to_value(&result).unwrap();
        let arr = json.as_array().expect("expected array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "before");
        assert_eq!(arr[1]["type"], "image");
        assert_eq!(arr[1]["source"]["media_type"], "image/png");
        assert_eq!(arr[1]["source"]["data"], "abc123");
        assert_eq!(arr[2]["type"], "text");
        assert_eq!(arr[2]["text"], "after");
    }

    #[test]
    fn tool_result_content_image_only() {
        let input = "data:image/jpeg;base64,/9j/xyz";
        let result = AnthropicProvider::parse_tool_result_content(input);
        let json = serde_json::to_value(&result).unwrap();
        let arr = json.as_array().expect("expected array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "image");
        assert_eq!(arr[0]["source"]["media_type"], "image/jpeg");
        assert_eq!(arr[0]["source"]["data"], "/9j/xyz");
    }

    #[test]
    fn parse_tool_result_message_with_image_builds_vision_blocks() {
        let payload = serde_json::json!({
            "tool_call_id": "id-1",
            "content": "data:image/png;base64,iVBORw0KGgo"
        })
        .to_string();

        let msg = AnthropicProvider::parse_tool_result_message(&payload).unwrap();
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content.len(), 1);
        let json = serde_json::to_value(&msg.content[0]).unwrap();
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["tool_use_id"], "id-1");
        let blocks = json["content"].as_array().expect("expected blocks array");
        assert_eq!(blocks[0]["type"], "image");
        assert_eq!(blocks[0]["source"]["data"], "iVBORw0KGgo");
    }

    #[test]
    fn parse_tool_result_message_plain_uses_string_content() {
        let payload = serde_json::json!({
            "tool_call_id": "id-2",
            "content": "just text"
        })
        .to_string();

        let msg = AnthropicProvider::parse_tool_result_message(&payload).unwrap();
        let json = serde_json::to_value(&msg.content[0]).unwrap();
        // Plain text — content must be a JSON string, not an array.
        assert!(json["content"].is_string(), "content should be a plain string");
        assert_eq!(json["content"], "just text");
    }
}
