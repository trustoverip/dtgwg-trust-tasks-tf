# Changelog

All notable changes to `trust-tasks-rs` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to a `MAJOR.MINOR` versioning scheme that tracks
the corresponding `SPEC.md` framework version.

## [0.2.16] — 2026-07-14

### Fixed

- **`vault/delete` documents `force`.** The reference implementation has always
  accepted it: it skips the grace window and hard-deletes, zeroising the secret
  bytes. The spec did not mention it, and its `consequences` promised that "the
  entry is retained only for a grace period" — which is false when `force` is set.
  A consent surface with no dry-run renders exactly that text, so it would have
  told a human they had a recovery window they did not have.

  `force` is now specified, the consequences describe both cases, and the prose
  says normatively that a consumer gating this task on human approval MUST compute
  per-request effects rather than rely on the static text: `consequences` are
  per-task, and `force` is per-request.

- **`vault/list` documents `status`** (`active` | `archived` | `deleted` | `all`).
  The archival view selector the implementation has always accepted.

Both were found by turning on payload-schema validation in a consumer — which is
the point of turning it on.

## [0.2.15] — 2026-07-14

### Added

- **`vta/webvh/dids/update/1.0`.** The task a caller uses to ask an agent to
  publish a `did:webvh` log entry it holds the update key for. Classified
  `destructive`: supplying `document` rotates the DID's update key, which SPEC
  §7.3 item 13 calls authority-shifting — and the rotation happens whether or not
  the caller asked for it.

- **`schema_index::schema_for(type_uri)`** (feature `validate`). Type URI →
  payload schema, generated from the specs.

  Without it a consumer that dispatches on a Type URI could not *find* the schema
  for the payload it was about to run: `ValidatedPayload::SCHEMA_JSON` is a
  per-type associated const, and a generic gate has no type to name. It could only
  validate by hand-writing a match arm per task and remembering to add one with
  every new task — which is to say, it would validate whatever somebody
  remembered.

## [0.2.14] — 2026-07-13

### Added

- **`task-consent/*` Trust Task family.** New `task-consent/request/0.1` and
  `task-consent/decision/0.1`, plus the shared `task-consent/_shared/0.1`
  (`Effect`, `StatePin`, `Exposure`, `Decision`).

  These are the documents a `PolicyDecision.requireConsent` (added to
  `policy/_shared/0.3` in #111) has been referring to without them existing: it
  requires "a signed consent decision from the named set, bound to the request's
  `payloadDigest`", and no such document was specified. `task-consent/decision`
  is that document.

  `task-consent/request` is the other half — the executor asks an enrolled
  approver to authorize one pending privileged task, and carries the `effects`
  it computed by **dry-running the real handler** against its own prior state.
  That matters because a payload says what was asked for, while only the code
  about to run knows what will happen: a `did:webvh` document update whose
  payload adds one service endpoint also rotates the DID's update keys, and no
  diff of the payload can show that.

  Distinct from the existing `consent/*` family, which gates an inbound
  *messaging conversation* for an AI agent (`ConsentSubject` is a
  platform/conversation pair). Additive — `consent/*` 1.0 is untouched.
  Consumers pick up `0.2.14` via `cargo update -p trust-tasks-rs`.

## [0.2.13] — 2026-07-09

### Added

- **`registry/*` Trust Task family.** New `registry/recognition/0.1` and
  `registry/authorization/0.1` (TRQP v2.0 recognition / authorization queries),
  plus `registry/record/{create,update,delete,read,list}/0.1` for Trust Registry
  record CRUD. Generated from the `registry/*` specs added in #108; writes carry
  `IS_PROOF_REQUIRED`. Additive — no change to existing tasks. Consumers pick up
  `0.2.13` via `cargo update -p trust-tasks-rs`.

  (The `registry/*` source was merged in #108 without a version bump, so it never
  reached crates.io; this release publishes it.)

## [0.2.12] — 2026-06-24

### Added

- **`vta/memory/*` Trust Task family.** New `vta/memory/put/0.1`,
  `vta/memory/list/0.1`, and `vta/memory/delete/0.1` — a generic per-context
  key/value store for AI-agent memory (cross-session recall, context-isolated),
  regenerated from the registry. Consumers pick up `0.2.12` via
  `cargo update -p trust-tasks-rs`.

## [0.2.11] — 2026-06-24

### Added

- **`messaging/admin/*` Trust Task family** — the admin-management surface, mirroring
  the messaging mediator's admin protocol: `admin/add` and `admin/strip` (grant /
  revoke admin rights), `admin/list` (page the admin accounts), `admin/audit-log`
  (page the privileged-change log, newest-first), and `admin/config` (read the
  mediator's version + configuration). Adds the shared `AdminAccount`, `AuditEntry`,
  and `AuditAction` `$def`s to `messaging/_shared`. Additive — no change to existing
  tasks.

## [0.2.10] — 2026-06-24

### Added

- **`vta/credentials/*` Trust Task family.** New `vta/credentials/issue/0.1`
  (issue a scoped, time-boxed Verifiable Credential to a holder, gated by
  operator step-up) and `vta/credentials/revoke/0.1` (withdraw an issued
  credential), regenerated from the registry. Consumers pick up `0.2.10` via
  `cargo update -p trust-tasks-rs`.

## [0.2.9] — 2026-06-23

### Fixed

- **Publish the `messaging` Trust Task family.** The messaging tasks (`ping`,
  `account/*`, `acl/*`, `access-list/*`) were added in #96 but landed without a
  version bump, so they never reached crates.io — the published 0.2.8 predates
  them. This patch republishes with `specs::messaging` included. No source change
  beyond the version; the specs are exactly as merged.

## [0.2.8] — 2026-06-22

Additive new `messaging/*` Trust Task family, regenerated from the registry.
Consumers pick up `0.2.8` via `cargo update -p trust-tasks-rs`.

### Added

- **`messaging/*` family** — generated payload modules for the new
  messaging-infrastructure Trust Tasks (`specs::messaging::*`): `ping`,
  `acl::{get,set}`, `access_list::{add,remove,clear,get,list}`, and
  `account::{add,get,list,remove,change_type,change_queue_limits}`. These
  re-express mediator account / ACL / queue administration and liveness as
  transport-agnostic Trust Tasks, sharing the `messaging/_shared` `Account`,
  `MediatorAcl`, `QueueLimits`, `AccountType`, and `Vid` definitions. No change
  to existing modules.

## [0.2.7] — 2026-06-18

Additive `chat/message` routing flags, regenerated from the registry.
Consumers pick up `0.2.7` via `cargo update -p trust-tasks-rs`.

### Added

- **`chat/message` `isGroup` / `isMention`** — optional booleans on the payload
  (`specs::chat::message::v0_1`). `isGroup` records group/channel vs 1:1 DM;
  `isMention` records whether an inbound message addresses the agent (an
  @-mention of the agent, or any DM) — distinct from `mentions`, which lists the
  participants referenced in the body. Both are signed routing context so the
  audit chain captures where a message was sent. Generated as `Option<bool>`
  (omitted when absent), so a `false` flag can be omitted for byte-lean DMs.

## [0.2.6] — 2026-06-18

`chat/message` renumbered `1.0` → **`0.1`** (aligning with the registry's `0.x`
draft convention) and extended with @-mentions, regenerated from the registry.
Consumers pick up `0.2.6` via `cargo update -p trust-tasks-rs`.

### Changed

- **`specs::chat::message::v0_1`** replaces `v1_0` — the type URI is now
  `https://trusttasks.org/spec/chat/message/0.1`. The `chat/message` task was
  the lone `1.0` outlier among `0.x` drafts; renumbered while still `draft`.

### Added

- **`chat/message` `mentions`** — an optional, ordered array of platform-neutral
  @-mentions on the payload. Each `Mention` references the mentioned party by an
  **opaque `participant` handle** (never a raw address — same model as
  `conversationId`) with an optional `displayName` hint and advisory
  `start`/`length` offsets. The body carries one `U+FFFC` sentinel per mention;
  the Nth sentinel binds positionally to the Nth entry.

## [0.2.3] — 2026-06-17

Additive `vta/did-templates/*` and `vta/contexts/did-templates/*` Trust Task families, regenerated from the registry — the previously-implemented-but-unspecced DID-template management endpoints (global + context scope), now published. Additive; consumers pick up `0.2.3` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::vta::did_templates::{list,get,create,update,delete,render}::v1_0`** — `vta/did-templates/<op>/1.0`: manage **global** DID templates on a VTA. `create`/`update`/`delete` are super-admin gated; `list`/`get`/`render` are open to any authenticated caller. `create`/`get`/`update` return the persisted `DidTemplateRecord`; `list` returns `{ templates }`; `delete` returns `{ name, deleted }`; `render` returns `{ document }`.
- **`specs::vta::contexts::did_templates::{list,get,create,update,delete,render}::v1_0`** — `vta/contexts/did-templates/<op>/1.0`: the context-scoped counterparts, gated on the context's admin (or super-admin) for writes and context access for reads. Each request carries `contextId`.

All twelve are in the `did-management` category, proof REQUIRED (`IS_PROOF_REQUIRED`, `IS_RECIPIENT_REQUIRED`), with member names in lowerCamelCase per SPEC §4.10.

## [0.2.2] — 2026-06-16

New `chat` Trust Task category, regenerated from dtgwg PR #85. Additive; consumers pick up `0.2.2` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::chat::message::v1_0`** — `chat/message/1.0`: a conversational message between an AI agent and a messaging-platform bridge, signed by its author (`eddsa-jcs-2022`, proof REQUIRED) and hash-linked (`prev`) into a verifiable per-conversation chain for audit and dispute resolution. Conversations are referenced by opaque bridge-issued handles. Fire-and-forget (no response document).

## [0.2.1] — 2026-06-07

Additive `vta/passkey-vms/*` Trust Task family, regenerated from dtgwg PR #81 — the previously-implemented-but-unspecced passkey-as-verificationMethod endpoints, now published in the registry. Additive; consumers pick up `0.2.1` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::vta::passkey_vms::enroll_challenge::v0_1`** — `vta/passkey-vms/enroll-challenge/0.1`: a DID administrator requests a WebAuthn registration challenge for a VTA-managed DID. Payload `{ did, label? }`; response carries the challenge + relying-party / user parameters.
- **`specs::vta::passkey_vms::enroll_submit::v0_1`** — `vta/passkey-vms/enroll-submit/0.1`: the administrator submits the WebAuthn registration result; the VTA re-derives the public key from the attestation and, on success, publishes the verificationMethod via a WebVH log entry. Response `{ verificationMethod, webvhVersion }`.
- **`specs::vta::passkey_vms::list::v0_1`** — `vta/passkey-vms/list/0.1`: enumerate the passkey verificationMethods on a DID. Response `{ verificationMethods }`.
- **`specs::vta::passkey_vms::revoke::v0_1`** — `vta/passkey-vms/revoke/0.1`: remove a passkey verificationMethod by fragment via a WebVH log entry. Empty success body.
- **`PasskeyVerificationMethod`** shared shape (`vta/_shared/0.1/passkey-vm`) — a WebAuthn passkey published as a `Multikey` verificationMethod (purpose `authentication`); reused by the `enroll-submit` and `list` responses.

These are admin-gated (`IS_PROOF_REQUIRED`, `IS_RECIPIENT_REQUIRED`), in the `authentication` category.

## [0.1.8] — 2026-06-04

`did-management/did/check-name/0.1` gains an **auto-assign** mode and the shared `DidRecord` gains a `didUrl` locator. Additive; consumers pick up `0.1.8` via `cargo update -p trust-tasks-rs`.

### Added

- **`DidRecord.didUrl`** (optional) — the resolvable URL of the DID's log document (e.g. `https://did.example.com/alice/did.jsonl`), stable from the initial reservation (`versionCount: 0`). Propagates to every `did-management/did/*` response that carries a `record` (`check-name`, `register`, `publish`, `info`, `list`, `change-owner`, `enable`, `disable`, `rollback`, `delete`).

### Changed

- **`check-name/0.1` request `path` is now optional.** Omitting `path` with `reserve: true` requests an **auto-assign** reservation: the host generates a fresh server-side mnemonic, reserves it under the caller, and returns `available: true, reserved: true, record` (with the generated `mnemonic` + `didUrl`). A path-less request without `reserve: true` remains invalid — that conditional is stated in the spec's §Conformance and enforced by the consumer, because the Rust codegen (typify) cannot model JSON-Schema `if/then/else`.

## [0.1.7] — 2026-06-03

Additive `push/*` Trust Task family, regenerated from dtgwg PR #72 (the push-gateway control plane modeled as Trust Tasks). Additive; consumers pick up `0.1.7` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::push::register::v0_1`** — `push/register/0.1`: a device registers its platform push token with a gateway and receives an opaque `WakeHandle` (response). Payload `{ registration, controllerVtaDid }`.
- **`specs::push::provision::v0_1`** — `push/provision/0.1`: the controller VTA sets a handle's trigger allowlist. Payload `{ handle, policy: WakeTriggerPolicy }`.
- **`specs::push::wake::v0_1`** — `push/wake/0.1`: a trigger requests a contentless wake. Payload `{ handle, v, mediator?, count?, urgency? }`; response `{ status }`.

These are addressed to the push gateway and reuse `WakeHandle` / `WakeTriggerPolicy` / `PushRegistration` from `device/_shared`. New `notifications` spec category.

## [0.1.6] — 2026-06-02

Additive push wake-up support, regenerated from the spec changes in dtgwg PR #68 (the push-gateway / VTA-owned-trigger-allowlist model). The change set is additive; existing consumers pick up `0.1.6` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::device::set_wake::v0_1`** — the new `device/set-wake/0.1` Trust Task: a device conveys its opaque `WakeHandle` (`{ gateway, handle }`) to its VTA so the VTA can own the trigger allowlist and provision the push gateway. Payload carries `wake_handle` (absent = clear), optional `push_platform` and `suggested_triggers`; the `#response` carries the effective `WakeTriggerPolicy` and `push_capable`. The raw platform push token never appears — only the opaque handle.
- **`WakeHandle`** and **`WakeTriggerPolicy`** shapes (from `device/_shared/0.1/device-binding`) — the opaque gateway handle and the VTA-owned allowlist of DIDs permitted to trigger a wake.

### Changed

- **`DeviceBinding.pushCapable`** doc — clarified that the push token is held by the gateway alone; the VTA holds only the opaque handle and the allowlist (reflected in `device/register` and `device/list`).

## [0.1.5] — 2026-06-01

Additive step-up policy support, regenerated from the spec changes in this PR (`auth/step-up/policy/0.1` + the `AclEntry.stepUp` field). The wire change set is additive; existing consumers pick up `0.1.5` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::auth::step_up::policy::v0_1`** — the new `auth/step-up/policy/0.1` Trust Task: a relying party's per-operation-class step-up *floor* (`Floor { operation, mode, allowAal1IfNonEscalating }`, `FloorMode` = `none`/`self`/`delegated`/`delegated-any`) plus the `enabled` master switch, with a `#response` carrying the effective policy.
- **`AclEntry.stepUp`** (`AclEntryStepUp { approver, require }`) on the shared `acl/_shared` `AclEntry`, surfaced in every `specs::acl::*` binding. Per-entry, additive-only step-up override: names the subject's approver VID and an optional minimum mode that may raise — never lower — the maintainer's system-wide floor.

### Changed

- Regenerated the `specs::*` modules. `AclEntry` gains the optional `step_up` field; Rust consumers constructing `AclEntry` with a struct literal must add `step_up: None` (deserialization is unaffected — the field is optional and absent ⇒ `None`).

## [0.1.4] — 2026-06-01

Dependency-maintenance release; no spec changes and no behavioural changes. Existing consumers pick up `0.1.4` via `cargo update -p trust-tasks-rs`.

### Changed

- Regenerated the `specs::*` modules with `typify` 0.6, which no longer emits the redundant reflexive `impl From<&T> for T` (a `value.clone()` no-op) on generated types. These auto-derived conversions are extremely unlikely to be referenced directly; the typed payloads and their fields, the `FromStr` / `Display` / `TryFrom` impls, and `validate`-feature validators are all unchanged.
- Bumped the internal `regress` dependency 0.10 → 0.11. `regress` is used only inside the codegen-emitted pattern validators (it does not appear in any public signature), so this is not an observable API change.

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

[0.1.4]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.4
[0.1.3]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.3
[0.1.1]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.1
[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
