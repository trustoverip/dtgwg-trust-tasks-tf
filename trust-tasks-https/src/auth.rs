//! Authentication seam: resolve an incoming HTTP request to a Verifiable
//! Identifier.
//!
//! [`Auth`] is the trait the server uses to convert a bearer token into a
//! VID. [`BearerAuth`] is a minimal `HashMap`-backed implementation
//! suitable for demos and tests; real deployments would implement [`Auth`]
//! against their identity provider, peer-certificate database, JWT
//! verifier, etc.

use std::collections::HashMap;

/// Resolves bearer tokens to Verifiable Identifiers.
///
/// Implementations are consulted on every inbound request before the
/// framework's [`TransportHandler`](trust_tasks_rs::TransportHandler) sees
/// the document. Returning `None` means the request is unauthenticated;
/// the server may still accept it (the framework falls back to in-band
/// `issuer` + `proof`), or apply policy (e.g. require auth for non-bearer
/// specs).
pub trait Auth: Send + Sync + 'static {
    /// Resolve `token` to a VID, or return `None` if the token is unknown.
    fn resolve(&self, token: &str) -> Option<String>;
}

/// Static `HashMap`-backed [`Auth`] implementation. Constructed from a
/// list of `(token, vid)` pairs.
///
/// ```rust,ignore
/// use trust_tasks_https::BearerAuth;
///
/// let auth = BearerAuth::from_pairs([
///     ("alice-token", "did:web:alice.example"),
///     ("bob-token",   "did:web:bob.example"),
/// ]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct BearerAuth {
    tokens: HashMap<String, String>,
}

impl BearerAuth {
    /// Empty bearer-auth table; every token resolves to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from `(token, vid)` pairs. Either argument can be anything
    /// `Into<String>`-convertible.
    pub fn from_pairs<I, T, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, V)>,
        T: Into<String>,
        V: Into<String>,
    {
        let mut auth = Self::new();
        for (t, v) in iter {
            auth.insert(t, v);
        }
        auth
    }

    /// Insert a single `(token, vid)` mapping, builder-style.
    pub fn with(mut self, token: impl Into<String>, vid: impl Into<String>) -> Self {
        self.insert(token, vid);
        self
    }

    /// Insert a single `(token, vid)` mapping in place.
    pub fn insert(&mut self, token: impl Into<String>, vid: impl Into<String>) {
        self.tokens.insert(token.into(), vid.into());
    }
}

impl Auth for BearerAuth {
    fn resolve(&self, token: &str) -> Option<String> {
        self.tokens.get(token).cloned()
    }
}
