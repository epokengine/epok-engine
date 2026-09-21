//! Repository-wide policy that no product name of another engine or tool
//! reaches a tracked file.
//!
//! Registered by the crate root only under `cfg(test)`: this keeps production
//! dependency and staging surfaces unchanged. The rule was enforced by hand
//! until a comment naming another engine survived several reviews, so it is a
//! test rather than a convention.
use std::{path::Path, process::Command};

/// The term list, which is the one tracked file the sweep skips: it necessarily
/// spells every term it bans.
const TERMS: &str = "tools/forbidden-terms.txt";

fn terms(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join(TERMS))
        .expect("the forbidden term list is tracked next to the tools")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Tracked paths, from git rather than a directory walk, so ignored build
/// output and the contents of submodules stay out of the sweep.
fn tracked(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .expect("git must be available to enumerate tracked files");
    assert!(output.status.success(), "git ls-files failed: {output:?}");
    String::from_utf8(output.stdout)
        .expect("git reports paths as UTF-8 here")
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whole-word so ordinary prose survives: "community" does not contain the term
/// "unity", because the characters around a match must not be word characters.
fn hits(haystack: &str, term: &str) -> bool {
    let lowered = haystack.to_ascii_lowercase();
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    lowered.match_indices(term).any(|(at, _)| {
        let before = lowered[..at].chars().next_back();
        let after = lowered[at + term.len()..].chars().next();
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}

#[test]
fn forbidden_terms_are_absent_from_tracked_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let terms = terms(root);
    assert!(!terms.is_empty(), "the term list must not be empty");
    let mut found = vec![];
    for path in tracked(root) {
        if path == TERMS {
            continue;
        }
        for term in &terms {
            if hits(&path, term) {
                found.push(format!("{path}: the path itself names `{term}`"));
            }
        }
        // Binary and non-UTF-8 files carry no prose to review, so a failed read
        // is a skip rather than a failure.
        let Ok(text) = std::fs::read_to_string(root.join(&path)) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for term in &terms {
                if hits(line, term) {
                    found.push(format!("{path}:{}: names `{term}`", number + 1));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "tracked files name another product; describe the behaviour instead:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_sweep_detects_a_planted_term() {
    // A sweep that silently matched nothing would pass for the wrong reason.
    assert!(hits("matching Unreal's Delay node", "unreal"));
    assert!(hits("docs/architecture/unreal-notes.md", "unreal"));
    assert!(!hits("the community agreed", "unity"));
    assert!(!hits("disunity_flag", "unity"));
}
