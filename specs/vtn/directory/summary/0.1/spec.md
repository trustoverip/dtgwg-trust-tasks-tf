---
slug: vtn/directory/summary
version: "0.1"
title: "VTN Directory — Summary"
summary: "Counts of the communities and networks a network publishes, and how many are members, shown to anyone when the network shows its counts publicly."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, directory, curation, counts]
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
    The counts are public when the network shows them publicly or its directory visibility is `public`, so a page that has not signed in can show them. Otherwise the network decides who may read from the signer, and refuses an unproved request with `permissionDenied`.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects. A restricted network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Counts the published set; changes nothing.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: four counts, naming nobody. In: at most a person's membership credential (VMC), for a network that gates its counts, used only to decide admission.
retention:
  class: transient
  rationale: >-
    The counts are computed on request; nothing in the request is kept.
errorCodes: []
related:
  - vtn/directory/search
  - vtn/directory/federated
---

## Abstract

A network may restrict who reads its directory and still want its size known — a landing page that says "42 members" names none of them. **VTN Directory — Summary** returns counts over the network's published set: how many communities and networks it publishes, and of those how many are members ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-013) and how many are listed but not members. When the network's configuration shows counts publicly the counts are answered to anyone; otherwise they are read under the same directory visibility as [`vtn/directory/search`](../../search/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** sign the request when it carries `membership`, and **SHOULD** send `membership` only when the network's profile shows that it neither shows counts publicly nor has `public` visibility, but has `members` visibility.
2. **MUST NOT** read `acknowledged` as anything but members of this network: a member network's own members are not counted (VTI-VTN-033).

A conforming **network** (`recipient`):

1. **MUST**, when its configuration shows counts publicly, answer any caller, with or without a proof, and ignore `membership`.
2. **MUST** otherwise admit the request exactly as [`vtn/directory/search`](../../search/0.1/spec.md) network items 1 and 2 do — by its configured directory visibility, read at execution time, and for a person a verified `membership` — and **MUST** refuse any other caller with `permissionDenied`.
3. **MUST** count only published entries (every state but `removed` and `departed`), each once by kind (`communities` or `networks`) and once by membership: `acknowledged` for an entry whose state is `acknowledged`, `unacknowledged` for every other published entry. So `communities + networks` equals `acknowledged + unacknowledged`.
4. **MUST NOT** keep `membership` past admission, nor use it for anything but admission.

## Definitions

- **Shows counts publicly** — the network's configuration setting that lets anyone read these counts, whatever its directory visibility. Its profile states it.
- **`membership`** — as in [`vtn/directory/search`](../../search/0.1/spec.md): a person's membership credential from a member community, holder-bound to the signer.
- **`summary`** — the counts ([`DirectorySummary`](../../../_shared/0.1/directory.schema.json)). `unacknowledged` counts `listed`, `repudiated` and `suspended` entries alike: none is a member.

## Request

A reader (`issuer`) asks the network (`recipient`) for its counts. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A landing page asks for the counts

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000521",
  "type": "https://trusttasks.org/spec/vtn/directory/summary/0.1#request",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {}
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The network's counts

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000522",
  "type": "https://trusttasks.org/spec/vtn/directory/summary/0.1#response",
  "issuer": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000521",
  "payload": {
    "summary": {
      "communities": 40,
      "networks": 3,
      "acknowledged": 38,
      "unacknowledged": 5
    }
  }
}
```

## Security & Privacy

### Data carried

The response carries four integers. The request carries nothing, or a person's membership credential for a network that gates its counts.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find it. The reader declares `identifierScope: any`: an anonymous reader sends no identifier, and a signed-in person may use whatever identifier their community's credential is bound to. Counts name no party, so nothing here is correlatable beyond the fact of asking. A network that is small, or whose counts change one at a time, does reveal joins and departures to a reader who polls; a network that considers that sensitive does not show its counts publicly.

### Retention

Nothing in the request is retained, and `membership` is never logged.

### Consent/purpose

The counts disclose only what the network's administrators chose to make public by showing counts publicly. The person's membership credential is used solely to decide whether they may read.
