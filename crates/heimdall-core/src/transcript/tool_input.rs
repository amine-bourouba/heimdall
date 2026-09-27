//! Tool input pretty-printing (Utilities/JSONBeautifier.swift).

use serde_json::Value;

/// Pretty-prints `s` with sorted keys when it is a JSON object or array (also when that JSON is
/// wrapped in a JSON string); anything else comes back unchanged.
pub fn beautify(s: &str) -> String {
    let mut trimmed = s.trim().to_string();
    if trimmed.starts_with('"')
        && trimmed.ends_with('"')
        && trimmed.chars().count() > 2
        && let Ok(Value::String(inner)) = serde_json::from_str(&trimmed)
    {
        trimmed = inner;
    }
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return s.to_string();
    }
    match serde_json::from_str::<Value>(&trimmed) {
        Ok(v) => pretty(&v),
        Err(_) => s.to_string(),
    }
}

/// Two-space indented JSON with keys sorted at every level.
pub fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(&sorted(v)).unwrap_or_default()
}

/// Compact JSON with sorted keys (TranscriptParser.stringifyValue for arrays and objects).
pub fn compact(v: &Value) -> String {
    serde_json::to_string(&sorted(v)).unwrap_or_default()
}

/// Rebuilds maps in key order, so output is sorted whether or not serde_json preserves insertion order.
fn sorted(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut entries: Vec<_> = m.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            Value::Object(entries.into_iter().map(|(k, v)| (k.clone(), sorted(v))).collect())
        }
        Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from PoirotTests/Utilities/JSONBeautifierTests.swift. Deviation: serde_json writes
    // `"key": value` where Foundation wrote `"key" : value`, so the expected strings drop that space.
    #[test]
    fn beautify_valid_object_pretty_prints() {
        let r = beautify(r#"{"b":1,"a":2}"#);
        assert!(r.contains(r#""a": 2"#) && r.contains(r#""b": 1"#));
    }
    #[test]
    fn beautify_valid_array_pretty_prints() {
        let r = beautify("[1,2,3]");
        assert!(r.contains('1') && r.contains('2') && r.contains('3') && r.contains('\n'));
    }
    #[test]
    fn beautify_non_json_returns_original() {
        assert_eq!(beautify("just plain text"), "just plain text");
    }
    #[test]
    fn beautify_doubly_encoded_decodes_and_formats() {
        let doubly = serde_json::to_string(r#"{"key":"value"}"#).unwrap();
        assert!(beautify(&doubly).contains(r#""key": "value""#));
    }
    #[test]
    fn beautify_empty_string_returns_original() {
        assert_eq!(beautify(""), "");
    }
    #[test]
    fn beautify_whitespace_only_returns_original() {
        assert_eq!(beautify("   \n  "), "   \n  ");
    }
    #[test]
    fn beautify_sorted_keys_verified() {
        let r = beautify(r#"{"zebra":1,"apple":2,"mango":3}"#);
        let keys: Vec<_> = r
            .lines()
            .filter(|l| l.contains(':'))
            .map(|l| l.trim().replace('"', "").split(':').next().unwrap().to_string())
            .collect();
        assert_eq!(keys, ["apple", "mango", "zebra"]);
    }
}
