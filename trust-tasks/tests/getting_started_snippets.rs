//! The gate that stops `GETTING-STARTED.md` rotting.
//!
//! The document's Rust snippets are not written by hand: each one is a region
//! of `examples/acl_grant_roundtrip.rs`, an example CI compiles (via
//! `cargo clippy --workspace --all-targets --all-features`) and a human can
//! run. This test asserts the two are byte-identical, so a change to the
//! example that is not carried into the document fails the build rather than
//! shipping a snippet that no longer compiles.
//!
//! Why this and not doctests: `cargo test --workspace --all-targets` — what CI
//! runs — **excludes** doctests, and the snippets need a live TCP listener on
//! both ends, which is a poor fit for a doctest anyway. An example is compiled
//! by the checks this repo already runs and is executable end to end; this test
//! is the thread that ties the prose to it.
//!
//! In the document a snippet is introduced by
//! `<!-- snippet: trust-tasks/examples/acl_grant_roundtrip.rs#<region> -->`
//! immediately followed by a ```rust fence. In the example a region is the
//! lines between `// GETTING-STARTED:begin <region>` and
//! `// GETTING-STARTED:end <region>`.

use std::collections::BTreeSet;
use std::path::PathBuf;

const EXAMPLE_REL: &str = "trust-tasks/examples/acl_grant_roundtrip.rs";
const DOC_REL: &str = "GETTING-STARTED.md";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("trust-tasks/ has a parent")
        .to_path_buf()
}

/// Lines strictly between the begin/end markers for `region`.
fn region_of(source: &str, region: &str) -> String {
    let begin = format!("// GETTING-STARTED:begin {region}");
    let end = format!("// GETTING-STARTED:end {region}");
    let mut out = Vec::new();
    let mut inside = false;
    for line in source.lines() {
        if line.trim() == begin {
            assert!(!inside, "region `{region}` opened twice in {EXAMPLE_REL}");
            inside = true;
            continue;
        }
        if line.trim() == end {
            assert!(inside, "region `{region}` closed before it opened");
            return out.join("\n");
        }
        if inside {
            out.push(line);
        }
    }
    panic!("region `{region}` is not delimited in {EXAMPLE_REL} (marker missing or unterminated)");
}

/// Every region the example declares.
fn declared_regions(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("// GETTING-STARTED:begin ")
                .map(str::to_string)
        })
        .collect()
}

/// Every `(region, code)` pair the document claims to have copied.
fn doc_snippets(doc: &str) -> Vec<(String, String)> {
    let marker_prefix = format!("<!-- snippet: {EXAMPLE_REL}#");
    let mut out = Vec::new();
    let mut lines = doc.lines().peekable();
    while let Some(line) = lines.next() {
        let Some(rest) = line.trim().strip_prefix(&marker_prefix) else {
            continue;
        };
        let region = rest.strip_suffix("-->").unwrap_or(rest).trim().to_string();
        let fence = lines
            .next()
            .unwrap_or_else(|| panic!("snippet marker for `{region}` is not followed by a fence"));
        assert_eq!(
            fence.trim(),
            "```rust",
            "snippet marker for `{region}` must be immediately followed by a ```rust fence"
        );
        let mut code = Vec::new();
        for body in lines.by_ref() {
            if body.trim_end() == "```" {
                break;
            }
            code.push(body);
        }
        out.push((region, code.join("\n")));
    }
    out
}

#[test]
fn getting_started_rust_snippets_match_the_example() {
    let root = repo_root();
    let example = std::fs::read_to_string(root.join(EXAMPLE_REL))
        .unwrap_or_else(|e| panic!("read {EXAMPLE_REL}: {e}"));
    let doc = std::fs::read_to_string(root.join(DOC_REL))
        .unwrap_or_else(|e| panic!("read {DOC_REL}: {e}"));

    let snippets = doc_snippets(&doc);
    assert!(
        !snippets.is_empty(),
        "{DOC_REL} carries no `<!-- snippet: … -->` blocks — either the markers were \
         dropped or the Rust was inlined by hand, which is exactly what this test exists \
         to prevent"
    );

    for (region, in_doc) in &snippets {
        let in_example = region_of(&example, region);
        assert_eq!(
            in_doc.trim_end(),
            in_example.trim_end(),
            "\n{DOC_REL} snippet `{region}` has drifted from {EXAMPLE_REL}.\n\
             Re-copy the region between the `GETTING-STARTED:begin/end {region}` markers \
             into the fenced block after its `<!-- snippet: … #{region} -->` marker.\n"
        );
    }

    // The reverse direction: a region added to the example and never wired into
    // the document is a snippet nobody reads, and a region *renamed* would
    // otherwise silently stop being checked.
    let referenced: BTreeSet<String> = snippets.into_iter().map(|(r, _)| r).collect();
    let declared = declared_regions(&example);
    assert_eq!(
        declared, referenced,
        "the regions marked in {EXAMPLE_REL} and the ones {DOC_REL} embeds must be the same set"
    );
}
