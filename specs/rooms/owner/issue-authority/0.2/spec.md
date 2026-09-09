---
slug: rooms/owner/issue-authority
version: "0.2"
title: "Rooms Owner — Issue Authority"
summary: "A room's owner grants a party the authority to read, write, curate or administer the room — a chain root at the room's own scope."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, owner, issue, authority, a-term-a-searcher-would-use]
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
  rationale: "The recipient signs as the ROOM, and what it signs is a grant of authority. Nothing about this may rest on the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A grant is time-boxed, and reissuing one to change what a party may do is the ordinary operation — an undated request cannot be ordered against the grant it replaces."
sideEffects:
  level: mutating
  rationale: >-
    Signs with the room's key and returns the credential. Storing nothing is deliberate: the host verifies a chain the presenter brings, so a grant this recipient remembered would be a grant nobody consults.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Discloses the grantee and what they were granted, to the owner's own key holder.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: transient
  rationale: >-
    Returned, not kept. The holder presents it; a copy retained here would be an authority record the room's own verification never reads.
errorCodes:
  - code: rooms/owner/issue-authority:noSigningKey
    meaning: "The recipient holds no key by that identifier, or the key is not one its caller may name."
    retryable: false
  - code: rooms/owner/issue-authority:emptyActions
    meaning: "`actions` was empty. An empty grant confers nothing rather than everything, and is refused rather than interpreted."
    retryable: false
related:
  - rooms/create
  - rooms/keys/welcome
  - rooms/records/put
---

## Abstract

A room's owner grants a party the authority to act in the room — `read`, `write`, `curate` or `admin` — as a **chain root** at the room's own scope.

Authority is separate from membership on purpose. A [membership credential](../../issue-membership/0.1/spec.md) says *you are in this room*; this says *what you may do*, so changing someone's access reissues one small credential and leaves the membership edge alone.

**A chain root is the grant the room itself made**, and it is the thing everything else descends from. A holder who wants to give an agent less than they hold does not come back here: they **attenuate** their own grant — fewer verbs, shorter expiry, a different subject — which needs no issuer and is what makes a leaked agent capability not a leaked member capability.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`validUntil` is **REQUIRED**. `0.1` made it optional, which let a caller ask this task for a grant that never lapses — and DTG Core Credentials requires the property on every authority credential, for a reason stated below. So `0.1` described a request no conforming implementation could honour: an issuer refuses to build the credential, and a verifier refuses a chain link that carries no expiry, so a grant minted without one could never have been used.

`0.2` is a breaking change to a `draft` specification and is therefore a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A caller upgrading supplies a `validUntil` it was previously free to omit. There is no default to fall back on: see [Every grant expires](#every-grant-expires).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is control of the room's signing key**, and nothing else. This recipient signs as the room; a caller who may name the room's key can issue in the room's name, and a caller who may not, cannot. That is the whole story, and it is deliberately the same story the VTA already tells about every key it holds: `require_context` on the key's context, then the context policy's own limit on what may sign, which binds every actor including a super-admin.

Note what this does **not** rest on. It does not check that the caller is the room's owner, because "owner" is a fact about the room's DID controller and this recipient is not a DID resolver. Controlling the signing key and controlling the DID are the same thing when the key is the one the DID document names — and when they have come apart, the credential this mints simply fails to verify. The failure is real, immediate, and at the point of use.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. The key gate is the entitlement; the proof is not.

## Definitions

**`roomId`** — the room whose identity issues this, and whose scope it governs.

**`signingKeyId`** — which held key signs.

**`subject`** — the party granted to.

**`actions`** — what they may do. One or more of `read`, `write`, `curate`, `admin`.

**`validUntil`** — when the grant lapses. REQUIRED; see [Every grant expires](#every-grant-expires).

**`credential`**, **`credentialId`** (response) — the signed VAC and its `id`.

### `curate` is not implied by `write`

Pinning, deprecating and retracting are judgements over shared knowledge, not writes, and a room may reasonably grant one without the other. Nothing stops a room granting `curate` to an agent; the default posture should not.

### An empty grant is refused, not interpreted

`actions` **MUST** carry at least one verb. An empty list confers nothing rather than everything — but "confers nothing" and "confers everything" are exactly the two readings a careless consumer might pick between, so the schema refuses the input rather than leaving the choice open.

### Every grant expires

`validUntil` **MUST** be present, and a consumer **MUST** reject a request that omits it.

This is not this task being strict. It is [DTG Core Credentials](https://github.com/trustoverip/dtgwg-cred-spec) requiring it of the credential being minted — *"Unlike the base structure, `validUntil` is REQUIRED for a VAC … nothing about the subject's current standing is consulted when a VAC is verified, so authority that does not expire is authority nobody can withdraw by waiting."*

The reasoning is worth restating here, because it is what makes the constraint load-bearing rather than tidy. This task mints a **chain root**, and roots are the grants nothing else can withdraw. There is no status list consulted at verification and no membership check behind it: a verifier presented with a chain asks whether it reaches the room and whether every link is live, and nothing else. Expiry is therefore not one of several ways a grant ends — for a root it is the only one that works without the room reissuing or revoking, and a root minted without it is authority the room cannot take back at all.

It is also unusable in practice, which is what makes the `0.1` shape a defect rather than a laxity: a verifier refuses a chain link carrying no expiry outright. A grant minted without one would have failed at its first use, at a host, for a reason the holder could not act on.

**There is deliberately no default.** A consumer **MUST NOT** substitute one for an absent value. How long a room's authority should last is the owner's judgement about their own room, and a recipient that picked a lifetime would be making that judgement silently, in the one place the owner cannot see it. Refusing the request puts the choice back where it belongs, and a caller with no view on it should say so explicitly with a short value rather than by omission.

Note that a **shorter** value is always safe and often right: a holder who needs a longer-lived capability than the root's cannot obtain one by attenuation — attenuation may only narrow — so the root's lifetime is the ceiling on everything derived from it.

### Why this mints roots and not links

A chain root is a grant by the party governing the scope. Minting one is how a self-issued grant of arbitrary authority would get in, so it is restricted to the party that controls the room's signing key — and every *narrower* grant is derived by attenuation instead, by the holder, without an issuer.

A task that minted arbitrary links would be a task that let anyone holding it manufacture a chain that reaches the room.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### `read` and `write`, not `curate` — curation is a separate judgement and is not implied.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/issue-authority/0.2#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "signingKeyId": "room-northwind-signing",
    "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "actions": ["read", "write"],
    "validUntil": "2027-01-01T00:00:00Z"
  }
}
```

## Response

The **KeyHolder** returns the signed credential, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The signed VAC. The holder attenuates it themselves to equip an agent.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/issue-authority/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "credential": "eyJhbGciOiJFZERTQSJ9.authority-credential",
    "credentialId": "urn:uuid:33333333-3333-4333-8333-333333333333"
  }
}
```

## Security & Privacy

### Data carried

A room, a key, a grantee and a list of verbs — and a signed credential conferring them.

**What this credential confers is the sensitive part.** A VAC naming `admin` at a room's scope is the authority to mint epochs and transfer the room. A producer **MUST NOT** put anything in `ext` that a verifier might mistake for scope or action: the members that decide authority are the ones the schema names, and a consumer reads no others.

### Correlation

The recipient learns who its principal is granting what. It already mints every credential the room issues.

Downstream, an attenuated derivative names the agent as its subject, and only its subject may present it — which is what makes an agent's capability traceable to the agent rather than to its principal. That is a property of attenuation rather than of this task, but it is the reason this task does not need to know about agents at all: the member equips their own.

### Retention

The recipient signs and returns. It keeps no copy, and keeps no record of having issued — a room's membership and authority live in the credentials themselves, and a list here would be the roster the whole design keeps away from any single party.

The consequence is that **the owner is the only party who knows what they have issued**, which is invariant I1 of the design working as intended rather than a gap: a room has an accountable party, and this is one of the things they are accountable for.

### Consent/purpose

The credential is minted so that the named subject can enter, remain in, or act in the room. It confers nothing on the caller — it confers on the *subject*, and a caller who could already name the room's key could already have signed it.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
