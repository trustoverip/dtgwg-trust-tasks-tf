# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.15 — 2026-09-22


### Added

- **persona**: Where a face may be worn, where it is, and what it has done (#577)

* feat(persona): where a face may be worn, where it is, and what it has done

  Design note (VTI docs/05-design-notes/persona-context-first.md) §5.4 and
  §9.6, and `until` on compose.

  FaceReach — a pool face may carry `reach`: `{kind: anywhere}` (the
  default, and what absent means) or `{kind: only, contextIds: [...]}`.
  binding/set refuses to wear a face outside it (`outsideReach`), and
  profile/put refuses to narrow it past a context the face is worn in now
  (`boundOutsideReach`, naming them). A tagged object rather than a bare
  context list, so "unrestricted" and "nowhere" cannot be confused: `only`
  needs at least one context, and nowhere is a retired face.

  persona/profile/usage/1.0 — where one face is worn now, with each
  binding's `until` and the face's reach beside them. Holder-only: it is
  the map of which personas are one face.

  persona/profile/timeline/1.0 — one face's history joined, oldest first:
  composed, worn, unworn, expired, disclosed, valueChanged, promoted,
  retired, reinstated. Never a value and never a private label. A
  maintainer records from now on what it would not otherwise keep — a
  binding taken off leaves no trace in the one that replaces it — and may
  omit what happened before and left no record.

  profile/compose takes `until`, for the face composed at the door of a
  conference; `untilNotFuture` refuses one in the past or without a
  personaDid.

## 0.1.14 — 2026-09-21


### Added

- **trust-task-discovery**: Publish 0.2 with a three-part frameworkVersion (#571)

trust-task-discovery/0.1's response admits frameworkVersion only as
  MAJOR.MINOR, so a responder cannot advertise a framework release as the
  framework has written it since 0.4.0 (SPEC §5.1.1) — 0.6.0, or a later
  PATCH. Widening 0.1 in place would make every 0.1 discoverer that
  validates the response reject a conforming one, so this is 0.2.

  - specs/trust-task-discovery/0.2: identical to 0.1 except the response's
    frameworkVersion is MAJOR.MINOR.PATCH; adds "Relationship to 0.1"
    (answer each version in the version asked; discoverers SHOULD ask 0.2)
    and the four Security & Privacy sub-sections. Bindings generated.
  - trust-tasks-rs: DiscoveryRegistry::respond_to_v0_2; the configured
    release is held three-part and written MAJOR.MINOR in a 0.1 response.
    Default "0.2" (stale) -> "0.6.0".
  - trust-tasks-https: with_discovery/enable_discovery answer both 0.1 and
    0.2, and advertise both.
  - trust-tasks-dart-https: enableDiscovery answers both versions; default
    frameworkVersion '0.5' -> '0.6.0'; example asks in 0.2.

- **persona**: Retire a face, warn before deleting one, and let a binding end on its own (#570)

Removing a face means one of three things to the person doing it, and
  the family had tasks for two: "not here" (a null binding/set) and "gone"
  (profile/delete). This adds the middle one.

  persona/profile/retire/1.0 — stop being a face everywhere and keep it.
  Every binding to it is cleared and reported, it drops out of pickers and
  default listings (profile/list `includeRetired`), and binding/set and
  local/binding/set refuse it (`profileRetired`). Values and disclosure
  history are kept. persona/profile/reinstate/1.0 undoes it and binds
  nothing: wearing a face in a context is decided in that context.
  Profile gains `status` and `retiredAt`.

  profile/get and profile/delete return `disclosedTo` {partyCount,
  contextCount}, and the delete spec tells a producer to show it before
  sending: deleting a face does not un-tell anyone.

  binding/set and local/binding/set take `until`; binding/get and
  binding/list return it. At expiry the binding clears and, when the face
  is then worn nowhere, it is retired — never deleted, and never retired
  while another context still wears it. `untilNotFuture` refuses a past
  `until` or one with a null profileId.

## 0.1.13 — 2026-09-21

## 0.1.12 — 2026-09-21


### Added

- **messaging**: Add mediator queue, message, stats and traffic-monitor specifications (#549)

Ten new messaging/* tasks (all 0.1, draft) so a mediator can be operated
  entirely through Trust Tasks — by an administrator across every account, or
  by an account controller over its own queues — instead of the REST
  `/admin/status`, `/purge` and `/queue/status` routes and the retired DIDComm
  admin protocols.



### Other

- Declare outcome evidence for witness/session and vetting/session (#550)

Implements framework §7.3 item 20 in the registry
(trustoverip/dtgwg-trust-tasks-spec#15 and #17).

- spec.meta.schema.json: optional `outcomeEvidence` key (response Type
  URI, binding by id + task digest or by fresh challenge, rationale).
  Excluded from every binding generator, so no codegen change.
- build-registry: checkOutcomeEvidence fails the build when the declared
  response is missing, lacks a response payload, or does not require
  proof and issuedAt (20.1-20.3); when the initiating spec is bearer or
  has no REQUIRED recipient; or when a binding pointer names an
  undeclared payload member (20.5). Warns when retention is not durable.
- witness/session/0.1 declares witness/session/submit#response, bound
  by the VWC's taskContext and taskDigestMultibase.
- vetting/session/0.1 declares its own #response, bound by the card's
  challenge; permitted under 20.5.2 because the statement's issuer is
  the session's issuer.
- witness/session/submit/0.1 cites SPEC §4.9.4 instead of its own
  pairing rule, keeping only the witnessing-specific checks.
- SPEC.md mirror: §4.9.4, §7.3 item 20, the outcome-evidence term, the
  §10 correlator paragraph and the [VTI] reference, ported from the
  canonical repo.

Signed-off-by: Glenn Gore <glenn.g@affinidi.com>

## 0.1.11 — 2026-09-21


### Added

- **vtc**: A community can ask an applicant to tell it about themselves (#543)

From the persona context-first design note (verifiable-trust-infrastructure,
  docs/05-design-notes/persona-context-first.md §5.2): a join manifest could
  ask only for credentials, so a community wanting a display name and a
  country had nothing to put on the "what's required" screen, and nothing
  connected the join ceremony to the applicant's persona.

  All additive, no new versions:

  - vtc/join-requests/manifest/0.2: optional `requestedAttributes` --
    claim types (never values), each `required` (default true) with a
    `purpose` shown to the applicant. Outside every criterion, so no
    requirementsDigest covers it. Answers are self-asserted: a community
    MUST NOT describe one as verified or decide as if it were; a community
    needing an attested value asks for a credential instead.
  - vtc/join-requests/submit/0.2: optional `attributes` -- [{type, value}],
    bound to the applicant by the document proof. New codes
    attributesMissing (a required one is absent) and attributesUnrequested
    (a type the manifest does not request -- refused rather than stored, so
    an over-sharing client cannot leave data with a community that never
    asked). SHOULD be sent through the applicant's own disclosure path so
    their record of what they told whom includes it.
  - vtc/_shared JoinRequest: optional `attributes`, so show and list carry
    the answers to reviewers, as the applicant's own statement.

  Bindings regenerated in Rust, TS, Go and Dart.

## 0.1.10 — 2026-09-21


### Added

- **persona**: Honest pins, purge-version, and entry slots (#538)

## 0.1.9 — 2026-09-21


### Specifications

- **persona**: Lifecycle answers — disclosure currency, edit reach, binding label (#533)

Additive members on existing 1.0 schemas, plus normative prose. From the
  persona context-first design note (verifiable-trust-infrastructure,
  docs/05-design-notes/persona-context-first.md, sections 3.2, 3.4, 9.2, 9.3).

## 0.1.8 — 2026-09-21


### Specifications

- **vtc/join-requests/supplement**: Vetting travels in the presentation, so it is replaced with it (#531)

#526 stated that vetting attestations are "attached to the request, not to the
  presentation", and that a consumer MUST NOT discard them when the presentation
  is replaced. That is wrong, and I found it while implementing the consumer
  side.

  A community counts vetting by reading the attestations **out of the
  presentation** it was handed — in the reference implementation,
  `vetting_credentials(vp)` filters the VP's `verifiableCredential` array. The
  per-request vetting record that does exist is a *record*: it answers the admin
  view, the vetter sweep, and the question of which admissions a later-withdrawn
  statement counted toward. No decision reads it. So there is nothing on the
  request for a consumer to decline to discard, and a supplement whose
  presentation omits the attestations is one with no vetting.

  The claim was also internally inconsistent with the rule immediately above it.
  Carrying forward attestations from a superseded presentation is accumulation —
  it decides the request on evidence the applicant is no longer presenting, which
  is exactly the defect the replace-not-merge rule exists to prevent. It merely
  arrives by a different route, so the corrected text forbids it explicitly.

  What survives is the promise underneath, which is the part that mattered: a
  vetting attestation is a credential the **applicant** holds, so re-presenting
  it costs the vetter nothing and no one attests twice. The obligation this puts
  on a deferring community is now stated — name the attestations in the
  `presentationDefinition` alongside everything else required.

  The Correlation subsection carried the same error ("it is what lets vetting
  survive") and is corrected to what the linkage actually buys: the applicant
  keeps their place rather than starting again.

  Prose only, in place, per SPEC §5.2's draft rule — no schema change, so the
  bindings are untouched.

## 0.1.7 — 2026-09-21


### Added

- **vtc/join-requests/supplement**: Let a deferred applicant answer, instead of starting over (#526)

Keyring finding KR-03, the half not closed by `join-requests/withdraw/0.1`
  (#518). A community that cannot decide a request on what it was given defers it
  and says what more it needs — and until now the applicant had nowhere to put
  the answer. The request stays open, `join-requests/submit`'s dedup rule refuses
  a second application, and the only ways out are to withdraw (discarding the
  vetting already gathered) or to wait for a retention sweep neither party
  controls. A deferral was a dead end dressed as a question.

  This task is the answer: the applicant re-presents against the request that
  already exists, the community re-runs its admission policy, and returns a fresh
  verdict.

  ## The response is submit's response

  `{requestId, verdict}` over the shared `Verdict`, exactly as
  `join-requests/submit/0.2` returns. A supplement has precisely the outcomes a
  submission has — including a further `requestMore`, because the community is
  entitled to still not be satisfied, and including `deny` — so a client that can
  read one reads the other with no second code path.

  ## Three decisions that had a tempting wrong answer

  **The presentation replaces, it does not accumulate.** Merging a new
  presentation with the old one produces a claim set the applicant never
  presented and never signed as a whole, covered by no single proof; a consumer
  could not then say what the applicant actually asserted at the moment it
  admitted them. The cost is that the applicant re-presents everything rather
  than only the shortfall, so the spec tells a deferring community to describe
  the whole requirement in its `presentationDefinition`.

  **Replacement governs the presentation and nothing else.** Vetting attestations
  are attached to the request, not to the presentation, and a consumer MUST NOT
  discard them when the presentation is replaced. A vetter who has already
  attested does not attest again because the applicant answered a question about
  a credential. This is what makes supplementing meaningfully different from
  withdraw-and-resubmit, which throws that work away.

  **A merely-queued request cannot be supplemented** — `notAwaitingEvidence`, a
  precondition and not an authorization rule. A `refer`red request waits on the
  community, not on the applicant; accepting new evidence into it would replace
  what a maintainer is reviewing underneath them, leaving the document they were
  reading no longer the one they were asked to decide. An applicant who wants to
  change a request nobody asked them to change withdraws and submits afresh,
  which is visible to everyone.

  ## Other declarations

  Authorization is ownership, as on withdraw: the proven issuer must be the
  applicant on the request, which is the whole entitlement because an applicant
  holds no membership or capability. `notFound` covers both "no open request" and
  "not yours", so the task cannot be used to probe which request ids exist.

  `issuedAt` is REQUIRED because a supplement is replayable in a way a submission
  is not: the request it targets outlives it, so a replay re-runs the policy
  against evidence the applicant has since replaced and can overwrite a newer
  verdict with an older one.

## 0.1.6 — 2026-09-20


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

## 0.1.5 — 2026-09-20


### Added

- **vtc/join-requests**: Withdraw/0.1 — an applicant closes their own request (#518)

An applicant whose request is answered `requestMore` has no way to end it.
  The community holds it open, the applicant cannot submit another, and the
  only thing that resolves it is a retention sweep neither party controls.

  The registry already assumed this task existed: `join-requests/status/0.1`
  lists `withdrawn` among the states it can report, and nothing could reach
  it. This is the task that does.

  Modelled on `members/self-remove/0.1` — the same shape of act, a subject
  exercising authority over their own record — and scaffolded with
  `npm run new-spec` rather than copied from it, per CONTRIBUTING-SPECS.md.

  Three things worth review:

  **`requestId` is OPTIONAL**, following `join-requests/status/0.1` and for
  the same reason: an applicant whose submit response was lost never received
  an id, and would otherwise have no way to reach their own request. A
  consumer given one MUST prefer it over inferring the request from the
  caller.

  **`issuedAt` is REQUIRED for a reason specific to this task**, beyond the
  §7.3 item 17 floor. An applicant may withdraw and then apply again — that
  is the point of it — so a captured withdrawal replayed later does not
  repeat a harmless act: it closes the *next* request, which the applicant
  never withdrew. The two documents are identical apart from when they were
  issued.

  **`notFound` deliberately conflates "no open request" with "not yours"**,
  so a caller cannot probe whether a given request id exists on a community.
  `alreadyDecided` is kept distinct because an applicant is entitled to know
  the outcome of their own request, and because retrying will never change
  it.

  Bindings regenerated for all four targets; `check-bindings` reports 452
  specs against 452 TypeScript, 447 Rust, 452 Go and 452 Dart modules, all
  agreeing. No version or changelog edits — release-plz owns those here.

## 0.1.4 — 2026-09-19


### Added

- **vta/contexts**: Preview the whole subtree a delete destroys, and report host copies it could not remove

Deleting a context deletes its sub-contexts and everything they hold, to any depth. Neither vta/contexts/delete/1.0 nor its preview said so, and the preview had nowhere to say what the cascade would reach.

  vta/contexts/preview-delete/1.0 gains subContexts, with the existing arrays defined as the union over the whole subtree. The narrower reading — preview the named context alone — is the natural one and is dangerous: a context whose children hold keys and DIDs previews as holding nothing, and a caller deciding whether the delete needs force decides about the wrong thing.

  vta/contexts/delete/1.0 gains daemonCleanupErrors, the subtree-wide form of the daemonCleanupError that vta/webvh/dids/delete/1.0 already reports for a single DID, and now requires that a did:webvh DID in the subtree be deleted the way that task deletes one — published log removed from its hosting server, credentials revoked, authority withdrawn. Removing only the local record is not a deletion: the log keeps resolving for every party except its owner, and the records that could remove it are the ones just destroyed.

  Both members are optional and additive; the refusal and cascade prose describes behaviour that was already implemented but unstated.

## 0.1.3 — 2026-09-17


### Added

- **vta/did-templates**: A template can declare which algorithms its keys use (#508)

* feat(vta/did-templates): a template can declare which algorithms its keys use

  Adds `vta/_shared/0.2/did-template.schema.json` and `create`, `update`, `get`
  and `list` at `3.0`. A template's `keys` block names each key slot's purpose and
  its acceptable algorithms, most preferred first:

      "keys": {
        "signing": { "purpose": "signing", "algorithms": ["mldsa44", "ed25519"] },
        "ka":      { "purpose": "keyAgreement", "algorithms": ["x25519"] }
      }

  A list rather than a single value because a fleet does not migrate atomically:
  that says mint ML-DSA-44 where the implementation can and Ed25519 otherwise, so
  one template serves a VTA with post-quantum support and one without.

  ## Why a new shared version rather than an edit

  `DidTemplate` lives in `_shared/0.1`, which the shipped 1.0 and 2.0 specs
  reference. Editing it in place would retroactively change what those versions
  mean, which is what versioning exists to prevent. `_shared/0.2` follows the
  precedent `credentials` and `device` already set, and 1.0/2.0 keep pointing at
  0.1.

  ## Why four task families, not two

  `create` and `update` carry a `DidTemplate`; `get` and `list` return a
  `DidTemplateRecord`, which flattens the same fields. A stored v2 template
  fetched through `get/2.0` would fail validation, so all four move together —
  otherwise the template can be written and not read back.

  ## The change is breaking, for a stronger reason than expected

## 0.1.2 — 2026-09-16

## 0.1.1 — 2026-09-16


### Fixed

- **dart**: Take trust_tasks to 160/160 on pub.dev (#473)

pub.dev scored the published 0.1.0 at 140/160, with two specific deductions.

  -10 "The package description is too long." The field was 196 characters against
  a 60-180 window; search results only render the first part of it, which is why
  pub.dev treats an over-long one as a failed pubspec check rather than a style
  note. Now 148, with the window recorded in a comment above it so the next edit
  does not silently fall outside it again.

  -10 "No example found." Adds example/main.dart, which is runnable rather than
  illustrative: it decodes an inbound document, runs it through consumeInbound
  against the generated acl/grant policy, and delivers it three times to show both
  halves of SPEC §7.2 item 11 —

    first delivery    : handled -> .../acl/grant/0.1#response
    identical resend  : duplicate (already executed, inFlight=false)
    escalated resend  : rejected -> idConflict
    handler ran 1 time(s) — the effect happened exactly once.

  Writing it as something that runs is what caught a bug in it: the first draft
  omitted the proof acl/grant declares REQUIRED, so all three deliveries returned
  proofRequired and the example demonstrated a rejection rather than the pipeline.

  Verified with pana 0.23.19 — the analyzer pub.dev runs — against the working
  tree: Points: 160/160. `dart analyze --fatal-infos`, the format check and the
  74 tests all still pass with example/ included in the analysed set.

  The score will not move until a release carries these files, so the improvement
  lands with 0.1.1 — which is also the first version the trusted-publishing
  automation will publish end to end.

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `lib/src/runtime/`.
