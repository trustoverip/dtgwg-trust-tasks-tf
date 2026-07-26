---
slug: vtc/backup/import
version: "0.1"
title: VTC Backup — Import
summary: Restore a Verifiable Trust Community from an encrypted backup envelope. Previews by default; `confirm` clears the backed-up keyspaces and applies it.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - backup
  - import
  - restore
  - disaster-recovery
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: super administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A confirmed import clears and replaces every backed-up keyspace. The caller must be attributable.
sideEffects:
  level: destructive
  rationale: "With `confirm: true` the backed-up keyspaces are cleared and replaced wholesale. With `confirm: false` nothing is written."
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: Accepts an envelope containing the community signing key bundle, so an import installs key material and not merely rows.
errorCodes:
  - code: vtc/backup/import:decryptionFailed
    meaning: The password does not decrypt the envelope, or the ciphertext is corrupt.
    retryable: false
  - code: vtc/backup/import:identityMismatch
    meaning: The envelope's `sourceDid` names a different community than the running one.
    retryable: false
  - code: vtc/backup/import:permissionDenied
    meaning: The consumer lacks the community super-admin capability.
    retryable: false
---

## Abstract

The **VTC Backup — Import** Trust Task restores a community from an envelope produced by [`vtc/backup/export`](../../export/0.1/).

It is **two-phase by default**. `confirm: false` — the default — decrypts, checks identity, and returns the per-keyspace row counts the restore *would* write, without mutating anything. `confirm: true` performs the restore. An operator sees the blast radius before authorising it.

## Conformance

Producer: send the `backup` envelope and its `password`. Call once with `confirm` absent or `false` to preview, then again with `confirm: true` to apply.

Consumer: verify the super-admin capability. Decrypt; on failure return `decryptionFailed` without partial writes. Cross-check the envelope's `sourceDid` against the running community DID — a **fresh install with no DID accepts any envelope**, but a configured community MUST reject a mismatch with `identityMismatch` rather than adopt another community's state. Return `status: "preview"` and the counts when `confirm` is not set; otherwise clear the backed-up keyspaces, replay the envelope, return `status: "imported"`, and audit the restore. A preview MUST NOT be audited as a restore.

Where the envelope carries the audit log, its signed checkpoints MUST be restored with it. Restoring one without the other leaves the log contradicting its own attestations and reads as truncation.

## Security & Privacy

**This is the most destructive operation the community exposes.** A confirmed import discards live state wholesale, which is why `confirm` is opt-in rather than a flag that defaults on, and why the preview returns real row counts rather than a bare acknowledgement.

The identity cross-check is what stops a community being silently overwritten with another's members, ACL, and credentials. It is deliberately asymmetric: a fresh install has nothing to protect, a configured one has everything to lose.

`exposure.discloses` is `secret` because the envelope carries the signing key bundle — an import installs key material, not just rows.
