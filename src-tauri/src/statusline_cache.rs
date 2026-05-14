use crate::paths;
use std::collections::HashMap;
use std::fs;

/// What we surface to the UI from a cached statusline tick.
#[derive(Debug, Clone)]
pub struct LiveEntry {
    pub used_percentage: Option<f64>,
    pub model_id: Option<String>,
    pub model_display_name: Option<String>,
    pub updated_at_ms: i64,
}

/// Load every cached statusline entry under ~/.claude-hub/ctx-cache.
/// Returns a map keyed by sessionId (UUID).
pub fn read_all() -> HashMap<String, LiveEntry> {
    let mut out = HashMap::new();
    let Some(dir) = paths::hub_ctx_cache_dir() else { return out; };
    let Ok(entries) = fs::read_dir(&dir) else { return out; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue; };
        // Ignore in-flight temp files written by the wrapper.
        if stem.starts_with(".claude-hub-") { continue; }
        let Ok(text) = fs::read_to_string(&path) else { continue; };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { continue; };
        let Some(entry) = parse_entry(&v) else { continue; };
        let sid = v.get("session_id").and_then(|s| s.as_str()).unwrap_or(stem);
        out.insert(sid.to_string(), entry);
    }
    out
}

fn parse_entry(v: &serde_json::Value) -> Option<LiveEntry> {
    let updated_at_ms = v
        .get("updated_at_ms")
        .and_then(|n| n.as_i64())
        .unwrap_or(0);
    let model_obj = v.get("model");
    let model_id = model_obj
        .and_then(|m| m.get("id"))
        .and_then(|s| s.as_str())
        .map(String::from);
    let model_display_name = model_obj
        .and_then(|m| m.get("display_name"))
        .and_then(|s| s.as_str())
        .map(String::from);
    let used_percentage = v
        .get("context_window")
        .and_then(|c| c.get("used_percentage"))
        .and_then(|n| n.as_f64());
    Some(LiveEntry { used_percentage, model_id, model_display_name, updated_at_ms })
}

/// Snap a derived window size to the closest known Claude context window.
/// CC only ships 200k or 1M today; rounding errors from `tokens / pct` should
/// not produce 217,341-token windows in the UI.
pub fn snap_window(derived: u64) -> u64 {
    const KNOWN: [u64; 2] = [200_000, 1_000_000];
    if derived == 0 { return 200_000; }
    *KNOWN.iter().min_by_key(|w| (**w as i128 - derived as i128).abs()).unwrap()
}

/// Given the JSONL-observed `context_tokens` and the cached `used_percentage`,
/// derive what window CC was dividing against. Returns None if percentage is
/// unusable (zero / negative / missing) — too noisy at the low end.
pub fn derive_window(context_tokens: u64, used_percentage: f64) -> Option<u64> {
    if used_percentage <= 0.5 { return None; }
    let derived = (context_tokens as f64 / (used_percentage / 100.0)).round() as u64;
    Some(snap_window(derived))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_picks_closest_known_window() {
        assert_eq!(snap_window(987_654), 1_000_000);
        assert_eq!(snap_window(205_000), 200_000);
        assert_eq!(snap_window(599_000), 200_000); // midpoint 600k; 599 closer to 200
        assert_eq!(snap_window(601_000), 1_000_000);
        assert_eq!(snap_window(0), 200_000); // fallback
    }

    #[test]
    fn derive_window_recovers_1m_from_recorded_percent() {
        // The bug-report case: 69,350 tokens at 6.9% should resolve to 1M.
        assert_eq!(derive_window(69_350, 6.9), Some(1_000_000));
    }

    #[test]
    fn derive_window_recovers_200k_from_recorded_percent() {
        // 70k tokens at ~35% means a 200k window.
        assert_eq!(derive_window(69_350, 34.7), Some(200_000));
    }

    #[test]
    fn derive_window_returns_none_on_zero_percent() {
        assert_eq!(derive_window(1_000, 0.0), None);
    }

    #[test]
    fn parse_entry_extracts_fields() {
        let v: serde_json::Value = serde_json::from_str(r#"{
            "session_id":"abc",
            "model":{"id":"claude-opus-4-7[1m]","display_name":"Opus 4.7 (1M)"},
            "context_window":{"used_percentage":6.9},
            "updated_at_ms":1700000000000
        }"#).unwrap();
        let e = parse_entry(&v).unwrap();
        assert_eq!(e.used_percentage, Some(6.9));
        assert_eq!(e.model_id.as_deref(), Some("claude-opus-4-7[1m]"));
        assert_eq!(e.model_display_name.as_deref(), Some("Opus 4.7 (1M)"));
        assert_eq!(e.updated_at_ms, 1700000000000);
    }
}
