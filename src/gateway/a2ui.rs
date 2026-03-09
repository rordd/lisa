//! A2UI v0.8 message handling for WebSocket channels.
//!
//! Parses A2UI card data from LLM responses and formats userAction events
//! per the A2UI protocol specification (Section 5.2).

use serde_json::Value;

/// Extract A2UI messages from an LLM response.
///
/// Supports two patterns:
/// 1. Explicit delimiter: `text\n---a2ui_JSON---\n[{...}]`
/// 2. Inline detection: text followed by `[{"surfaceUpdate":...}]`
///
/// Returns (text_only, a2ui_messages). If no A2UI data is found,
/// returns the original text and an empty vec.
pub fn parse_response(raw: &str) -> (String, Vec<Value>) {
    // Try delimiter-based parsing first
    if raw.contains(DELIMITER) {
        return parse_delimiter(raw);
    }
    // Fallback: detect inline A2UI JSON arrays
    if let Some(result) = try_extract_inline(raw) {
        return result;
    }
    (raw.to_string(), vec![])
}

/// Format a client userAction payload as a message for the LLM history.
///
/// Follows A2UI spec Section 5.2 userAction format.
pub fn format_user_action(payload: &Value) -> String {
    let surface_id = payload["surfaceId"].as_str().unwrap_or("unknown");
    let action_name = payload["name"].as_str().unwrap_or("unknown");
    let component_id = payload["sourceComponentId"].as_str().unwrap_or("unknown");
    let context = payload
        .get("context")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let user_action = serde_json::json!({
        "userAction": {
            "name": action_name,
            "surfaceId": surface_id,
            "sourceComponentId": component_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "context": context
        }
    });
    format!(
        "[A2UI userAction]\n{}",
        serde_json::to_string_pretty(&user_action).unwrap_or_default()
    )
}

// ── Internal helpers ─────────────────────────────────────────

const DELIMITER: &str = "---a2ui_JSON---";

/// Try to parse a JSON array starting at `input` using serde_json's streaming
/// deserializer. Returns `Some((parsed_values, bytes_consumed))` on success.
/// This correctly handles brackets inside JSON string values, unlike simple
/// bracket-depth counting.
fn try_parse_json_array(input: &str) -> Option<(Vec<Value>, usize)> {
    let mut de = serde_json::Deserializer::from_str(input).into_iter::<Vec<Value>>();
    if let Some(Ok(messages)) = de.next() {
        let consumed = de.byte_offset();
        Some((messages, consumed))
    } else {
        None
    }
}

fn parse_delimiter(raw: &str) -> (String, Vec<Value>) {
    // NOTE: split is naive — if the delimiter string appears inside a JSON
    // string value, it would cause an incorrect split. In practice the delimiter
    // is unusual enough that this is unlikely.
    let parts: Vec<&str> = raw.split(DELIMITER).collect();
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
        // Fallback: treat entire segment as text
        if !trimmed.is_empty() {
            text_parts.push(trimmed);
        }
    }

    let text = text_parts.join("\n").trim().to_string();
    (text, a2ui_messages)
}

fn try_extract_inline(raw: &str) -> Option<(String, Vec<Value>)> {
    // Find the first `[{` which could be an A2UI JSON array.
    // NOTE: only checks the first occurrence — if a non-A2UI `[{` appears
    // before the real A2UI array, the A2UI data will be missed.
    let start = raw.find("[{")?;
    let json_candidate = &raw[start..];
    let (messages, consumed) = try_parse_json_array(json_candidate)?;

    // Verify at least one message has an A2UI key
    let has_a2ui = messages.iter().any(|m| {
        m.get("surfaceUpdate").is_some()
            || m.get("beginRendering").is_some()
            || m.get("dataModelUpdate").is_some()
            || m.get("deleteSurface").is_some()
    });
    if !has_a2ui {
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

    #[test]
    fn delimiter_splits_text_and_json() {
        let raw = "Here is the weather.\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"w1\",\"components\":[]}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Here is the weather.");
        assert_eq!(a2ui.len(), 1);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "w1");
    }

    #[test]
    fn delimiter_absent_returns_full_text() {
        let raw = "Just text, no cards.";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, raw);
        assert!(a2ui.is_empty());
    }

    #[test]
    fn delimiter_invalid_json_fallback() {
        let raw = "Text\n---a2ui_JSON---\n{not valid json";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Text\n{not valid json");
        assert!(a2ui.is_empty());
    }

    #[test]
    fn delimiter_multiple_blocks() {
        let raw = "Hello\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"s1\"}}]\nWorld\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"s2\"}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "Hello\nWorld");
        assert_eq!(a2ui.len(), 2);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "s1");
        assert_eq!(a2ui[1]["surfaceUpdate"]["surfaceId"], "s2");
    }

    #[test]
    fn delimiter_empty_text_with_json() {
        let raw = "---a2ui_JSON---\n[{\"beginRendering\":{\"surfaceId\":\"s1\",\"root\":\"r\"}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "");
        assert_eq!(a2ui.len(), 1);
    }

    #[test]
    fn delimiter_text_only_before() {
        let raw = "   \n---a2ui_JSON---\n[{\"deleteSurface\":{\"surfaceId\":\"s1\"}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "");
        assert_eq!(a2ui.len(), 1);
    }

    #[test]
    fn inline_json_without_delimiter() {
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
        // Verify that brackets inside JSON string values don't break parsing
        let raw = "결과!\n---a2ui_JSON---\n[{\"surfaceUpdate\":{\"surfaceId\":\"s1\",\"components\":[{\"id\":\"t1\",\"component\":{\"Text\":{\"text\":\"점수는 [3/5] 입니다\"}}}]}}]";
        let (text, a2ui) = parse_response(raw);
        assert_eq!(text, "결과!");
        assert_eq!(a2ui.len(), 1);
        assert_eq!(a2ui[0]["surfaceUpdate"]["surfaceId"], "s1");
    }
}
