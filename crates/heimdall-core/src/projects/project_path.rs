//! Project path resolution, cwd first (ADR 0003). Poirot turned the folder name back into a path,
//! which is lossy (`-` stands for every separator and punctuation mark); transcripts record the
//! real folder in `cwd`.

use std::collections::HashMap;

use crate::models::project::PathSource;
use crate::paths::{self, TargetOs};

/// Picks a project's path, in order:
/// 1. a transcript `cwd` whose encoding is the folder name (case-insensitive: Claude Code writes
///    both `C:` and `c:`), the most frequent if several match — sessions that `cd` into a
///    subfolder do not move the project;
/// 2. the most frequent `cwd`;
/// 3. the sessions-index path;
/// 4. the folder name decoded by probing the disk ([`paths::decode_project_dir_name`]).
pub fn resolve(dir_name: &str, cwds: &HashMap<String, u32>, index_path: Option<&str>, os: TargetOs,
               exists: impl Fn(&str) -> bool) -> (String, PathSource) {
    let most_frequent = |matching: &dyn Fn(&str) -> bool| {
        cwds.iter()
            .filter(|(cwd, _)| matching(cwd))
            // Ties go to the lexically smaller path, so the result does not depend on map order.
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
            .map(|(cwd, _)| cwd.clone())
    };
    if let Some(cwd) = most_frequent(&|cwd| paths::encodes_to(cwd, dir_name)).or_else(|| most_frequent(&|_| true)) {
        return (cwd, PathSource::TranscriptCwd);
    }
    if let Some(p) = index_path {
        return (p.to_string(), PathSource::SessionsIndex);
    }
    (paths::decode_project_dir_name(dir_name, os, exists), PathSource::Decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn matching_cwd_beats_a_more_frequent_subfolder() {
        let cwds = counts(&[(r"C:\Users\a\tech\proj-1", 455), (r"C:\Users\a\tech", 27), (r"c:\Users\a\tech", 20)]);
        assert_eq!(resolve("C--Users-a-tech", &cwds, Some("/idx"), TargetOs::Windows, |_| false), (r"C:\Users\a\tech".into(), PathSource::TranscriptCwd));
        assert_eq!(resolve("c--users-a-tech", &cwds, None, TargetOs::Windows, |_| false).0, r"C:\Users\a\tech");
    }

    #[test]
    fn falls_back_through_frequent_cwd_index_and_decoding() {
        let other = counts(&[("/srv/x", 3), ("/srv/y", 1)]);
        assert_eq!(resolve("-home-me-app", &other, None, TargetOs::Linux, |_| false), ("/srv/x".into(), PathSource::TranscriptCwd));
        let none = HashMap::new();
        assert_eq!(resolve("-home-me-app", &none, Some("/home/me/app"), TargetOs::Linux, |_| false), ("/home/me/app".into(), PathSource::SessionsIndex));
        assert_eq!(resolve("-home-me-my-app", &none, None, TargetOs::Linux, |p| p == "/home" || p == "/home/me"), ("/home/me/my-app".into(), PathSource::Decoded));
    }
}
