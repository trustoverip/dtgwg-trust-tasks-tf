---
slug: auth/step-up/approver/list
version: "0.1"
title: "Auth — Step-up Approver List"
summary: A subject lists the step-up approvers bound to them — or an administrator lists another subject's within its authority — so a factor can be recognised before it is replaced or revoked.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - second-factor
parties:
  - role: Subject, or an administrator
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
subjectPath: /subject
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The list is an inventory of a subject's step-up factors and when each was last used — reconnaissance for anyone deciding which factor to steal or which to revoke to lock the subject out of every step-up. Enumeration is therefore tied to a signature the relying party resolves to the subject or to an administrator over them, rather than to a bearer token.
sideEffects:
  level: none
  rationale: >-
    Reads the subject's approver bindings. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses a subject's approver bindings — approver DIDs, labels, timestamps and the anchor each was bound on — to the subject or an administrator over them. No key material, and no other subject's bindings.
retention:
  class: transient
  rationale: >-
    A read. The relying party needs nothing from it after answering, and the caller's copy is a point-in-time view that goes stale at the next enrolment or revocation.
errorCodes: []
related:
  - auth/step-up/approver/enroll
  - auth/step-up/approver/revoke
  - auth/step-up/approver/invite
  - auth/step-up/approver/redeem/finish
---

## Abstract

The **Auth — Step-up Approver List** Trust Task returns every live step-up approver bound to one subject: which `did:key`s answer that subject's step-ups at this relying party, the label each was given, when and on what anchor each was bound, and when each was last used. It is the input to [`revoke`](../../revoke/0.1/spec.md) and to a rotation through [`enroll`](../../enroll/0.1/spec.md) `replaces`, and what a console shows under a member's factors.

By default it lists the caller's own approvers. An administrator may name another subject within its authority — for incident response, or to see whether a member it is about to invite already holds a factor.

**Why there is no `show` sibling.** The registry's default for a collection is a list-and-show pair, because a filtered list cannot say `notFound`. This collection does not need one: it is bounded at five approvers per subject, so the whole of it is returned every time and a caller wanting one entry finds it by `approverDid`; and whether a given approver exists is not load-bearing for anybody but its subject and the administrators over them, who already receive the complete list. A per-approver lookup would add a probe surface (is this `did:key` bound here, and to whom?) and answer nothing the list does not. `list` is the only read of the family.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the relying party):

1. **MUST** resolve the proof's signer to an identity — the signer itself, or the identity a delegated key acts for.
2. **MUST** treat an absent `subject` as that identity.
3. **MUST** answer a `subject` other than the caller only when the caller holds administrative standing that covers that subject, and otherwise refuse with the framework `permissionDenied` — given identically whether or not the subject is known, so the refusal reveals nothing about who holds approvers.
4. **MUST** return every live approver bound to the subject, and nothing else: no revoked approver, no other subject's. A subject with none receives an empty list.
5. **MUST NOT** treat the read as using any approver: `lastUsedAt` is unchanged.

## Definitions

- **`subject`** — whose approvers to list; the caller when absent.
- **`approvers`** — the subject's live bindings, each an [`Approver`](../../_shared/0.1/approver.schema.json).

## Request

The caller (`issuer`) sends the request to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Alice lists her own approvers

```json
{
  "id": "urn:uuid:b2e6f9a3-4c7d-4e8f-a01b-2c3d4e5f6a01",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/list/0.1",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T10:59:00Z",
  "threadId": "urn:uuid:b2e6f9a3-4c7d-4e8f-a01b-2c3d4e5f6aff",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid4:wallet.example:alice#key-1",
    "created": "2026-10-02T10:59:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The relying party answers with the sub-schema reachable via `$anchor: "response"`: the subject's live approvers. Refusals use `trust-task-error`.

### Alice's approvers

```json
{
  "id": "urn:uuid:b2e6f9a3-4c7d-4e8f-a01b-2c3d4e5f6a02",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/list/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-02T10:59:01Z",
  "threadId": "urn:uuid:b2e6f9a3-4c7d-4e8f-a01b-2c3d4e5f6aff",
  "payload": {
    "approvers": [
      {
        "approverDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
        "label": "Browser plugin — work laptop",
        "enrolledAt": "2026-10-01T15:04:06Z",
        "enrolledVia": "invite",
        "lastUsedAt": "2026-10-02T09:00:06Z"
      }
    ]
  }
}
```

## Security & Privacy

**Metadata only.** The list carries public identifiers and timestamps, never key material and never the statements the approvers made.

**Administrators see within their authority.** An administrator over a subject may need to know whether that subject can answer a step-up at all before deciding whether to invite them; an administrator without standing over them learns nothing, not even whether they exist.

### Data carried

The request carries at most a subject DID. The response carries, per approver, its DID, the subject, a label, when and on what anchor it was bound, and when it was last used. The label is free text the subject (or an inviter) chose, and a surface rendering it **MUST** attribute it to its author.

### Correlation

The approver DIDs are stable handles for the subject at this relying party, disclosed only to the subject and the administrators over them. `lastUsedAt` reveals when the subject last answered a step-up; that is the information a holder needs to recognise a stale or stolen factor, and why the read is proof-gated.

### Retention

Nothing is retained by the relying party beyond what any authenticated read leaves in its logs. The caller's copy is a snapshot.

### Consent/purpose

The list exists so a subject, or an administrator over them, can recognise the subject's factors before replacing or revoking one. It **SHOULD NOT** be used to profile when a subject is active.
