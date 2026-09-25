//! Streaming JSONL reading for transcripts and history (task M1-07).
//!
//! Lines are read one at a time (never the whole file), a trailing `\r` is trimmed so
//! CRLF files parse, blank lines are ignored, and lines that do not parse are skipped and
//! counted. Claude Code may still be appending, so a truncated last line is just one more
//! skipped line.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::marker::PhantomData;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::{AppError, Loaded, Result};

/// The header tier reads transcripts in chunks of this size (TranscriptParser.swift:148).
pub const HEAD_CHUNK_BYTES: usize = 8192;

/// Iterator over the records of a JSONL stream that parse as `T`.
pub struct Records<R, T> {
    reader: R,
    buf: Vec<u8>,
    first_line: bool,
    skipped: u32,
    io_error: Option<std::io::Error>,
    _t: PhantomData<T>,
}

impl<R: BufRead, T: DeserializeOwned> Records<R, T> {
    pub fn new(reader: R) -> Self {
        Self { reader, buf: Vec::new(), first_line: true, skipped: 0, io_error: None, _t: PhantomData }
    }

    /// Lines that were not empty but did not parse as `T`.
    pub fn skipped(&self) -> u32 {
        self.skipped
    }

    /// The I/O error that stopped iteration early, if any.
    pub fn take_io_error(&mut self) -> Option<std::io::Error> {
        self.io_error.take()
    }
}

impl<R: BufRead, T: DeserializeOwned> Iterator for Records<R, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            self.buf.clear();
            match self.reader.read_until(b'\n', &mut self.buf) {
                Ok(0) => return None,
                Ok(_) => {}
                Err(e) => {
                    self.io_error = Some(e);
                    return None;
                }
            }
            let mut line: &[u8] = &self.buf;
            if std::mem::take(&mut self.first_line) {
                line = line.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(line); // UTF-8 byte-order mark
            }
            while let [rest @ .., b'\n' | b'\r'] = line {
                line = rest;
            }
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            match serde_json::from_slice::<T>(line) {
                Ok(v) => return Some(v),
                Err(_) => self.skipped += 1,
            }
        }
    }
}

/// Open `path` for streaming. Missing or unreadable files become an [`AppError`] naming the path.
pub fn open<T: DeserializeOwned>(path: &Path) -> Result<Records<BufReader<File>, T>> {
    let file = File::open(path).map_err(|e| AppError::from_io(&e, path))?;
    Ok(Records::new(BufReader::new(file)))
}

/// Read every record of `path`.
pub fn read_all<T: DeserializeOwned>(path: &Path) -> Result<Loaded<Vec<T>>> {
    let mut records = open::<T>(path)?;
    let values: Vec<T> = records.by_ref().collect();
    if let Some(e) = records.take_io_error() {
        return Err(AppError::from_io(&e, path));
    }
    Ok(Loaded::new(values, records.skipped()))
}

/// Read records in 8 KiB chunks until `max` of them satisfy `qualifies`, then stop reading.
/// Used by the header tier, which looks at the first 50 qualifying lines only.
pub fn read_head<T: DeserializeOwned>(path: &Path, max: usize, mut qualifies: impl FnMut(&T) -> bool) -> Result<Loaded<Vec<T>>> {
    let file = File::open(path).map_err(|e| AppError::from_io(&e, path))?;
    head_from(BufReader::with_capacity(HEAD_CHUNK_BYTES, file), max, &mut qualifies)
        .map_err(|e| AppError::from_io(&e, path))
}

fn head_from<R: Read, T: DeserializeOwned>(
    reader: BufReader<R>,
    max: usize,
    qualifies: &mut impl FnMut(&T) -> bool,
) -> std::io::Result<Loaded<Vec<T>>> {
    let mut records = Records::<_, T>::new(reader);
    let mut values = Vec::new();
    while values.len() < max {
        match records.next() {
            Some(v) if qualifies(&v) => values.push(v),
            Some(_) => {}
            None => break,
        }
    }
    if let Some(e) = records.take_io_error() {
        return Err(e);
    }
    Ok(Loaded::new(values, records.skipped()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::{Cursor, Write};

    #[derive(Debug, Deserialize, PartialEq)]
    struct Rec {
        n: u32,
    }

    fn parse(bytes: &[u8]) -> (Vec<u32>, u32) {
        let mut r = Records::<_, Rec>::new(Cursor::new(bytes.to_vec()));
        let v = r.by_ref().map(|x| x.n).collect();
        (v, r.skipped())
    }

    #[test]
    fn reads_lf_crlf_and_mixed_endings() {
        assert_eq!(parse(b"{\"n\":1}\n{\"n\":2}\n"), (vec![1, 2], 0));
        assert_eq!(parse(b"{\"n\":1}\r\n{\"n\":2}\r\n"), (vec![1, 2], 0));
        assert_eq!(parse(b"{\"n\":1}\r\n{\"n\":2}\n{\"n\":3}"), (vec![1, 2, 3], 0));
    }

    #[test]
    fn ignores_blank_lines_and_a_byte_order_mark() {
        assert_eq!(parse(b"\xEF\xBB\xBF{\"n\":1}\n\n  \r\n{\"n\":2}\n"), (vec![1, 2], 0));
    }

    #[test]
    fn skips_and_counts_invalid_and_truncated_lines() {
        assert_eq!(parse(b"{\"n\":1}\nnot json\n{\"n\":2}\n{\"n\":"), (vec![1, 2], 2));
        assert_eq!(parse(b"{\"x\":1}\n{\"n\":2}\n"), (vec![2], 1), "wrong shape counts as skipped");
    }

    #[test]
    fn read_head_stops_after_max_qualifying_records() {
        let mut text = String::new();
        for i in 0..1000 {
            text.push_str(&format!("{{\"n\":{i}}}\n"));
        }
        let loaded = head_from(BufReader::with_capacity(HEAD_CHUNK_BYTES, Cursor::new(text)), 50, &mut |r: &Rec| r.n.is_multiple_of(2)).unwrap();
        assert_eq!(loaded.value.len(), 50);
        assert_eq!(loaded.value.last(), Some(&Rec { n: 98 }));
    }

    #[test]
    fn streams_a_large_file_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.jsonl");
        let mut f = File::create(&path).unwrap();
        let pad = "x".repeat(1000);
        for i in 0..5000 {
            writeln!(f, "{{\"n\":{i},\"pad\":\"{pad}\"}}").unwrap(); // about 5 MB
        }
        drop(f);
        let mut records = open::<Rec>(&path).unwrap();
        assert_eq!(records.by_ref().count(), 5000);
        assert_eq!(records.skipped(), 0);
    }

    #[test]
    fn missing_file_is_not_found_with_path() {
        let err = read_all::<Rec>(Path::new("definitely/missing.jsonl")).unwrap_err();
        assert!(matches!(err, AppError::NotFound { path: Some(_), .. }));
    }
}
