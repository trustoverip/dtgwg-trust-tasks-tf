---
slug: vtc/config/import
version: "0.1"
title: VTC Config — Import
summary: Apply a portable configuration document to a Verifiable Trust Community, or preview what applying it would change. Previews by default.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - config
  - import
  - portability
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Rewrites the community's profile and configuration in one call. The caller must be attributable, and the change auditable to them.
sideEffects:
  level: destructive
  rationale: On confirm, overwrites the community profile and stored configuration overrides with the document's values. Prior values are not retained by this task. Previewing has no side effects at all.
consequences:
  - Community profile members named by the document are overwritten.
  - Stored configuration overrides named by the document are overwritten; keys the document omits are left alone, not cleared.
  - Applied keys that are restart-gated do not take effect until the maintainer restarts.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: The response echoes both the old and the new value of every changed field, so it discloses the configuration in force before the import as well as after.
errorCodes:
  - code: vtc/config/import:communityDidMismatch
    meaning: The document was taken from a different community than the one importing it.
    retryable: false
  - code: vtc/config/import:unsupportedSchemaVersion
    meaning: The document's schemaVersion is one this consumer does not implement.
    retryable: false
  - code: vtc/config/import:permissionDenied
    meaning: The consumer lacks the community administrator capability.
    retryable: false
related:
  - vtc/config/export
  - config/patch
  - vtc/community/profile/update
---

## Abstract

The **VTC Config — Import** Trust Task applies a [`ConfigExportDocument`](../../export/0.1/) — the document [`vtc/config/export`](../../export/0.1/) produced — to this community. It is how an operator stands up a replacement maintainer, moves a community between hosts, or restores a configuration they previously captured.

**It previews by default.** `confirm` is `false` unless the caller sets it, and a preview computes the whole diff, reports every rejection, and writes nothing. Preview and apply return the same shape, so an operator UX renders the preview, shows what would change, and re-submits the identical payload with `confirm: true`.

## Why preview is the default

The safe direction of a default is the one whose mistake is recoverable. A caller who meant to apply and previewed instead loses a round-trip; a caller who meant to preview and applied instead has overwritten a live community's configuration with a document they had not yet read. Defaulting to `confirm: true` would make the second mistake the one a forgotten member produces, so the default is `false`.

This is the same reason `rejected` is populated on a preview rather than only on apply. A rejection discovered *after* writing the accepted half of a document leaves the community in a state that matches neither the old configuration nor the imported one.

## Identity is checked, not imported

A document carries `communityDid` so an import can refuse one taken from somewhere else. A consumer whose community already has a profile MUST compare that DID against its own and reject a mismatch with `communityDidMismatch` — before applying anything, including the configuration overrides, which are otherwise independent of the profile. Importing another community's configuration into a live community is not a supported operation and there is no flag to force it.

A consumer with **no** profile yet — a fresh install — accepts any `communityDid`, because that is the standing-up-a-replacement case this task exists for.

The DID is never *written* by an import. Neither is `createdAt`. Identity is established at install; an import that could re-point it would be a community-takeover primitive, so `communityDid` is read for the comparison above and otherwise ignored.

## Omission means "leave alone"

A field the document omits is left at its current value. It is **not** cleared. A caller that intends to clear a nullable profile member sends an explicit `null`, which appears in the diff as a `ConfigFieldChange` with `newValue: null` — distinct from a change with `newValue` absent, which is not a change at all. A consumer MUST NOT treat the two alike; conflating them turns every partial document into a destructive one.

The same rule governs `configOverrides`: keys the document does not name keep whatever the target has. An import is therefore additive-by-omission, which is what makes a document from a differently-deployed host safe to apply.

## Conformance

Producer: send the document exactly as `vtc/config/export` returned it. Preview first — submit without `confirm`, read `rejected` and `pendingRestart`, then re-submit with `confirm: true` if the diff is what was intended.

Consumer:

1. Reject a `schemaVersion` you do not implement with `unsupportedSchemaVersion`. Do this first — a document whose shape you cannot vouch for must not be diffed, because the diff would be against members you may be misreading.
2. If a profile exists, compare `communityDid` and reject a mismatch with `communityDidMismatch`.
3. Compute the diff for both the profile and the overrides. Validate every override key against your configuration registry; a key that is unknown, mistyped, or out of range goes to `rejected` and **not** to `overrideChanges`.
4. If `confirm` is not set, return `status: "preview"` with the diff and stop. Nothing is written.
5. Otherwise apply. Write the profile before the overrides, so a failure part-way leaves the identity-bearing half consistent. Return `status: "imported"` with the changes actually written, and record an audit event naming the caller and the changed keys.

Report restart-gated applied keys under `pendingRestart` on both paths — an operator learns that the import implies downtime while previewing, not after.

## Security & Privacy

`sideEffects.level` is `destructive` because a confirmed import overwrites values this task does not retain. There is no undo: an operator who wants one takes an export first, which is cheap and is the reason these two tasks are specified together.

The response echoes `oldValue` alongside `newValue` for every change, so it discloses the configuration that was in force as well as the one being installed. That is deliberate — a diff a caller cannot see both sides of is not a diff they can approve — but it means the preview response carries the same `metadata` exposure as an export, and should be handled the same way.
