# Changelog

All notable changes to `trust-tasks-https` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

## [Unreleased] — staged into `0.1.1`

### Fixed — security

- The server previously accepted documents carrying a `proof` member
  without verifying it (the binding has no in-band verifier). A producer
  that signed its document and saw a 200 had no way to learn that the
  signature was never checked. The server now rejects proof-bearing
  documents with `malformed_request` at the framework-checks stage,
  matching the same rule `consume_inbound` now applies. Deployments
  that need to accept proof-bearing documents must layer in a verifier;
  the binding does not provide one out of the box.
- The server previously accepted *proofless* documents on specs whose
  front matter declares `proofRequirement.requirement: REQUIRED` —
  silently violating SPEC §7.2 item 7. The dispatch closure now
  consults `Payload::IS_PROOF_REQUIRED` after downcast and rejects with
  `proof_required` when the spec requires a proof and none is present,
  regardless of whether the binding is configured to verify. The
  combination of this check and the proof-bearing rejection above
  means specs with `proofRequirement: REQUIRED` (e.g. `acl/grant`,
  `acl/revoke`, `acl/change-role`) are *unreachable* on this binding
  until a verifier plug-in point lands; see the test fixture in
  `tests/end_to_end.rs` for the workaround using `acl/list`
  (`RECOMMENDED`).

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

[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
