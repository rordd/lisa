//! A2UI v0.9 message handling for WebSocket channels.
//!
//! Parses A2UI card data from LLM responses and formats userAction events
//! per the Google A2UI v0.9 protocol specification.

use serde_json::Value;

/// Extract A2UI messages from an LLM response.
///
/// Supports these patterns (in priority order):
/// 1. Google `<a2ui-json>` tags: text + `<a2ui-json>[...]</a2ui-json>` (official v0.9 format)
/// 2. Explicit delimiter: `text\n---a2ui---\n{...}\n{...}` (JSONL after delimiter)
/// 3. Legacy delimiter: `text\n---a2ui_JSON---\n[{...}]` (v0.8 JSON array)
/// 4. Inline detection: text followed by JSONL or `[{"updateComponents":...}]`
///
/// Returns (text_only, a2ui_messages). If no A2UI data is found,
/// returns the original text and an empty vec.
pub fn parse_response(raw: &str) -> (String, Vec<Value>) {
    // Try Google official <a2ui-json> tags first (v0.9 primary format)
    if raw.contains(A2UI_TAG_OPEN) && raw.contains(A2UI_TAG_CLOSE) {
        return parse_a2ui_tags(raw);
    }
    // Try v0.9 delimiter
    if raw.contains(DELIMITER_V09) {
        return parse_delimiter_v09(raw);
    }
    // Try legacy v0.8 delimiter
    if raw.contains(DELIMITER_V08) {
        return parse_delimiter_v08(raw);
    }
    // Fallback: detect inline A2UI data
    if let Some(result) = try_extract_inline(raw) {
        return result;
    }
    (raw.to_string(), vec![])
}

/// Extract a compact context summary from A2UI messages for LLM history.
///
/// Keeps only the data model values (which contain actual content like option text)
/// and strips bulky component definitions. This lets the LLM remember what was shown
/// (e.g., "A = Sahara Desert") without bloating the context with UI structure.
pub fn summarize_for_history(a2ui_messages: &[Value]) -> String {
    let mut parts = Vec::new();
    for msg in a2ui_messages {
        if let Some(dm) = msg.get("updateDataModel") {
            if let Some(value) = dm.get("value") {
                parts.push(format!(
                    "[A2UI data] {}",
                    serde_json::to_string(value).unwrap_or_default()
                ));
            }
        }
    }
    parts.join("\n")
}

/// Format a client A2UI action payload for the LLM.
///
/// Passes the original payload as-is (v0.9 standard).
/// The client includes dataModel when sendDataModel is enabled.
pub fn format_user_action(payload: &Value) -> String {
    format!(
        "[A2UI action]\n{}",
        serde_json::to_string_pretty(payload).unwrap_or_default()
    )
}

// ── Internal helpers ─────────────────────────────────────────

/// Google official A2UI v0.9 XML-style tags.
const A2UI_TAG_OPEN: &str = "<a2ui-json>";
const A2UI_TAG_CLOSE: &str = "</a2ui-json>";

const DELIMITER_V09: &str = "---a2ui---";
const DELIMITER_V08: &str = "---a2ui_JSON---";

/// A2UI v0.9 message keys (server-to-client).
const A2UI_V09_KEYS: &[&str] = &[
    "createSurface",
    "updateComponents",
    "updateDataModel",
    "deleteSurface",
];

/// Legacy A2UI v0.8 message keys (server-to-client).
const A2UI_V08_KEYS: &[&str] = &[
    "surfaceUpdate",
    "beginRendering",
    "dataModelUpdate",
    "deleteSurface",
];

/// Check if a JSON value has any A2UI key (v0.8 or v0.9).
fn is_a2ui_message(v: &Value) -> bool {
    // Check top-level keys (standard format)
    if A2UI_V09_KEYS.iter().any(|k| v.get(k).is_some())
        || A2UI_V08_KEYS.iter().any(|k| v.get(k).is_some())
    {
        return true;
    }
    // Check nested "v0.9": { "createSurface": ... } format (Gemini style)
    if let Some(inner) = v.get("v0.9") {
        if let Some(obj) = inner.as_object() {
            return A2UI_V09_KEYS.iter().any(|k| obj.contains_key(*k));
        }
        // "v0.9": "v0.9" version marker — also valid
        if inner.is_string() {
            return A2UI_V09_KEYS.iter().any(|k| v.get(k).is_some());
        }
    }
    false
}

/// Parse JSONL (one JSON object per line) from a string.
/// Returns all successfully parsed objects that contain A2UI keys.
fn parse_jsonl(input: &str) -> Vec<Value> {
    let mut messages = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('{') {
            continue;
        }
        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            if is_a2ui_message(&val) {
                messages.push(val);
            }
        }
    }
    messages
}

/// Parse Google official `<a2ui-json>` tag format.
/// Extracts JSON arrays from one or more `<a2ui-json>...</a2ui-json>` blocks.
/// Text between/around blocks is collected as conversational text.
fn parse_a2ui_tags(raw: &str) -> (String, Vec<Value>) {
    let mut text_parts = Vec::new();
    let mut a2ui_messages = Vec::new();
    let mut remaining = raw;

    while let Some(open_pos) = remaining.find(A2UI_TAG_OPEN) {
        let before = remaining[..open_pos].trim();
        if !before.is_empty() {
            text_parts.push(before.to_string());
        }
        let after_open = &remaining[open_pos + A2UI_TAG_OPEN.len()..];
        if let Some(close_pos) = after_open.find(A2UI_TAG_CLOSE) {
            let json_content = after_open[..close_pos].trim();
            // Try parsing as JSON array first (standard format)
            if let Some((messages, _)) = try_parse_json_array(json_content) {
                for msg in messages {
                    if is_a2ui_message(&msg) {
                        a2ui_messages.push(msg);
                    }
                }
            } else if let Ok(val) = serde_json::from_str::<Value>(json_content) {
                // Single JSON object within tags (LLM may wrap each message individually)
                if is_a2ui_message(&val) {
                    a2ui_messages.push(val);
                }
            } else {
                // Fallback: try JSONL within tags
                a2ui_messages.extend(parse_jsonl(json_content));
            }
            remaining = &after_open[close_pos + A2UI_TAG_CLOSE.len()..];
        } else {
            // No closing tag found; treat rest as text
            text_parts.push(remaining.to_string());
            remaining = "";
            break;
        }
    }
    let trailing = remaining.trim();
    if !trailing.is_empty() {
        text_parts.push(trailing.to_string());
    }
    let text = text_parts.join("\n").trim().to_string();
    (text, a2ui_messages)
}

/// Parse v0.9 delimiter format: text + `---a2ui---` + JSON array or JSONL lines.
fn parse_delimiter_v09(raw: &str) -> (String, Vec<Value>) {
    let parts: Vec<&str> = raw.splitn(2, DELIMITER_V09).collect();
    let text = parts[0].trim().to_string();
    let data_part = parts.get(1).map(|s| s.trim()).unwrap_or("");
    // Try JSON array first (e.g. [{...},{...}])
    if let Some((messages, _)) = try_parse_json_array(data_part) {
        let a2ui: Vec<Value> = messages.into_iter().filter(is_a2ui_message).collect();
        if !a2ui.is_empty() {
            return (text, a2ui);
        }
    }
    // Fallback to JSONL (one JSON object per line)
    let messages = parse_jsonl(data_part);
    (text, messages)
}

/// Try to parse a JSON array starting at `input` using serde_json's streaming
/// deserializer. Returns `Some((parsed_values, bytes_consumed))` on success.
fn try_parse_json_array(input: &str) -> Option<(Vec<Value>, usize)> {
    let mut de = serde_json::Deserializer::from_str(input).into_iter::<Vec<Value>>();
    if let Some(Ok(messages)) = de.next() {
        let consumed = de.byte_offset();
        Some((messages, consumed))
    } else {
        None
    }
}

/// Parse legacy v0.8 delimiter format: text + `---a2ui_JSON---` + JSON array.
fn parse_delimiter_v08(raw: &str) -> (String, Vec<Value>) {
    let parts: Vec<&str> = raw.split(DELIMITER_V08).collect();
    let mut text_parts: Vec<&str> = Vec::new();
    let mut a2ui_messages: Vec<Value> = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            let t = part.trim();
            if !t.is_empty() {
                text_parts.push(t);
            }
            continue;
        }
        let trimmed = part.trim();
        if let Some(bracket_start) = trimmed.find('[') {
            let json_candidate = &trimmed[bracket_start..];
            if let Some((messages, consumed)) = try_parse_json_array(json_candidate) {
                a2ui_messages.extend(messages);
                let after = trimmed[bracket_start + consumed..].trim();
                if !after.is_empty() {
                    text_parts.push(after);
                }
                continue;
            }
        }
        // Also try JSONL parsing (v0.8 delimiter but v0.9 content)
        let jsonl = parse_jsonl(trimmed);
        if !jsonl.is_empty() {
            a2ui_messages.extend(jsonl);
            continue;
        }
        // Fallback: treat entire segment as text
        if !trimmed.is_empty() {
            text_parts.push(trimmed);
        }
    }

    let text = text_parts.join("\n").trim().to_string();
    (text, a2ui_messages)
}

fn try_extract_inline(raw: &str) -> Option<(String, Vec<Value>)> {
    // Try JSONL detection: look for lines starting with `{"version"` or `{"createSurface"`
    if let Some(result) = try_extract_inline_jsonl(raw) {
        return Some(result);
    }
    // Try legacy JSON array detection
    try_extract_inline_array(raw)
}

/// Try to extract inline JSONL A2UI messages from the text.
fn try_extract_inline_jsonl(raw: &str) -> Option<(String, Vec<Value>)> {
    let mut text_lines = Vec::new();
    let mut a2ui_messages = Vec::new();
    let mut in_a2ui_block = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                if is_a2ui_message(&val) {
                    a2ui_messages.push(val);
                    in_a2ui_block = true;
                    continue;
                }
            }
        }
        if !in_a2ui_block {
            text_lines.push(line);
        }
    }

    if a2ui_messages.is_empty() {
        return None;
    }

    let text = text_lines.join("\n").trim().to_string();
    Some((text, a2ui_messages))
}

/// Try to extract a legacy inline JSON array of A2UI messages.
fn try_extract_inline_array(raw: &str) -> Option<(String, Vec<Value>)> {
    let start = raw.find("[{")?;
    let json_candidate = &raw[start..];
    let (messages, consumed) = try_parse_json_array(json_candidate)?;

    // Verify at least one message has an A2UI key
    if !messages.iter().any(is_a2ui_message) {
        return None;
    }

    let text_before = raw[..start].trim();
    let text_after = raw[start + consumed..].trim();
    let text = if text_after.is_empty() {
        text_before.to_string()
    } else {
        format!("{text_before}\n{text_after}")
    };
    Some((text.trim().to_string(), messages))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── <a2ui-json> tag tests (Google official format) ──────

    #[test]
    fn a2ui_tags_parse_json_array() {
        let raw = "Here is the weather.\n\n<a2ui-json>\n[\n  {\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"w1\",\"catalogId\":\"basic\"}},\n  {\"version\":\"v0.9\",\"updateComponents\":{\"surfaceId\":\"w1\",\"components\":[]}}\n]\n</a2ui-json>";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Here is the weather.");
        assert_eq!(a2ui.len(), 2);
        assert!(a2ui[0].get("createSurface").is_some());
        assert!(a2ui[1].get("updateComponents").is_some());
    }

    #[test]
    fn a2ui_tags_multiple_blocks() {
        let raw = "First card:\n\n<a2ui-json>\n[{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s1\",\"catalogId\":\"basic\"}}]\n</a2ui-json>\n\nSecond card:\n\n<a2ui-json>\n[{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s2\",\"catalogId\":\"basic\"}}]\n</a2ui-json>";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "First card:\nSecond card:");
        assert_eq!(a2ui.len(), 2);
    }

    #[test]
    fn a2ui_tags_with_data_model() {
        let raw = "Weather data:\n<a2ui-json>\n[\n{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"w\",\"catalogId\":\"basic\"}},\n{\"version\":\"v0.9\",\"updateComponents\":{\"surfaceId\":\"w\",\"components\":[]}},\n{\"version\":\"v0.9\",\"updateDataModel\":{\"surfaceId\":\"w\",\"path\":\"/\",\"value\":{\"temp\":\"25°C\"}}}\n]\n</a2ui-json>";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Weather data:");
        assert_eq!(a2ui.len(), 3);
        assert_eq!(a2ui[2]["updateDataModel"]["value"]["temp"], "25°C");
    }

    #[test]
    fn a2ui_tags_trailing_text() {
        let raw = "Before.\n<a2ui-json>\n[{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s\",\"catalogId\":\"basic\"}}]\n</a2ui-json>\nAfter text.";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Before.\nAfter text.");
        assert_eq!(a2ui.len(), 1);
    }

    #[test]
    fn a2ui_tags_empty_text() {
        let raw = "<a2ui-json>\n[{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s\",\"catalogId\":\"basic\"}}]\n</a2ui-json>";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "");
        assert_eq!(a2ui.len(), 1);
    }

    #[test]
    fn a2ui_tags_single_object_per_tag() {
        // LLM wraps each message in its own <a2ui-json> tag as a single object (not array)
        let raw = "날씨 알려줄게~\n\n<a2ui-json>{\n  \"version\": \"v0.9\",\n  \"createSurface\": {\n    \"surfaceId\": \"w1\",\n    \"catalogId\": \"basic\"\n  }\n}</a2ui-json>\n\n<a2ui-json>{\n  \"version\": \"v0.9\",\n  \"updateComponents\": {\n    \"surfaceId\": \"w1\",\n    \"components\": []\n  }\n}</a2ui-json>";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "날씨 알려줄게~");
        assert_eq!(a2ui.len(), 2);
        assert!(a2ui[0].get("createSurface").is_some());
        assert!(a2ui[1].get("updateComponents").is_some());
    }

    #[test]
    fn a2ui_tags_priority_over_delimiter() {
        // If both tags and delimiter exist, tags should win
        let raw = "Text\n---a2ui---\nshould not parse this\n<a2ui-json>\n[{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s\",\"catalogId\":\"basic\"}}]\n</a2ui-json>";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(a2ui.len(), 1);
        assert!(text.contains("---a2ui---"));
    }

    // ── v0.9 delimiter tests ────────────────────────────────

    #[test]
    fn v09_delimiter_splits_text_and_jsonl() {
        let raw = "Here is the weather.\n---a2ui---\n{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"w1\",\"catalogId\":\"basic\"}}\n{\"version\":\"v0.9\",\"updateComponents\":{\"surfaceId\":\"w1\",\"components\":[]}}";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Here is the weather.");
        assert_eq!(a2ui.len(), 2);
        assert!(a2ui[0].get("createSurface").is_some());
        assert!(a2ui[1].get("updateComponents").is_some());
    }

    #[test]
    fn v09_delimiter_with_data_model() {
        let raw = "Weather data:\n---a2ui---\n{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"w\",\"catalogId\":\"basic\"}}\n{\"version\":\"v0.9\",\"updateComponents\":{\"surfaceId\":\"w\",\"components\":[{\"id\":\"root\",\"component\":\"Card\",\"child\":\"t\"},{\"id\":\"t\",\"component\":\"Text\",\"text\":{\"path\":\"/temp\"}}]}}\n{\"version\":\"v0.9\",\"updateDataModel\":{\"surfaceId\":\"w\",\"path\":\"/\",\"value\":{\"temp\":\"25°C\"}}}";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Weather data:");
        assert_eq!(a2ui.len(), 3);
        assert!(a2ui[2].get("updateDataModel").is_some());
        assert_eq!(a2ui[2]["updateDataModel"]["value"]["temp"], "25°C");
    }

    #[test]
    fn v09_delimiter_absent_returns_full_text() {
        let raw = "Just text, no cards.";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, raw);
        assert!(a2ui.is_empty());
    }

    #[test]
    fn v09_delimiter_empty_text() {
        let raw = "---a2ui---\n{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s1\",\"catalogId\":\"basic\"}}";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "");
        assert_eq!(a2ui.len(), 1);
    }

    #[test]
    fn v09_delete_surface() {
        let raw = "Removing card.\n---a2ui---\n{\"version\":\"v0.9\",\"deleteSurface\":{\"surfaceId\":\"old\"}}";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Removing card.");
        assert_eq!(a2ui.len(), 1);
        assert!(a2ui[0].get("deleteSurface").is_some());
    }

    #[test]
    fn v09_skips_non_a2ui_json_lines() {
        let raw = "Mixed content.\n---a2ui---\n{\"random\":\"data\"}\n{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"s\",\"catalogId\":\"basic\"}}";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Mixed content.");
        assert_eq!(a2ui.len(), 1);
        assert!(a2ui[0].get("createSurface").is_some());
    }

    // ── v0.9 inline JSONL detection ────────────────────────

    #[test]
    fn v09_inline_jsonl() {
        let raw = "퀴즈 가져왔어!\n{\"version\":\"v0.9\",\"createSurface\":{\"surfaceId\":\"quiz\",\"catalogId\":\"basic\"}}\n{\"version\":\"v0.9\",\"updateComponents\":{\"surfaceId\":\"quiz\",\"components\":[]}}";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "퀴즈 가져왔어!");
        assert_eq!(a2ui.len(), 2);
    }

    // ── Legacy v0.8 backward compatibility ──────────────────

    #[test]
    fn v08_delimiter_splits_text_and_json() {
        let raw = "Here is the weather.\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"w1\",\"components\":[]}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Here is the weather.");
        assert_eq!(a2ui.len(), 1);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "w1");
    }

    #[test]
    fn v08_delimiter_invalid_json_fallback() {
        let raw = "Text\n---a2ui_JSON---\n{not valid json";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Text\n{not valid json");
        assert!(a2ui.is_empty());
    }

    #[test]
    fn v08_delimiter_multiple_blocks() {
        let raw = "Hello\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"s1\"}}]\nWorld\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"s2\"}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Hello\nWorld");
        assert_eq!(a2ui.len(), 2);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "s1");
        assert_eq!(a2ui[1]["surfaceUpdate"]["surfaceId"], "s2");
    }

    #[test]
    fn v08_delimiter_empty_text_with_json() {
        let raw = "---a2ui_JSON---\n[{\"beginRendering\":{\"surfaceId\":\"s1\",\"root\":\"r\"}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "");
        assert_eq!(a2ui.len(), 1);
    }

    #[test]
    fn v08_inline_json_without_delimiter() {
        let raw = "퀴즈 가져왔어! ✨ [{\"surfaceUpdate\":{\"surfaceId\":\"quiz\",\"components\":[]}},{\"beginRendering\":{\"surfaceId\":\"quiz\",\"root\":\"root\"}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "퀴즈 가져왔어! ✨");
        assert_eq!(a2ui.len(), 2);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "quiz");
    }

    #[test]
    fn inline_no_false_positive() {
        let raw = "Here is some [{\"random\": \"json\"}] in text";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, raw);
        assert!(a2ui.is_empty());
    }

    #[test]
    fn brackets_inside_json_strings() {
        let raw = "결과!\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"s1\",\"components\":[{\"id\":\"t1\",\"component\":{\"Text\":{\"text\":\"점수는 [3/5] 입니다\"}}}]}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "결과!");
        assert_eq!(a2ui.len(), 1);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "s1");
    }

    // ── format_user_action ──────────────────────────────────

    #[test]
    fn format_user_action_passes_payload_as_is() {
        let payload = serde_json::json!({
            "surfaceId": "quiz",
            "name": "submit",
            "sourceComponentId": "btn1",
            "context": {"answer": "a"}
        });
        let result = format_user_action(&payload);
        assert!(result.starts_with("[A2UI action]"));
        let json_part = result.strip_prefix("[A2UI action]\n").unwrap();
        let parsed: Value = serde_json::from_str(json_part).unwrap();
        assert_eq!(parsed["surfaceId"], "quiz");
        assert_eq!(parsed["name"], "submit");
        assert_eq!(parsed["context"]["answer"], "a");
    }

    #[test]
    fn format_user_action_includes_data_model() {
        let payload = serde_json::json!({
            "surfaceId": "todo",
            "name": "submit",
            "sourceComponentId": "btn_save",
            "context": {},
            "dataModel": {
                "items": [
                    {"text": "빨래", "checked": true},
                    {"text": "장보기", "checked": false}
                ]
            }
        });
        let result = format_user_action(&payload);
        assert!(result.contains("dataModel"));
        assert!(result.contains("빨래"));
    }
}
