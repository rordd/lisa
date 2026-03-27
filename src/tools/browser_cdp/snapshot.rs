//! DOM snapshot for LLM consumption.
//!
//! Reuses the same snapshot JS as the native_backend (`snapshot_script()` in
//! browser.rs) to ensure consistent ref format (@e1, data-zc-ref) across all
//! browser backends. This file duplicates the JS to avoid modifying browser.rs
//! (which would cause upstream merge conflicts).

use anyhow::Result;
use chromiumoxide::Page;

/// Resolve a selector string (supports @eN, @N, ref=N, and CSS selectors).
/// Uses `data-zc-ref` attribute — same as native_backend.
pub fn resolve_selector(selector: &str) -> String {
    let s = selector.trim();
    // @e5, @5
    if let Some(rest) = s.strip_prefix('@') {
        let num = rest.strip_prefix('e').unwrap_or(rest);
        if num.chars().all(|c| c.is_ascii_digit()) && !num.is_empty() {
            return format!("[data-zc-ref=\"@e{num}\"]");
        }
    }
    // ref=N
    if let Some(num) = s.strip_prefix("ref=") {
        if num.chars().all(|c| c.is_ascii_digit()) && !num.is_empty() {
            return format!("[data-zc-ref=\"@e{num}\"]");
        }
    }
    // Plain CSS selector
    s.to_string()
}

/// Generate the snapshot JS script.
/// This is the same logic as `native_backend::snapshot_script()` in browser.rs.
/// Kept as a separate copy to avoid modifying browser.rs (upstream sync).
fn snapshot_script(interactive_only: bool, compact: bool, depth: Option<i64>) -> String {
    // Enforce minimum depth — shallow depths miss critical UI elements
    // like cart buttons on e-commerce sites.
    let depth_literal = match depth {
        Some(d) if d < 8 => "null".to_string(), // too shallow, use unlimited
        Some(d) => d.to_string(),
        None => "null".to_string(),
    };

    // Output format: role-based category grouping.
    // Elements are grouped into buttons/inputs/links/other sections
    // so the LLM can immediately find action buttons without scanning
    // hundreds of elements. This works on ALL websites since it uses
    // HTML tag types, not site-specific ARIA landmarks.
    format!(
        r#"(() => {{
  const interactiveOnly = {interactive_only};
  const compact = {compact};
  const maxDepth = {depth_literal};
  const buttons = [];
  const inputs = [];
  const links = [];
  const other = [];
  const root = document.body || document.documentElement;
  let counter = 0;
  let total = 0;

  const isVisible = (el) => {{
    const style = window.getComputedStyle(el);
    if (style.display === 'none' || style.visibility === 'hidden' || Number(style.opacity) === 0) {{
      return false;
    }}
    const rect = el.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  }};

  const isInteractive = (el) => {{
    if (el.matches('a,button,input,select,textarea,summary,[role],*[tabindex]')) return true;
    return typeof el.onclick === 'function';
  }};

  const getCategory = (el) => {{
    const tag = el.tagName;
    const explicit = el.getAttribute('role');
    if (tag === 'BUTTON' || explicit === 'button') return 'buttons';
    if (tag === 'INPUT' && (el.getAttribute('type') || 'text') === 'submit') return 'buttons';
    if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA') return 'inputs';
    if (tag === 'A' || explicit === 'link') return 'links';
    return 'other';
  }};

  const describe = (el) => {{
    const interactive = isInteractive(el);
    const text = (el.innerText || el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 140);
    if (interactiveOnly && !interactive) return;
    if (compact && !interactive && !text) return;

    const refId = '@e' + (++counter);
    el.setAttribute('data-zc-ref', refId);

    let line = refId;

    // Add attributes based on category
    if (text) line += ' "' + text.replace(/"/g, '\\"') + '"';
    if (el.tagName === 'INPUT') {{
      const t = el.getAttribute('type') || 'text';
      line += ' [type=' + t + ']';
      if (el.placeholder) line += ' placeholder="' + el.placeholder.replace(/"/g, '\\"') + '"';
      if (el.value && t !== 'password' && t !== 'hidden') line += ' value="' + el.value.slice(0, 50).replace(/"/g, '\\"') + '"';
    }}
    if (el.tagName === 'A' && el.href) {{
      const href = el.getAttribute('href') || '';
      if (href.length < 80) line += ' href="' + href + '"';
    }}
    if (el.tagName === 'H1' || el.tagName === 'H2' || el.tagName === 'H3') {{
      line += ' [level=' + el.tagName.charAt(1) + ']';
    }}

    const cat = getCategory(el);
    if (cat === 'buttons') buttons.push(line);
    else if (cat === 'inputs') inputs.push(line);
    else if (cat === 'links') links.push(line);
    else {{
      // Add role label for other category (mixed types need identification)
      const role = el.getAttribute('role') || el.tagName.toLowerCase();
      other.push(refId + ' [' + role + ']' + line.slice(refId.length));
    }}
    total++;
  }};

  const walk = (el, depth) => {{
    if (!(el instanceof Element)) return;
    if (maxDepth !== null && depth > maxDepth) return;
    if (isVisible(el)) {{
      describe(el);
    }}
    for (const child of el.children) {{
      walk(child, depth + 1);
      if (total >= 400) return;
    }}
  }};

  if (root) walk(root, 0);

  let out = 'title: ' + document.title + '\nurl: ' + window.location.href + '\nelements: ' + total + '\n---\n';
  if (buttons.length) out += '── buttons ──\n' + buttons.join('\n') + '\n';
  if (inputs.length) out += '── inputs ──\n' + inputs.join('\n') + '\n';
  if (links.length) out += '── links ──\n' + links.join('\n') + '\n';
  if (other.length) out += '── other ──\n' + other.join('\n') + '\n';
  return out;
}})();"#
    )
}

/// Take a DOM snapshot of the page for LLM consumption.
pub async fn take_snapshot(
    page: &Page,
    interactive_only: bool,
    compact: bool,
    depth: Option<i64>,
) -> Result<String> {
    let js = snapshot_script(interactive_only, compact, depth);

    // Take snapshot with progressive retry: if first attempt yields too few refs,
    // wait progressively longer and retry. No reload — just re-read the same DOM
    // as JS continues rendering dynamic content.
    let mut result: String = String::new();
    let wait_schedule = [2000u64, 3000, 4000, 5000]; // progressive backoff (ms)

    for attempt in 0..=wait_schedule.len() {
        result = page
            .evaluate(js.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Snapshot evaluation failed: {e}"))?
            .into_value()
            .unwrap_or_default();

        // Count refs in the JSON result
        let ref_count = result.matches("@e").count();
        tracing::debug!(
            snapshot_len = result.len(),
            ref_count = ref_count,
            attempt = attempt,
            "DOM snapshot captured"
        );

        // If snapshot has reasonable content, accept it
        if result.len() > 100 && ref_count > 5 {
            break;
        }

        // Wait and retry if we have more attempts
        if attempt < wait_schedule.len() {
            let wait_ms = wait_schedule[attempt];
            tracing::info!(
                snapshot_len = result.len(),
                ref_count = ref_count,
                wait_ms = wait_ms,
                "Snapshot too sparse, waiting for dynamic content"
            );
            tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
        }
    }

    if result.len() < 50 {
        tracing::warn!(
            snapshot_len = result.len(),
            "Snapshot suspiciously short — page may not have loaded"
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_selector_ref_formats() {
        assert_eq!(resolve_selector("@e5"), "[data-zc-ref=\"@e5\"]");
        assert_eq!(resolve_selector("@5"), "[data-zc-ref=\"@e5\"]");
        assert_eq!(resolve_selector("ref=15"), "[data-zc-ref=\"@e15\"]");
    }

    #[test]
    fn resolve_selector_css() {
        assert_eq!(resolve_selector("#my-btn"), "#my-btn");
        assert_eq!(resolve_selector(".cls"), ".cls");
        assert_eq!(resolve_selector("input[type=text]"), "input[type=text]");
    }

    #[test]
    fn resolve_selector_trims() {
        assert_eq!(resolve_selector("  @e7  "), "[data-zc-ref=\"@e7\"]");
    }

    #[test]
    fn resolve_selector_at_non_numeric_passthrough() {
        assert_eq!(resolve_selector("@abc"), "@abc");
    }

    #[test]
    fn resolve_selector_empty_passthrough() {
        assert_eq!(resolve_selector(""), "");
    }

    #[test]
    fn snapshot_script_generates_valid_js() {
        let js = snapshot_script(true, true, None);
        assert!(js.contains("interactiveOnly"));
        assert!(js.contains("data-zc-ref"));
        assert!(js.contains("@e"));
        assert!(js.contains("getCategory"));
        // Category labels
        assert!(js.contains("'buttons'"));
        assert!(js.contains("'links'"));
        assert!(js.contains("'inputs'"));
    }

    #[test]
    fn snapshot_script_with_depth() {
        // depth < 8 should be overridden to null
        let js = snapshot_script(false, false, Some(5));
        assert!(js.contains("const maxDepth = null;"));
        // depth >= 8 should be kept
        let js = snapshot_script(false, false, Some(10));
        assert!(js.contains("const maxDepth = 10;"));
    }

    #[test]
    fn snapshot_script_null_depth() {
        let js = snapshot_script(true, true, None);
        assert!(js.contains("const maxDepth = null;"));
    }

    #[test]
    fn snapshot_script_contains_category_mappings() {
        let js = snapshot_script(true, true, None);
        // Verify category logic for buttons/inputs/links
        assert!(js.contains("=== 'BUTTON'")); // <button> → buttons
        assert!(js.contains("=== 'A'")); // <a> → links
        assert!(js.contains("=== 'INPUT'")); // <input> → inputs
        assert!(js.contains("=== 'SELECT'")); // <select> → inputs
        assert!(js.contains("=== 'TEXTAREA'")); // <textarea> → inputs
        assert!(js.contains("'submit'")); // input[submit] → buttons
    }

    #[test]
    fn snapshot_script_output_format() {
        let js = snapshot_script(true, true, None);
        // Verify category section headers
        assert!(js.contains("── buttons ──"));
        assert!(js.contains("── inputs ──"));
        assert!(js.contains("── links ──"));
        assert!(js.contains("── other ──"));
        // Verify header contains title, url, elements
        assert!(js.contains("title: "));
        assert!(js.contains("url: "));
        assert!(js.contains("elements: "));
    }

    #[test]
    fn snapshot_script_max_nodes_limit() {
        let js = snapshot_script(true, true, None);
        assert!(js.contains("total >= 400"));
    }
}
