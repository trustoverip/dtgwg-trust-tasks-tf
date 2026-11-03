---
slug: vtc/networks/notice
version: "0.1"
title: "VTC Networks — Notice"
summary: "The community passes a Verifiable Trust Network's broadcast on to a member of the group registered for its role, carrying the network's signed envelope unchanged."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, broadcast, relay, notice]
parties:
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
  - role: member of the registered group
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The member must be able to tell that their own community passed this on, as well as — from the envelope — which network originated it and through which networks it came. The community's proof is the first; the envelope's signatures are the second.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A member's agent holds notices and absorbs a repeat by the envelope's message id; placing the document in time lets it discard a late copy.
sideEffects:
  level: none
  rationale: "Delivers a broadcast the network already sent; the notice changes nothing at the recipient beyond what its agent chooses to hold."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Out: the network's DID, the role and group, and the network's message — content the network chose to broadcast to every party serving the role. No personal data of the recipient.
errorCodes: []
related:
  - vtc/networks/groups/register
  - vtc/networks/evidence
  - notifications/notify
---

## Abstract

The **VTC Networks — Notice** Trust Task is the last step of a Verifiable Trust Network's broadcast ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063)). A network broadcasts to a role; each member network relays to the groups its own members registered for the role, adding a hop it signs; and a community that registered a group receives the envelope at its relay endpoint for that group. It passes the envelope on, unchanged, to each active member of the group with this task.

The envelope is the network's: the message signed by the network that originated it and one hop signed by each network that relayed it, the last delivering to the community's group. A member verifies the community's proof on this document and, independently, the envelope — so they can tell who said it, who passed it on, and that nobody changed it.

**Not a notification.** [`notifications/notify`](../../../../notifications/notify/0.1/spec.md) tells a party that a thing exists and carries nothing of its content (VTI-NTF-003). A broadcast *is* the content: a network's message to everyone serving a role. A broadcast that announces a thing a member may take part in — a network room — is better sent as a notification.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (the producer, `issuer`):

1. **MUST** send a notice only for an envelope it verified on receipt: the originating network's signature, an unbroken chain of signed hops ending with one delivering to the group for `groupId`, by `network` — a network it is an acknowledged member of, and with which it registered that group for `role`.
2. **MUST** carry the envelope unchanged, and **MUST** send it only to active members of the group, at most once each per message.
3. **MUST NOT** add to, summarise or rephrase the message; the community's own words, if any, belong in a separate document.

A conforming **member's agent** (the consumer, `recipient`):

1. **MUST** verify the community's proof and **SHOULD** verify the envelope as the community did before presenting the message as the originating network's.
2. **SHOULD** absorb a repeat by the envelope's message id.
3. **MUST NOT** treat a notice as evidence of anything beyond what the originating network said: it grants nothing, and a room or task it mentions decides admission when opened.

## Definitions

- **`network`** — the network the community received the broadcast from (the last hop's signer).
- **`role`** — the network's role the group serves.
- **`groupId`** — the community group the recipient is in.
- **`envelope`** — the network's envelope: **`message`**, signed by the originating network (`origin`, `subject`, `body`, `class`, `sentAt`, `expiresAt`), and **`hops`**, one per relay step, each signed by the network that made it.

## Request

The community (`issuer`) sends a member of the group (`recipient`) the top-level schema in [`payload.schema.json`](payload.schema.json). The task has no response.

### A network's exercise notice, delivered to a member of the community's admins group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000361",
  "type": "https://trusttasks.org/spec/vtc/networks/notice/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:member",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "network": "did:example:whn",
    "role": "admins",
    "groupId": "admins",
    "envelope": {
      "message": {
        "type": "VtnBroadcast",
        "id": "urn:uuid:00000000-0000-4000-8000-00000000b001",
        "origin": "did:example:whn",
        "role": "admins",
        "class": "notice",
        "subject": "Regional exercise",
        "body": "A coordination exercise runs on 12 November at 09:00 UTC.",
        "sentAt": "2026-11-03T09:59:00Z",
        "expiresAt": "2026-11-06T09:59:00Z",
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:whn#key-0",
          "proofValue": "z3abc"
        }
      },
      "hops": [
        {
          "type": "VtnRelayHop",
          "message": "urn:uuid:00000000-0000-4000-8000-00000000b001",
          "index": 0,
          "from": "did:example:whn",
          "to": "did:example:community-admins-group",
          "role": "admins",
          "at": "2026-11-03T09:59:01Z",
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "proofPurpose": "assertionMethod",
            "verificationMethod": "did:example:whn#key-0",
            "proofValue": "z3def"
          }
        }
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

The network's DID, the role, the group's identifier, and the envelope — the message a network chose to send to every party serving the role, and the DIDs of the networks that relayed it. Nothing about the recipient beyond being the `recipient`.

### Correlation

Every member of a group receives the same envelope, so its message id links their copies; that is inherent to a broadcast and harmless, since it is the network's message to all of them. The community does not tell the network who received it.

### Retention

A member's agent keeps a notice for as long as it is useful to them, at most until the message's `expiresAt`. The community keeps no record of individual deliveries beyond what its delivery queue needs.

### Consent/purpose

The member's community registered their group to serve the role; a member who does not want a network's broadcasts asks their community to register another group, or leaves the group.
