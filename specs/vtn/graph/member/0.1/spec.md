---
slug: vtn/graph/member
version: "0.1"
title: "VTN Graph — Member Network"
summary: "Expand one acknowledged member network into its own trust graph, read from that network's verified signed index."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, dtg, trust-graph, nested-membership, federation]
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
    Expanding a member network is reading this network's published set one level further, which is public when the directory visibility is `public`, so a page that has not signed in can do it. A network whose visibility is `members` or `admins` decides who may read from the signer, so it refuses an unproved request with `permissionDenied`.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects on the network's state. A restricted network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Fetches and verifies a member network's published index; changes nothing. A network may cache the index, which is not a change to any party's state.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: the member network's published entries and their credentials — organisations' public identifiers. In: a member network's DID and, in a `members` network, a person's membership credential (VMC), used only to decide admission.
retention:
  class: transient
  rationale: >-
    Nothing in the request is kept. A member index the network caches is a published document, not the reader's data.
errorCodes:
  - code: vtn/graph/member:notMember
    meaning: "`did` is not a network whose entry here is `acknowledged` and names the URL it is reached at — it is absent, a community, not (or no longer) a member, or names no URL."
    retryable: false
  - code: vtn/graph/member:memberUnreachable
    meaning: "The member network's signed index could not be fetched, answered for another network, did not verify, or could not be read. Its graph is indeterminate, not empty (VTI-REG-012). `details.reason` says which."
    retryable: true
    detailsSchema:
      type: object
      required: [reason]
      properties:
        reason:
          type: string
related:
  - vtn/graph/show
  - vtn/directory/federated
---

## Abstract

[`vtn/graph/show`](../../show/0.1/spec.md) draws a network's own credentials; a member network appears in it as a single node. **VTN Graph — Member Network** expands that node: the network fetches the member network's signed index, verifies it as that network's own, and returns the graph it publishes. The expansion is the member network's own evidence, relayed — it is not this network's, and a member network's members are not members here ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-033).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** meet reader items 1 and 2 of [`vtn/directory/search`](../../../directory/search/0.1/spec.md): sign when sending `membership` or when visibility is not `public`, and send `membership` only to a `members` network.
2. **MUST** read the returned graph as the member network's: its root is that network, and its nodes' states are their entries there, not here (VTI-VTN-033).
3. **MUST** treat `memberUnreachable` as indeterminate, never as a member network with no entries (VTI-REG-012).

A conforming **network** (`recipient`):

1. **MUST** admit the request exactly as [`vtn/directory/search`](../../../directory/search/0.1/spec.md) network items 1 and 2 do — by its configured directory visibility, read at execution time, and for a person a verified `membership` — and **MUST** refuse any other caller with `permissionDenied`.
2. **MUST** refuse with `vtn/graph/member:notMember` unless `did` names an entry of kind `network`, in state `acknowledged`, whose profile names the URL it is reached at.
3. **MUST** fetch that network's signed network index from that URL, and **MUST** accept it only if it names `did` as its network and its proof verifies for `assertionMethod` against `did`. On any failure it **MUST** refuse with `vtn/graph/member:memberUnreachable`, giving the reason, and **MUST NOT** return a partial or empty graph instead.
4. **MUST** return the index as a graph: the member network as the root node, a node for each entry the index publishes with that entry's own state, and an edge for each credential the index carries — listing, grant, acknowledgement, repudiation — from issuer to subject, every one with status `unchecked`, because this network does not hold the member network's status lists.
5. **MAY** reuse an index it fetched and verified recently instead of fetching it again; the reused index **SHOULD** be no older than a few minutes.
6. **MUST NOT** forward the reader's identity or `membership` to the member network, and **MUST NOT** keep `membership` past admission.

## Definitions

- **`did`** — the member network to expand: a node of kind `network` in this network's graph.
- **`membership`** — as in [`vtn/directory/search`](../../../directory/search/0.1/spec.md): a person's membership credential from a member community, holder-bound to the signer.
- **Signed network index** — the document in which a network publishes its set, signed by its own DID (VTI-VTN-024).
- **`graph`** — the member network's graph ([`Graph`](../../../_shared/0.1/graph.schema.json)). Its edges are `unchecked`: a reader that relies on one resolves its status against its issuer's status lists.

## Request

A reader (`issuer`) asks the network (`recipient`) to expand a member network. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Expand a member network

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000541",
  "type": "https://trusttasks.org/spec/vtn/graph/member/0.1#request",
  "recipient": "did:example:state",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "did": "did:example:whn"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The member network's graph

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000542",
  "type": "https://trusttasks.org/spec/vtn/graph/member/0.1#response",
  "issuer": "did:example:state",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000541",
  "payload": {
    "graph": {
      "nodes": [
        { "id": "did:example:whn", "kind": "network", "label": "Western Health Network", "root": true },
        { "id": "did:example:community", "kind": "community", "label": "St Vincent's Hospital", "state": "listed", "root": false }
      ],
      "edges": [
        {
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011",
          "source": "did:example:whn",
          "target": "did:example:community",
          "role": "listing",
          "credType": "StatementCredential",
          "status": "unchecked",
          "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" }
        }
      ]
    }
  }
}
```

The credential is abbreviated to its `id`; the network returns it in full. The community is only listed by the member network, so there is one edge and no membership.

## Security & Privacy

### Data carried

The response carries what the member network itself publishes in its signed index: organisations' public identifiers, names and the credentials between them. The request carries a DID and, for a `members` directory, a person's membership credential.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it, and it reaches the member network the same way. The reader declares `identifierScope: any`: an anonymous reader of a public directory sends no identifier, and a signed-in person may use whatever identifier their community's credential is bound to. The member network learns only that this network fetched its index; it learns nothing about the reader (network item 6). A member network whose own index is restricted will not answer that fetch, and the reader receives `memberUnreachable`.

### Retention

Nothing in the request is retained. A network **MAY** cache the member index briefly (network item 5); it is a published document.

### Consent/purpose

The graph is what the member network chose to publish, relayed with its own signature verified; this network adds nothing to it and grants nothing over the member network's members (VTI-VTN-060 – VTI-VTN-063). The person's membership credential is used solely to decide whether they may read.
