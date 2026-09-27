---
slug: did-management/agent-name/resolve
version: "0.1"
title: DID Management — Agent Name Resolve
summary: A DID owner or an administrator asks which agent names a batch of hosted DIDs currently serve — the reverse of the `/@name` redirect — seeing only DIDs they may read.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, agent-name, alsoKnownAs, reverse-lookup, read]
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
  rationale: "Asked about an arbitrary DID, the answer would say whether this service hosts it — the association between a DID and its hosting control plane that the service never publishes. The service answers only for DIDs the proven issuer may read, so the proof is what bounds the answer on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits reads."
sideEffects:
  level: none
  rationale: "Reads the agent-name registry; nothing is written."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries DIDs; the response, the names each serves. Every name returned is one the DID's own document already claims in `alsoKnownAs` and the service already serves as a public redirect."
retention:
  class: transient
  rationale: "The request is needed only to answer it."
errorCodes: []
related:
  - did-management/agent-name/list
  - did-management/agent-name/check
  - did-management/did/list
---

## Abstract

An agent name (`did.example.com/@alice`) is a redirect a hosting service serves to a DID it hosts, which that DID's document claims in `alsoKnownAs`. **Agent Name Resolve** answers the reverse question in bulk: for these DIDs, which names do they serve right now? It is what a console listing an owner's DIDs calls once, rather than once per row.

It differs from [`did-management/agent-name/list`](../../list/0.1/spec.md), which reads one slot's whole name registry — parked names included — for its owner. This task returns served names only, for many DIDs, and says nothing about a DID the caller may not read.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. For each requested DID, includes an entry only when it hosts the DID, the caller is the DID's owner or an administrator, and the DID serves at least one name. Every other DID is simply absent: a DID not hosted here, one the caller may not read, and one serving no name **MUST** be indistinguishable in the response, including by its order and size.
3. Lists only served names — enabled, and claimed by the DID's current document — never a parked one.
4. Answers with entries in request order and changes nothing. A request in which no DID qualifies is answered with an empty `entries`, not an error.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is, per DID, **ownership of its slot** or **administrator standing on the hosting service**, read from the service's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A DID the caller may not read is filtered out rather than refused, so that a batch mixing the caller's DIDs with others' cannot be used as a membership test.

## Definitions

**Served name** — an agent name the service currently redirects to the DID: registered, enabled, and claimed by the DID's current document.

## Request

```json
{
  "id": "urn:uuid:05b1de32-c2e0-4134-8d3e-1a2b3c4d5e01",
  "type": "https://trusttasks.org/spec/did-management/agent-name/resolve/0.1",
  "issuer": "did:webvh:QmVtaScid5:vta.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {
    "dids": [
      "did:webvh:QmAliceScid7:did.example.com:alice",
      "did:webvh:QmBobScid8:did.example.com:bob"
    ]
  }
}
```

## Response

Bob's DID serves no name, so it is absent.

```json
{
  "id": "urn:uuid:05b1de32-c2e0-4134-8d3e-1a2b3c4d5e02",
  "type": "https://trusttasks.org/spec/did-management/agent-name/resolve/0.1#response",
  "threadId": "urn:uuid:05b1de32-c2e0-4134-8d3e-1a2b3c4d5e01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "entries": [
      { "did": "did:webvh:QmAliceScid7:did.example.com:alice", "names": ["did.example.com/@alice"] }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries up to a hundred DIDs; the response, the served names of those the caller may read. Every name returned is already public — claimed in the DID's own document and served as a redirect — so the response adds nothing to what a resolver of each DID could learn, except, without rule 2, whether this service hosts it.

### Correlation

A DID-to-control-plane association is the one fact this task must not leak, and rule 2 is how it does not: a DID outside the caller's reach looks exactly like one this service has never seen. The request does disclose to the service which DIDs the caller is interested in; a producer **SHOULD** send only DIDs it expects to own. Both parties declare `identifierScope: public`, and must: the owner's DID is the one the slot records name, and the service's DID is the recipient the owner registered with.

### Retention

Nothing of the request needs keeping beyond the exchange.

### Consent/purpose

The names are disclosed so an owner's console can show each DID's addresses. The task **MUST NOT** be used, by producer or consumer, to probe which DIDs a service hosts.
