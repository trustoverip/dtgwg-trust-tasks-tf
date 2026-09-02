# Changelog

All notable changes to `trust-tasks-https` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate versions independently of `trust-tasks-rs` — it takes its own
leading bump when a `trust-tasks-rs` break reaches it, rather than aligning
to that crate's number (see the `0.6.5` → `0.7.0` release for the shape).

## [0.17.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.17.5...trust-tasks-https-v0.17.7) — 2026-09-02


## [0.17.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.17.4...trust-tasks-https-v0.17.5) — 2026-09-02


## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.17.3...trust-tasks-https-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.17.2...trust-tasks-https-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.17.1...trust-tasks-https-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.17.0...trust-tasks-https-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-https-v0.16.0...trust-tasks-https-v0.17.0) — 2026-08-27


### Changed

- **versioning**: Release the trust-tasks-rs-exposing crates in lockstep ([#315](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/315))


### Specifications

- Bound every free-text payload member with a maxLength (§7.3) ([#296](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/296))

* spec: bound every free-text payload member with a maxLength

  SPEC.md §7.3 (framework 0.5.0) requires that any member holding free
  text declare a `maxLength`. 92 free-text string members across 83 draft
  schemas carried none, leaving the wire contract unbounded and every
  consumer to invent its own ceiling — or none, which is what §10.3
  (schema-validation DoS) exists to prevent.

  Bounds are chosen per member from the vocabulary the registry already
  uses rather than applied uniformly:

    256   `label`, `comment` — a display name or an OpenSSH key comment;
          matches the existing 256 on provision/integration `label` and
          the `name` members alongside it.
    500   requester-authored prose that a surface renders to a human who
          is deciding something; matches task-consent/request/0.1 `note`,
          the registry's considered consent-surface bound.
    1024  `reason`, `description`, `message` — operator or service prose
          recorded for audit or returned as a diagnostic; matches the six
          existing `reason: 1024` and the `description: 1024` in policy/
          and vtc/endorsement-type.
    16384 chat/message `text` — the task's actual content rather than
          metadata about it; matches the corpus's long-form bound on
          vault `secureNotes`.

  All amended specifications are `status: draft`, so the change is made in
  place per SPEC §5.2. Deliberately untouched:

    * 17 members in `retired` specifications, frozen by SPEC §6.4.
    * messaging/_shared/0.1 `AuditEntry.detail` and did-management/
      _shared/0.1 `DomainEntry.label` — shared $defs reachable from a
      retired specification, so bounding them would change a frozen
      specification's effective wire contract.
    * vault/_shared/{0.1,0.2,0.3} `TspMessageEnvelope.message` — opaque
      base64url TSP bytes, not free text.

  The `label` description in vault/_shared/*/vault-entry.schema.json said
  the wire spec enforced no maximum length. It now does, so the sentence
  is corrected rather than left contradicting the schema it annotates.

  `npm run validate` re-checks all 533 fenced example documents against
  the amended schemas; none is rejected by a new bound.



## [0.16.0] - 2026-08-26

### Changed

- **BREAKING: `HttpsServer::on` takes one type parameter and requires
  `RequestPayload`.** It was `on::<P, Resp, _>` with `Resp` unconstrained, so
  `on::<acl::grant::Payload, acl::revoke::Response, _>` compiled and
  registered a handler answering `acl/grant` with an `acl/revoke` document.
  The response type is now `P::Response`, inferred, and a mismatched pair is
  a compile error — the same correction #287 made to `HttpsClient::send`.

  Migration: drop the response type parameter.
  `.on::<grant::v0_1::Payload, grant::v0_1::Response, _>(h)` →
  `.on::<grant::v0_1::Payload, _>(h)`.

- **Added `HttpsServer::on_ack` for fire-and-forget specifications.** A spec
  defining no `$defs.Response` gets no `RequestPayload` impl, so `on` cannot
  take it — without this, those specs would have become unregisterable. The
  handler returns `Ok(())` and the server answers `204 No Content`, matching
  the status the binding already gives a duplicate of a completed
  fire-and-forget execution. `DispatchFn` now returns `Option<Value>`:
  SPEC §4.4.1 distinguishes a `#response` whose payload is empty (200 with a
  body) from a specification with no response at all (204).

## [0.15.0] - 2026-08-26

### Changed

- **BREAKING. `HttpsClient::send` takes one type parameter, not two.** The
  response type is now `<Req as RequestPayload>::Response`, which the codegen
  pairs with the request from the one schema:

  ```rust
  // before
  let resp = client.send::<grant::Payload, grant::Response>(req).await?;
  // after
  let resp = client.send::<grant::Payload>(req).await?;
  ```

  The pair could previously disagree — `send::<grant::Payload,
  revoke::Response>(req)` compiled, and the mistake surfaced as a decode
  failure against a live server. It no longer type-checks. A specification
  defining no success response implements no `RequestPayload`, so `send` does
  not compile for it, which is the right answer for an exchange with no reply.

  Call sites written without the turbofish (`let r: TrustTask<grant::Response>
  = client.send(req).await?;`) are unaffected.

- **BREAKING.** Requires `trust-tasks-rs` 0.14, whose generated payload types
  are `#[non_exhaustive]` and built through a builder. A handler returning a
  response payload constructs it with `Response::builder()` rather than a
  struct literal — see that crate's migration note.

  `HttpsServer::on` deliberately keeps both type parameters
  (`on::<Req, Resp, _>`) in this release. Constraining it to
  `Req: RequestPayload` is the same win on the server side and is worth doing,
  but it also decides whether a server may register a handler for a
  response-less specification, which is a separate question from this release's.

## [0.14.1] - 2026-08-26

### Fixed

- **The end-to-end demo now signs, so it completes.** `acl/grant/0.1`
  declares `proof` REQUIRED and this server enforces it, so `client_demo`
  had never succeeded — it returned `422 proofRequired`. It now derives a
  `did:key` from a fixed seed, sets `issuer`/`recipient`/`issuedAt` before
  signing, and signs via `ProofExt`; `server_demo` maps the `alice` bearer
  token to the same `did:key` and verifies with `Verifier::for_did_key()`.

  Examples and dev-dependencies only — no library code changed, and
  dev-dependencies are stripped at publish.

## [0.14.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.13`.** That release flips
  `IS_PROOF_REQUIRED` on `vta/memory/list/0.1`'s response, so a consumer rejects
  an unproofed response it used to accept. This crate re-exports the generated
  types, so the leading component moves with it. No change to this crate's own
  API.

## [0.13.0] - 2026-08-26

### Added

- **SPEC §7.2 item 11 — the duplicate-execution record, on by default.** The
  server now claims each document `id` before dispatch and keeps a record of
  what it executed, so a bit-for-bit resend is absorbed instead of executing a
  second time and a *different* document under a reused `id` is refused
  `idConflict`. `trust-tasks-rs` 0.12.0 shipped the `ReplayGuard` seam; this
  release is the wiring that makes an HTTPS deployment actually keep the
  record.

  The claim sits at the `validated → accepted` transition — after validation,
  attribution, the DID-method pre-screen and proof verification, immediately
  before the handler runs. Claiming earlier would burn the `id` of every
  document the server then refuses, so a corrected resend came back
  `idConflict` forever and a stranger could pre-burn an `id` it had merely
  observed.

  Verdicts are disposed of per §7.2 (*Disposition of a duplicate*), which is
  explicit that a duplicate is never a failure:

  | Verdict | Answer |
  |---|---|
  | `Fresh` | dispatch, then record the response |
  | `Duplicate` with a recorded response | `200` with that response, no second dispatch |
  | `Duplicate`, original still in flight | `202 Accepted` — accepted, executing, no result yet |
  | `Duplicate`, nothing recorded | `204 No Content` — silence, the fire-and-forget case |
  | `Conflict` | `409` `idConflict` |
  | guard error | `503` `unavailable`, `retryable` — **fail closed**, never execute |

  A refusal or a panic downstream of the claim releases it, so a legitimate
  retry is not blocked for the whole retention window.

- **`HttpsServerBuilder::with_replay_guard`** — supply a shared-store
  `ReplayGuard`. The default `InMemoryReplayGuard` is correct for a
  single-process consumer and **not** correct behind a load balancer, where two
  replicas would each accept the same document once.

- **`HttpsServerBuilder::freshness`** — the acceptance window, defaulting to
  `FreshnessPolicy::consequential()` (`issuedAt` REQUIRED, five-minute window).
  Its `record_expiry` is what bounds the replay record: SPEC §7.2 makes the
  window and the record's retention one bound, so there is deliberately no
  second TTL to configure.

- **`HttpsServerBuilder::replay_protection(false)`** — the documented opt-out.
  Its rustdoc says plainly what turning it off re-opens.

- **`ClientError::DuplicateAbsorbed`** — a 2xx with an empty body (the `202` /
  `204` answers above) now surfaces as its own variant rather than as a
  response-decode failure. It is *not* a failure: the effect happened, once.

### Changed

- **BREAKING: the duplicate-execution record and the freshness bound are on by
  default.** Both change what a consumer observes. A deployment that resends a
  document under a reused `id` will now see `idConflict`, and one whose
  producers omit `issuedAt` — or whose intermediaries hold a request longer
  than five minutes — will now see `malformedRequest` / `expired` where the
  document used to be executed. The safe behaviour is the default, as with
  `require_attribution` in 0.11.0; both opt-outs are one builder call and both
  say what they cost.

### Fixed

- **SPEC §10.4: the deserializer's rendering no longer reaches the wire.** Both
  places the server deserialized a body — the document parse and the payload
  downcast — put `serde_json::Error`'s `Display` into the rejection message,
  which spells the member path, the byte offset, and sometimes the full set of
  expected members ("invalid type: string \"not-a-number\", expected a nonzero
  u64"). That describes this consumer's internal type layout to anyone willing
  to POST malformed JSON. Both now use `RejectReason::malformed_from_serde`,
  which `trust-tasks-rs` 0.12.0 exported for exactly this; the detail stays in
  the operator's log.

## [0.12.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.12`.** That crate's `0.12.0` makes
  the duplicate-execution record of SPEC §7.2 item 11 and the freshness bound of
  item 13 part of `consume_inbound`. This crate re-exports its types, so the
  leading component moves with it.

  This server does **not** yet get the guard: its pipeline runs its own §7.2
  checks and never reaches `consume_inbound`, so a `ReplayGuard` still has to be
  threaded through the handler path. Until it is, an HTTPS deployment has no
  duplicate-execution defence.

## [0.11.1] - 2026-08-26

### Changed

- **The two fallback error documents now carry the `trust-task-error` version
  the rest of the stack emits.** `synthesise_error` and
  `suppressed_error_response` spelled `trust-task-error/0.2` out for
  themselves, while `trust-tasks-rs` and the TypeScript runtime both emitted
  `0.5` — so the version a producer saw depended on which branch of the
  rejection path its request hit, and neither README described either number.
  Both now call `trust_tasks_rs::trust_task_error_type_uri()`, which
  `trust-tasks-rs` 0.11.18 made public as the single source of truth.

  `0.5` is a forward-minor move from `0.2`; per SPEC §5.2 a `0.2` consumer
  SHOULD accept it. Every *other* error response this server emits already
  carried `0.5`, because they are built by `TrustTask::reject_with` in
  `trust-tasks-rs` — these two branches were the only ones that disagreed.

### Fixed

- `proof_invalid_when_verifier_rejects` asserted that the verifier's own error
  description reaches the wire. It does not any more, and should not have:
  SPEC §10.4 makes that a resolver-reachability and DID-document oracle for a
  sender who is by construction unauthenticated. The test is renamed
  `proof_invalid_wire_message_withholds_the_verifier_description` and now
  asserts both halves — the constant on the wire, and the description still
  present on `RejectReason`'s `Display` for the operator's log. The sanitising
  itself is in `trust-tasks-rs` 0.11.18.

## [0.11.0] - 2026-08-26

Security release. **Every item under "Changed" alters the behaviour of an
existing deployment**, most of them by refusing traffic that used to be
accepted. Nothing on the wire moved; read the first entry before upgrading.

### Changed

- **BREAKING (behaviour, not API): unattributable documents are now rejected.**
  A new `HttpsServerBuilder::require_attribution` option **defaults to `true`**.
  A document arriving with neither a transport-authenticated peer nor a `proof`
  is refused with `proofRequired` before any handler runs.

  Previously `build()` installed a `BearerAuth` with an empty token map, the
  peer resolved to `None`, and SPEC §4.8.1 then fell back **entirely** to the
  document's in-band `issuer` — an unverified string the sender chose. The
  per-spec `IS_PROOF_REQUIRED` check was the only thing standing behind it, and
  133 of the 344 generated spec modules never set it — including mutating ones
  (`auth/step-up/approve-response/0.2`, `did-management/did/register/0.1`,
  `messaging/admin/add/0.1`). So an unauthenticated, proofless POST claiming
  `"issuer": "did:web:victim"` reached a handler with that string presented as
  the caller's identity. The binding specification says plainly that this
  binding "does not permit `proof` to be omitted"; the runtime had no
  representation of that rule.

  **A deployment that relied on unauthenticated, proofless requests will start
  seeing `proofRequired`/422.** The fix is to authenticate the caller (or have
  it sign). `.require_attribution(false)` restores the old behaviour and is
  documented for local development and tests only — its rustdoc spells out what
  it re-opens.

- **BREAKING: the request pipeline is reordered.** Route lookup, transport auth
  and the attribution gate now run **before** proof verification. Verifying a
  proof resolves its `verificationMethod` DID, which for `did:web` is an
  outbound HTTPS request to a host the *sender* named — and the proof block used
  to run at step 4a, ahead of route lookup at step 5. A stranger could therefore
  make a server built `with_verifier` fetch an arbitrary host by POSTing a
  document whose `type` that server does not even implement, unauthenticated and
  unrated-limited. An unknown type or an unattributable sender is now rejected
  before anything touches the network. Observable consequence: a request that
  was both unroutable *and* carried a bad proof now reports `unsupportedType`
  where it used to report `proofInvalid`.

- **BREAKING: the status table now matches the binding specification's §4.**
  `proofRequired`, `proofInvalid`, `identityMismatch`, `wrongRecipient`,
  `unsupportedType`, `unsupportedVersion` and `expired` all map to **422**;
  they previously mapped to 401, 401, 403, 403, 400, 400 and 400. Any client
  branching on those HTTP statuses needs updating (the framework error code in
  the body is unchanged and remains authoritative). The finer split was itself
  the identity oracle the specification's flat 422 exists to avoid: it told an
  unauthenticated prober, without reading a body, whether its proof or its
  claimed identity was the problem. `malformedRequest`/400,
  `permissionDenied`/403, `idConflict`/409, `unavailable`/503 and
  `internalError`/500 are unchanged.

- **BREAKING: discovery requires an authenticated discoverer by default.**
  `enable_discovery()` / `with_discovery()` built the registry with no auth
  predicate, so any unauthenticated POST got back the server's full route table.
  SPEC §10 says a responder **SHOULD** authenticate the discoverer first. Opt
  back in with the new `HttpsServerBuilder::public_discovery()`.

- **BREAKING: `Content-Type: application/json` is now required**, per the
  binding specification §2; anything else (or an absent header) is refused with
  **415**. `dispatch_handler` took raw `Bytes` and never looked. `text/plain` is
  one of the media types a cross-origin `fetch` or HTML form may send *without*
  a CORS preflight, so any page in a victim's browser could drive this endpoint;
  requiring JSON forces the preflight and the browser refuses on our behalf.

- **BREAKING: `HttpsClient::send` binds each response to its request.** It used
  to return any 2xx body that merely deserialised as `TrustTask<Resp>`. It now
  requires the response's `threadId` to equal the request's `threadId ?? id`,
  its `type` to be the request's `type` with `#response`, its `issuer` to be the
  configured `server_vid` and its `recipient` to be `my_vid` — each with its own
  `ClientError` variant. On an error response, `inResponseTo.id` must name the
  request where the responder populated it (absent under `identityMismatch` per
  SPEC §8.1, which stays acceptable). `ClientError` gained seven variants, which
  is breaking for exhaustive `match`.

- The binding identifier is `https://trusttasks.org/binding/https/0.2`.
  `BINDING_URI`, the crate-level module docs and the README all said `0.1`.

### Added

- `HttpsServerBuilder::allowed_did_methods` — pre-screens the DID method in
  `proof.verificationMethod` before the verifier is called, bounding where
  proof-verification egress can go. Off by default. Depth (resolved-host
  allow-listing, redirect policy, per-resolution timeouts) belongs in the
  resolver behind the verifier, not at this boundary.
- `HttpsServerBuilder::request_timeout` and
  `HttpsServerBuilder::max_concurrent_requests`, with the constants
  `DEFAULT_REQUEST_TIMEOUT` (30s) and `DEFAULT_MAX_CONCURRENT_REQUESTS` (512).
  `into_router` now applies a tower `TimeoutLayer` and `ConcurrencyLimitLayer`;
  `serve()` was previously a bare `axum::serve` with neither, so a stalled
  request held a connection and a task open indefinitely (slowloris). An expired
  request answers `408`.
- `MAX_BODY_BYTES` is now public and exported. A caller replacing the router's
  `DefaultBodyLimit` layer wants to know what it is replacing.
- `HttpsClientBuilder::with_response_verifier` — optional verification of the
  `proof` on response documents. Configuring it makes a signed response
  mandatory: a proofless response is rejected rather than silently downgraded.

## [0.8.0] - 2026-08-16

### Changed

- Requires `trust-tasks-rs` 0.8, which adds the `cancelled` standard error code
  (framework 0.4, SPEC §8.3) and the `trust-task-control/0.1` payload types.
  Additive on the Rust side — `StandardCode` has been `#[non_exhaustive]` since
  0.7.0 — so this crate needed no source change.
- `cancelled` maps to **HTTP 422**, the same bucket as `taskFailed`: a
  deliberate stop is neither a server fault nor a malformed request.

## [0.7.0] - 2026-08-15

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.7, whose `StandardCode` is now
  `#[non_exhaustive]` and carries the new `idConflict` code (framework 0.4,
  SPEC §8.3). Any `match` over `StandardCode` that this crate's types reach
  needs a wildcard arm.
- `idConflict` maps to **HTTP 409 Conflict**. The status table gained a
  wildcard arm sending any future unmapped code to 500, so a framework
  revision no longer fails to compile here.

## [0.6.0] - 2026-08-10

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.6, which narrows `DigestMultibase`
  to the multibase headers CID 1.0 requires. The core types cross this crate's
  public API, so a graph mixing 0.5 with this crate will not type-check. No API
  of this crate changed on its own account.

## [0.5.0] - 2026-08-09

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.5. That release adds a field to
  `TrustTask<P>` for the framework 0.4 `ceremony` member, and the core types
  cross this crate's public API, so a dependency graph mixing 0.4 with this
  crate will not type-check. No API of this crate changed on its own account.

## [0.4.0] - 2026-08-09

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.4. That release changes digest
  payload members from `String` to the validating `DigestMultibase` newtype, and
  the core types cross this crate's public API, so a dependency graph mixing
  `trust-tasks-rs` 0.3 with this crate will not type-check. No API of this crate
  changed on its own account.

## [0.2.2] — 2026-07-17

### Fixed

- `DEFAULT_TIMEOUT` / `DEFAULT_CONNECT_TIMEOUT` (added in 0.2.1) are now
  re-exported from the crate root as intended; previously they were
  documented as public API but unreachable, which also broke the
  workspace doc build under `-D warnings`.

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
