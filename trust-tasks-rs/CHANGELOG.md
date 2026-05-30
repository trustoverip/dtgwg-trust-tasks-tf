# Changelog

All notable changes to `trust-tasks-rs` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to a `MAJOR.MINOR` versioning scheme that tracks
the corresponding `SPEC.md` framework version.

## [0.1.3] — 2026-05-30

Additive cross-device step-up + push-wake-up support, regenerated from the spec changes in PRs #61 and #62. The change set is purely additive, so existing consumers pick up `0.1.3` via `cargo update -p trust-tasks-rs` without code changes.

### Changed — existing specs

- **`auth/step-up/approve-response/0.1`**: adds optional `evidence` — a tagged union (`did-signed` | `webauthn`) selecting the elevation gate. The `webauthn` variant carries an `AuthenticatorAssertionResponse` over the step-up challenge, enabling cross-device AAL2 (a browser session elevated by a passkey on the phone) alongside the existing DID-signed gate. Adds the `assertion_invalid` and `no_gate` error codes. Generated as the `Evidence` enum.
- **`auth/step-up/approve-request/0.1`**: adds optional `acceptableEvidence` (which evidence gates the relying party will accept) and `webauthn` (`PublicKeyCredentialRequestOptions`) so a relying party can drive a passkey-backed elevation.
- **`device/_shared/0.1/device-binding`** (`DeviceBinding`): adds the non-secret `pushCapable` flag for `device/list` visibility. The push token itself is held by the mediator (per the push wake-up binding), never by the maintainer/VTA.

## [0.1.2] — 2026-05-27

This is a roll-up release covering everything merged since 0.1.1 (PRs #40–#56). The change set is overwhelmingly additive — new spec families regenerated into `specs::*` — so existing consumers should pick up `0.1.2` via `cargo update -p trust-tasks-rs` without code changes.

### Added — new spec families

- **`did-management/`** (24 specs) — full DID hosting lifecycle: `did/{check-name, register, publish, info, list, delete, disable, enable, change-owner, rollback, problem-report}`, `domain/{create, update, disable, enable, purge, set-default, assign, unassign}`, `me/domains`, `server/{register, health, stats-sync}`, `registry/{admin-register, deregister}`. Shared schemas for `DidRecord`, `DomainEntry`, `ServiceInstance`, and the webvh method extension.
- **`webvh/`** (3 specs) — did:webvh-protocol-internal mechanics: `witness/publish`, `sync/update`, `sync/delete`.
- **`vault/`** (11 specs) — credential manager surface: `list, get, upsert, delete, sync, proxy-login, release, sign-trust-task, usage` and shared schemas (`VaultEntry`, `VaultSecret`, `SessionBlob`, `SealedEnvelope`, `ConsumerContext`).
- **`device/`** (5 specs) — Companion / Service lifecycle: `register, list, disable, wipe, heartbeat`.
- **`policy/`** (4 specs) — Rego policy CRUD: `list, upsert, delete, evaluate`.
- **`sync/`** — `sync/event/0.1` server-push event envelope.
- **`provision/integration/0.1`** — generic provisioning Trust Task for template-driven integration bootstrap.
- **`auth/`** (15 specs) — full session lifecycle: `challenge, authenticate, refresh, revoke-session, whoami, sessions/list, step-up/{approve-request, approve-response}, passkey/{enroll, login}/{start, finish, invite}`.
- **`acl/swap-key/0.1`** — atomic ACL key rotation for the swap-then-rotate enrolment pattern.
- **`confirm/{request, response}/0.1`** — generic confirmation envelope.

### Changed — existing specs

- `did-management/did/register/0.1`: adds `did-management/did/register:invalid_path` error code, mirroring `did/check-name`'s identical code for the atomic register flow.
- `did-management/server/register/0.1`: adds optional `enabledMethods[]` and `protocolVersion` request fields so hosting servers can declare their capabilities. Both default cleanly when omitted.
- `provision/integration/0.1`: makes `contextId` optional with inference rules from the template's declared scope.
- `vault/proxy-login/0.1`: adds optional `nonce` and `ttlSecondsHint` fields for RP-issued challenges and caller-hinted TTLs.
- `vault/_shared/0.1/vault-entry`: adds optional `principalDid` metadata so vault entries can carry the DID they would act AS in a proxy-login.
- `vault/_shared/0.1/vault-secret`: `SealedSecret`/`SealedSessionBlob` reshape into pluggable envelopes; `VaultSecret::Password` gains a `PasswordLoginConfig` for site-specific form quirks.

### Added — framework

- `category` taxonomy is now an enforced enum at the spec.meta level — invalid categories fail validation.

## [0.1.1] — 2026-05-19

### Changed — consumer-pipeline hardening (SPEC §7.2 items 6 + 7)

- **BREAKING**: `consume_inbound`'s handler signature changes from
  `FnOnce(TrustTask<P>) -> Future<Result<TrustTask<R>, RejectReason>>`
  to `FnOnce(TrustTask<P>, ResolvedParties) -> Future<Result<TrustTask<R>, ErrorResponse>>`.
  Handlers now receive the SPEC §4.8.1-resolved parties (no need to call
  `transport.resolve_parties` themselves) and return a fully-routed
  `ErrorResponse` on refusal, freeing them to mint extended codes
  (SPEC §8.5), attach task-specific `details`, and apply spec-specific
  routing without being constrained to the framework's `RejectReason`
  vocabulary. The docstring spells out that handler-built errors are
  passed through verbatim — handlers that reject for identity-style
  reasons MUST use `reject_with_recipient` or `TransportHandler::reject`
  to preserve §8.1 routing.
- **BREAKING**: `consume_inbound`'s `verifier: Option<&V>` parameter is
  replaced by `policy: ProofPolicy<'_, V>` with three explicit variants:
  `Verify(&V)`, `RejectIfPresent`, and `AcceptUnverified`. Forces the
  security tradeoff to be a deliberate, audit-able choice at the call
  site instead of an `Option::None` whose meaning was ambiguous. The
  `AcceptUnverified` variant is the documented opt-out for transports
  whose integrity guarantees live outside the in-band proof (signed
  DIDComm envelopes, mTLS-bound HTTPS).
- `consume_inbound` now reads `Payload::IS_PROOF_REQUIRED`
  authoritatively for the SPEC §7.2 item 7 proof-required check,
  replacing the `verifier.is_some() && !P::IS_BEARER` heuristic. Per-
  spec proof contracts are enforced regardless of the chosen policy.
- **SECURITY**: under `ProofPolicy::RejectIfPresent`, `consume_inbound`
  rejects documents carrying an in-band proof with `malformed_request`.
  Silently dropping a producer-supplied proof previously misled the
  producer about the integrity guarantees of the exchange. The wire-
  exposed `message` is a neutral constant — it cites the spec section
  but does not name the consumer's configuration, so an unauthenticated
  probe cannot fingerprint deployments by verifier coverage.

### Added

- `Payload::IS_PROOF_REQUIRED` (default `false`). Codegen emits an
  explicit `const IS_PROOF_REQUIRED: bool = true;` override when a spec's
  front matter declares `proofRequirement.requirement: REQUIRED`. Mirrors
  the existing `IS_BEARER` plumbing.
- `Payload::extended_code(local)` convenience trait method — builds a
  `TrustTaskCode::Extended` under the payload's own slug (sourced from
  `Self::TYPE_URI`). Eliminates slug-literal drift in handler code and
  makes the SPEC §8.5 namespace rule enforceable by construction.
  `TrustTaskCode::new_extended(slug, local) -> Result<Self, ParseCodeError>`
  is the runtime-input-safe constructor.
- `DynProofVerifier` trait + `ErasedVerifier<V>` adapter + `erase_verifier`
  helper. Object-safe wrapper around [`ProofVerifier`] for transport
  bindings that need to store a verifier behind `Arc<dyn …>` on shared
  state (the generic method on `ProofVerifier::verify` is not
  object-safe). Reusable across bindings (HTTPS, future DIDComm, …).
- `PROOF_NOT_ACCEPTED_BY_POLICY` constant — the wire-safe message
  shared by `consume_inbound` and transport bindings for the
  proof-without-verifier rejection. Sanitised: no mention of the
  consumer's configuration that could be used as a probe oracle.

## [0.1.0] — initial pre-release, tracks `SPEC.md` 0.1

### Added — framework primitives

- `TrustTask<P>` document envelope (SPEC §4.2) with serde round-trip,
  forward-compatible extra members, and JSON-LD `@context` support.
- `TypeUri` (SPEC §4.4, §6.1) — parses
  `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` with `#request` /
  `#response` fragments, accepts private-authority variants from
  SPEC §6.5, rejects reserved-namespace slugs. Rejects `http://`
  scheme per the tightened SPEC §6.1 (HTTPS only).
- `Proof` data structure (SPEC §4.7) with forward-compat extras for
  future cryptosuites.
- `ErrorPayload`, `StandardCode`, `TrustTaskCode` — the
  `trust-task-error/0.1` payload (SPEC §8.2) with standard codes
  (§8.3) and namespaced extension codes (§8.5).
- `RejectReason` enum modelling the SPEC §7.2 rejection conditions;
  impls `std::error::Error` and converts to `ErrorPayload` via `From`.
- `ErrorResponse` type alias and `impl Error for TrustTask<ErrorPayload>`
  so error responses `?`-propagate.
- `TrustTask::reject_with` / `respond_with` / `validate_basic` — wire
  the §4.4.1 success and failure response paths and apply §7.2 items 4
  and 5. `is_expired_at` / `validate_basic` use `now ≥ expiresAt`
  (inclusive bound), matching SPEC §4.2 / §7.2 item 4.
- `TransportHandler` trait (SPEC §9.2) with the §4.8.1 in-band-wins
  precedence baked into a default `resolve_parties` method. Reference
  implementations: `NoopHandler`, `InMemoryHandler`.
- `Payload` trait + `TrustTask::for_payload` for typed per-spec payloads.
- `specs::*` module tree — per-spec `Payload` / `Response` structs
  generated by `trust-tasks-codegen` from
  `specs/<slug>/<version>/payload.schema.json`.
- `#[cfg(test)] mod conformance` inside each generated module —
  round-trip tests harvested from each spec's `spec.md`.

### Added — audience binding + identity-mismatch routing (SPEC §4.8.2, §8.1)

- `TrustTask::enforce_audience_binding()` checks `proof.is_some() &&
  recipient.is_none() && !P::IS_BEARER` and rejects with
  `MalformedRequest` per SPEC §7.2 item 8.
- `Payload::IS_BEARER: bool = false` — codegen emits an override when a
  spec opts in via `bearer: true` front-matter.
- `TrustTask::reject_with_recipient` for explicit recipient override.
  Used by `TransportHandler::reject` to apply SPEC §8.1 routing —
  identity-mismatch rejections go to the transport-authenticated peer,
  never the contested in-band issuer.
- `RejectReason::wire_message()` returns sanitised strings for
  identity-bearing rejections; `From<RejectReason> for ErrorPayload`
  uses it so the consumer's expected VID isn't leaked over the wire.

### Added — opt-in validation, proof verification trait

- `validate` Cargo feature — runtime JSON Schema validation against
  the embedded `payload.schema.json` files via the `jsonschema` crate
  (Draft 2020-12). `ValidatedPayload` trait emitted by the codegen on
  every request payload.
- `ProofVerifier` trait (async via `async-trait`) + `VerificationError`
  enum — the seam where cryptosuite crates plug in. No suites
  implemented in this crate; companion crates live elsewhere
  (`trust-tasks-proof` with the `affinidi` feature).
- `Dispatcher<R>` keys its routes on `TypeUri::for_routing()` so the
  `#request`-fragmented and bare forms route together, per SPEC
  §4.4.1 item 1.

### Added — discovery (SPEC §11)

- `discovery` module with `match_slug` / `query_matches` primitives and a
  `DiscoveryRegistry` builder. `respond_to(&query)` consumes a typed
  `trust-task-discovery/0.1` request and emits the matching subset of
  registered Type URIs.
- `DiscoveryRegistry` implements `FromIterator<impl Into<String>>` so
  routing tables (e.g. `HttpsServer`'s) can `.collect()` directly into
  a registry.
- Generated `specs::trust_task_discovery::v0_1::{Payload, Response}`
  via the codegen, with the spec.md request/response examples wired
  into the standard `#[cfg(test)] mod conformance`.
- `TypeUri` parser accepts `trust-task-discovery` as a framework-defined
  slug per the SPEC §6.1 reserved-slug list.

[0.1.3]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.3
[0.1.1]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.1
[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
