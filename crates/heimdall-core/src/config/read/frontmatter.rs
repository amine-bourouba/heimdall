//! Markdown frontmatter parser (Services/Config/FrontmatterParser.swift): a `---` block of
//! `key: value` lines at the top of a command, skill, agent or output-style file. Not full YAML:
//! the only multi-line form is a `>` (folded) or `|` (literal) block scalar with indented lines.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub metadata: BTreeMap<String, String>,
    pub body: String,
}

/// Splits `content` into metadata and body. Without an opening and a closing `---`, the metadata
/// is empty and the body is `content` unchanged. CRLF files parse like LF files.
pub fn parse(content: &str) -> Frontmatter {
    let unchanged = || Frontmatter {
        metadata: BTreeMap::new(),
        body: content.to_string(),
    };
    let Some(after_opening) = content.trim().strip_prefix("---") else {
        return unchanged();
    };
    let Some(close) = after_opening.find("\n---") else {
        return unchanged();
    };
    let yaml = &after_opening[..close];
    let body = after_opening[close + "\n---".len()..].trim().to_string();

    let mut metadata = BTreeMap::new();
    let mut block: Option<(String, bool, Vec<String>)> = None; // (key, folded, lines)
    let flush = |block: &mut Option<(String, bool, Vec<String>)>, metadata: &mut BTreeMap<String, String>| {
        if let Some((key, folded, lines)) = block.take() {
            metadata.insert(key, lines.join(if folded { " " } else { "\n" }));
        }
    };
    for line in yaml.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)) {
        let trimmed = line.trim_matches([' ', '\t']);
        // An indented line under `>` or `|` continues the block (FrontmatterParser.swift:32-36).
        let continues = block.is_some()
            && !trimmed.is_empty()
            && line.starts_with(char::is_whitespace)
            && (!trimmed.contains(':') || line.starts_with("  "));
        if continues {
            if let Some((_, _, lines)) = &mut block {
                lines.push(trimmed.to_string());
            }
            continue;
        }
        flush(&mut block, &mut metadata);
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let (key, value) = (
            key.trim_matches([' ', '\t']).to_string(),
            value.trim_matches([' ', '\t']),
        );
        match value {
            ">" | "|" => block = Some((key, value == ">", Vec::new())),
            _ => {
                metadata.insert(key, value.to_string());
            }
        }
    }
    flush(&mut block, &mut metadata);
    Frontmatter { metadata, body }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta<'a>(f: &'a Frontmatter, key: &str) -> Option<&'a str> {
        f.metadata.get(key).map(String::as_str)
    }

    // Ported from PoirotTests/Services/FrontmatterParserTests.swift.
    #[test]
    fn parse_no_frontmatter_returns_original_body() {
        let r = parse("Just some plain text");
        assert!(r.metadata.is_empty());
        assert_eq!(r.body, "Just some plain text");
    }
    #[test]
    fn parse_unclosed_frontmatter_returns_original_body() {
        let input = "---\nkey: value\nno closing marker";
        assert_eq!(
            parse(input),
            Frontmatter {
                metadata: BTreeMap::new(),
                body: input.into()
            }
        );
    }
    #[test]
    fn parse_well_formed_extracts_metadata() {
        let r = parse("---\ntitle: Hello World\n---\nBody content here");
        assert_eq!(
            (meta(&r, "title"), r.body.as_str()),
            (Some("Hello World"), "Body content here")
        );
    }
    #[test]
    fn parse_colon_in_value_preserves_full_value() {
        assert_eq!(
            meta(&parse("---\ndescription: foo: bar\n---\nBody"), "description"),
            Some("foo: bar")
        );
    }
    #[test]
    fn parse_empty_lines_in_yaml_skipped() {
        let r = parse("---\nkey1: value1\n\nkey2: value2\n---\nBody");
        assert_eq!(
            (r.metadata.len(), meta(&r, "key1"), meta(&r, "key2")),
            (2, Some("value1"), Some("value2"))
        );
    }
    #[test]
    fn parse_body_trimmed_whitespace_removed() {
        assert_eq!(
            parse("---\nkey: val\n---\n  Body with surrounding whitespace\n").body,
            "Body with surrounding whitespace"
        );
    }
    #[test]
    fn parse_multiple_keys_all_extracted() {
        let r = parse("---\ntitle: Test\nauthor: Alice\nversion: 3\n---\nContent");
        assert_eq!(
            (
                r.metadata.len(),
                meta(&r, "title"),
                meta(&r, "author"),
                meta(&r, "version")
            ),
            (3, Some("Test"), Some("Alice"), Some("3"))
        );
    }
    #[test]
    fn parse_empty_body_after_frontmatter() {
        let r = parse("---\nkey: value\n---");
        assert_eq!((meta(&r, "key"), r.body.as_str()), (Some("value"), ""));
    }
    #[test]
    fn parse_folded_block_scalar_joins_with_spaces() {
        let r = parse("---\nname: my-skill\ndescription: >\n  Multi-line description\n  goes here\n---\nBody");
        assert_eq!(
            (meta(&r, "name"), meta(&r, "description"), r.body.as_str()),
            (Some("my-skill"), Some("Multi-line description goes here"), "Body")
        );
    }
    #[test]
    fn parse_literal_block_scalar_joins_with_newlines() {
        let r = parse("---\nname: my-skill\ndescription: |\n  Line one\n  Line two\n---\nBody");
        assert_eq!(
            (meta(&r, "name"), meta(&r, "description")),
            (Some("my-skill"), Some("Line one\nLine two"))
        );
    }
    #[test]
    fn parse_block_scalar_followed_by_key_both_extracted() {
        let r = parse("---\ndescription: >\n  A long\n  description\nmodel: opus\n---\nBody");
        assert_eq!(
            (meta(&r, "description"), meta(&r, "model")),
            (Some("A long description"), Some("opus"))
        );
    }
    #[test]
    fn parse_block_scalar_at_end_extracted() {
        let r = parse("---\nname: test\nallowed-tools: >\n  Bash, Read,\n  Glob, Grep\n---\nBody");
        assert_eq!(
            (meta(&r, "name"), meta(&r, "allowed-tools")),
            (Some("test"), Some("Bash, Read, Glob, Grep"))
        );
    }

    #[test]
    fn crlf_files_parse_like_lf_files() {
        let lf =
            "---\nname: my-skill\ndescription: >\n  Multi-line description\n  goes here\nmodel: opus\n---\nBody\nmore";
        let crlf = lf.replace('\n', "\r\n");
        let (a, b) = (parse(lf), parse(&crlf));
        assert_eq!(a.metadata, b.metadata);
        assert_eq!(b.body, "Body\r\nmore");
    }
}
