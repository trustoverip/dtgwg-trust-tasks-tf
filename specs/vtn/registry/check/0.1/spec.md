---
slug: vtn/registry/check
version: "0.1"
title: "VTN Registry — Check"
summary: "Ask a network to ask its trust registry whether a party is published as its member, and to return the registry's own signed answer."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, trust-registry, trqp, membership, verification]
parties:
  - role: verifier
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: OPTIONAL
  response: RECOMMENDED
  rationale: >-
    Anyone may ask: the answer is what the trust registry already publishes about the party, so the request's proof authenticates nothing the network needs. The network's own answer is only a relay; what a verifier relies on is the registry's signed answer carried inside it, so a signed response is RECOMMENDED rather than required.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read changes nothing, so a replay discloses no more than the original did.
sideEffects:
  level: none
  rationale: >-
    The network reads the trust registry and changes nothing, there or here.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: whether the registry publishes a party under the network's authority, and the registry's answer, which names organisations by their public DIDs. In: the DID asked about.
retention:
  class: transient
  rationale: >-
    Nothing in the request is kept beyond operational logs and the duplicate-detection window.
errorCodes:
  - code: vtn/registry/check:noRegistry
    meaning: "The network publishes its membership to no trust registry, so there is nothing to ask."
    retryable: false
related:
  - vtn/registry/status
  - vtn/entries/show
  - registry/record/query
---

## Abstract

A Verifiable Trust Network may publish its membership to a trust registry, one TRQP record per member under the network's DID as authority. A verifier that can reach the registry asks it directly. One that cannot — a browser, or a tool with no DIDComm or TSP session where the registry serves no HTTPS — asks the network to ask for it. **VTN Registry — Check** is that relay: the network queries the registry for the records it holds about the party under this network's authority, and returns what the registry answered, **including the registry's own signed answer document**, so the verifier can check the answer came from the registry rather than take the network's word for it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** answer any requester, signed or not, whatever its directory visibility: the answer is what the registry publishes, not the network's directory.
2. **MUST** answer `noRegistry` when it publishes to no trust registry.
3. **MUST** ask the registry it names in its DID document's `TrustRegistry` referral, for the records under `authority` (its own DID) and `resource` about `did`, and **MUST** return them as the registry answered: `member` is `true` only when the registry holds an authorised record for the party with the member action, and `listed` likewise for the listed action.
4. **MUST** return the registry's answer document verbatim as `answer` when one arrived, and **MUST NOT** alter it.
5. **MUST** answer `reachable: false`, with `member` and `listed` absent and a `reason`, when the registry could not be asked or did not answer. That is indeterminate, never "not a member" (VTI-REG-012).

A conforming **verifier** (`issuer`):

1. **SHOULD** verify `answer`'s proof against the registry's DID before relying on the result, and **SHOULD** check that `answer` names the network's DID as authority. Without that check the result rests on the network's word alone.
2. **MUST** treat `reachable: false` as indeterminate.
3. **SHOULD** compare the result with the network's own answer ([`vtn/entries/show`](../../../entries/show/0.1/spec.md)): a disagreement is itself a finding.

## Definitions

- **`did`** — the party asked about.
- **`registry`** — the trust registry's DID.
- **`authority`** — the network's DID, the `authorityId` of every record the network publishes.
- **`resource`** — the TRQP resource the network's records are published under.
- **Member action** and **listed action** — the TRQP actions the network publishes acknowledged members and, where it publishes them, listed parties under (`actions`).
- **`answer`** — the registry's own answer document to the network's query, as it arrived: signed by the registry.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000a01",
  "type": "https://trusttasks.org/spec/vtn/registry/check/0.1#request",
  "issuer": "did:example:verifier",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-05T10:00:00Z",
  "payload": {
    "did": "did:example:msf"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000a02",
  "type": "https://trusttasks.org/spec/vtn/registry/check/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:verifier",
  "issuedAt": "2026-11-05T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000a01",
  "payload": {
    "registry": "did:example:registry",
    "authority": "did:example:whn",
    "resource": "vtn",
    "actions": { "member": "member", "listed": "lists" },
    "did": "did:example:msf",
    "reachable": true,
    "member": true,
    "listed": false,
    "answer": { "id": "urn:uuid:00000000-0000-4000-8000-000000000a03", "issuer": "did:example:registry" }
  }
}
```

`answer` is abbreviated; the network returns the registry's document in full.

## Security & Privacy

### Data carried

A DID in; out, what the registry publishes about it under the network's authority. Both name organisations by public identifiers.

### Correlation

The network, and the registry it asks, learn which party a verifier asked about. A verifier that wants neither to know asks the registry directly, where it can.

### Retention

Nothing is kept beyond operational logs and the duplicate-detection window.

### Consent/purpose

The records are published by the network on its own authority, for members with their acknowledgement. The check discloses only what the registry already publishes to anyone who asks it.
