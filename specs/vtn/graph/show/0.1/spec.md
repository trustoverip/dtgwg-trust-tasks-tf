---
slug: vtn/graph/show
version: "0.1"
title: "VTN Graph — Show"
summary: "A network's trust graph: the network and the parties it publishes as nodes, every credential between them as an edge with its status, as far as the network's directory visibility admits the caller."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, dtg, trust-graph, curation]
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
    The graph shows the published set, which is public when the network's directory visibility is `public`, so a page that has not signed in can draw it. A network whose visibility is `members` or `admins`, or a request asking for exited entries, is decided by the signer, so the network refuses an unproved request with `permissionDenied` there.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects. A restricted network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads the published set and the network's status lists; changes nothing.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: organisations' public identifiers and the credentials between them, with their status. In: at most a person's membership credential (VMC), used only to decide admission.
retention:
  class: transient
  rationale: >-
    The graph is computed on request; nothing in the request is kept.
errorCodes: []
related:
  - vtn/graph/member
  - vtn/directory/search
  - vtn/entries/show
---

## Abstract

In a Verifiable Trust Network every relationship is a credential: the network's listing of a party, its membership grant, the party's acknowledgement, a repudiation. **VTN Graph — Show** returns those credentials as a graph — the parties as nodes, each credential as an edge from issuer to subject, with its status as the network determines it — so that the picture is the evidence ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-033). A member network appears as a node; [`vtn/graph/member`](../../member/0.1/spec.md) expands it. Who may read is the network's directory visibility; administrators may also ask for entries that have left the published set.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** meet reader items 1 and 2 of [`vtn/directory/search`](../../../directory/search/0.1/spec.md), and **MUST** sign the request when it sets `includeExited`.
2. **MUST NOT** read a membership from a node alone: a party is a member only where both its grant edge and its acknowledgement edge are present and the grant's status is `valid` (VTI-VTN-013). An `unchecked` edge is one whose status the reader resolves itself, against its issuer's status lists, if it relies on it.

A conforming **network** (`recipient`):

1. **MUST** admit the request exactly as [`vtn/directory/search`](../../../directory/search/0.1/spec.md) network items 1 and 2 do — by its configured directory visibility, read at execution time, and for a person a verified `membership` — and **MUST** refuse any other caller with `permissionDenied`.
2. **MUST** refuse with `permissionDenied` a request whose `includeExited` is `true` unless the signer is an administrator (in the network's administrator list, or a key one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md)). It **MUST NOT** answer such a request with the published graph instead.
3. **MUST** return one root node for itself (`root: true`, no `state`), and one node per published entry — every state but `removed` and `departed`, and those too when `includeExited` is `true` — carrying the entry's kind, label, state and, for a party that names one, its URL.
4. **MUST** return one edge per credential the entry holds — the listing, the grant, the party's acknowledgement, the party's repudiation — from the credential's issuer to its subject, with the credential in full.
5. **MUST** give an edge for a credential it issued (listing, grant) the status its own status lists show — `revoked`, else `suspended`, else `valid` — and an edge for a credential the party issued (acknowledgement, repudiation) the status `unchecked`, because it does not hold that party's status lists.
6. **MUST NOT** add an edge for a credential it does not hold, nor any node or edge from a member network's own set: a member network's members are not this network's (VTI-VTN-033, VTI-VTN-060).

## Definitions

- **`includeExited`** — when `true`, the graph also shows entries that have left the published set (`removed` and `departed`), with their credentials. For administrators only. Absent means `false`.
- **`membership`** — as in [`vtn/directory/search`](../../../directory/search/0.1/spec.md): a person's membership credential from a member community, holder-bound to the signer.
- **`graph`** — the nodes and edges ([`Graph`](../../../_shared/0.1/graph.schema.json)). A membership is two edges, the grant and the acknowledgement; a listing alone is one.

## Request

A reader (`issuer`) asks the network (`recipient`) for its graph. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator asks for the full graph

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000531",
  "type": "https://trusttasks.org/spec/vtn/graph/show/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "includeExited": true
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### A network with one member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000532",
  "type": "https://trusttasks.org/spec/vtn/graph/show/0.1#response",
  "issuer": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000531",
  "payload": {
    "graph": {
      "nodes": [
        { "id": "did:example:whn", "kind": "network", "label": "Western Health Network", "root": true },
        { "id": "did:example:community", "kind": "community", "label": "St Vincent's Hospital", "state": "acknowledged", "root": false, "url": "https://stvincents.example" }
      ],
      "edges": [
        {
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011",
          "source": "did:example:whn",
          "target": "did:example:community",
          "role": "listing",
          "credType": "StatementCredential",
          "status": "valid",
          "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" }
        },
        {
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012",
          "source": "did:example:whn",
          "target": "did:example:community",
          "role": "grant",
          "credType": "MembershipCredential",
          "status": "valid",
          "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" }
        },
        {
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021",
          "source": "did:example:community",
          "target": "did:example:whn",
          "role": "acknowledgement",
          "credType": "MembershipCredential",
          "status": "unchecked",
          "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021" }
        }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The response carries organisations' public identifiers, names and URLs, and the credentials between them. With `includeExited` it also carries parties that have left or been removed, which is why only administrators may ask for it. The request carries at most a flag and a person's membership credential.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it, and every edge it returns names it. The reader declares `identifierScope: any`: an anonymous reader of a public directory sends no identifier, a signed-in person may use whatever identifier their community's credential is bound to, and an administrator is recognised by the network's own configuration, not by any other party. The graph is no more correlatable than the published set and credentials it is drawn from.

### Retention

Nothing in the request is retained, and `membership` is never logged.

### Consent/purpose

Each credential in the published graph is published with the party's knowledge; each node states the party's own state, and an acknowledgement edge exists only because the party issued it. A departed or removed party is shown only to the network's administrators, who keep its record (network item 2). The person's membership credential is used solely to decide whether they may read.
