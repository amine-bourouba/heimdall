//! Load-time benchmark against the budgets in docs/port-plan.md §9 (task M1-57).
//!
//! Usage: `cargo run --release -p heimdall-core --example bench_load -- <claude_home>`
//! `<claude_home>` is a directory containing `projects/` (e.g. `fixtures/windows/claude-home`).
//! Always also benchmarks a freshly generated tree of 30 projects x 500 sessions in a temp
//! directory, regardless of the argument, per this task's own acceptance criterion.
//!
//! "Warm" here means a same-process repeat call, not heimdall-core's own cache — heimdall-core
//! has none (ADR 0002: it never depends on tauri, and the mtime/size-validated `Cache` that
//! actually backs "cached reload" in the real app lives in `src-tauri/src/state.rs`, M1-38). This
//! benchmark can only measure what re-reading the same files immediately affords from the OS's
//! own filesystem cache — see docs/perf-baseline.md for what that does and doesn't tell us.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery;
use heimdall_core::transcript::full::{self, ParseContext};

fn main() {
    let arg = std::env::args().nth(1);

    if let Some(path) = arg {
        let claude_home = PathBuf::from(path);
        println!("== {} ==", claude_home.display());
        bench_claude_home(&claude_home);
        println!();
    } else {
        println!("(no <claude_home> argument given — skipping the real-fixture benchmark)");
        println!();
    }

    println!("== generated tree: 30 projects x 500 sessions ==");
    let tmp = tempfile::tempdir().expect("create temp dir for the generated tree");
    let projects_dir = generate_tree(tmp.path(), 30, 500);
    bench_discover(&projects_dir);
    println!();

    // No real fixture or the tree above reaches anywhere near 50 MB (largest real transcript
    // available is under 1 MB), so the "full parse of a 50 MB transcript <= 2s" budget has no
    // real data point without this — generated separately from the 30x500 tree since it targets
    // a different budget (single large file, not many small ones).
    println!("== generated single transcript: ~50 MB ==");
    let big_path = tmp.path().join("bench-50mb.jsonl");
    let bytes = generate_large_transcript(&big_path, 50 * 1024 * 1024);
    let ctx = ParseContext {
        project_path: "/bench",
        session_id: "bench-50mb",
        index_started_at: None,
        is_sidechain: false,
    };
    let (loaded, elapsed) = time(|| full::parse(&big_path, ctx).expect("full parse should not error"));
    println!(
        "  full parse of {bytes} bytes: {elapsed:?} ({} messages, {} skipped)",
        loaded.value.map_or(0, |s| s.messages.len()),
        loaded.skipped
    );
}

fn bench_claude_home(claude_home: &Path) {
    let projects_dir = claude_home.join("projects");
    bench_discover(&projects_dir);

    match largest_transcript(&projects_dir) {
        Some((path, bytes)) => {
            let ctx = ParseContext {
                project_path: "/bench",
                session_id: "bench",
                index_started_at: None,
                is_sidechain: false,
            };
            let start = Instant::now();
            let loaded = full::parse(&path, ctx).expect("full parse should not error");
            let elapsed = start.elapsed();
            println!(
                "  full parse of largest transcript ({bytes} bytes, {}): {elapsed:?} ({} messages, {} skipped)",
                path.display(),
                loaded.value.map_or(0, |s| s.messages.len()),
                loaded.skipped
            );
        }
        None => println!("  no .jsonl transcripts found under {}", projects_dir.display()),
    }
}

fn bench_discover(projects_dir: &Path) {
    let cold = time(|| discovery::discover(projects_dir, TargetOs::current()).expect("discover should not error"));
    let warm = time(|| discovery::discover(projects_dir, TargetOs::current()).expect("discover should not error"));
    let (project_count, session_count) = {
        let loaded = discovery::discover(projects_dir, TargetOs::current()).unwrap();
        (
            loaded.value.len(),
            loaded.value.iter().map(|p| p.sessions.len()).sum::<usize>(),
        )
    };
    println!("  discover(): {project_count} projects, {session_count} sessions");
    println!("  cold: {:?}   warm (same-process repeat): {:?}", cold.1, warm.1);
}

fn time<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = f();
    (value, start.elapsed())
}

/// The largest `.jsonl` file anywhere under `projects_dir` (main sessions, agent files and
/// sub-agent transcripts alike) — whichever real transcript this `claude_home` actually has, not
/// a synthetic stand-in, so this number is only ever as meaningful as the fixture it is run
/// against (see docs/perf-baseline.md).
fn largest_transcript(projects_dir: &Path) -> Option<(PathBuf, u64)> {
    fn walk(dir: &Path, best: &mut Option<(PathBuf, u64)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, best);
            } else if path.extension().is_some_and(|e| e == "jsonl")
                && let Ok(meta) = entry.metadata()
                && best.as_ref().is_none_or(|(_, size)| meta.len() > *size)
            {
                *best = Some((path, meta.len()));
            }
        }
    }
    let mut best = None;
    walk(projects_dir, &mut best);
    best
}

/// A single transcript file, one growing conversation's worth of user/assistant turn pairs,
/// written until at least `min_bytes` — for the "full parse of a 50 MB transcript" budget, which
/// no committed fixture or the 30x500 tree's own small per-session files comes anywhere near.
fn generate_large_transcript(path: &Path, min_bytes: u64) -> u64 {
    use std::io::Write;
    let file = std::fs::File::create(path).unwrap();
    let mut writer = std::io::BufWriter::new(file);
    let mut turn = 0u64;
    let mut written = 0u64;
    while written < min_bytes {
        let ts = format!("2026-01-01T00:{:02}:{:02}.000Z", (turn / 60) % 60, turn % 60);
        let line1 = format!(
            r#"{{"type":"user","uuid":"u{turn}","timestamp":"{ts}","isSidechain":false,"message":{{"role":"user","content":"Bench prompt {turn} with a little more text to be realistic-sized."}}}}"#
        );
        let line2 = format!(
            r#"{{"type":"assistant","uuid":"a{turn}","timestamp":"{ts}","isSidechain":false,"message":{{"id":"m{turn}","role":"assistant","model":"claude-opus-4-6","content":[{{"type":"text","text":"Bench reply {turn}, also padded a bit so this file grows at a realistic rate per turn."}}],"usage":{{"input_tokens":100,"output_tokens":50}}}}}}"#
        );
        written += (line1.len() + line2.len() + 2) as u64;
        writeln!(writer, "{line1}").unwrap();
        writeln!(writer, "{line2}").unwrap();
        turn += 1;
    }
    writer.flush().unwrap();
    written
}

/// A minimal real-shaped project tree: each session is a two-line JSONL (one user turn, one
/// assistant reply with usage) — enough for `discovery::discover`'s summary-tier parse to succeed
/// with zero skips, matching the exact record shape `projects::discovery`'s own tests already use.
fn generate_tree(root: &Path, project_count: u32, sessions_per_project: u32) -> PathBuf {
    let projects_dir = root.join("projects");
    for p in 0..project_count {
        let dir = projects_dir.join(format!("bench-project-{p}"));
        std::fs::create_dir_all(&dir).unwrap();
        for s in 0..sessions_per_project {
            let session_id = format!("{p:08x}-0000-0000-0000-{s:012x}");
            let ts = format!("2026-01-{:02}T{:02}:{:02}:00.000Z", (s % 28) + 1, p % 24, s % 60);
            let line1 = format!(
                r#"{{"type":"user","uuid":"u{s}","timestamp":"{ts}","isSidechain":false,"message":{{"role":"user","content":"Bench prompt {s}"}}}}"#
            );
            let line2 = format!(
                r#"{{"type":"assistant","uuid":"a{s}","timestamp":"{ts}","isSidechain":false,"message":{{"id":"m{s}","role":"assistant","model":"claude-opus-4-6","content":[{{"type":"text","text":"Bench reply {s}"}}],"usage":{{"input_tokens":100,"output_tokens":50}}}}}}"#
            );
            std::fs::write(dir.join(format!("{session_id}.jsonl")), format!("{line1}\n{line2}\n")).unwrap();
        }
    }
    projects_dir
}
