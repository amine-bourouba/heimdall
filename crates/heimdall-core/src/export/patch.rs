//! Unified-diff patch export for Edit blocks (`SessionExporter.toPatch`/`buildHunks`), built on
//! the line diff (M1-35).

use crate::diff::{diff, DiffKind, DiffLine};

const CONTEXT_SIZE: usize = 3;

/// A standard unified diff: `--- a/<path>` / `+++ b/<path>`, one or more `@@ -o,oc +n,nc @@` hunks
/// with up to 3 lines of context on each side, ending with a trailing newline. Applies cleanly
/// with `git apply`.
pub fn to_patch(old: &str, new: &str, file_path: &str) -> String {
    // A real file's own trailing newline is not a content line; diffing it as one (as a naive
    // split('\n') would) adds a spurious trailing blank line both sides "agree" on, which is
    // enough to make `git apply` reject the hunk's line count against the real file on disk.
    let result = diff(old.strip_suffix('\n').unwrap_or(old), new.strip_suffix('\n').unwrap_or(new));
    let mut lines = vec![format!("--- a/{file_path}"), format!("+++ b/{file_path}")];
    for hunk in build_hunks(&result.lines) {
        let old_start = hunk.iter().find_map(|l| l.old_line).unwrap_or(1);
        let new_start = hunk.iter().find_map(|l| l.new_line).unwrap_or(1);
        let old_count = hunk.iter().filter(|l| matches!(l.kind, DiffKind::Context | DiffKind::Remove)).count();
        let new_count = hunk.iter().filter(|l| matches!(l.kind, DiffKind::Context | DiffKind::Add)).count();
        lines.push(format!("@@ -{old_start},{old_count} +{new_start},{new_count} @@"));
        for line in hunk {
            let prefix = match line.kind {
                DiffKind::Context => ' ',
                DiffKind::Add => '+',
                DiffKind::Remove => '-',
            };
            lines.push(format!("{prefix}{}", line.text));
        }
    }
    lines.join("\n") + "\n"
}

/// Groups changed lines with up to [`CONTEXT_SIZE`] lines of surrounding context, starting a new
/// hunk when a gap between changes is too large to bridge with context alone
/// (`SessionExporter.buildHunks`, ported index for index).
fn build_hunks(diff_lines: &[DiffLine]) -> Vec<Vec<DiffLine>> {
    if diff_lines.is_empty() {
        return Vec::new();
    }
    let mut hunks: Vec<Vec<DiffLine>> = Vec::new();
    let mut current: Vec<DiffLine> = Vec::new();
    let mut last_change: isize = -1;

    for (index, line) in diff_lines.iter().enumerate() {
        if line.kind == DiffKind::Context {
            continue;
        }
        let index = index as isize;
        let context_start = (last_change + 1).max(index - CONTEXT_SIZE as isize);
        if current.is_empty() {
            current.extend(diff_lines[context_start as usize..index as usize].iter().cloned());
        } else if context_start > last_change + 1 {
            let trail_end = (last_change + CONTEXT_SIZE as isize + 1).min(index) as usize;
            current.extend(diff_lines[(last_change + 1) as usize..trail_end].iter().cloned());
            hunks.push(std::mem::take(&mut current));
            current.extend(diff_lines[context_start as usize..index as usize].iter().cloned());
        } else {
            current.extend(diff_lines[(last_change + 1) as usize..index as usize].iter().cloned());
        }
        current.push(line.clone());
        last_change = index;
    }

    if !current.is_empty() {
        let trail_end = (last_change + CONTEXT_SIZE as isize + 1).min(diff_lines.len() as isize) as usize;
        current.extend(diff_lines[(last_change + 1) as usize..trail_end].iter().cloned());
        hunks.push(current);
    }
    if hunks.is_empty() {
        hunks.push(diff_lines.to_vec());
    }
    hunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn a_single_change_produces_one_hunk_with_surrounding_context() {
        let old = (1..=10).map(|n| format!("line{n}")).collect::<Vec<_>>().join("\n");
        let new = old.replace("line5", "CHANGED");
        let patch = to_patch(&old, &new, "src/a.txt");
        assert!(patch.starts_with("--- a/src/a.txt\n+++ b/src/a.txt\n"));
        assert_eq!(patch.matches("@@").count(), 2); // one hunk header (two "@@" markers)
        assert!(patch.contains("-line5") && patch.contains("+CHANGED"));
        assert!(patch.contains("line2") && patch.contains("line8")); // 3 lines of context each side
        assert!(!patch.contains("line1\n") || patch.contains(" line2")); // line1 is outside the context window
    }

    #[test]
    fn changes_far_apart_produce_separate_hunks() {
        let old = (1..=30).map(|n| format!("line{n}")).collect::<Vec<_>>().join("\n");
        let new = old.replace("line5", "A").replace("line25", "B");
        let patch = to_patch(&old, &new, "f.txt");
        assert_eq!(patch.matches("@@ -").count(), 2);
    }

    #[test]
    fn changes_close_together_merge_into_one_hunk() {
        let old = (1..=30).map(|n| format!("line{n}")).collect::<Vec<_>>().join("\n");
        let new = old.replace("line10", "A").replace("line14", "B"); // 3 lines apart: within 2*context
        let patch = to_patch(&old, &new, "f.txt");
        assert_eq!(patch.matches("@@ -").count(), 1);
    }

    #[test]
    fn identical_content_still_produces_one_all_context_hunk() {
        // `buildHunks`' own fallback ("if no changes found, return all lines as a single hunk"),
        // ported as-is: a no-op patch is one hunk of pure context, not an empty patch.
        let patch = to_patch("same\ntext", "same\ntext", "f.txt");
        assert_eq!(patch, "--- a/f.txt\n+++ b/f.txt\n@@ -1,2 +1,2 @@\n same\n text\n");
    }

    #[test]
    fn the_patch_applies_cleanly_with_git_apply() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Command::new("git").args(["init", "-q"]).current_dir(dir.path()).status().unwrap().success());
        let old = "fn main() {\n    println!(\"a\");\n    println!(\"b\");\n    println!(\"c\");\n}\n";
        let new = "fn main() {\n    println!(\"a\");\n    println!(\"changed\");\n    println!(\"c\");\n}\n";
        std::fs::write(dir.path().join("src.rs"), old).unwrap();
        let patch = to_patch(old, new, "src.rs");
        std::fs::write(dir.path().join("change.patch"), &patch).unwrap();
        let output = Command::new("git").args(["apply", "--check", "change.patch"]).current_dir(dir.path()).output().unwrap();
        assert!(output.status.success(), "git apply --check failed:\n{}\n--- patch ---\n{patch}", String::from_utf8_lossy(&output.stderr));
    }

    #[test]
    fn a_patch_with_an_added_and_a_removed_file_line_also_applies() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Command::new("git").args(["init", "-q"]).current_dir(dir.path()).status().unwrap().success());
        let old = "one\ntwo\nthree\n";
        let new = "one\ntwo point five\nthree\nfour\n";
        std::fs::write(dir.path().join("f.txt"), old).unwrap();
        let patch = to_patch(old, new, "f.txt");
        std::fs::write(dir.path().join("change.patch"), &patch).unwrap();
        let output = Command::new("git").args(["apply", "--check", "change.patch"]).current_dir(dir.path()).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
}
