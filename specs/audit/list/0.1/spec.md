---
slug: audit/list
version: "0.1"
title: Audit — List
summary: Page through a maintainer's append-only audit log, newest first, with optional filters and an opaque continuation cursor.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - audit
  - list
  - pagination
  - oversight
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
  rationale: Read-only inventory of the audit tail. Recommended so the read is itself attributable.
sideEffects:
  level: none
  rationale: "Reads the audit log; persists nothing."
subjectPath: /contextId
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: "Audit envelopes carry plaintext principal DIDs and full event payloads — the maintainer's operations record, and the tightest-gated read it offers."
errorCodes:
  - code: audit/list:permissionDenied
    meaning: The consumer lacks the audit-read capability. This is the tightest-gated read a maintainer offers; a context-scoped admin does not qualify for the whole-log tail.
    retryable: false
  - code: audit/list:invalidCursor
    meaning: The supplied cursor failed to verify — malformed, or minted before an audit-key rotation. The consumer restarts from the first page rather than retrying.
    retryable: false
---

## Abstract

The **Audit — List** Trust Task pages through a maintainer's append-only audit log, returning [`AuditEnvelope`](../_shared/0.1/audit.schema.json) entries **newest first**. It is the enumeration companion to [`audit/verify`](../verify/0.1/): verify says whether the chain is intact, list shows what is in it. Optional filters (`from`/`to`, `action`, `actor`, `outcome`, `contextId`) narrow the result; an opaque `cursor` continues a previous page.

## Paging

Paging is by **opaque continuation cursor**, not offset. The response returns `cursor` when `truncated` is true; the consumer passes it back verbatim to fetch the next page. The cursor encodes the resume position — typically `(recordedAt, eventId)` of the oldest entry on the page just returned — so paging is stable under concurrent appends: a new entry written at the head does not shift the pages a consumer is walking, the way an offset would.

The cursor is opaque and SHOULD be signed by the maintainer, so it cannot be forged to skip entries or read a slice the caller could not otherwise reach. Filters are bound into the cursor's position; a consumer continuing a page MUST NOT also change the filters (start a fresh query instead). A cursor that fails to verify — malformed, or minted under an audit key that has since rotated — yields `invalidCursor`, and the consumer restarts from the first page.

## Conformance

Producer: optional filters; `cursor` to continue, `pageSize` to bound the page.

Consumer: verify the audit-read capability. Return matching entries newest-first as `AuditEnvelope`s, populating every field the maintainer tracks (only `eventId`/`recordedAt`/`action` are universal). Set `truncated` and return a `cursor` iff more entries match. Reject an unverifiable cursor with `invalidCursor`. A maintainer whose log is hash-chained SHOULD populate `prevHash`/`entryHash` so a consumer can spot-check links against `audit/verify`.

## Security & Privacy

**Highest-sensitivity read.** Audit envelopes carry plaintext principal DIDs and full event payloads (`detail`) — the wire form of the maintainer's operations record. That is why `exposure.discloses` is `secret`, not `metadata`: this is the tightest-gated read surface a maintainer offers. A context-scoped admin can manage their own context but MUST NOT see the whole-log tail; the audit-read gate is community-wide. Erring to the stricter classification is deliberate — the exposure of an audit read is its records, not merely their shape.

**Redaction.** An `actor`/`target` filter matches the plaintext DID, so entries a right-to-be-forgotten override has redacted (plaintext nulled) do not match an actor filter and are returned with `actor: null`. A maintainer that keeps a keyed correlation hash exposes it under `ext`, never as a queryable plaintext.

**Cursor as a capability.** Because the cursor encodes a reachable position and is signed, it is a bearer handle to that slice of the log. A maintainer SHOULD bind it to the caller and key so a leaked cursor is not a cross-principal read; at minimum it MUST NOT be forgeable into a position the filters did not authorize.

**Auditing the audit read.** Reading the log is itself an auditable event; a maintainer SHOULD record `audit/list` calls (at a sampled rate) so the oversight surface is not a blind spot in its own trail.
