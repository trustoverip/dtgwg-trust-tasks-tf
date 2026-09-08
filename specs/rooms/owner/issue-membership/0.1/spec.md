---
slug: rooms/owner/issue-membership
version: "0.1"
title: "Rooms Owner — Issue Membership"
summary: "A room's owner mints the membership credential that lets a member re-enter, issued in the room's own name."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, owner, issue, membership, a-term-a-searcher-would-use]
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
  rationale: "The recipient signs as the ROOM. A membership credential is what a host verifies to admit an operation, so one whose origin depended on the transport would let a compromised channel add members to a room whose roster no host can check."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Membership credentials are renewed rather than reissued blindly, and telling a renewal from a re-send needs both an `id` and a time."
sideEffects:
  level: mutating
  rationale: >-
    Signs with the room's key and returns the credential. It keeps no roster: a room's membership lives in the credentials it issued, and a list here would be the roster the design exists to avoid.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Discloses the member to the recipient, which is the owner's own key holder and already the party that mints every credential in the room.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: transient
  rationale: >-
    Returned, not kept. The member holds it; the room's owner knows who they issued to; no third copy is needed and a third copy is a roster.
errorCodes:
  - code: rooms/owner/issue-membership:noSigningKey
    meaning: "The recipient holds no key by that identifier, or the key is not one its caller may name."
    retryable: false
related:
  - rooms/create
  - rooms/keys/welcome
  - rooms/records/put
---

## Abstract

A room's owner mints the **Verifiable Membership Credential** that lets a member re-enter, issued in the room's own name.

This is the credential a member presents on every subsequent operation, and it is what a host verifies to admit one. The host holds no roster — by design, because a roster would make the room unmovable and make the host part of its membership definition — so this credential *is* the membership.

Membership and authority stay in different credentials. This says *you are in this room*; a [Verifiable Authority Credential](../../issue-authority/0.1/spec.md) says *what you may do*. Changing someone's access reissues one small credential and leaves the membership edge alone.

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

**`roomId`** — the room whose identity issues this.

**`signingKeyId`** — which held key signs.

**`subject`** — the member being admitted. Membership is an explicit set of parties, **never a role**: a room that admitted a role would be a room whose membership its host could compute, and the host is the party the design keeps membership away from.

**`validUntil`** — when membership lapses. Absent means it does not, and removal is then an epoch advance rather than an expiry.

**`credential`**, **`credentialId`** (response) — the signed VMC and its `id`, which is how a re-send is told from a renewal.

### Expiry is not removal

Worth stating because the two look similar and behave differently. Letting a credential lapse stops a member presenting it; it does **not** stop them reading what they could already read, because they hold the epoch keys. Removal is [`rooms/epoch/mint`](../../../epoch/mint/0.1/spec.md): a new epoch, sealed to the members who remain.

A surface that offers "remove" and issues a short expiry has mis-stated the guarantee.

### One half of an edge, and the half that is checked

A membership credential in the DTG is one half of a pair — the room grants, and the member acknowledges, forming an edge. Room authorization verifies the **grant** half: the presented credential's subject must match the root of the authority chain.

The member's half is not consulted, and this task does not mint it — it could not, since that half is issued by the member. A room implementation that wants the full edge issues it from the member's side; nothing here depends on it, and this specification does not require it.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### No `validUntil`: membership that does not lapse, where removal is an epoch advance rather than an expiry.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/issue-membership/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "signingKeyId": "room-northwind-signing",
    "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
  }
}
```

## Response

The **KeyHolder** returns the signed credential, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The signed VMC, for delivery to the member.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/issue-membership/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "credential": "eyJhbGciOiJFZERTQSJ9.membership-credential",
    "credentialId": "urn:uuid:22222222-2222-4222-8222-222222222222"
  }
}
```

## Security & Privacy

### Data carried

A room identifier, a key identifier, a member identifier — and a signed credential naming that member as a member of that room.

**The credential is the membership**, so possession of it matters in a way possession of most credentials does not: there is no roster to check it against. A producer **MUST NOT** place anything in `ext` that it would not want a host to see, because the member presents this and the host reads what it carries.

### Correlation

The recipient learns who its principal admits. It already mints every other credential in the room.

Downstream, this credential is presented to a host on every operation, so on `open` and `attributed` rooms the host can join a member's reads and writes across time. That is the tier's stated bargain and is the reason the `private` tier presents membership in zero knowledge instead. Nothing about how this credential is *issued* changes that; what changes it is how it is presented.

### Retention

The recipient signs and returns. It keeps no copy, and keeps no record of having issued — a room's membership and authority live in the credentials themselves, and a list here would be the roster the whole design keeps away from any single party.

The consequence is that **the owner is the only party who knows what they have issued**, which is invariant I1 of the design working as intended rather than a gap: a room has an accountable party, and this is one of the things they are accountable for.

### Consent/purpose

The credential is minted so that the named subject can enter, remain in, or act in the room. It confers nothing on the caller — it confers on the *subject*, and a caller who could already name the room's key could already have signed it.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
