//! A cache of what is derived from a schema's text — its parsed JSON, or a
//! compiled validator — keyed by the text's address.
//!
//! The schemas the runtime evaluates are `Payload::PAYLOAD_SCHEMA` constants:
//! `&'static str`s of tens of KB that never move, so their address identifies
//! them and the expensive derivation (a parse, or a parse plus a compile with
//! every `pattern` regex) is needed once per schema, not once per document.
//!
//! The callers take `&str`, though, not `&'static str` — [`PayloadValidator`]
//! and [`prune_payload`] are public and accept any schema text — and the
//! address of a string that has since been freed can be reused by a different
//! one. So a hit is confirmed by comparing the text too (a `memcmp`, against
//! the parse it saves), and the cache is bounded: a caller handing in a fresh
//! allocation per call can make it recompute, never grow without limit or
//! answer for the wrong schema.
//!
//! [`PayloadValidator`]: crate::PayloadValidator
//! [`prune_payload`]: crate::extension::prune_payload

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;

/// More distinct schemas than every generated module together carries; past
/// it the cache starts over rather than grow.
const MAX_ENTRIES: usize = 4096;

/// `(address, length)` of a schema text → that text, and what was derived from it.
type Entries<T> = BTreeMap<(usize, usize), (Box<str>, Arc<T>)>;

/// Derived values of type `T`, one per schema text.
pub(crate) struct SchemaCache<T> {
    /// `(address, length)` of the text → the text it was derived from, and
    /// the value.
    entries: Mutex<Entries<T>>,
}

impl<T> SchemaCache<T> {
    pub(crate) const fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }

    /// The value derived from `schema`, deriving it with `derive` on a miss.
    /// A failed derivation is not cached.
    ///
    /// `derive` runs outside the lock, so two threads missing on the same
    /// schema at once may both derive it; the second insert wins and both
    /// results are equivalent.
    pub(crate) fn get_or_try_insert<E>(
        &self,
        schema: &str,
        derive: impl FnOnce(&str) -> Result<T, E>,
    ) -> Result<Arc<T>, E> {
        let key = (schema.as_ptr() as usize, schema.len());
        {
            let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((text, value)) = entries.get(&key) {
                if **text == *schema {
                    return Ok(Arc::clone(value));
                }
            }
        }
        let value = Arc::new(derive(schema)?);
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() >= MAX_ENTRIES && !entries.contains_key(&key) {
            entries.clear();
        }
        entries.insert(key, (schema.into(), Arc::clone(&value)));
        Ok(value)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

/// Parsed schemas, shared by everything in the runtime that reads a schema's
/// JSON without compiling it.
static PARSED: SchemaCache<Value> = SchemaCache::new();

/// `schema_json` parsed, from the process-wide cache; `None` if it does not
/// parse.
pub(crate) fn parsed(schema_json: &str) -> Option<Arc<Value>> {
    PARSED
        .get_or_try_insert(schema_json, |s| serde_json::from_str::<Value>(s))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hit_is_derived_once() {
        let cache: SchemaCache<String> = SchemaCache::new();
        static SCHEMA: &str = r#"{"type":"object"}"#;
        let mut calls = 0;
        for _ in 0..3 {
            let got = cache
                .get_or_try_insert(SCHEMA, |s| {
                    calls += 1;
                    Ok::<_, ()>(s.to_uppercase())
                })
                .unwrap();
            assert_eq!(*got, SCHEMA.to_uppercase());
        }
        assert_eq!(calls, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn a_reused_address_with_other_text_is_a_miss() {
        let cache: SchemaCache<String> = SchemaCache::new();
        // The same buffer, rewritten in place: same address and length,
        // different text.
        let mut buf = String::from(r#"{"a":1}"#);
        let first = cache
            .get_or_try_insert(&buf, |s| Ok::<_, ()>(s.to_string()))
            .unwrap();
        buf.replace_range(.., r#"{"b":2}"#);
        let second = cache
            .get_or_try_insert(&buf, |s| Ok::<_, ()>(s.to_string()))
            .unwrap();
        assert_eq!(*first, r#"{"a":1}"#);
        assert_eq!(*second, r#"{"b":2}"#);
    }

    #[test]
    fn failures_are_not_cached() {
        let cache: SchemaCache<Value> = SchemaCache::new();
        assert!(cache
            .get_or_try_insert("{", |s| serde_json::from_str::<Value>(s))
            .is_err());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn the_cache_is_bounded() {
        let cache: SchemaCache<()> = SchemaCache::new();
        let texts: Vec<String> = (0..MAX_ENTRIES + 10).map(|i| i.to_string()).collect();
        for t in &texts {
            cache.get_or_try_insert(t, |_| Ok::<_, ()>(())).unwrap();
        }
        assert!(cache.len() <= MAX_ENTRIES);
    }
}
