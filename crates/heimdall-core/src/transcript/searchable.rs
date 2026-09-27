//! Searchable text extraction with a size cap (TranscriptParser.appendSearchableText).

use serde_json::Value;

/// Per-session ceiling on indexed characters (TranscriptParser.searchableTextCap).
pub const SEARCHABLE_TEXT_CAP: usize = 20_000;

/// Collects the free text of user and assistant records — prose, reasoning, tool names and inputs,
/// tool output — until the cap. Each piece is followed by a space, so the result can pass the cap by one.
#[derive(Debug, Default)]
pub struct Searchable {
    text: String,
    chars: usize,
}

impl Searchable {
    pub fn is_full(&self) -> bool {
        self.chars >= SEARCHABLE_TEXT_CAP
    }

    /// Adds one record's `message` object; `kind` is the record type (`user` or `assistant`).
    pub fn append(&mut self, message: &Value, kind: &str) {
        let content = message.get("content");
        if kind == "user" {
            match content {
                Some(Value::String(s)) => self.push(Some(s)),
                Some(Value::Array(blocks)) => {
                    for block in blocks {
                        match block_type(block) {
                            Some("text") => self.push(str_field(block, "text")),
                            Some("tool_result") => self.push(flatten_tool_result(block.get("content")).as_deref()),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            return;
        }
        let Some(Value::Array(blocks)) = content else { return };
        for block in blocks {
            match block_type(block) {
                Some("text") => self.push(str_field(block, "text")),
                Some("thinking") => self.push(str_field(block, "thinking")),
                Some("tool_use") => {
                    self.push(str_field(block, "name"));
                    if let Some(Value::Object(input)) = block.get("input") {
                        for value in input.values() {
                            self.push(value.as_str());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn push(&mut self, piece: Option<&str>) {
        let Some(piece) = piece.filter(|p| !p.is_empty()) else {
            return;
        };
        if self.is_full() {
            return;
        }
        let remaining = SEARCHABLE_TEXT_CAP - self.chars;
        match piece.char_indices().nth(remaining) {
            Some((cut, _)) => {
                self.text.push_str(&piece[..cut]);
                self.chars += remaining;
            }
            None => {
                self.text.push_str(piece);
                self.chars += piece.chars().count();
            }
        }
        self.text.push(' ');
        self.chars += 1;
    }

    /// The collected text, lower-cased.
    pub fn finish(self) -> String {
        self.text.to_lowercase()
    }
}

fn block_type(block: &Value) -> Option<&str> {
    block.get("type").and_then(Value::as_str)
}

fn str_field<'a>(block: &'a Value, key: &str) -> Option<&'a str> {
    block.get(key).and_then(Value::as_str)
}

/// A tool result's text: a plain string, or its text blocks joined with spaces.
fn flatten_tool_result(content: Option<&Value>) -> Option<String> {
    match content? {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => Some(
            blocks
                .iter()
                .filter(|b| block_type(b) == Some("text"))
                .filter_map(|b| str_field(b, "text"))
                .collect::<Vec<_>>()
                .join(" "),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn indexes_every_kind_of_text_and_lower_cases() {
        let mut s = Searchable::default();
        s.append(&json!({"content": [{"type": "text", "text": "Hi"}, {"type": "tool_result", "content": [{"type": "text", "text": "Out"}]}]}), "user");
        s.append(&json!({"content": [{"type": "thinking", "thinking": "Hmm"}, {"type": "tool_use", "name": "Bash", "input": {"command": "LS", "n": 1}}]}), "assistant");
        assert_eq!(s.finish(), "hi out hmm bash ls ");
    }

    #[test]
    fn stops_at_the_cap_plus_one_separator() {
        let mut s = Searchable::default();
        s.append(&json!({"content": "é".repeat(SEARCHABLE_TEXT_CAP + 50)}), "user");
        s.append(&json!({"content": "more"}), "user");
        assert!(s.is_full());
        assert_eq!(s.finish().chars().count(), SEARCHABLE_TEXT_CAP + 1);
    }
}
