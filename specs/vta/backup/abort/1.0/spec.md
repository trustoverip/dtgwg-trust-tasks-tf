---
slug: vta/backup/abort
version: "1.0"
title: "VTA Backup — Abort"
summary: "Cancel an in-flight export or import bundle and discard its bytes."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - cancellation
parties:
  - role: backup operator
    requirement: REQUIRED
    member: issuer
  - role: verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Abort destroys staged bytes and, on the export side, closes the only window in which a copy of the agent could have been retrieved. A recipient that cannot attribute the request cannot answer the question an operator asks when a bundle they were about to download has vanished: who cancelled it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Bundle identifiers are reused by nothing, so a replayed abort finds a terminal bundle and is answered idempotently. It still needs placing in time, because the abort is often the last event in a bundle's life and is therefore the document a reconstruction of that life is anchored to.
sideEffects:
  level: destructive
  rationale: >-
    Discards the staged bytes and moves the bundle to a terminal state. On the export side the encrypted copy is deleted and cannot be re-minted — a new export must be initiated, which re-serializes the agent and produces different bytes. On the import side an uploaded bundle is discarded unapplied, and the operator must upload it again.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
retention:
  class: durable
  rationale: >-
    Nothing of the request survives beyond the transition, but the recipient keeps the fact of it. A cancelled export is the case where no copy was made, and being able to say so later — rather than seeing only a bundle that stopped existing — is what makes the trail readable.
errorCodes:
  - code: vta/backup/abort:notFound
    meaning: >-
      The recipient holds no bundle under this identifier that this producer may act on. Deliberately conflates "no such bundle" with "not yours" — see Correlation.
    retryable: false
related:
  - vta/backup/initiate-export
  - vta/backup/initiate-import
---

## Abstract

The **VTA Backup — Abort** Trust Task cancels a bundle that is still in flight — an export whose bytes were staged but never fetched, or an import that was uploaded but never applied — and discards the bytes behind it.

It is the safety valve for a family whose failure mode is a copy of an agent sitting at a fetchable address longer than anyone intended. Bundles expire on their own, but expiry is a deadline, not a decision: an operator who realises mid-export that they used the wrong password, or that the download went somewhere it should not have, wants the window shut now rather than in the remainder of its slot.

Abort is also the only way out of a stuck bundle. A recipient caps how many bundles one operator may hold open at once, so a bundle abandoned in a non-terminal state consumes a slot until it expires. Aborting returns it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **having created the bundle**. A recipient **MUST** refuse an abort from any party other than the one that initiated the bundle, and **MUST** answer such a request as `notFound` rather than as a refusal — see [Correlation](#correlation).

The entitlement is deliberately no wider than that. Abort is destructive, but it destroys only a thing the producer itself brought into existence and holds nothing of the agent's own state, so requiring more than "you started this" would leave operators unable to clean up after themselves. The narrowness is also a containment property: a party that has somehow obtained a bundle identifier still cannot use this task to interfere with another operator's export.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who sent this document and never that they own the bundle it names. That comparison is separate, and a recipient that omits it lets any authenticated operator destroy any bundle whose identifier they can quote.

## Definitions

**`bundleId`** — the handle returned in the descriptor from [`initiate-export`](../../initiate-export/1.0/spec.md) or [`initiate-import`](../../initiate-import/1.0/spec.md). Opaque: a producer quotes what it was given and **MUST NOT** derive, guess or enumerate one. Abort takes bundles of either kind, which is why this task has one specification rather than two — the operator's intent ("stop this, discard the bytes") is identical, and the recipient already knows which kind it holds.

**`aborted`** — whether this request is what moved the bundle to a terminal state. `false` means the bundle was already terminal: completed, expired, or aborted by an earlier identical request. That is a success, not a failure — see [Idempotence](#idempotence) — and the member exists so a producer can tell "I cancelled it" from "it was already over" without either being an error.

## Idempotence

Abort is idempotent. A request naming a bundle that is already terminal succeeds and returns `aborted: false`; it **MUST NOT** be refused.

This is not merely a convenience. The situation abort exists for — a network that dropped, an operator unsure whether their cancellation landed — is exactly the one that produces duplicate requests, and a specification that made the second an error would push producers toward reading the bundle's state first and racing on the answer. Making the repeat harmless removes the race instead of documenting it.

The consequence is that `aborted: false` carries no information about *why* the bundle was terminal. A completed export and a bundle someone else's request expired look the same from here. A producer that needs the distinction has it in the response to whichever task produced the terminal state, not in this one.

## Request

The producer is the operator that initiated the bundle; the recipient is the agent holding it. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Cancelling a staged export

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/vta/backup/abort/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:03:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "bundleId": "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with the code declared in the front matter, not a `#response` document.

### The bundle was live and is now cancelled

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/vta/backup/abort/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T00:03:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "bundleId": "3f2504e0-4f89-41d3-9a0c-0305e82c3301",
    "aborted": true
  }
}
```

## Security & Privacy

### Data carried

One opaque handle in, the same handle and a boolean out. No password, no token, no URL, and no description of what the bundle held.

A recipient **MUST NOT** report in this response what the aborted bundle contained, how large it was, or where it could have been fetched from. The request is a cancellation, and answering it with a description of the thing being cancelled would make abort a read of exactly the material it exists to destroy.

A recipient **SHOULD** delete the staged bytes rather than only marking the bundle terminal. Where deletion cannot be completed synchronously — bytes on storage that is briefly unavailable — the recipient **MUST** still refuse the transport token immediately, so that the bundle is unfetchable from the moment it is aborted rather than from the moment the deletion succeeds.

### Correlation

The recipient learns that this operator cancelled this bundle and when. Combined with the initiating document that gives the elapsed time before cancellation, which is a weak signal about operator behaviour and is intrinsic: a bundle cannot be aborted without the abort being recorded.

`bundleId` joins this document to the `initiate-export` or `initiate-import` that produced the bundle; `threadId` joins request to response. Both are intrinsic to the exchange.

The producer's identifier **MUST** be the same one that initiated the bundle, because the ownership check compares against it.

Both "no such bundle" and "a bundle owned by another operator" are answered as `notFound`. Conflating them is deliberate: distinguishing them turns an opaque handle into an oracle telling a stranger whether a given bundle exists and belongs to someone else, which is a map of the agent's backup activity built one guess at a time. The abort case makes this sharper than elsewhere in the family, because a refusal that said "not yours" would confirm the existence of a live bundle to precisely the party who should not know about it.

### Retention

Nothing of the request is retained. The bytes are destroyed, which is the point.

The recipient **SHOULD** keep a `durable` record that this bundle was aborted, by whom and when. A bundle that simply stops appearing is indistinguishable from one that was quietly fetched, and the difference between "no copy was ever made" and "a copy left and nobody logged it" is the entire question a later investigation asks.

### Consent/purpose

The purpose is cancelling an operation the same producer started. Nothing about this document licenses anything else: a recipient **MUST NOT** treat an abort as authorization to mint a replacement bundle, and **MUST NOT** treat it as a request to delete anything beyond the named bundle's staged bytes. In particular, aborting an import discards the uploaded bundle and **MUST NOT** touch state the agent has already applied from an earlier one.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement.
