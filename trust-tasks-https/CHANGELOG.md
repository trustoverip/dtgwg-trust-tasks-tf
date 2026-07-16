# Changelog

All notable changes to `trust-tasks-https` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

## [0.2.1] — 2026-07-16

### Added

- `HttpsClientBuilder::timeout(Duration)` and
  `HttpsClientBuilder::connect_timeout(Duration)`, plus the
  `DEFAULT_TIMEOUT` (30s) / `DEFAULT_CONNECT_TIMEOUT` (10s) constants.

### Changed

- **The client's `reqwest::Client` now carries finite timeouts by
  default.** Previously it was built with no timeout at all, so a peer
  that accepted the connection and never answered would hang the caller
  forever. An exchange that legitimately needs longer than 30s can raise
  the limits via the new builder methods. Behavioral change: calls that
  previously hung indefinitely now fail with a `ClientError::Http`
  timeout error.

## [0.1.2] — 2026-05-27

### Changed

- Track `trust-tasks-rs` 0.1.2. No public API changes in this crate; the
  bump exists so consumers can `cargo update -p trust-tasks-https` and
  pick up the new spec families (`did-management/*`, `webvh/*`,
  `vault/*`, `device/*`, `policy/*`, `provision/integration`, etc.) over
  the HTTPS transport without further dependency surgery.

## [0.1.1] — 2026-05-19

### Added

- `HttpsServerBuilder::with_verifier(verifier)` — plug in any
  `trust_tasks_rs::ProofVerifier` implementation. When configured, the
  server verifies the `proof` member of every proof-bearing inbound
  document and rejects `proof_invalid` on failure; when absent, the
  server rejects proof-bearing documents with `malformed_request` as
  before. Stored internally as `Arc<DynProofVerifier>` for object-safe
  dispatch. This lets `acl/grant` / `acl/revoke` / `acl/change-role`
  (all `proofRequirement: REQUIRED`) flow end-to-end on the binding
  once a verifier is configured — without one, `IS_PROOF_REQUIRED`
  enforcement still fires per the security fix above.
- `RequestContext.resolved: ResolvedParties` — handlers can now read
  the SPEC §4.8.1-resolved issuer/recipient pair directly instead of
  re-running `TransportHandler::resolve_parties` to re-derive what the
  dispatch pipeline already computed. Matches the ergonomic improvement
  in `trust-tasks-rs` `consume_inbound`'s handler signature.

### Fixed — security

- The server previously accepted documents carrying a `proof` member
  without verifying it (the binding has no in-band verifier). A producer
  that signed its document and saw a 200 had no way to learn that the
  signature was never checked. The server now rejects proof-bearing
  documents with `malformed_request` at the framework-checks stage,
  matching the same rule `consume_inbound` now applies under
  `ProofPolicy::RejectIfPresent`. The wire-exposed reason is the
  framework-shared `PROOF_NOT_ACCEPTED_BY_POLICY` constant — naming
  the server's configuration would let an unauthenticated probe
  fingerprint verifier coverage across a fleet.
- The server previously accepted *proofless* documents on specs whose
  front matter declares `proofRequirement.requirement: REQUIRED` —
  silently violating SPEC §7.2 item 7. The dispatch closure now
  consults `Payload::IS_PROOF_REQUIRED` after downcast and rejects with
  `proof_required` when the spec requires a proof and none is present,
  regardless of whether the binding is configured to verify.
  Combined with the verifier plug-in point added under "Added" below,
  REQUIRED specs (e.g. `acl/grant`, `acl/revoke`, `acl/change-role`)
  now flow end-to-end on the binding when a verifier is wired in via
  `HttpsServerBuilder::with_verifier`; without a verifier they are
  correctly refused with `proof_required` (proofless) or
  `malformed_request` (proof-bearing, the policy rejection above).

## [0.1.0] — initial pre-release, tracks `trust-tasks-rs` 0.1, `SPEC.md` 0.1

### Added

- HTTPS transport binding for the Trust Tasks framework (SPEC §9).
  Binding URI: `https://trusttasks.org/binding/https/0.1`.
- `HttpsServer` — axum-based, single `POST /trust-tasks` endpoint.
  Builder API: `HttpsServer::builder().local_vid(...).with_auth(...).on::<P, Resp, _>(handler).build()`.
  Runs the full SPEC §7.2 pipeline per request: `resolve_parties`
  (§4.8.1 in-band-wins), `validate_basic` (items 4 + 5),
  `enforce_audience_binding` (item 8 / §4.8.2), dispatch by canonical
  Type URI (§4.4.1 item 1), then the user handler. Success ⇒
  `respond_with`; failure ⇒ `TransportHandler::reject` (§8.1 routing).
- `HttpsServer::with_discovery(registry)` and
  `HttpsServer::enable_discovery()` — one-line wiring of
  `trust-task-discovery/0.1` on the server. `enable_discovery()`
  snapshots every Type URI registered via `.on(...)` (plus discovery
  itself) and serves the matching subset on each query.
- `HttpsClient` — reqwest-based typed `send::<Req, Resp>` with bearer
  auth, automatic in-band identity defaulting, and `ClientError` that
  distinguishes transport-level failures, framework
  `trust-task-error/0.1` documents, and untyped non-2xx fallbacks.
- `HttpsHandler` — `TransportHandler` impl that maps the bearer-
  authenticated sender into the framework's transport peer.
- `BearerAuth` — minimal `HashMap<token, VID>` `Auth` implementation
  for demos and tests; production deployments plug in their own
  `Auth` impl.
- `status_for_code(code) -> u16` — informative HTTP status mapping
  from `StandardCode` (400 / 401 / 403 / 422 / 500 / 503).
- `examples/server_demo.rs` + `examples/client_demo.rs` — runnable
  end-to-end demo on `localhost:3000`.
- `tests/end_to_end.rs` — full HTTP loop covering happy path,
  identity-mismatch routing + sanitised wire message,
  unsupported-type, spec-handler rejection, plus discovery
  advertisement and pattern-filtered discovery.

### Cargo features

- `client` (default) — `HttpsClient` + `reqwest`.
- `server` (default) — `HttpsServer` + `axum` + `tokio` + `tower`.

[0.1.1]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-https-v0.1.1
[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
