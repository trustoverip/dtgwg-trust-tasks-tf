# Changelog

All notable changes to `trust-tasks-https` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

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
