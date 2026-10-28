---
slug: understanding/offer
version: "0.1"
title: "Understanding — Offer"
summary: "A joining party asks to begin an engagement; the host answers with a resolvable, versioned reference to the terms that would govern it."
status: draft
targetFrameworkVersion: "0.6.0"
category: consent
parties:
  - role: joining-party
    requirement: REQUIRED
    member: issuer
  - role: host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: RECOMMENDED
  response: REQUIRED
  rationale: >-
    The request is an inquiry under transport authentication; a proofless request costs
    nothing to ignore. The response is different: its `termsRef` is what `understanding/exchange`
    and `understanding/seal` are later pinned to, and a dispute over which terms were on offer
    at a given moment has to be settled from a document the host cannot repudiate.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    This document opens the thread `understanding/exchange` and `understanding/seal` nest
    under, and the response names the terms version current at one moment. A replayed
    request would reopen that thread; a replayed response would restate a terms version
    that may since have changed, with nothing to say it had.
sideEffects:
  level: none
  rationale: >-
    Answering an offer does not add, remove, or alter anything durable — no account, no
    membership, no ACL entry, no grant. It resolves a reference. A host MAY log the inquiry
    for its own purposes, but nothing in this task requires it to.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries the joining party's stated purpose and, optionally, an identifier it
    proposes to use for the relationship if one forms — neither is secret, and a purpose
    string is not personal data on its own. The response discloses where the host's terms
    live and which version is current, not the terms' content.
errorCodes:
  - code: understanding/offer:notAccepting
    meaning: The host is not currently accepting engagements of any kind.
    retryable: true
  - code: understanding/offer:purposeNotSupported
    meaning: The host does not offer engagement for the declared purpose.
    retryable: false
related:
  - understanding/exchange
  - understanding/seal
---

## Abstract

The **Understanding — Offer** Trust Task opens an engagement between a *joining party* and a
*host* by asking the host to name the terms that would govern it. The host's response is a
resolvable, version-identified, hash-pinned reference — not the terms themselves — so the
joining party can fetch, cache, and later cite the exact document version it was shown. This
has to be a Trust Task rather than a plain fetch of a well-known URL because the reference
needs to be attributable to the host at a specific moment: `understanding/exchange` and
`understanding/seal` both depend on being able to point back at precisely which terms version
was on offer when the engagement began.

Discovering that a host exists, and where to reach it, is out of scope for this task — that is
the job of whatever identifier resolution or registry mechanism located the host in the first
place. This task begins only once the joining party already has an address for the host.

*Joining party* and *host* are deliberately generic: either may be an individual, an
organization, or an agent acting for one — see
[`understanding/seal`](../../seal/0.1/spec.md#relationship-to-ieee-7012-2025) for how this
family's later handshake relates to, and extends, IEEE 7012-2025's own individual-and-entity
framing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets
framework version 0.6.0 and may change without a version bump while it remains a draft
([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described
in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all
capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements)
in addition to the requirements stated here.

## Definitions

* **Joining party** (`joining-party`). The person, organization, agent, or community seeking
  to begin an engagement; identified by `issuer`.
* **Host** (`host`). The party whose relationship requirements would govern the engagement;
  identified by `recipient`.
* **Purpose.** A short, host-defined string naming the kind of engagement sought (for example
  `membership`, `conversation`, `transaction`). Opaque to this specification — each host
  defines its own vocabulary and states it out of band.
* **Terms reference.** A resolvable pointer to the document stating the host's relationship
  requirements — the terms a MyTerms-style (IEEE 7012-2025) negotiation would proceed from, a
  governance framework, or any other host-defined policy document. Its `{ uri, version,
  digestMultibase }` shape is the governance-framework reference the companion Verifiable Trust
  Infrastructure specification places in a DID document (VTI-KEY-052), and `digestMultibase` is
  the multibase-encoded multihash of the document's bytes. This task carries the pointer, never
  the document's content.

## Request

A *request* document carries `type: https://trusttasks.org/spec/understanding/offer/0.1` with
a payload that validates against the top-level schema in
[`payload.schema.json`](payload.schema.json). Issued by the joining party, addressed to the
host.

### A joining party asks to begin an engagement

```json
{
  "id": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/understanding/offer/0.1",
  "issuer": "did:example:alice",
  "recipient": "did:example:host",
  "issuedAt": "2026-09-24T10:00:00Z",
  "threadId": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000001",
  "payload": {
    "purpose": "membership"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:alice#key-1",
    "created": "2026-09-24T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

### With a proposed relationship identifier

```json
{
  "id": "urn:uuid:1a2b3c4d-0002-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/understanding/offer/0.1",
  "issuer": "did:example:alice",
  "recipient": "did:example:host",
  "issuedAt": "2026-09-24T10:00:00Z",
  "threadId": "urn:uuid:1a2b3c4d-0002-4000-8000-000000000001",
  "payload": {
    "purpose": "conversation",
    "proposedIdentifier": "did:key:z6Mkabc123..."
  }
}
```

## Response

A success *response* document carries `type:
https://trusttasks.org/spec/understanding/offer/0.1#response`, with a payload that validates
against the `$anchor: "response"` sub-schema in `payload.schema.json`. Issued by the host.

The response payload is `{ termsRef }`, a `RegisterRef` (shared with `understanding/seal`, in
[`_shared/0.1/agreement.schema.json`](../../_shared/0.1/agreement.schema.json)). A joining party proceeding to
`understanding/exchange` **MUST** carry this document's `id` forward as `offerId`, so the
exchange is bound to the exact terms version this response named.

### Where the host's DID document names a governance framework

Where the host's DID document carries a governance-framework reference — a service of type
`GovernanceFramework` whose `serviceEndpoint` is the framework's primary document URI, with
`version` and `digestMultibase` members, as the companion Verifiable Trust Infrastructure
specification defines it (VTI-KEY-052) — the response's `termsRef` **MUST** name the same
document: its `uri` equals that `serviceEndpoint`, and its `version` and `digestMultibase` equal
the service's members of the same names. A host does not name one set of terms in its DID
document and another in its offer. A joining party that resolved the host's DID document before
opening the offer **SHOULD** compare the two, and treat a difference as a reason not to proceed
rather than as a newer version of the terms — the DID document is the host's public statement,
and an offer that contradicts it is either stale or not the host's.

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response`
variant.

### The host names its current terms

```json
{
  "id": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/understanding/offer/0.1#response",
  "threadId": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000001",
  "issuer": "did:example:host",
  "recipient": "did:example:alice",
  "issuedAt": "2026-09-24T10:00:01Z",
  "payload": {
    "termsRef": {
      "uri": "https://host.example/governance/terms",
      "version": "3.2",
      "digestMultibase": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbd",
      "mediaType": "application/ld+json"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:host#key-1",
    "created": "2026-09-24T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6ab..."
  }
}
```

## Security & Privacy

### Data carried

The request carries `purpose` — a short, non-secret category string — and, optionally, a
`proposedIdentifier` the joining party is willing to be reached at if the engagement proceeds.
Neither is sensitive on its own. A producer **MUST NOT** use the free-form nature of `purpose`
to smuggle personal data; the field names a category, not a request.

The response carries only `termsRef` — a URI, a version label, a digest, and an optional media
type. It never carries the terms document's content.

### Correlation

`issuer` on the request is whatever identifier the joining party chooses to use for this
inquiry; nothing in this task requires it to be durable or public, and a joining party that
prefers not to be linkable across separate offers **SHOULD** mint a fresh identifier per
engagement. `threadId` correlates the request with its response and with the
`understanding/exchange` and `understanding/seal` documents that may follow it, by design —
that correlation is the point of the family, not an incidental leak.

### Retention

A host has no obligation under this task to retain a request or response beyond answering it.
Where a host does retain a record — for its own audit or rate-limiting purposes — it retains
only what it needs to answer `understanding/offer:notAccepting` or
`understanding/offer:purposeNotSupported` consistently.

### Consent/purpose

The joining party's `purpose` states what the engagement is for, so the host can decide
whether to name terms for it at all. This specification does not require the host to obtain
consent, approval, or a step-up before responding — it only requires the response to say what
it says.
