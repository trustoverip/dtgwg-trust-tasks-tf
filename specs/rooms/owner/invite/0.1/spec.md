---
slug: rooms/owner/invite
version: "0.1"
title: "Rooms Owner — Invite"
summary: "A room's owner mints an invitation credential in the room's own name, so joining is a two-party act rather than something done to a member."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, owner, invite, a-term-a-searcher-would-use]
#   authors:
#     - Your Name (https://github.com/your-handle)
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The recipient signs as the ROOM, using key material its principal controls. A request whose integrity depended on the transport would let a compromised channel invite whoever it liked into a room the host cannot see the membership of."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An invitation is time-bounded and single-use; an undated request for one cannot be placed in the window its principal authorized."
sideEffects:
  level: mutating
  rationale: >-
    Signs with the room's key and returns the credential. It stores no invitation record — on a `private` room a host-side invitation store would hand the host the membership at invite time, so the owner tracks consumption.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Discloses the invitee to the recipient, which is the owner's own key holder. The credential names its subject, which is why a VIC is consumed on entry rather than presented on every access.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: transient
  rationale: >-
    The signed credential is returned and not kept. What must be remembered is that it was issued, and that is the owner's record to keep — the recipient is a signer here, not a registry.
errorCodes:
  - code: rooms/owner/invite:noSigningKey
    meaning: "The recipient holds no key by that identifier, or the key is not one its caller may name."
    retryable: false
related:
  - rooms/create
  - rooms/keys/welcome
  - rooms/records/put
---

## Abstract

A room's owner mints a **Verifiable Invitation Credential** in the room's own name, for delivery to someone they want to admit.

**Joining is consent, and this is the artefact that makes it so.** Without an invitation step the owner seals a room key to a party's key holder and they are simply *in* — holding keys to material they may not want, having agreed to nothing, and on a `private` room with nobody outside the room able to tell them they are there.

The VIC and the membership credential do not collapse into one. A VIC is single-use and consumed on entry, and it names its subject — so presenting one per access would both contradict its semantics and disclose the member on a tier that exists to prevent exactly that.

It is a Trust Task rather than a library call because the signing key is held by the owner's key holder and never leaves it. The alternative is an owner who exports a room's signing key to whatever is building the invitation, which is the arrangement this whole family exists to avoid.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is control of the room's signing key**, and nothing else. This recipient signs as the room; a caller who may name the room's key can issue in the room's name, and a caller who may not, cannot. That is the whole story, and it is deliberately the same story the VTA already tells about every key it holds: `require_context` on the key's context, then the context policy's own limit on what may sign, which binds every actor including a super-admin.

Note what this does **not** rest on. It does not check that the caller is the room's owner, because "owner" is a fact about the room's DID controller and this recipient is not a DID resolver. Controlling the signing key and controlling the DID are the same thing when the key is the one the DID document names — and when they have come apart, the credential this mints simply fails to verify. The failure is real, immediate, and at the point of use.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. The key gate is the entitlement; the proof is not.

## Definitions

**`roomId`** — the room whose identity issues this. The recipient signs **as** the room, not as itself.

**`signingKeyId`** — which held key signs. See below on why it is named rather than derived.

**`subject`** — the party being invited.

**`validUntil`** — when the invitation expires. An invitation that never expires is a standing right to enter, held by whoever ends up with the bytes.

**`credential`**, **`credentialId`** (response) — the signed VIC and its `id`.

### Why the key is named and not looked up

Nothing maps a DID to the key it was minted with, and inventing that mapping would add a lifecycle to get wrong — a binding that can go stale, be rebuilt incorrectly, or disagree with the DID document after a rotation.

Naming the key is also the honest shape: the surface that minted the room *has* the key identifier, and one that did not can enumerate keys. A wrong key produces a credential that fails to verify against the room's DID document, which is loud and immediate rather than silently wrong.

### Consumption is the owner's record, not the host's

On `open` and `attributed` rooms an invitation may ride a host's invitation machinery, which already does replay protection. On `private` it **MUST NOT**: a server-side invitation store would hand the host the room's membership at invite time, which is precisely what the tier withholds. There the owner tracks consumption — they issued it, and they know who they invited.

This recipient stores nothing either way. It is a signer, not a registry.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An invitation with a month to live, for one named party.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/invite/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "signingKeyId": "room-northwind-signing",
    "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "validUntil": "2026-02-01T00:00:00Z"
  }
}
```

## Response

The **KeyHolder** returns the signed credential, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The signed VIC, for delivery to the invitee — over DIDComm on a `private` room, where a host must not learn who was asked.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/invite/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "credential": "eyJhbGciOiJFZERTQSJ9.invitation-credential",
    "credentialId": "urn:uuid:11111111-1111-4111-8111-111111111111"
  }
}
```

## Security & Privacy

### Data carried

The request names a room, a key and an invitee. The response carries a signed credential that **names the invitee**.

That naming is the reason a VIC is consumed rather than presented: a credential that identifies its subject, presented on every access, would disclose the member to the host on every read. The design's answer is that this one is spent on entry and a membership credential takes over.

A producer **MUST NOT** put a reason, a note, or anything about *why* someone is being invited in `ext`. The invitee receives this credential; anything placed beside it travels with it.

### Correlation

The recipient learns who its principal is inviting. It is the owner's own key holder, and it already mints every credential the room issues.

The invitation itself is the correlation risk, and it travels: whoever routes it learns that this room admitted someone, and on a `private` room that is the one fact the tier is built to withhold. This is why the design routes a `private` room's VIC over DIDComm rather than a host's invitation store — the choice is made at delivery, not here.

### Retention

The recipient signs and returns. It keeps no copy, and keeps no record of having issued — a room's membership and authority live in the credentials themselves, and a list here would be the roster the whole design keeps away from any single party.

The consequence is that **the owner is the only party who knows what they have issued**, which is invariant I1 of the design working as intended rather than a gap: a room has an accountable party, and this is one of the things they are accountable for.

### Consent/purpose

The credential is minted so that the named subject can enter, remain in, or act in the room. It confers nothing on the caller — it confers on the *subject*, and a caller who could already name the room's key could already have signed it.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
