//! Reading a registry bundle (`bundleVersion` 1).
//!
//! A bundle is one specification version, self-contained: its request and
//! response schemas with every cross-file `$ref` inlined, the front-matter
//! policy the runtime enforces, its declared error codes and its lifecycle.
//! The registry publishes one per version at
//! `https://trusttasks.org/specs/<slug>/<M.N>/bundle.json`; an application
//! vendors the ones its `trust-tasks.lock` pins.

use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error_codes::{error_code_decls, ErrorCodeDecl};

/// The bundle shape this crate reads.
pub const BUNDLE_VERSION: u64 = 1;

/// A value declared separately for the request and the response document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct PerVariant<T> {
    /// The request document's value.
    pub request: T,
    /// The response document's value. Meaningless when the specification
    /// defines no response (`responseSchema` is `null`).
    pub response: T,
}

/// The consumption policy a specification version declares (SPEC §7.2/§7.3),
/// as the runtime's `Payload` constants carry it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Policy {
    /// `Payload::IS_BEARER` (SPEC §4.8.3).
    pub bearer: bool,
    /// `Payload::IS_PROOF_REQUIRED`, per variant (SPEC §7.3 item 8).
    pub proof_required: PerVariant<bool>,
    /// `Payload::IS_ISSUED_AT_REQUIRED`, per variant (SPEC §7.3 item 17).
    pub issued_at_required: PerVariant<bool>,
    /// `Payload::IS_RECIPIENT_REQUIRED`, per variant. The response's tracks the
    /// request's *issuer* party, because a response swaps the parties (SPEC
    /// §7.2 item 5).
    pub recipient_required: PerVariant<bool>,
    /// `Payload::MAX_DOCUMENT_BYTES`, per variant.
    pub max_document_bytes: PerVariant<Option<usize>>,
}

/// One parsed registry bundle.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Bundle {
    /// The request Type URI, e.g. `https://trusttasks.org/spec/vault/upsert/0.4`.
    pub type_uri: String,
    /// e.g. `vault/upsert`.
    pub slug: String,
    /// `MAJOR.MINOR`, e.g. `0.4`.
    pub version: String,
    /// Front matter `status`: `draft`, `candidate`, `standard`, `retired` or
    /// `withdrawn`.
    pub status: String,
    /// The lifecycle note (`superseded by …`, `retired; use …`, `retired`), or
    /// `None` while the version is current.
    pub deprecation: Option<String>,
    /// The earlier `MAJOR.MINOR` this version declares it extends (SPEC §5.2).
    pub extends: Option<String>,
    /// The consumption policy.
    pub policy: Policy,
    /// The extended error codes the version declares, in declaration order.
    pub error_codes: Vec<ErrorCodeDecl>,
    /// The request payload schema, self-contained.
    pub schema: Value,
    /// The response document schema, or `None` when the version defines no
    /// `$defs.Response`.
    pub response_schema: Option<Value>,
    /// The authored order of every `properties` object in `schema` and
    /// `responseSchema`, keyed by the JSON Pointer (from the bundle root) of
    /// the schema owning the `properties`. Empty for a bundle written before
    /// the member was added.
    pub property_order: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBundle {
    bundle_version: u64,
    type_uri: String,
    slug: String,
    version: String,
    status: String,
    deprecation: Option<String>,
    extends: Option<String>,
    policy: RawPolicy,
    error_codes: Vec<Value>,
    schema: Value,
    response_schema: Option<Value>,
    #[serde(default)]
    property_order: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPolicy {
    bearer: bool,
    proof_required: PerVariant<bool>,
    issued_at_required: PerVariant<bool>,
    recipient_required: PerVariant<bool>,
    max_document_bytes: PerVariant<Option<u64>>,
}

impl Bundle {
    /// Parse a bundle's bytes.
    ///
    /// Refuses a `bundleVersion` other than [`BUNDLE_VERSION`], a `typeUri`
    /// that does not name `slug`/`version`, an `extends` that is not a
    /// strictly earlier MINOR of the same MAJOR, and an error code the runtime
    /// could not put on the wire.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, crate::Error> {
        Ok(Self::parse_bytes(bytes)?)
    }

    pub(crate) fn parse_bytes(bytes: &[u8]) -> Result<Self> {
        let raw: RawBundle = serde_json::from_slice(bytes).context("parse bundle")?;
        if raw.bundle_version != BUNDLE_VERSION {
            return Err(anyhow!(
                "{}/{}: bundleVersion {} is not supported (this trust-tasks-build reads {})",
                raw.slug,
                raw.version,
                raw.bundle_version,
                BUNDLE_VERSION
            ));
        }
        let ctx = format!("{}/{}", raw.slug, raw.version);
        if !raw
            .type_uri
            .ends_with(&format!("/spec/{}/{}", raw.slug, raw.version))
        {
            return Err(anyhow!(
                "{ctx}: typeUri {:?} does not name this slug and version",
                raw.type_uri
            ));
        }
        if !raw.schema.is_object() {
            return Err(anyhow!("{ctx}: schema is not an object"));
        }
        if let Some(target) = &raw.extends {
            extends_shape(&raw.version, target)
                .map_err(|why| anyhow!("{ctx}: extends: '{target}' {why}"))?;
        }
        let size = |n: Option<u64>| -> Result<Option<usize>> {
            n.map(|n| {
                usize::try_from(n).map_err(|_| anyhow!("{ctx}: maxDocumentBytes {n} out of range"))
            })
            .transpose()
        };
        let policy = Policy {
            bearer: raw.policy.bearer,
            proof_required: raw.policy.proof_required,
            issued_at_required: raw.policy.issued_at_required,
            recipient_required: raw.policy.recipient_required,
            max_document_bytes: PerVariant {
                request: size(raw.policy.max_document_bytes.request)?,
                response: size(raw.policy.max_document_bytes.response)?,
            },
        };
        // The same reader the repository's codegen applies to front matter,
        // so a bundle is held to exactly the rules a spec.md is.
        let error_codes = error_code_decls(&raw.slug, &json!({ "errorCodes": raw.error_codes }))
            .with_context(|| format!("{ctx}: errorCodes"))?;
        Ok(Bundle {
            type_uri: raw.type_uri,
            slug: raw.slug,
            version: raw.version,
            status: raw.status,
            deprecation: raw.deprecation,
            extends: raw.extends,
            policy,
            error_codes,
            schema: raw.schema,
            response_schema: raw.response_schema,
            property_order: raw.property_order,
        })
    }
}

/// `Ok` when `target` is a `MAJOR.MINOR` with `version`'s MAJOR and a strictly
/// lower MINOR — the only links SPEC §5.2 permits.
fn extends_shape(version: &str, target: &str) -> std::result::Result<(), &'static str> {
    let parse = |v: &str| -> Option<(u32, u32)> {
        let (major, minor) = v.split_once('.')?;
        Some((major.parse().ok()?, minor.parse().ok()?))
    };
    let (Some((major, minor)), Some((t_major, t_minor))) = (parse(version), parse(target)) else {
        return Err("is not a MAJOR.MINOR version");
    };
    if t_major != major || t_minor >= minor {
        return Err("must name a strictly earlier version with the same MAJOR");
    }
    Ok(())
}
