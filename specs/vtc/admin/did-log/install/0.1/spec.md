---
slug: vtc/admin/did-log/install
version: "0.1"
title: "VTC Admin DID Log Install"
summary: "Hand a community that self-hosts its did:webvh log a copy extended by entries its key holder signed, so the community verifies it and serves it in place of the one it serves now."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
  - role: community
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Replaces the identity document a community publishes to everyone who resolves it. The log
    authenticates its own entries, but not who chose to deliver it, and the audit record of that
    choice must be attributable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A mutating task. A replay of an earlier install cannot move the served log backwards — the
    extension rule refuses it — but placing each install in time is what lets an operator
    reconstruct when the community's document changed.
sideEffects:
  level: mutating
  rationale: >-
    Changes the did:webvh log the community serves, and so the DID document every resolver of
    the community's DID receives. Idempotent: installing the log already served changes nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
retention:
  class: durable
  rationale: >-
    The installed log is served publicly for the life of the DID; a did:webvh log is its own
    permanent record, and its earlier entries are what later ones are verified against.
errorCodes:
  - code: vtc/admin/did-log/install:invalidLog
    meaning: The supplied log does not verify as a did:webvh log — an entry fails to parse, a proof does not verify under the update keys in force, or the SCID or entry-hash chain is broken.
    retryable: false
  - code: vtc/admin/did-log/install:wrongDid
    meaning: The supplied log verifies, but for a DID other than the community's own.
    retryable: false
  - code: vtc/admin/did-log/install:notAnExtension
    meaning: The supplied log does not keep every entry the community serves now, unchanged and in order — it is shorter, rewritten or forked.
    retryable: false
  - code: vtc/admin/did-log/install:notServedHere
    meaning: The community does not self-host its did:webvh log — it is published by a DID host, which receives new entries from the key holder directly — so there is nothing here to replace.
    retryable: false
related: []
---

## Abstract

The **VTC Admin DID Log Install** Trust Task delivers a community's own `did:webvh` log to the community, extended by entries its key holder has signed, so the community verifies it and serves it in place of the log it serves now.

A community's `did:webvh` log is published one of two ways. Where a DID host publishes it, the agent holding the DID's keys sends each new entry to that host, and this task is not needed. A community can instead **self-host** its log, serving `did.jsonl` itself — and then it serves a log it cannot extend, because the keys that sign it are held by the agent that provisioned it, which has no way to reach the community's copy. So any change to a self-hosted community's DID document after it was minted — a transport added to its services, a key rotated — produces an entry the community has no way to receive. Before this task an operator copied the file onto the community's host by hand. As a Trust Task the delivery is authorised, audited, and checked by the party that will serve the result.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

**Producer.** Supply the community DID's **complete** log, every entry from the first. A delta is not a conforming `log`: the consumer verifies the chain from its start.

**Consumer.** Before serving anything, the consumer MUST:

1. verify `log` as a `did:webvh` log in full — every entry's proof under the update keys in force at that entry, the SCID, and the entry-hash chain — and refuse with `invalidLog` if any check fails;
2. refuse with `wrongDid` if the log's DID is not the community's own;
3. refuse with `notAnExtension` unless every entry it serves now appears in `log`, byte for byte and in the same order, as its first entries — so a shorter log, a rewritten entry, or a fork from any earlier entry is refused;
4. refuse with `notServedHere` if it does not self-host its `did:webvh` log — a log a DID host publishes is extended at that host, and a copy installed here would never be the one resolvers read.

It MUST then replace the served log atomically — a resolver reads either the whole previous log or the whole new one, never a mixture — and answer with the resulting `versionId`, the previous one, and the number of entries added. Installing the log already served is not an error: it changes nothing and answers `entriesAdded: 0`.

A consumer MUST NOT modify `log`, and MUST NOT serve a log it has not verified under rule 1, whatever the authority of the producer.

## Authorization

The entitlement is **administration of the community**: the producer holds the community's super-administrator capability, which the consumer verifies against its own access-control list — never against the log. Verifying the `proof` establishes who asked; per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements) it does not establish that they may.

That authority covers **delivery only**. What the log *says* is authorised by the log itself: every entry carries a proof under the update keys its predecessor committed to, so the conformance rules above admit only entries the DID's key holder signed. A producer with full administrative authority cannot use this task to publish a document the key holder did not sign, and cannot move the served log backwards.

## Definitions

- **`log`** (request) — the community DID's complete `did:webvh` log, in JSON Lines, exactly as it is to be served. Chosen by the producer; typically fetched from the agent that holds the DID's keys, after that agent appended the new entries.
- **`did`** (response) — the community's DID, whose log is now served.
- **`versionId`** (response) — the `versionId` of the last entry of the log now served.
- **`previousVersionId`** (response) — the `versionId` of the last entry served before the request; equal to `versionId` when nothing was added.
- **`entriesAdded`** (response) — the number of entries the installed log added. `0` when `log` was the log already served.

## Request

Sent by a community administrator to the community, carrying the payload defined by the top-level schema in [`payload.schema.json`](payload.schema.json).

### Installing a log extended by one entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/admin/did-log/install/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:webvh:QmCommunityScid:community.example",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "log": "{\"versionId\":\"1-QmFirst\",\"versionTime\":\"2025-12-01T00:00:00Z\",\"parameters\":{},\"state\":{},\"proof\":[]}\n{\"versionId\":\"2-QmSecond\",\"versionTime\":\"2026-01-01T00:00:00Z\",\"parameters\":{},\"state\":{},\"proof\":[]}"
  }
}
```

The entries are abbreviated for the example; a real `log` carries each entry's full parameters, state and proof, and would be refused with `invalidLog` as written here.

## Response

Sent by the community, the recipient of the request, carrying the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error`, not a `#response` document.

### The log now served ends at the new entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/admin/did-log/install/0.1#response",
  "issuer": "did:webvh:QmCommunityScid:community.example",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "did": "did:webvh:QmCommunityScid:community.example",
    "versionId": "2-QmSecond",
    "previousVersionId": "1-QmFirst",
    "entriesAdded": 1
  }
}
```

## Security & Privacy

### Data carried

A `did:webvh` log: public by design, since every resolver of the DID fetches it. It carries verification methods, services and log parameters, and no personal data about members. The response carries identifiers and a count. Nothing secret moves in either direction — no private key is needed to deliver a log, and a producer **MUST NOT** place one in `ext`.

### Correlation

The community's DID is public and stable, and the log it serves is world-readable, so nothing here is newly joinable except the timing of an install and which administrator made it — both of which belong in the community's own audit record.

### Retention

The installed log is served for the life of the DID: a `did:webvh` log is append-only, and its earlier entries are what later ones are verified against, so none of it can be discarded. The request document is audit evidence of when and by whom the served document changed.

### Consent/purpose

The log is received for one purpose — to be served as the community's DID log — and the consumer uses it for nothing else. Whether an install needs a further approval is the consumer's policy, not this specification's.
