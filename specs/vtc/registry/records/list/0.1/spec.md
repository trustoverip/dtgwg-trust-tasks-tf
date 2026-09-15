---
slug: vtc/registry/records/list
version: "0.1"
title: "VTC Registry Records — List"
summary: Enumerate the trust records backing a community's recognition graph — either as the trust registry holds them, or as the community believes it published them — so the two can be compared.
status: draft
targetFrameworkVersion: "0.5.0"
category: reputation
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Read-only enumeration. RECOMMENDED because the transport already
    authenticates the caller and guarantees integrity; what governs the
    disclosure is the authorization rule, not proof strength.
sideEffects:
  level: none
  rationale: >-
    Reads. Enumerating the registry view causes an onward query to the trust
    registry, but that query is itself a read and persists nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related: []
---

## Abstract

Recognition is decided per-DID: a community asks its trust registry whether it recognises an identity, and gets back a yes or no. That is the right shape for a decision and the wrong shape for an operator, who needs to see the whole graph — and, more often, needs to see where the community's own view of it has diverged from the registry's.

This task enumerates those records from either side of that divergence. `source: "registry"` asks the trust registry what it actually holds; `source: "local"` asks the community what it believes it published. A consumer that returns them from one store would defeat the point: the two are different questions, and the answer worth having is the difference.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A consumer **MUST** echo the `source` it actually served in the response. A producer **MUST** read it, and **MUST NOT** infer the source from the request it believes it sent.

A consumer **MUST NOT** serve `source: "registry"` from a local cache. If it cannot reach the registry it **MUST** fail with `trust-task-error` rather than substituting its own view — a stale local answer presented as the registry's is precisely the fault this task exists to detect.

A consumer **MUST** omit `recognized` and `authorized` on record types that do not carry them, and a producer **MUST NOT** read an absent member as `false`. Absent means the record makes no such assertion.

A consumer **MUST** order results stably across pages so that `cursor` enumerates without repeating or skipping. Where the underlying registry supplies its own continuation token, a consumer **MAY** pass it through opaquely.

A nullable member of this response may be sent either as `null` or omitted, and a producer **MUST** treat the two identically. In particular, an absent `nextCursor` means the last page exactly as `null` does, and an absent `recognized` means the record carries no such assertion exactly as `null` does — neither absence means `false`.

## Definitions

- **`source`** — which view to enumerate. `registry` is the authoritative one; `local` is the community's record of what it published. Chosen by the producer, defaulting to `registry`, because the authoritative answer is the one a caller who did not think about it should get.
- **`entityId`, `authorityId`, `action`, `resource`** — the four-part key of a trust record, used as optional filters on the request and always present on a returned record. Supplying all four narrows to at most one record; supplying none enumerates.
- **`recordType`** — the registry's own discriminator for what kind of assertion the record makes. Passed through as the registry states it rather than mapped to a local vocabulary, so a consumer of this task sees what the registry sees.
- **`recognized` / `authorized`** — the assertion itself, present only on the record type that carries it.

## Request

The community administrator (`issuer`) sends the request to the community maintainer (`recipient`); the top-level schema is in [`payload.schema.json`](payload.schema.json). Every member is optional: an empty payload enumerates the registry's records.

### Enumerating what the registry holds

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/registry/records/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "source": "registry",
    "limit": 50
  }
}
```

## Response

The community maintainer — the `recipient` of the request, now responding — returns the matching records, the source they came from, and a continuation token, per the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document; an empty graph is not a failure and is answered with an empty `items` array.

### One recognition record

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/registry/records/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "source": "registry",
    "items": [
      {
        "entityId": "did:example:member",
        "authorityId": "did:example:community",
        "action": "recognise",
        "resource": "trust-graph",
        "recordType": "recognition",
        "recognized": true
      }
    ]
  }
}
```

`nextCursor` is absent here rather than explicitly `null`: this is the last page, and both spellings mean that.

## Security & Privacy

### Data carried

The request carries filters and a page size. The response carries the community's trust graph: who it recognises, under whose authority, for what. `entityId` and `authorityId` are identifiers of real parties and are the sensitive members — together they are a map of the community's inter-community trust relationships, which is not otherwise public. A producer **MUST NOT** place identifiers or assertions in `ext`; every member this task needs is named.

The smallest payload that answers the task is the four-part key plus the assertion, which is what a record *is*. `recordType` is included rather than inferred because a consumer that guessed it would eventually guess wrong about a record type it had not seen.

### Correlation

Enumeration is the disclosure. A reader of the answer learns the complete set of entities the community recognises and can join it against any other list of DIDs it holds — that is inherent to the task and cannot be varied by the producer. The `local` source additionally reveals what the community *believes*, which, compared against `registry`, reveals where its publication has failed; an observer who can read both learns which members are currently invisible to the wider trust fabric.

`threadId` correlates request and response, and the four-part key is a stable handle across calls.

### Retention

The recipient retains nothing on account of this task. A producer that stores the answer has taken a copy of the community's trust graph and **SHOULD** retain it under the policy it applies to that graph, not to operational logs. An answer from `source: "registry"` is a point-in-time observation of a third party's state and goes stale silently; retaining it as though it were current is the failure mode to avoid.

### Consent/purpose

The purpose is operational visibility: letting an administrator inspect the recognition graph and detect divergence between what the community published and what the registry holds. The graph **MUST NOT** be reused to profile the entities in it, and a record's absence from the registry **MUST NOT** be read as a decision about that entity — as the divergence case makes plain, absence is as likely to mean a write that failed.
