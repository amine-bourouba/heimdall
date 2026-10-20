//! The only way Heimdall changes a Claude Code JSON file (ADR 0009). The Swift reference app fell back to an empty
//! document when a file did not parse and wrote that back, wiping the user's configuration.
//!
//! [`JsonDoc`] fails closed on a file that does not parse, keeps key order (serde_json
//! `preserve_order`), unknown keys, indentation, line endings, a byte-order mark and the trailing
//! newline, backs the file up once to `<file>.heimdall.bak`, refuses to write if the file changed
//! since it was read, and replaces it atomically ([`super::atomic`]).

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Map, Value};

use super::atomic;
use crate::error::{AppError, Result, WriteOutcome};

const BOM: &[u8] = b"\xEF\xBB\xBF";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Style {
    /// `None` writes compact JSON.
    indent: Option<Vec<u8>>,
    crlf: bool,
    trailing_newline: bool,
    bom: bool,
}

impl Style {
    /// Two spaces and a trailing newline, like Claude Code's own files.
    fn new_file() -> Self {
        Self {
            indent: Some(b"  ".to_vec()),
            crlf: false,
            trailing_newline: true,
            bom: false,
        }
    }

    fn detect(bytes: &[u8]) -> Self {
        let indent = bytes
            .split(|b| *b == b'\n')
            .skip(1)
            .map(|line| {
                line.iter()
                    .take_while(|b| **b == b' ' || **b == b'\t')
                    .copied()
                    .collect::<Vec<u8>>()
            })
            .find(|ws| !ws.is_empty());
        let has_newline_inside = bytes.trim_ascii_end().contains(&b'\n');
        Self {
            indent: if has_newline_inside {
                Some(indent.unwrap_or_else(|| b"  ".to_vec()))
            } else {
                None
            },
            crlf: bytes.windows(2).any(|w| w == b"\r\n"),
            trailing_newline: bytes.ends_with(b"\n"),
            bom: bytes.starts_with(BOM),
        }
    }
}

#[derive(Debug)]
pub struct JsonDoc {
    path: PathBuf,
    value: Value,
    style: Style,
    /// The bytes on disk when the document was read; `None` for a file that did not exist.
    original: Option<Vec<u8>>,
}

impl JsonDoc {
    /// Reads and parses `path`. A missing file is an empty object when `create_if_missing` is set
    /// (a project's first `.claude/settings.json`), else `NotFound`. A file that does not parse is
    /// `AppError::Parse`, and nothing will be written.
    pub fn open(path: &Path, create_if_missing: bool) -> Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let body = bytes.strip_prefix(BOM).unwrap_or(&bytes);
                let value: Value = serde_json::from_slice(body).map_err(|e| {
                    AppError::parse(format!("not valid JSON, so it was left untouched: {e}")).with_path(path)
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                    value,
                    style: Style::detect(&bytes),
                    original: Some(bytes),
                })
            }
            Err(e) if e.kind() == ErrorKind::NotFound && create_if_missing => Ok(Self {
                path: path.to_path_buf(),
                value: Value::Object(Map::new()),
                style: Style::new_file(),
                original: None,
            }),
            Err(e) => Err(AppError::from_io(&e, path)),
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// The top-level object; a file whose top level is not an object is a parse error.
    pub fn root_mut(&mut self) -> Result<&mut Map<String, Value>> {
        let path = self.path.clone();
        self.value
            .as_object_mut()
            .ok_or_else(|| AppError::parse("the top level is not a JSON object").with_path(path))
    }

    /// The document as it would be written, in the file's own style.
    pub fn render(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        if self.style.bom {
            out.extend_from_slice(BOM);
        }
        let json = match &self.style.indent {
            Some(indent) => {
                let mut ser = Serializer::with_formatter(Vec::new(), PrettyFormatter::with_indent(indent));
                self.value
                    .serialize(&mut ser)
                    .map_err(|e| AppError::parse(e.to_string()))?;
                ser.into_inner()
            }
            None => serde_json::to_vec(&self.value).map_err(|e| AppError::parse(e.to_string()))?,
        };
        let newline: &[u8] = if self.style.crlf { b"\r\n" } else { b"\n" };
        for (i, line) in json.split(|b| *b == b'\n').enumerate() {
            if i > 0 {
                out.extend_from_slice(newline);
            }
            out.extend_from_slice(line);
        }
        if self.style.trailing_newline {
            out.extend_from_slice(newline);
        }
        Ok(out)
    }

    /// Writes the document if it changed. Before the first write the original is copied to
    /// `<file>.heimdall.bak` (an existing backup is never replaced). If the file changed on disk
    /// since it was read, nothing is written and the error asks for a reload.
    pub fn save(&mut self) -> Result<WriteOutcome> {
        let bytes = self.render()?;
        if self.original.as_deref() == Some(bytes.as_slice()) {
            return Ok(WriteOutcome::Unchanged);
        }
        let current = match std::fs::read(&self.path) {
            Ok(b) => Some(b),
            Err(e) if e.kind() == ErrorKind::NotFound => None,
            Err(e) => return Err(AppError::from_io(&e, &self.path)),
        };
        if current != self.original {
            return Err(
                AppError::busy("the file changed since it was read; reload and try again").with_path(&self.path),
            );
        }
        if let Some(original) = &self.original {
            back_up_once(&self.path, original)?;
        }
        atomic::replace(&self.path, &bytes)?;
        self.original = Some(bytes);
        Ok(WriteOutcome::Written)
    }
}

pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".heimdall.bak");
    path.with_file_name(name)
}

fn back_up_once(path: &Path, original: &[u8]) -> Result<()> {
    let backup = backup_path(path);
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&backup) {
        Ok(mut f) => std::io::Write::write_all(&mut f, original)
            .and_then(|()| f.sync_all())
            .map_err(|e| AppError::from_io(&e, &backup)),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(AppError::from_io(&e, &backup)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(rel: &str) -> Vec<u8> {
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures").join(rel)).unwrap()
    }
    fn file_with(bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".claude.json");
        std::fs::write(&p, bytes).unwrap();
        (d, p)
    }

    #[test]
    fn a_corrupt_file_is_a_parse_error_and_is_left_alone() {
        let corrupt = fixture("synthetic/corrupt/claude.json");
        let (d, p) = file_with(&corrupt);
        let err = JsonDoc::open(&p, true).unwrap_err();
        assert!(matches!(err, AppError::Parse { path: Some(_), .. }), "{err:?}");
        assert_eq!(std::fs::read(&p).unwrap(), corrupt);
        assert!(!backup_path(&p).exists());
        assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 1);
    }

    #[test]
    fn the_real_claude_json_round_trips_byte_for_byte() {
        let original = fixture("windows/claude.json");
        let (_d, p) = file_with(&original);
        let mut doc = JsonDoc::open(&p, false).unwrap();
        let rendered = doc.render().unwrap();
        if let Some(i) = rendered.iter().zip(&original).position(|(a, b)| a != b) {
            let ctx = |v: &[u8]| String::from_utf8_lossy(&v[i.saturating_sub(80)..(i + 80).min(v.len())]).into_owned();
            panic!(
                "differs at byte {i}:
--- rendered
{}
--- original
{}",
                ctx(&rendered),
                ctx(&original)
            );
        }
        assert_eq!(rendered.len(), original.len());
        assert_eq!(doc.save().unwrap(), WriteOutcome::Unchanged);
    }

    #[test]
    fn changing_one_key_changes_only_its_bytes() {
        let original = fixture("windows/claude.json");
        let (_d, p) = file_with(&original);
        let mut doc = JsonDoc::open(&p, false).unwrap();
        let before = doc.root_mut().unwrap()["numStartups"].as_u64().unwrap();
        doc.root_mut().unwrap()["numStartups"] = json!(before + 1);
        assert_eq!(doc.save().unwrap(), WriteOutcome::Written);
        let (old, new) = (
            String::from_utf8(original).unwrap(),
            std::fs::read_to_string(&p).unwrap(),
        );
        let changed: Vec<_> = old.lines().zip(new.lines()).filter(|(a, b)| a != b).collect();
        assert_eq!(old.lines().count(), new.lines().count());
        assert_eq!(
            changed,
            [(
                format!("  \"numStartups\": {before},").as_str(),
                format!("  \"numStartups\": {},", before + 1).as_str()
            )]
        );
    }

    #[test]
    fn the_backup_is_made_once_and_never_replaced() {
        let (_d, p) = file_with(b"{\n  \"a\": 1\n}\n");
        let mut doc = JsonDoc::open(&p, false).unwrap();
        doc.root_mut().unwrap().insert("a".into(), json!(2));
        doc.save().unwrap();
        doc.root_mut().unwrap().insert("a".into(), json!(3));
        doc.save().unwrap();
        assert_eq!(std::fs::read(backup_path(&p)).unwrap(), b"{\n  \"a\": 1\n}\n");
        assert_eq!(std::fs::read(&p).unwrap(), b"{\n  \"a\": 3\n}\n");
    }

    #[test]
    fn indentation_line_endings_bom_and_trailing_newline_are_kept() {
        let cases: [&[u8]; 6] = [
            b"{\n  \"b\": 1,\n  \"a\": [\n    1\n  ]\n}\n",
            b"{\n    \"b\": 1,\n    \"a\": [\n        1\n    ]\n}",
            b"{\n\t\"b\": 1,\n\t\"a\": [\n\t\t1\n\t]\n}\n",
            b"{\r\n  \"b\": 1,\r\n  \"a\": [\r\n    1\r\n  ]\r\n}\r\n",
            b"{\"b\":1,\"a\":[1]}",
            b"\xEF\xBB\xBF{\n  \"b\": 1,\n  \"a\": [\n    1\n  ]\n}\n",
        ];
        for original in cases {
            let (_d, p) = file_with(original);
            let mut doc = JsonDoc::open(&p, false).unwrap();
            assert_eq!(doc.render().unwrap(), original, "{}", String::from_utf8_lossy(original));
            doc.root_mut().unwrap().insert("b".into(), json!(2));
            doc.save().unwrap();
            let expected = String::from_utf8_lossy(original)
                .replace("\"b\": 1", "\"b\": 2")
                .replace("\"b\":1", "\"b\":2");
            assert_eq!(String::from_utf8_lossy(&std::fs::read(&p).unwrap()), expected);
        }
    }

    #[test]
    fn a_missing_file_is_created_only_when_allowed() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        assert!(matches!(
            JsonDoc::open(&p, false).unwrap_err(),
            AppError::NotFound { .. }
        ));
        let mut doc = JsonDoc::open(&p, true).unwrap();
        doc.root_mut().unwrap().insert("model".into(), json!("opus"));
        assert_eq!(doc.save().unwrap(), WriteOutcome::Written);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "{\n  \"model\": \"opus\"\n}\n");
        assert!(!backup_path(&p).exists());
    }

    #[test]
    fn a_file_changed_since_it_was_read_is_not_overwritten() {
        let (_d, p) = file_with(b"{\"a\":1}");
        let mut doc = JsonDoc::open(&p, false).unwrap();
        std::fs::write(&p, b"{\"a\":1,\"added\":\"by Claude Code\"}").unwrap();
        doc.root_mut().unwrap().insert("a".into(), json!(2));
        assert!(matches!(doc.save().unwrap_err(), AppError::Busy { .. }));
        assert_eq!(std::fs::read(&p).unwrap(), b"{\"a\":1,\"added\":\"by Claude Code\"}");
    }

    #[test]
    fn a_top_level_that_is_not_an_object_cannot_be_edited() {
        let (_d, p) = file_with(b"[1, 2]");
        let mut doc = JsonDoc::open(&p, false).unwrap();
        assert!(matches!(doc.root_mut().unwrap_err(), AppError::Parse { .. }));
    }
}
