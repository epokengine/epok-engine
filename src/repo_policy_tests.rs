//! Repository-wide policy that no product name of another engine or tool
//! reaches a tracked file.
//!
//! Registered by the crate root only under `cfg(test)`: this keeps production
//! dependency and staging surfaces unchanged. The rule was enforced by hand
//! until a comment naming another engine survived several reviews, so it is a
//! test rather than a convention.
use std::{path::Path, process::Command};

/// The term list the sweep reads.
const TERMS: &str = "tools/forbidden-terms.txt";

/// The only tracked files the sweep skips, because enforcing the rule requires
/// spelling the terms: the list itself, and this file, whose negative control
/// plants them deliberately. Nothing else is exempt, so a term cannot be hidden
/// by moving it somewhere quieter.
const SPELLS_THE_TERMS: [&str; 2] = [TERMS, "src/repo_policy_tests.rs"];

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
        if SPELLS_THE_TERMS.contains(&path.as_str()) {
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
        tracked(root).len() > 500,
        "the sweep must cover the repository, not a handful of files"
    );
    assert!(
        found.is_empty(),
        "tracked files name another product; describe the behaviour instead:\n{}",
        found.join("\n")
    );
}

/// Every shipped example must open in the editor that ships beside it. Two
/// examples were left at `0.1.0` over startup maps in a retired format, so no
/// editor since v0.1.0 could open them, and the portable archive shipped both
/// anyway because it copies `examples/` whole. Reviewing a version string is the
/// wrong instrument for that, so the release gate now reads the descriptors.
#[test]
fn shipped_examples_open_in_this_editor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let projects: Vec<String> = tracked(root)
        .into_iter()
        .filter(|path| path.starts_with("examples/") && path.ends_with(".epokproject"))
        .collect();
    assert!(
        projects.len() >= 3,
        "the shipped examples must be discoverable; found {projects:?}"
    );
    let mut rejected = vec![];
    for project in projects {
        // The descriptor's own directory is the project root.
        let folder = root
            .join(&project)
            .parent()
            .expect("a rooted path")
            .to_owned();
        match crate::workspace::project_editor_version(&folder) {
            Ok(version) if version == env!("CARGO_PKG_VERSION") => {}
            Ok(version) => rejected.push(format!(
                "{project}: declares {version}, but this editor is {}",
                env!("CARGO_PKG_VERSION")
            )),
            Err(error) => rejected.push(format!("{project}: {error}")),
        }
    }
    assert!(
        rejected.is_empty(),
        "shipped examples that this editor cannot open:\n{}",
        rejected.join("\n")
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
