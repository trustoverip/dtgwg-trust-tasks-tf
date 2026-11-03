---
slug: vtn/memberships/repudiate
version: "0.1"
title: "VTN Memberships — Repudiate"
summary: "An administrator rejects a parent network's listing of this network: the network signs a repudiation of the listing as itself, which the parent must publish beside it."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, nested-membership, repudiation, listing, curation]
parties:
  - role: network administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The task makes the network sign a public statement to a third party — the parent — on the authority of the administrator who asked, so the asker has to be attributable on every transport and afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay would repeat a statement the network makes to its parent, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Signs a repudiation of the parent's listing as the network, sends it to the parent, which publishes it beside the listing, and records the membership in state repudiated.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the parent's DID and URL. Out: the membership record — the parent's and this network's public identifiers, the listing and the repudiation. No personal data.
retention:
  class: durable
  rationale: >-
    The record is the network's evidence that it rejected the parent's listing, and when; the change is in the audit trail.
errorCodes:
  - code: vtn/memberships/repudiate:notListed
    meaning: "The parent does not list this network — no listing signed by the parent names it with `nodeType` `VTN` under the DTG `lists/1` predicate — so there is nothing to repudiate."
    retryable: false
  - code: vtn/memberships/repudiate:networkMismatch
    meaning: "The network at `url` answers as a DID other than `network`."
    retryable: false
  - code: vtn/memberships/repudiate:networkRefused
    meaning: "The parent could not be reached or refused the repudiation; nothing is recorded. `details.code` and `details.message` carry the parent's error when it gave one."
    retryable: true
    detailsSchema:
      type: object
      properties:
        code:
          type: string
        message:
          type: string
related:
  - vtn/memberships/join
  - vtn/memberships/leave
  - vtn/memberships/list
  - vtn/membership/repudiate
  - vtc/networks/repudiate
---

## Abstract

A parent network may list this network without asking (VTI-VTN-010), and a listing establishes nothing (VTI-VTN-020). **VTN Memberships — Repudiate** is how this network says publicly that it does not accept such a listing: it signs, as itself, a repudiation naming the listing by its digest and sends it to the parent through [`vtn/membership/repudiate`](../../../membership/repudiate/0.1/spec.md), and the parent must publish it beside the entry ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-012). It is the mirror of [`vtc/networks/repudiate`](../../../../vtc/networks/repudiate/0.1/spec.md) for a network that is listed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`) **MUST** take `network` from a source other than the network at `url`, as for [`vtn/memberships/join`](../../join/0.1/spec.md).

A conforming **network** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, and a `network` that is its own DID with `malformedRequest`.
2. **MUST** check the parent's DID at `url` as [`vtn/memberships/join`](../../join/0.1/spec.md) network item 2 does, and refuse with `vtn/memberships/repudiate:networkMismatch` before signing anything.
3. **MUST** verify the listing as `vtn/memberships/join` network item 3 does — signed by `network` for `assertionMethod`, naming this network, predicate `https://registry.trustoverip.org/dtg/vsc/lists/1`, `object.value.nodeType` `VTN` — and refuse with `vtn/memberships/repudiate:notListed` when there is none.
4. **MUST** refuse with `alreadyExists` when it is an acknowledged member of `network`: a member withdraws by repudiating the grant, with [`vtn/memberships/leave`](../../leave/0.1/spec.md), not the listing.
5. **MUST** sign the repudiation as this network, for `assertionMethod`, naming the listing by its digest, send it with `vtn/membership/repudiate`, and record the membership in state `repudiated` only once the parent has accepted it; otherwise it refuses with `vtn/memberships/repudiate:networkRefused` and records nothing.
6. **MUST** record the change in its audit trail naming the administrator, and answer with the membership.

## Authorization

The authority this task presupposes is **administrator standing at this network**: the signer is one of the administrators in the network's configuration, read at execution time, or a key one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), in which case it is the standing of the administrator the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A network **MAY** require a further, operation-bound step-up before signing a public statement about a parent; that is its policy, not this specification's.

The parent decides whether to keep listing this network; the repudiation does not remove the listing, it is published beside it (VTI-VTN-012).

## Definitions

- **`url`**, **`network`** — as in [`vtn/memberships/join`](../../join/0.1/spec.md).
- **Repudiation** — a statement credential issued by this network, predicate `https://registry.trustoverip.org/dtg/vsc/repudiates/1`, whose subject is the parent and whose `object.digestMultibase` is the listing's digest.
- **`membership`** — the resulting record ([`UpstreamMembership`](../../../_shared/0.1/membership.schema.json)) in state `repudiated`, with the listing and the repudiation and no groups.

## Request

A network administrator (`issuer`) asks this network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Repudiate a parent's listing

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d31",
  "type": "https://trusttasks.org/spec/vtn/memberships/repudiate/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:regional",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "url": "https://whn.example.org",
    "network": "did:example:whn"
  }
}
```

## Response

This network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The repudiated listing

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d32",
  "type": "https://trusttasks.org/spec/vtn/memberships/repudiate/0.1#response",
  "issuer": "did:example:regional",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "membership": {
      "network": "did:example:whn",
      "url": "https://whn.example.org",
      "name": "World Health Network",
      "state": "repudiated",
      "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0011" },
      "repudiation": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0031" },
      "groups": [],
      "updatedAt": "2026-11-03T10:00:01Z"
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries the parent's URL and DID. The response carries the two networks' public identifiers, the parent's listing and this network's repudiation. No personal data is carried, and the repudiation carries no reason: it is published.

### Correlation

The repudiation is published by the parent beside its listing, by design: it is how a verifier learns the listed network does not accept it. Both networks are named by their public identifiers, which is why this network declares `identifierScope: public`; a pairwise identifier would be one the parent's listing and published index could not refer to. The administrator declares `identifierScope: any`: the parent sees only this network's signature, never which administrator asked.

### Retention

The network keeps the record, with the listing and the repudiation, as evidence that it rejected the listing and when. The change is recorded in its audit trail against the administrator who made it.

### Consent/purpose

The repudiation is this network's public refusal of the parent's listing and is used for nothing else. It is not consent to anything; a later [`vtn/memberships/join`](../../join/0.1/spec.md) against a renewed listing is a fresh decision.
