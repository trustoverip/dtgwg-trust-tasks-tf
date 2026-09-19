# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

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
