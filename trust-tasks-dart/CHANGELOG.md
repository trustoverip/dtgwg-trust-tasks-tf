# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

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
