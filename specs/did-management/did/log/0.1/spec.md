---
slug: did-management/did/log
version: "0.1"
title: DID Management — DID Log
summary: A DID owner or an administrator reads the full history a hosting service holds for a slot — every entry, parsed — and, on request, the stored log and witness artifacts verbatim.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, did-log, webvh, history, read]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner or administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The resolvable document is public, but the hosting service's record of a slot is not: a disabled or reserved slot's log is withheld from resolution, and the service is the only party that can say which slot a record belongs to. The read is owner-or-administrator, and the service authorises on the document's proven issuer on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits reads."
sideEffects:
  level: none
  rationale: "Reads the stored log; it is not counted as a resolve."
subjectPath: /mnemonic
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns the slot's DID documents and method parameters as recorded, and optionally the stored artifacts. For a slot that resolves publicly this is public data; for a disabled one it is data the service otherwise withholds."
retention:
  class: transient
  rationale: "The request names one slot and is needed only to answer it."
errorCodes:
  - code: did-management/did/log:notFound
    meaning: "No slot the caller may read has this `mnemonic`, or the slot holds no published content yet. Returned alike for a slot that does not exist and one the caller does not own."
    retryable: false
  - code: did-management:unknownDomain
    meaning: "The submitted `domain` is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error)."
    retryable: false
related:
  - did-management/did/info
  - did-management/did/rollback
  - did-management/did/register
---

## Abstract

[`did-management/did/info`](../../info/0.1/spec.md) returns a slot's record and a one-line summary of its log. **DID Log** returns the history itself: every entry the service holds, parsed into the DID document and the method parameters as of that entry — what an owner's console shows as a version history, and what an owner reads before choosing a version to [roll back](../../rollback/0.1/spec.md) to. With `raw: true` it also returns the stored artifacts byte for byte, for an owner who wants to verify or archive exactly what the service serves.

The task works across the hosted methods: for did:webvh an entry is a log line; for did:web, which keeps no history, there is one entry, the current document; for did:webs an entry is a document the key event log established.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the [category conventions](../../../_shared/0.1/CONVENTIONS.md). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Resolves the slot per the conventions (`domain` disambiguates). Answers `did-management/did/log:notFound` when it does not exist, when the caller is neither its owner nor an administrator, or when it holds no published content — the three alike.
3. Returns the entries oldest first, parsed from what it stores and serves — not re-derived from any other source — with the slot's `method`.
4. With `raw: true`, also returns `logContent` exactly as stored, and `witnessContent` when the slot holds witness proofs. The parsed `entries` **MUST** be the parse of that same `logContent`.
5. Changes nothing, and does not count the read as a resolve.

A consumer that holds a did:webs slot returns, as `logContent`, the key event stream it stores; it never stores a derived did.json, so it has none to return.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **ownership of the slot** (the slot's recorded owner is the proven issuer) or **administrator standing on the hosting service**, both read from the service's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Public resolvability of the DID confers no standing here: anyone may resolve it through the method, but this task reads the service's record, which includes what the method does not serve.

## Definitions

**Entry** — one version of the slot's DID document, together with the method parameters in force at it.

## Request

```json
{
  "id": "urn:uuid:f4a0cd21-b1df-4023-9c2d-0f1a2b3c4d01",
  "type": "https://trusttasks.org/spec/did-management/did/log/0.1",
  "issuer": "did:webvh:QmVtaScid5:vta.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "mnemonic": "alice" }
}
```

## Response

One entry shown, abbreviated.

```json
{
  "id": "urn:uuid:f4a0cd21-b1df-4023-9c2d-0f1a2b3c4d02",
  "type": "https://trusttasks.org/spec/did-management/did/log/0.1#response",
  "threadId": "urn:uuid:f4a0cd21-b1df-4023-9c2d-0f1a2b3c4d01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "mnemonic": "alice",
    "method": "webvh",
    "entries": [
      {
        "versionId": "1-QmHashOfEntryOne",
        "versionTime": "2026-09-01T10:00:00Z",
        "state": {
          "@context": ["https://www.w3.org/ns/did/v1"],
          "id": "did:webvh:QmAliceScid7:did.example.com:alice"
        },
        "parameters": { "method": "did:webvh:1.0", "scid": "QmAliceScid7", "updateKeys": ["z6MkUpdateKey1"] }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request names one slot. The response carries the slot's DID documents and method parameters — keys, services, `alsoKnownAs`, pre-rotation commitments — as the controller published them. For a publicly resolvable slot every byte is already public; for a disabled slot it is content the service withholds from resolution, which is the reason the task is owner-or-administrator.

### Correlation

The response is about one DID and discloses nothing about the slot's owner beyond what the DID's own documents say. The request itself links the caller to the slot, to a service that already holds that link. Both parties declare `identifierScope: public`, and must: the owner's DID is the one the slot record names, and the service's DID is the recipient the owner registered with.

### Retention

Nothing of the request needs keeping beyond the exchange.

### Consent/purpose

The history is disclosed so an owner can inspect, verify and archive what the service holds for them, and an administrator can diagnose it. It **MUST NOT** be disclosed to other owners.
