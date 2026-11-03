---
slug: vtn/entries/show
version: "0.1"
title: "VTN Entries — Show"
summary: "Read one listed party's published entry in a network, with the credentials a verifier needs and whether the party has acknowledged."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, directory, curation, nested-membership]
parties:
  - role: reader
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
    A public network answers anyone, so a page that has not signed in can still read an entry; a network whose directory is restricted refuses an unproved request with `permissionDenied`, which makes the proof the reader's authentication exactly where it matters. The answer is published material whose credentials carry their own proofs, so a signed response is RECOMMENDED rather than required.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read changes nothing, so a replay discloses no more than the original did; a restricted network still wants to place a proved request in its acceptance window.
sideEffects:
  level: none
  rationale: >-
    Reads the published set; changes nothing.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: one organisation's published entry and the credentials it carries. In: the DID asked about and, from a person reading a restricted network, that person's membership credential (VMC), which is personal data used only to decide whether the directory admits them.
retention:
  class: transient
  rationale: >-
    The network verifies a carried membership credential and discards it; it keeps nothing from a read beyond what its operational logs and duplicate-detection window need.
errorCodes: []
related:
  - vtn/directory/search
  - vtn/listings/show
  - vtn/membership/acknowledge
  - vtn/membership/repudiate
---

## Abstract

A Verifiable Trust Network publishes the communities and networks it lists, each with the credentials that make the listing — and, where the party agreed, the membership — verifiable ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-011, VTI-VTN-024). **VTN Entries — Show** reads one of those entries by the party's DID. The answer states whether the party has acknowledged, because a listing on its own is never membership (VTI-VTN-013). The network's `[directory] visibility` decides who may read; the party an entry names may always read its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** sign the request when it reads a network whose directory is not public, and **MUST** then carry `membership` if it is a person admitted only by a membership credential.
2. **MUST NOT** treat an entry whose `acknowledged` is `false` as a member of the network (VTI-VTN-013), and **MUST NOT** treat a listing as establishing anything about the party beyond the network's own claim (VTI-VTN-020).
3. **SHOULD** verify the credentials in the entry — their proofs for `assertionMethod` and their status — before relying on them, since the entry's shape asserts nothing about them.

A conforming **network** (`recipient`):

1. **MUST** answer a request whose proven signer is the DID named by `did` — directly, or through a key delegated by that DID ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md)) — without applying its directory visibility: a party may always read its own entry.
2. **MUST** otherwise apply its directory visibility before looking the entry up:
   - `public`: any requester, signed or not;
   - `members`: an administrator; a listed party whose entry is `acknowledged`, signing as itself; or a person whose `membership` verifies — a membership credential issued by a community whose entry here is `acknowledged`, naming the requester's identity as its subject, signed for `assertionMethod`, within its validity period and with a status that resolves as valid (an indeterminate status is refused);
   - `admins`: an administrator only.

   A requester the visibility does not admit, including any unproved request to a network that is not public, **MUST** be refused with `permissionDenied`.
3. **MUST** answer `notFound` for a DID it has no entry for, and **MUST** answer an entry that is not published (`removed` or `departed`) with exactly the same `notFound` — same code, no distinguishing detail — so the task is no oracle for who was once listed.
4. **MUST** return the entry in its published form ([`IndexEntry`](../../../_shared/0.1/entry.schema.json)), with `acknowledged` `true` only for state `acknowledged`, and with the listing credential and, where they exist, the grant, the acknowledgement and the repudiation.
5. **MUST NOT** use a carried membership credential for anything but this admission decision, and **MUST NOT** retain it.

## Definitions

- **`did`** — the DID of the listed party (a community or a network) whose entry is asked for. The task's subject.
- **`membership`** — a person's membership credential (VMC) from a community that is a member of this network, holder-bound to the requester's identity ([`Membership`](../../../_shared/0.1/entry.schema.json)). Carried only where `visibility = "members"` and the requester is admitted by nothing else; ignored otherwise.
- **Requester's identity** — the request's `issuer`, or, where the issuer is a key delegated through `auth/signing-key/enroll` or approved through `auth/oob/*`, the identity it was delegated by.
- **Published** — an entry in state `listed`, `acknowledged`, `repudiated` or `suspended`. `removed` and `departed` entries are no longer published.
- **`entry`** — the party's entry as the network publishes it ([`IndexEntry`](../../../_shared/0.1/entry.schema.json)).

## Request

A reader (`issuer`) asks the network (`recipient`) for one entry. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member community reads another entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000501",
  "type": "https://trusttasks.org/spec/vtn/entries/show/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "did": "did:example:health-au"
  }
}
```

### A person reads a members-only network

The membership credential is abbreviated to its `id`; the requester sends it in full.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000503",
  "type": "https://trusttasks.org/spec/vtn/entries/show/0.1#request",
  "issuer": "did:example:alice",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:05:00Z",
  "payload": {
    "did": "did:example:health-au",
    "membership": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0031" }
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the entry as published. A refusal — including `notFound` for an absent or unpublished entry — is a `trust-task-error`, not a `#response` document.

### A listed network that has acknowledged

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000502",
  "type": "https://trusttasks.org/spec/vtn/entries/show/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000501",
  "payload": {
    "entry": {
      "did": "did:example:health-au",
      "kind": "network",
      "state": "acknowledged",
      "acknowledged": true,
      "label": "Health Australia",
      "profile": { "tags": ["health", "au"] },
      "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0041" },
      "grant": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0042" },
      "acknowledgement": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0043" }
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries a DID and, from a person reading a restricted network, that person's membership credential. The response carries one organisation's published entry: its directory profile and the credentials of the listing and, where they exist, of the membership. The membership credential is the only personal data, and a requester **MUST NOT** carry it where the network's visibility does not call for it.

### Correlation

The network learns which entry a reader looked at, and — from a person admitted by credential — the person's identity and their community. The network declares `identifierScope: public` because it is an organisation found by its public DID: a reader can only check an entry against a network it can name the same way every time, and the entry's credentials are signed by that DID. The reader's scope is `any`: a public network serves anonymous readers, and a person admitted by credential is identified by whatever identity their community's credential names, which this task neither requires nor forbids to be pairwise.

### Retention

The network keeps nothing from a read beyond its operational logs and the identifiers its duplicate-detection window holds; a carried membership credential is verified and discarded (network item 5).

### Consent/purpose

The entry is published by the network on its own authority and, for a member, with the party's acknowledgement as its consent to being published as one. A carried membership credential is presented for one purpose — admission to read — and is used for nothing else.
