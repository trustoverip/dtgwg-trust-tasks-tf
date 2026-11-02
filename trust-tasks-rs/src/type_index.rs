//! A Type URI index an application generates, as data: one [`IndexEntry`] per
//! Type URI, in tables that several generated crates can contribute to.
//!
//! `trust_tasks_rs::schema_index` (feature `validate`) answers for the prebuilt modules this
//! build carries. An application that generates its tasks with
//! `trust-tasks-build` and turns on its `index` option gets the same lookups
//! for the versions it locked, and with them a `schema_index::INDEX` table of
//! these entries. An application split into several generated crates — one per
//! family, say — combines their tables with [`lookup`] instead of writing a
//! merge by hand:
//!
//! ```ignore
//! use trust_tasks_rs::type_index::{lookup, IndexEntry};
//!
//! static TABLES: &[&[IndexEntry]] =
//!     &[vault_tasks::schema_index::INDEX, keys_tasks::schema_index::INDEX];
//!
//! fn schema_for(type_uri: &str) -> Option<&'static str> {
//!     lookup(TABLES, type_uri).and_then(|entry| entry.schema)
//! }
//! ```
//!
//! An entry's members answer as the four `schema_index`
//! functions do for its Type URI, so a lookup that finds no entry is their
//! `None`: a Type URI this build knows no specification for. A consumer that
//! reads that as "skip validation" fails open for exactly the tasks it knows
//! least.
//!
//! Not behind the `validate` feature: an entry is constants, and building or
//! reading one needs no JSON Schema implementation.

use crate::{DeclaredErrorCode, SpecPolicy};

/// What the index knows about one Type URI: the request side under the bare
/// Type URI, the response side under the same URI suffixed `#response`.
///
/// `#[non_exhaustive]`, so a member can be added without a breaking change;
/// generated code builds one with [`IndexEntry::request`] or
/// [`IndexEntry::response`], both `const`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct IndexEntry {
    /// The Type URI: bare for a request, suffixed `#response` for a response.
    pub type_uri: &'static str,
    /// The payload schema — `schema_index::schema_for`.
    pub schema: Option<&'static str>,
    /// The consumer policy — `schema_index::spec_policy_for`. `None` for a
    /// response: the rules are about a document a consumer received.
    pub policy: Option<SpecPolicy>,
    /// The declared error codes, `Some(&[])` when the specification declares
    /// none — `schema_index::error_codes_for`. `None` for a response: the codes
    /// belong to the specification, which is keyed by its request Type URI.
    pub error_codes: Option<&'static [DeclaredErrorCode]>,
    /// The declared document size bound — `schema_index::max_document_bytes_for`.
    pub max_document_bytes: Option<usize>,
}

impl IndexEntry {
    /// The entry for a request Type URI.
    pub const fn request(
        type_uri: &'static str,
        schema: Option<&'static str>,
        policy: SpecPolicy,
        error_codes: &'static [DeclaredErrorCode],
        max_document_bytes: Option<usize>,
    ) -> Self {
        Self {
            type_uri,
            schema,
            policy: Some(policy),
            error_codes: Some(error_codes),
            max_document_bytes,
        }
    }

    /// The entry for a `#response` Type URI.
    pub const fn response(
        type_uri: &'static str,
        schema: Option<&'static str>,
        max_document_bytes: Option<usize>,
    ) -> Self {
        Self {
            type_uri,
            schema,
            policy: None,
            error_codes: None,
            max_document_bytes,
        }
    }
}

/// The entry for `type_uri` in the first of `tables` that has one, or `None`.
///
/// The comparison is on the whole string, so a Type URI under another
/// authority (SPEC §6.5) is its own entry. A linear scan: each table is a few
/// hundred entries at most, and a lookup is once per received document.
pub fn lookup<'a>(tables: &[&'a [IndexEntry]], type_uri: &str) -> Option<&'a IndexEntry> {
    tables
        .iter()
        .find_map(|table| table.iter().find(|entry| entry.type_uri == type_uri))
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: SpecPolicy = SpecPolicy {
        is_bearer: false,
        is_proof_required: true,
        is_recipient_required: true,
        is_issued_at_required: false,
    };
    const A: &[IndexEntry] = &[
        IndexEntry::request(
            "https://example.com/spec/a/0.1",
            Some("{}"),
            POLICY,
            &[],
            None,
        ),
        IndexEntry::response("https://example.com/spec/a/0.1#response", None, Some(10)),
    ];
    const B: &[IndexEntry] = &[IndexEntry::request(
        "https://example.com/spec/b/0.1",
        None,
        POLICY,
        &[],
        Some(5),
    )];

    #[test]
    fn lookup_searches_every_table_by_the_whole_uri() {
        let tables = &[A, B];
        let a = lookup(tables, "https://example.com/spec/a/0.1").unwrap();
        assert_eq!(a.policy, Some(POLICY));
        assert_eq!(a.error_codes, Some(&[][..]));
        let response = lookup(tables, "https://example.com/spec/a/0.1#response").unwrap();
        assert_eq!((response.policy, response.error_codes), (None, None));
        assert_eq!(response.max_document_bytes, Some(10));
        assert_eq!(
            lookup(tables, "https://example.com/spec/b/0.1")
                .unwrap()
                .max_document_bytes,
            Some(5)
        );
        assert!(lookup(tables, "https://example.com/spec/c/0.1").is_none());
        assert!(lookup(tables, "https://example.com/spec/a/0.1#request").is_none());
        assert!(lookup(&[], "https://example.com/spec/a/0.1").is_none());
    }
}
