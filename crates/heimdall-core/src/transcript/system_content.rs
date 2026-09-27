//! System-reminder segment parsing (Utilities/SystemContentParser.swift): user text is split around
//! `<tag>…</tag>` blocks, where a tag name is a letter followed by word characters or `-`.

use crate::models::message::{SystemSegment, SystemSegmentKind};

use super::tool_input::beautify;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// Text outside the tags, each piece trimmed, joined with a blank line.
    pub user_text: String,
    pub blocks: Vec<SystemBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemBlock {
    pub tag: String,
    /// Trimmed tag content.
    pub content: String,
}

/// Same matches as the Swift pattern `<([a-zA-Z][\w-]*)>([\s\S]*?)</\1>`: leftmost opening tag,
/// nearest closing tag of the same name.
pub fn parse(text: &str) -> Parsed {
    let mut blocks = Vec::new();
    let mut parts: Vec<&str> = Vec::new();
    let mut last = 0;
    let mut i = 0;
    while let Some(offset) = text[i..].find('<') {
        let start = i + offset;
        if let Some((tag, content_start)) = open_tag(text, start) {
            let close = format!("</{tag}>");
            if let Some(len) = text[content_start..].find(&close) {
                let before = text[last..start].trim();
                if !before.is_empty() {
                    parts.push(before);
                }
                blocks.push(SystemBlock {
                    tag: tag.to_string(),
                    content: text[content_start..content_start + len].trim().to_string(),
                });
                last = content_start + len + close.len();
                i = last;
                continue;
            }
        }
        i = start + 1;
    }
    if blocks.is_empty() {
        return Parsed {
            user_text: text.to_string(),
            blocks,
        };
    }
    let after = text[last..].trim();
    if !after.is_empty() {
        parts.push(after);
    }
    Parsed {
        user_text: parts.join("\n\n"),
        blocks,
    }
}

/// The tag name starting at the `<` at `start`, and the byte offset just past its `>`.
fn open_tag(text: &str, start: usize) -> Option<(&str, usize)> {
    let rest = &text[start + 1..];
    let mut chars = rest.char_indices();
    if !chars.next()?.1.is_ascii_alphabetic() {
        return None;
    }
    for (idx, c) in chars {
        if c == '>' {
            return Some((&rest[..idx], start + 1 + idx + 1));
        }
        if !(c.is_alphanumeric() || c == '_' || c == '-') {
            return None;
        }
    }
    None
}

/// The `system` field of a user text block: `None` when the text has no tags. Otherwise the user
/// text first (when not empty), then one segment per tag with its content beautified; a tag whose
/// content holds more tags also lists those as children, as SystemBlockContentView renders them.
pub fn segments(text: &str) -> Option<Vec<SystemSegment>> {
    let parsed = parse(text);
    if parsed.blocks.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    if !parsed.user_text.is_empty() {
        out.push(plain(&parsed.user_text));
    }
    for block in parsed.blocks {
        let inner = parse(&block.content);
        let children = (!inner.blocks.is_empty()).then(|| {
            let lead = (!inner.user_text.is_empty()).then(|| plain(&beautify(&inner.user_text)));
            lead.into_iter()
                .chain(inner.blocks.iter().map(|b| tagged(&b.tag, beautify(&b.content), None)))
                .collect()
        });
        out.push(tagged(&block.tag, beautify(&block.content), children));
    }
    Some(out)
}

fn plain(text: &str) -> SystemSegment {
    SystemSegment {
        kind: SystemSegmentKind::Text,
        tag: None,
        text: text.to_string(),
        children: None,
    }
}

fn tagged(tag: &str, text: String, children: Option<Vec<SystemSegment>>) -> SystemSegment {
    let kind = match tag {
        "system-reminder" => SystemSegmentKind::SystemReminder,
        "command-message" | "command-name" | "command-args" => SystemSegmentKind::CommandMessage,
        _ => SystemSegmentKind::Other,
    };
    SystemSegment {
        kind,
        tag: Some(tag.to_string()),
        text,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(p: &Parsed) -> Vec<&str> {
        p.blocks.iter().map(|b| b.tag.as_str()).collect()
    }

    // Ported from PoirotTests/Utilities/SystemContentParserTests.swift.
    #[test]
    fn parse_plain_text_returns_full_text_no_blocks() {
        let r = parse("Hello, can you fix this bug?");
        assert_eq!(r.user_text, "Hello, can you fix this bug?");
        assert!(r.blocks.is_empty());
    }
    #[test]
    fn parse_single_xml_tag_extracts_block() {
        let r = parse("Fix the login flow\n<system-reminder>You are a helpful assistant</system-reminder>");
        assert_eq!(r.user_text, "Fix the login flow");
        assert_eq!(
            r.blocks,
            [SystemBlock {
                tag: "system-reminder".into(),
                content: "You are a helpful assistant".into()
            }]
        );
    }
    #[test]
    fn parse_multiple_xml_tags_extracts_all_blocks() {
        let r = parse("Please help\n<system-reminder>Context A</system-reminder>\n<context>Context B</context>");
        assert_eq!(r.user_text, "Please help");
        assert_eq!(tags(&r), ["system-reminder", "context"]);
    }
    #[test]
    fn parse_text_between_tags_preserves_all_user_text() {
        let r = parse("First part\n<tag1>System 1</tag1>\nMiddle part\n<tag2>System 2</tag2>\nLast part");
        assert_eq!(r.user_text, "First part\n\nMiddle part\n\nLast part");
        assert_eq!(r.blocks.len(), 2);
    }
    #[test]
    fn parse_only_xml_tags_returns_empty_user_text() {
        let r = parse("<system-reminder>Only system content</system-reminder>");
        assert!(r.user_text.is_empty());
        assert_eq!(r.blocks[0].content, "Only system content");
    }
    #[test]
    fn parse_empty_input_returns_empty_result() {
        let r = parse("");
        assert!(r.user_text.is_empty() && r.blocks.is_empty());
    }
    #[test]
    fn parse_unmatched_tags_treats_as_plain_text() {
        let r = parse("Some <unclosed text here");
        assert_eq!(r.user_text, "Some <unclosed text here");
        assert!(r.blocks.is_empty());
    }
    #[test]
    fn parse_multiline_tag_content_preserves_content() {
        let r = parse("Help me\n<system-reminder>\nLine 1\nLine 2\nLine 3\n</system-reminder>");
        assert_eq!(r.user_text, "Help me");
        assert_eq!(r.blocks[0].content, "Line 1\nLine 2\nLine 3");
    }
    #[test]
    fn parse_trims_whitespace_in_system_content() {
        assert_eq!(
            parse("<tag>  content with spaces  </tag>").blocks[0].content,
            "content with spaces"
        );
    }

    #[test]
    fn nearest_closing_tag_wins_and_other_tags_are_skipped() {
        let r = parse("a <b>x</b> <c>y</c> <b>z</b> <1>no</1>");
        assert_eq!(tags(&r), ["b", "c", "b"]);
        assert_eq!(r.user_text, "a\n\n<1>no</1>");
    }

    #[test]
    fn segments_classify_tags_and_nest_one_level() {
        assert!(segments("plain").is_none());
        let s = segments(
            "run it <command-name>/test</command-name><system-reminder>A <file>{\"b\":1}</file></system-reminder>",
        )
        .unwrap();
        assert_eq!(s[0].kind, SystemSegmentKind::Text);
        assert_eq!(s[1].kind, SystemSegmentKind::CommandMessage);
        assert_eq!(s[2].kind, SystemSegmentKind::SystemReminder);
        let children = s[2].children.as_ref().unwrap();
        assert_eq!(
            (children[0].text.as_str(), children[1].tag.as_deref()),
            ("A", Some("file"))
        );
        assert_eq!(children[1].text, "{\n  \"b\": 1\n}");
    }
}
