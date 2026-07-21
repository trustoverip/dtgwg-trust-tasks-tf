---
slug: audit/verify
version: "0.1"
title: Audit — Verify
summary: Walk a maintainer's append-only audit hash chain and report whether it is internally consistent, locating the first break if it is not.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - audit
  - integrity
  - hash-chain
  - tamper-evidence
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: auditor
    requirement: REQUIRED
    member: issuer
  - role: audit maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only integrity check. Recommended so the verification itself is attributable in the maintainer's own audit trail.
sideEffects:
  level: none
  rationale: "Reads the audit log and recomputes hashes; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: audit/verify:permissionDenied
    meaning: The consumer lacks the audit-read capability (typically a super-admin or auditor role).
    retryable: false
---

## Abstract

The **Audit — Verify** Trust Task checks the integrity of an append-only audit log whose envelopes form a hash chain: each carries `prevHash` (its predecessor's `entryHash`) and `entryHash` (a commitment over its own immutable content). The maintainer walks the log in chronological order and re-derives both for every envelope, returning a [`VerifyResponse`](payload.schema.json) that says whether the chain held and — if not — where it first broke.

It detects:

- **Tampering** — an envelope edited after writing, whose `entryHash` no longer re-derives (`kind: tamperedEntry`).
- **Reorder, drop, insertion, or duplication** — an envelope whose `prevHash` does not match its predecessor's `entryHash` (`kind: brokenLink`).

`entriesExamined` / `entriesVerified` / `legacySkipped` / `unparseableSkipped` account for every envelope walked, so a `verified: true` over a log full of skips cannot be mistaken for a clean bill of health — the skip counts are reported at the same prominence as the result.

## Conformance

Producer: send with no parameters. Carry a proof if the maintainer's policy expects verification to be attributable.

Consumer: verify the audit-read capability. Walk the audit store in ascending (chronological) key order. For each envelope from the chain-carrying schema version onward, recompute `entryHash` and confirm `prevHash` equals the previous envelope's `entryHash`; on the first failure, stop and return `verified: false` with a `chainBreak` locating it. Step over envelopes that predate the chain format (counting them in `legacySkipped`) and envelopes that fail to deserialize (`unparseableSkipped`) rather than failing outright, but surface both counts. Return the `entryHash` of the newest envelope reached as `head`. When `verified` is true, `chainBreak` MUST be absent; when false, it MUST be present.

## Security & Privacy

**Consistency, not authenticity.** A `verified: true` proves the chain is internally consistent — it does **not** prove authenticity. If `entryHash` is an unkeyed digest (e.g. a bare SHA-256), an adversary with write access to the audit store holds everything needed to forge a suffix and restamp every subsequent envelope, and a truncation to a valid prefix is indistinguishable from a quiet period. Treat this task as detecting accident and careless tampering, not a determined adversary with store access. A maintainer that needs to resist that adversary keys the digest or anchors the head under a key the adversary does not hold, and publishes checkpoints out of band; this task reports on the chain it can see, and an independent copy verified out-of-band is the stronger check.

**Skips are findings, not passes.** `legacySkipped > 0` on a store that should hold no pre-chain rows is itself a finding: those envelopes are stepped over, not verified, so they are an insertion point that a naive reading of `verified: true` would miss. Consumers surface the skip counts; callers treat a non-zero count on a current store as a break-equivalent signal.

**Same-store caveat.** The result reflects the store the maintainer is reading. An adversary who can rewrite that store can also rewrite what this task reports. This is a self-check, not an external attestation.

**Exposure.** The response discloses metadata about the log's integrity and size, not its contents, so `discloses: metadata`. The audit-read gate still applies — the mere shape and health of the audit trail is admin-class information.
