---
slug: vtn/directory/federated
version: "0.1"
title: "VTN Directory — Federated Search"
summary: "Search a network's published set and, through the signed indexes of its acknowledged member networks, theirs in turn, with each result's path and each hop's own entry state."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, directory, federation, nested-membership, discovery]
parties:
  - role: reader
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: OPTIONAL
  rationale: >-
    The search is open to any caller when the network's directory visibility is `public`, so a page that has not signed in can still search. A network whose visibility is `members` or `admins` decides who may read from the signer, so it refuses an unproved request with `permissionDenied`.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects on the network's state. A restricted network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads this network's published set and fetches member networks' published indexes; changes nothing. A network may cache the indexes it fetched, which is not a change to any party's state.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: published entries of this network and its member networks — organisations' public identifiers, names and credential ids — with the nesting path to each. In: the query and, in a `members` network, a person's membership credential (VMC), used only to decide admission.
retention:
  class: transient
  rationale: >-
    Nothing in the request is kept: the query and any membership credential are discarded once the answer is sent. Member indexes the network caches are published documents, not the reader's data.
errorCodes: []
related:
  - vtn/directory/search
  - vtn/graph/member
  - vtn/entries/show
---

## Abstract

A network's members can themselves be networks, each publishing its own set in a signed index. **VTN Directory — Federated Search** runs the query of [`vtn/directory/search`](../../search/0.1/spec.md) over this network's set and then over the signed indexes of its acknowledged member networks, and theirs in turn, to a depth the network configures. Nesting is never flattened: membership does not chain ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-032, VTI-VTN-033), so every result carries the path of networks it was found through, each hop with its own entry state. A member network whose index cannot be read is reported, not silently dropped (VTI-REG-012).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** meet reader items 1 and 2 of [`vtn/directory/search`](../../search/0.1/spec.md): sign when sending `membership` or when visibility is not `public`, and send `membership` only to a `members` network.
2. **MUST NOT** read a result as membership of the searched network unless its `path` is empty and its `acknowledged` is `true`. A result found through a member network is a member of `listedBy` at most, and only where its own `acknowledged` says so; each hop says only whether that network is a member of the one before it (VTI-VTN-033).
3. **MUST** treat a network in `unreachable` as indeterminate: its set was not searched, and the absence of its entries from `results` says nothing about them (VTI-REG-012).

A conforming **network** (`recipient`):

1. **MUST** admit the request exactly as [`vtn/directory/search`](../../search/0.1/spec.md) network items 1 and 2 do — by its configured directory visibility and, for a person, a verified `membership` — and **MUST** refuse any other caller with `permissionDenied`.
2. **MUST** first search its own published set as `vtn/directory/search` does, returning each match with `listedBy` its own DID and an empty `path`.
3. **MUST** then follow, breadth first and to at most its configured federation depth, each entry of kind `network` whose state is `acknowledged` — in its own set at the first level, and in each fetched index after that — and that names the URL it is reached at. A depth of zero searches this network's own set only.
4. **MUST**, for each member network followed, fetch its signed network index, and **MUST** accept it only if it names that network's DID and its proof verifies for `assertionMethod` against that DID. It **MUST** apply the query to the index's published entries exactly as to its own (network items 3 and 4 of `vtn/directory/search`), returning each match with `listedBy` that network's DID.
5. **MUST** give every result found through a member network the `path` it was reached by: one hop per nesting, from this network's member to `listedBy`, each hop stating that network's own `state` and `acknowledged` in the network before it on the path. It **MUST NOT** derive one hop's state from another's or from the result's (VTI-VTN-033).
6. **MUST** report in `unreachable`, with its path and a reason, every member network it set out to search and could not — fetch failure, an index answering for another network, a proof that does not verify, an index it cannot read — and **MUST NOT** omit one.
7. **MUST** search each network at most once per request, so a cycle in the nesting ends.
8. **MAY** reuse an index it fetched and verified recently instead of fetching it again; `determinedAt` is when it assembled the answer, and a reused index **SHOULD** be no older than a few minutes.
9. **MUST NOT** keep `membership` past admission, nor use it for anything but admission; and **MUST NOT** forward it to a member network.

## Definitions

- **Query members** — `q`, `kind`, `state`, `membersOnly` and `membership`, as defined in [`vtn/directory/search`](../../search/0.1/spec.md). The filters apply to every set searched.
- **Federation depth** — how many levels of member networks the network follows. Its profile states it; it is the network's configuration, not the reader's to choose.
- **Signed network index** — the document in which a network publishes its set, signed by its own DID (VTI-VTN-024). It is what a federated search reads; a member network's own Trust Task endpoint is not consulted.
- **`determinedAt`** — when the network assembled the answer.
- **`results`** — the matches ([`FederatedEntry`](../../../_shared/0.1/directory.schema.json)), each with the network whose set it is in (`listedBy`) and the path to that network.
- **`unreachable`** — the member networks whose sets could not be searched ([`Unreachable`](../../../_shared/0.1/directory.schema.json)), with the path each was reached by and why.

## Request

A reader (`issuer`) sends its query to the network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Search the federation for members tagged cardiology

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000511",
  "type": "https://trusttasks.org/spec/vtn/directory/federated/0.1#request",
  "recipient": "did:example:national",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "q": "cardiology",
    "membersOnly": true
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### A result two levels down, and an unreachable member network

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000512",
  "type": "https://trusttasks.org/spec/vtn/directory/federated/0.1#response",
  "issuer": "did:example:national",
  "issuedAt": "2026-11-03T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000511",
  "payload": {
    "determinedAt": "2026-11-03T10:00:02Z",
    "results": [
      {
        "did": "did:example:community",
        "kind": "community",
        "state": "acknowledged",
        "acknowledged": true,
        "label": "St Vincent's Hospital",
        "tags": ["hospital", "cardiology"],
        "listingId": "urn:uuid:00000000-0000-4000-8000-0000000c0011",
        "grantId": "urn:uuid:00000000-0000-4000-8000-0000000c0012",
        "listedBy": "did:example:whn",
        "path": [
          { "network": "did:example:state", "name": "State Health Network", "state": "acknowledged", "acknowledged": true },
          { "network": "did:example:whn", "name": "Western Health Network", "state": "acknowledged", "acknowledged": true }
        ]
      }
    ],
    "unreachable": [
      {
        "network": "did:example:northern",
        "path": [
          { "network": "did:example:northern", "name": "Northern Network", "state": "acknowledged", "acknowledged": true }
        ],
        "reason": "index signature does not verify"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries organisations' public identifiers, names, tags and credential ids drawn from this network's set and its member networks' published indexes, and the nesting path to each. The request carries a query and, for a `members` directory, a person's membership credential. A reader **MUST NOT** put personal data in `q`.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it, and the same holds for every network on a path: a path can only be checked against the networks it names. The reader declares `identifierScope: any`: an anonymous reader of a public directory sends no identifier, and a signed-in person may use whatever identifier their community's credential is bound to. The member networks searched learn nothing about the reader — the network fetches their published indexes as itself, and forwards neither the query's author nor `membership` (network item 9). A member network whose own index is restricted will not answer that fetch, and appears in `unreachable`.

### Retention

Nothing in the request is retained. A network **MAY** cache member indexes briefly (network item 8); they are published documents.

### Consent/purpose

Each result was published by the network that lists it, with the listed party's knowledge and with its acknowledgement state stated (VTI-VTN-010, VTI-VTN-013). This network aggregates what its members already publish and grants nothing over their members (VTI-VTN-060 – VTI-VTN-063). The person's membership credential is used solely to decide whether they may read.
