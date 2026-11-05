---
slug: vtc/networks/preview
version: "0.1"
title: "VTC Networks — Preview"
summary: "An administrator reads a Verifiable Trust Network before joining it: its profile, the roles it requires a group for, and whether it lists the community."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, membership, preview]
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Only administrators may ask the community to contact a network on their behalf, so the asker has to be identified by more than the transport.
sideEffects:
  level: none
  rationale: >-
    Reads the network's profile and the community's own entry there, and the community's own membership record; changes nothing at the community or at the network.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Out: the network's public profile (DID, name, owner, governance framework, roles) and where the community stands with it. No personal data.
errorCodes:
  - code: vtc/networks/preview:networkUnreachable
    meaning: "Nothing at `url` answered as a Verifiable Trust Network. `details.message` carries the reason."
    retryable: true
  - code: vtc/networks/preview:networkMismatch
    meaning: "The network at `url` answers as a DID other than `network`, or its answer is not signed by the DID it answers as."
    retryable: false
related:
  - vtc/networks/join
  - vtc/networks/repudiate
  - vtc/networks/show
  - vtn/entries/show
---

## Abstract

The **VTC Networks — Preview** Trust Task lets a community administrator see a Verifiable Trust Network before joining it ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063)). A network may list a community without asking (VTI-VTN-010), and sends its administrators an invitation: a URL, and the DID they should find there. Before [`vtc/networks/join`](../../join/0.2/spec.md) signs anything, the administrator needs to know what that network is, which of its roles they must assign a group to, and whether it really lists the community.

The community reads, at `url`:

1. the network's profile: its DID, name, description, owner (VTI-VTN-002), governance framework and roles;
2. its own entry at the network, with [`vtn/entries/show`](../../../../vtn/entries/show/0.1/spec.md) signed as the community, and verifies the listing is the network's and lists the community as a community — the same check `vtc/networks/join` makes before it acknowledges.

It answers with what it found, and with its own record of the membership if it holds one. The administrator's console reads the task's answer, so a browser never has to reach the network itself.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, before contacting the network.
2. **MUST** refuse with `vtc/networks/preview:networkMismatch` when `network` is present and the network at `url` answers as another DID, and when the network's signed answer is not signed by the DID it answers as.
3. **MUST** refuse with `vtc/networks/preview:networkUnreachable` when nothing at `url` answers as a network.
4. **MUST** answer `listed: true` only when the network's entry for the community carries a listing that verifies as the network's, names the community as its subject with the `lists/1` predicate, and lists it as a community (`nodeType` `VTC`) — the check `vtc/networks/join` makes. Any other outcome, including no entry, is `listed: false`, not a refusal.
5. **MUST NOT** sign any statement to the network, register anything with it, or change the community's records. Its only request to the network is a read.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The task needs no step-up: it reads, and the community signs nothing to the network but its own read.

## Definitions

- **`url`** — the network's API base, as the invitation gave it.
- **`network`** — the DID the administrator expects at `url`.
- **`listed`** — whether the network verifiably lists the community.
- **`entryState`** — the network's state for the community's published entry.
- **`membershipState`** — the community's own record, as [`vtc/networks/show`](../../show/0.2/spec.md) reads it.

## Request

A community administrator (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### From an invitation

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000331",
  "type": "https://trusttasks.org/spec/vtc/networks/preview/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-05T10:00:11Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "url": "https://whn.example.org",
    "network": "did:example:whn"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### A network that lists the community and requires an admins group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000332",
  "type": "https://trusttasks.org/spec/vtc/networks/preview/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-05T10:00:12Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "network": "did:example:whn",
    "url": "https://whn.example.org",
    "name": "World Health Network",
    "description": "Health communities that recognise each other's members.",
    "owner": "World Health Network Foundation",
    "framework": {
      "iri": "https://whn.example.org/governance",
      "version": "1.0"
    },
    "roles": [
      {
        "name": "admins",
        "required": true,
        "description": "Each member's administrators, for notices from the network."
      },
      {
        "name": "news",
        "required": false
      }
    ],
    "listed": true,
    "entryState": "listed"
  }
}
```

### A network that does not list the community

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000333",
  "type": "https://trusttasks.org/spec/vtc/networks/preview/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-05T10:00:12Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "network": "did:example:whn",
    "url": "https://whn.example.org",
    "name": "World Health Network",
    "roles": [],
    "listed": false,
    "notListedReason": "The network holds no entry for this community."
  }
}
```

## Security & Privacy

### Data carried

The network's public profile — its DID, URL, name, description, owner, governance framework and roles — and where the community stands with it: whether it is listed, the network's state for its entry, and the community's own membership state. These are public identifiers and the network's own statements about itself. No member's personal data is carried.

### Correlation

The community reads its entry at the network signed as itself, so the network learns that the community looked. That is the identifier the network lists it under, and the same read `vtc/networks/join` makes, so it reveals nothing the network could not already link. A community that does not want a network to learn it is interested does not preview it.

The community is addressed by its public identifier (`identifierScope: public`) because that is the identifier the network lists and its entry is keyed by: a listing is a public relationship between two public parties (VTI-VTN-011), and a pairwise identifier would name no entry the network holds.

### Retention

The community keeps nothing from a preview: the answer is read, shown and dropped. The network may log the read like any other.

### Consent/purpose

The answer exists to let an administrator decide whether, and with which groups, to join; it is not reused for any other purpose.

### Server-side request

The community fetches a URL an administrator named, so it applies its outbound policy to it as `vtc/networks/join` does: an `https` URL, no private or loopback address unless the operator allowed one, bounded time and size.
