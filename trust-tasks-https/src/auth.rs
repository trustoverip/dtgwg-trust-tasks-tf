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

/// JWT-bearer [`Auth`] adapter — available behind the `jwt` Cargo
/// feature.
#[cfg(feature = "jwt")]
#[cfg_attr(docsrs, doc(cfg(feature = "jwt")))]
pub use jwt::JwtBearerAuth;

#[cfg(feature = "jwt")]
#[cfg_attr(docsrs, doc(cfg(feature = "jwt")))]
mod jwt {
    use std::marker::PhantomData;

    use jsonwebtoken::{decode, DecodingKey, Validation};
    use serde::de::DeserializeOwned;

    use super::Auth;

    /// [`Auth`] adapter that decodes an inbound JWT bearer token under a
    /// caller-supplied `DecodingKey` + [`Validation`], then delegates the
    /// subject-VID extraction to a closure over the typed claims.
    ///
    /// Most Trust-Tasks consumers running on a JWT-based session model
    /// (OIDC, Affinidi sessions, etc.) end up re-inventing this glue
    /// inside their [`Auth`] impl — the wrapper exists so the same six
    /// lines do not appear in every consumer.
    ///
    /// ```rust,ignore
    /// use jsonwebtoken::{DecodingKey, Validation, Algorithm};
    /// use serde::Deserialize;
    /// use trust_tasks_https::JwtBearerAuth;
    ///
    /// #[derive(Deserialize)]
    /// struct MyClaims {
    ///     sub: String,
    ///     /// Custom claim — your session JWT might carry the VID
    ///     /// separately from the OIDC `sub`.
    ///     vid: Option<String>,
    /// }
    ///
    /// let auth = JwtBearerAuth::<MyClaims>::new(
    ///     DecodingKey::from_secret(b"my-shared-secret"),
    ///     Validation::new(Algorithm::HS256),
    ///     |claims| claims.vid.clone().or(Some(claims.sub.clone())),
    /// );
    /// ```
    ///
    /// On any decode / signature / `Validation` failure the adapter
    /// returns `None` from [`Auth::resolve`], producing an
    /// unauthenticated request — the framework then falls back to
    /// in-band `issuer` + `proof` per SPEC.md §4.8.1.
    pub struct JwtBearerAuth<C: DeserializeOwned + Send + Sync + 'static> {
        key: DecodingKey,
        validation: Validation,
        extract: Box<ExtractFn<C>>,
        _phantom: PhantomData<fn() -> C>,
    }

    /// Caller-supplied subject-extraction closure: given the decoded
    /// typed claims, return the VID string (or `None` to fall back to
    /// unauthenticated, e.g. if the claim that names the VID is empty).
    type ExtractFn<C> = dyn Fn(&C) -> Option<String> + Send + Sync;

    impl<C: DeserializeOwned + Send + Sync + 'static> JwtBearerAuth<C> {
        /// Build a [`JwtBearerAuth`] over the supplied verification key,
        /// `jsonwebtoken` [`Validation`] policy, and a subject-extraction
        /// closure over the typed claims.
        pub fn new<F>(key: DecodingKey, validation: Validation, extract: F) -> Self
        where
            F: Fn(&C) -> Option<String> + Send + Sync + 'static,
        {
            Self {
                key,
                validation,
                extract: Box::new(extract),
                _phantom: PhantomData,
            }
        }
    }

    impl<C: DeserializeOwned + Send + Sync + 'static> std::fmt::Debug for JwtBearerAuth<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("JwtBearerAuth")
                .field("claims_type", &std::any::type_name::<C>())
                .finish_non_exhaustive()
        }
    }

    impl<C: DeserializeOwned + Send + Sync + 'static> Auth for JwtBearerAuth<C> {
        fn resolve(&self, token: &str) -> Option<String> {
            let data = decode::<C>(token, &self.key, &self.validation).ok()?;
            (self.extract)(&data.claims)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct Claims {
            sub: String,
            vid: Option<String>,
            exp: usize,
        }

        fn sign(secret: &[u8], claims: &Claims) -> String {
            encode(
                &Header::new(Algorithm::HS256),
                claims,
                &EncodingKey::from_secret(secret),
            )
            .unwrap()
        }

        fn auth(secret: &[u8]) -> JwtBearerAuth<Claims> {
            let mut v = Validation::new(Algorithm::HS256);
            v.validate_exp = true;
            v.required_spec_claims.clear();
            JwtBearerAuth::new(DecodingKey::from_secret(secret), v, |c: &Claims| {
                c.vid.clone().or_else(|| Some(c.sub.clone()))
            })
        }

        #[test]
        fn valid_jwt_resolves_to_vid_via_claims_closure() {
            let secret = b"test-secret";
            let token = sign(
                secret,
                &Claims {
                    sub: "user-123".into(),
                    vid: Some("did:web:alice.example".into()),
                    exp: 10_000_000_000,
                },
            );
            let auth = auth(secret);
            assert_eq!(
                auth.resolve(&token),
                Some("did:web:alice.example".to_string())
            );
        }

        #[test]
        fn jwt_without_vid_claim_falls_back_to_subject() {
            let secret = b"test-secret";
            let token = sign(
                secret,
                &Claims {
                    sub: "did:web:fallback.example".into(),
                    vid: None,
                    exp: 10_000_000_000,
                },
            );
            let auth = auth(secret);
            assert_eq!(
                auth.resolve(&token),
                Some("did:web:fallback.example".to_string())
            );
        }

        #[test]
        fn wrong_signature_returns_none() {
            let token = sign(
                b"correct-secret",
                &Claims {
                    sub: "x".into(),
                    vid: None,
                    exp: 10_000_000_000,
                },
            );
            let auth = auth(b"wrong-secret");
            assert!(auth.resolve(&token).is_none());
        }

        #[test]
        fn expired_jwt_returns_none() {
            let token = sign(
                b"test-secret",
                &Claims {
                    sub: "x".into(),
                    vid: None,
                    exp: 1, // 1970
                },
            );
            let auth = auth(b"test-secret");
            assert!(auth.resolve(&token).is_none());
        }

        #[test]
        fn malformed_jwt_returns_none() {
            let auth = auth(b"test-secret");
            assert!(auth.resolve("not-a-jwt").is_none());
        }
    }
}
