---
slug: vtc/members/removal-notice
version: "0.1"
title: VTC Members — Removal Notice
summary: A community tells a member it removed them — on whose authority, when, and why — so the most consequential decision a community makes is not the one it delivers in silence.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - members
  - removal
  - notice
  - due-process
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
  - role: removed member
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: This is the one message a member may need to show a third party. Without a proof it evidences nothing beyond the transport that carried it, and the member cannot demonstrate who removed them, when, or on what grounds — which is the whole reason to send it.
sideEffects:
  level: none
  rationale: "Reports a removal already carried out; the notice itself changes nothing at the recipient."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vtc/members/admin-remove
  - vtc/members/purge
  - vtc/members/self-remove-receipt
---

## Abstract

The **VTC Members — Removal Notice** Trust Task is how a community tells a
member that it removed them, naming the deciding administrator, the moment the
decision took effect, and the reason given.

It exists because removal is the most consequential thing a community can do to
a member and, without this, the one it delivers with the least information —
none. A removed member's only observable signal is a side effect: the revocation
bit on their membership credential flips. They must infer their own removal from
a status list, and can learn nothing about why.

## Not a receipt

[`vtc/members/self-remove-receipt`](../../self-remove-receipt/0.1/) is the
adjacent task and the contrast is the point. A receipt answers a request the
member made: it is correlated to that request, and the member is already
expecting it. A removal notice answers nothing. The member did not ask, is not
waiting, and may be offline. Everything below follows from that asymmetry.

It is also why this task carries `decidedBy` and `reason` where the receipt does
not. A departing member knows why they left.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the community) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vtc/members/removal-notice/0.1`, with itself as `issuer` and the removed member as `recipient`.
2. Send it **after** the removal has taken effect, not on deciding it. A notice for a removal that then fails is worse than none.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof) — see [Security & Privacy](#security--privacy) for why this is REQUIRED here.
4. Resolve `disposition` to a concrete value. A community's `policydefault` preference **MUST NOT** appear on the wire; the member learns nothing from being told a default was applied without being told which.
5. Populate `decidedAt` with when the removal took effect, **not** when the notice was sent. The two diverge whenever the member was offline, and it is the decision that has to be placeable in time.
6. **MUST NOT** send this for a member-initiated departure. That is
   [`self-remove-receipt`](../../self-remove-receipt/0.1/), and conflating them
   would tell a member who chose to leave that they were removed.

A conforming producer **SHOULD** include `reason` whenever the removing
administrator gave one. Omitting the member's only account of why is a choice,
and the schema keeps it distinguishable from an explicitly empty one.

A conforming **consumer** (the removed member) **MUST**:

1. Verify the `proof` before relying on the notice, and **MUST NOT** treat an unverified notice as evidence of anything.
2. Verify that the `issuer` is the community it purports to be — a notice is only meaningful from the community that held the membership.
3. Treat `did` in the payload as authoritative over any transport-level addressing, so that a notice retained or forwarded independently still names its subject.

## Delivery, and the member who cannot ask

A removal notice has a delivery problem no other member-facing task has: **the
act it reports is the act that ends the member's ability to ask about it.**
Removal withdraws the member's access, so a consumer implementation cannot fall
back on polling — the endpoint that would answer is the endpoint that now
refuses them.

Two consequences for producers:

- A community **MUST NOT** rely on the member being reachable at the moment of
  removal, and **SHOULD** use a durable, retrying delivery with a window
  measured in weeks rather than hours. A member offline across the delivery
  window learns nothing, permanently.
- A community **SHOULD** send the notice on a channel that survives the removal.
  Anything gated on the membership being current is unavailable by construction
  at the only moment it is needed.

This specification does not mandate a transport. It states the property a
transport must have, because a conforming notice that cannot arrive is not a
conforming implementation.

## Request

The notice is one-way: there is no response document, and the payload schema
carries no `$defs.Response`. A member has nothing to answer — they were not
asked.

### An administrator removes a member, with a reason

```json
{
  "id": "urn:uuid:6b1f8a3c-90de-4a71-b2f5-1c8e70d4a913",
  "type": "https://trusttasks.org/spec/vtc/members/removal-notice/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkRemovedMember",
  "issuedAt": "2026-08-23T09:14:07Z",
  "payload": {
    "did": "did:key:z6MkRemovedMember",
    "code": "adminRemoved",
    "disposition": "tombstone",
    "reason": "Repeated breach of the community code of conduct, after two warnings.",
    "decidedAt": "2026-08-23T09:14:02Z",
    "decidedBy": "did:key:z6MkCommunityAdmin"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-23T09:14:07Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

### A super-administrator purge, no reason given

`reason` is absent rather than empty: the community did not state one. A member
receiving this knows they were purged and by whom, and knows that no reason was
recorded — which is itself information.

```json
{
  "id": "urn:uuid:8d3a0c5e-b2f0-4c93-a417-3e0a92f6c135",
  "type": "https://trusttasks.org/spec/vtc/members/removal-notice/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkPurgedMember",
  "issuedAt": "2026-08-23T11:02:44Z",
  "payload": {
    "did": "did:key:z6MkPurgedMember",
    "code": "purged",
    "disposition": "purge",
    "decidedAt": "2026-08-23T11:02:41Z",
    "decidedBy": "did:key:z6MkSuperAdmin"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-23T11:02:44Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Security & Privacy

**The proof is the point, not a formality.** This is the one member-facing
message whose value depends on being shown to somebody else — an appeal, a
dispute, another community assessing a rejected applicant. Authenticated
transport establishes the sender to the *recipient* and stops there; a member
forwarding an unsigned notice forwards an assertion anyone could have written.
`proofRequirement` is REQUIRED for that reason, and a consumer that skips
verification has kept a document that evidences nothing.

**A notice is evidence against its sender, which is why it may not be sent.** A
community that removes members without notice is indistinguishable, to an
outside observer, from one that never removes anyone. Nothing in this
specification can compel a community to send. What it can do is make the notice
verifiable when it *is* sent, so that its presence is meaningful — and make its
absence a visible choice rather than a technical limitation.

**`decidedBy` names a person, and that is deliberate.** It is more disclosure
than the community strictly needs to make, and communities where removal is
contentious may find it uncomfortable. It is required because the alternative —
"the community removed you" — is unanswerable and unappealable. A member cannot
contest a decision whose maker is unnamed.

**`reason` is operator-authored free text and reaches the member verbatim.**
Producers **SHOULD** treat it as attacker-influenced on the consumer side and
consumers **MUST NOT** render it as markup. It is capped at 1024 characters to
bound both the envelope and the blast radius.

**The notice discloses only the member's own removal.** It carries nothing about
other members, the community's size, or any other decision.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
