---
slug: auth/step-up/approver/enroll
version: "0.1"
title: "Auth — Step-up Approver Enroll"
summary: A subject who already holds a step-up factor adds a step-up approver, or rotates one out, on the evidence of that existing factor and the new approver's signed proof of possession — without anyone else's help.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - enrollment
  - rotation
  - self-service
  - proof-of-possession
parties:
  - role: Subject
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
  - role: Step-up approver being enrolled
    requirement: REQUIRED
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The proof is the subject's own signature — by a verification method of the subject's own DID document, never a delegated key — and attributes the enrolment to the subject on every transport, so the relying party can tell whose factor is being changed and audit it against them. Every enrolment route carries it, as both redeem legs do. It is never the authority for the change: a factor bound on the strength of the subject's signing key alone would add nothing to it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed enrolment is refused once its approver is bound, but one replayed with a still-recorded approval could re-run a rotation its subject had abandoned; SPEC §7.2 item 11 absorbs that only inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Binds a step-up approver to the subject, and — with `replaces` — revokes one of the subject's existing approvers in the same step. The binding confers no role, scope or session, and is reversed by auth/step-up/approver/revoke.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries an approver DID, an optional label, an optional approver to replace and the new approver's signed statement; the response returns the stored binding. No key material moves either way.
retention:
  class: durable
  rationale: >-
    The binding is kept until revoked, and the enrolment — which existing factor authorized it, which approver it replaced — is the audit record of how the factor came to exist.
errorCodes:
  - code: auth/step-up/approver/enroll:subjectMismatch
    meaning: "The document is not signed by a verification method of the issuer's own DID document. A signing-key or console delegation acting for the subject, or the approver being enrolled, does not satisfy this: every enrolment carries the subject's own signature."
    retryable: false
  - code: auth/step-up/approver/enroll:noFactorHeld
    meaning: "The subject holds no step-up factor from which a self-service enrolment could be authorized. Ask an administrator for an auth/step-up/approver/invite; a subject never invites themselves."
    retryable: false
  - code: auth/step-up/approver/enroll:statementInvalid
    meaning: "`statement` is absent from an enrolment whose authority evidence was accepted, or is not a valid auth/step-up/approver/attest/0.1 document for it: its proof does not verify, its issuer is not `approverDid`, its `recipient` or `audience` is not this relying party, its `purpose` is not `enrol`, its `subject` is not the subject, its `challenge` is not the challenge of the step-up that authorized this enrolment, its `boundTo` is not this enrolment's terms digest, it was issued outside that step-up's lifetime, or its `id` was already spent."
    retryable: false
  - code: auth/step-up/approver/enroll:approverNotDistinct
    meaning: "`approverDid` is the subject's own DID, appears as a verification method in the subject's DID document, is a key the subject signs operations with at the relying party, or holds standing of its own there."
    retryable: false
  - code: auth/step-up/approver/enroll:approverAlreadyBound
    meaning: "`approverDid` is already bound — to this subject or another — or was bound and revoked. Generate a new one."
    retryable: false
  - code: auth/step-up/approver/enroll:tooManyApprovers
    meaning: "The subject already holds five live approvers and the enrolment names no `replaces`. Enrol again naming one in `replaces`, or revoke one first."
    retryable: false
  - code: auth/step-up/approver/enroll:replaceNotFound
    meaning: "`replaces` names no live approver of this subject. The same answer is given for an approver that does not exist, was revoked, or is bound to someone else."
    retryable: false
related:
  - auth/step-up/approver/attest
  - auth/step-up/approver/invite
  - auth/step-up/approver/list
  - auth/step-up/approver/revoke
  - auth/step-up/approve-request
  - auth/step-up/approve-response
---

## Abstract

A step-up approver ([`attest/0.1`](../../attest/0.1/spec.md)) must be bound on an anchor independent of the subject's signing key. For a subject's *first* approver that anchor is someone else — an administrator's [`invite`](../../invite/0.1/spec.md), or the community's install token. For every later one it can be the subject themselves, **through a factor they already hold**: a live approver, or a passkey bound to them as a step-up factor.

This task is that self-service path. It adds an approver — on a new browser, a new phone — or, with `replaces`, rotates one out in the same step, so a user who moves device needs nobody's help. Two pieces of evidence are involved, and they are deliberately different:

- **authority** — the subject's existing factor, exercised over this one enrolment, so a stolen signing key alone cannot add a factor for its thief;
- **possession** — the new approver's signed [`attest/0.1`](../../attest/0.1/spec.md) statement with `purpose: enrol`, over the same challenge, bound to a digest of this enrolment's terms, so a factor nobody holds cannot be bound.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Enrolment terms and their digest

The **enrolment terms** are this document's `payload` exactly as received with its `statement` member removed — `approverDid`, `label`, `replaces` and `ext` as present.

The **terms digest** is the value the new approver's statement binds as `boundTo`. It is a DigestMultibase ([framework `DigestMultibase`](/SPEC.md#66-shared-schema-components)): the multibase base58btc (`z`) encoding of the multihash sha2-256 (`0x12 0x20` ‖ digest) of the UTF-8 bytes of the [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonical JSON of the object

```
{
  "type":      "https://trusttasks.org/spec/auth/step-up/approver/enroll/0.1",
  "subject":   <the subject DID>,
  "challenge": <the challenge of the step-up authorizing this enrolment>,
  "terms":     <the enrolment terms>
}
```

The challenge salts the digest, so it is unlinkable across enrolments even of identical terms; the subject and type keep a digest computed for one subject or task from verifying for another. The producer and the approver compute it from the terms they are shown; the consumer recomputes it from the document as received.

A conforming **producer** (the subject):

1. **MUST** set `issuer` to the subject and sign the document with a verification method of the subject's **own** DID document — never with an [`auth/signing-key`](../../../../signing-key/enroll/0.2/spec.md) or console delegation acting for the subject, and never with the approver's key.
2. **MAY** first send the enrolment without `statement`, to obtain the challenge of the step-up that will authorize it (see [Authorization](#authorization)); **MUST** then carry, on the send that is to execute, the new approver's `attest/0.1` statement with `purpose: enrol`, `subject` the subject, `audience` and `recipient` the relying party, `challenge` that step-up's challenge, and `boundTo` the terms digest.
3. **MUST NOT** change any member of the terms between the send that obtained the authority evidence and the send that carries the statement: the evidence covers the terms, and changed terms are a different enrolment.
4. **MAY** name in `replaces` one of the subject's live approvers to revoke in the same step.

A conforming **consumer** (the relying party):

1. **MUST** require the proof's verification method to belong to the `issuer` DID's own document, and take that DID as the subject. A proof by a delegated key acting for the subject — a signing-key or console delegation — or by any other key → `subjectMismatch`, checked before anything else and changing nothing.
2. **MUST** refuse with `noFactorHeld` when the subject holds no step-up factor.
3. **MUST NOT** write the binding until it has accepted authority evidence that the subject exercised a step-up factor they already hold, over **these enrolment terms** (see [Authorization](#authorization)). The subject's signature on this document is never that evidence. Evidence bound to anything less than the whole terms — a session, the subject alone, terms with any member changed — **MUST NOT** be accepted: otherwise an approval taken to add one approver could add another, or remove one it never named in `replaces`.
4. **MUST** identify the enrolment, for the purpose of that evidence, by its type and its terms — never by a payload that includes `statement` — so the send carrying the statement is recognised as the enrolment the evidence was given for.
5. Once the authority evidence is accepted, **MUST** verify `statement` as [`attest/0.1`](../../attest/0.1/spec.md) requires of its consumer, over the object exactly as received, with its `issuer` equal to `approverDid` and `boundTo` equal to the terms digest it recomputes → `statementInvalid` (including when `statement` is absent). It **MUST** verify the statement **before** it consumes the authority evidence, and a `statementInvalid` refusal leaves that evidence in place until it expires, so a producer can correct the statement without asking the human again.
6. **MUST** refuse with `approverNotDistinct`, `approverAlreadyBound`, `replaceNotFound` or `tooManyApprovers` when the corresponding rule holds, deciding the cap only after accepting the authority evidence and counting an approver named in a valid `replaces` as already gone. `replaceNotFound` **MUST** be given identically whatever the reason.
7. **MUST** write the binding with `enrolledVia: selfService`, revoke any `replaces` approver exactly as [`revoke`](../../revoke/0.1/spec.md) would, and consume the authority evidence, all in one critical section serialised with every other write of the subject's approvers.
8. **MUST** audit the enrolment against the subject, naming the new approver, the factor that authorized it and any approver replaced.
9. **SHOULD** tell the subject, over a channel it already has with them, that an approver was added or replaced.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is **a step-up factor the subject already holds**, exercised over this enrolment's terms. The subject's own signature — by their own DID, never a delegation — establishes whose factors are being changed, as it does on both redeem legs, so a console key that a script could drive cannot even begin an enrolment. It is not the authority, because a second factor bound on the strength of a signing key adds nothing to that key.

The reference implementation — a VTC — obtains that evidence as an **operation-bound step-up** delivered inline ([`approve-request/0.4`](../../../approve-request/0.4/spec.md) *Inline delivery*):

1. The subject sends the enrolment, typically without `statement`. The relying party finds no approval bound to its terms and refuses with `permissionDenied`, carrying `details.stepUpRequest`: an `approve-request/0.4` payload for the subject, with a fresh `challenge`, `boundTo` its own salted digest of the type and terms, and `accepts` the kinds the subject's existing factors can answer — `approverSigned` naming the subject's live approvers, `webauthn` for a step-up passkey.
2. The subject answers with an [`approve-response/0.6`](../../../approve-response/0.6/spec.md) from an existing factor, and the relying party records the approval. Nothing is elevated.
3. The new approver signs its enrolment statement over the same `challenge`, bound to the terms digest, and the subject re-sends the enrolment with that statement. The relying party recognises the terms, verifies the statement, removes the approval and writes the binding.

The approver being replaced may be the factor that answers step 2: a subject rotating an approver they still hold uses it one last time. Which factors a relying party accepts, and whether it asks for this evidence at all, is its own policy; this section describes the evidence the task assumes.

## Definitions

- **`approverDid`** — the step-up approver to bind, an Ed25519 `did:key`.
- **`label`** — a name for where the approver lives.
- **`replaces`** — a live approver of the same subject to revoke in the same step.
- **`statement`** — the new approver's signed `attest/0.1` document with `purpose: enrol`, its proof of possession.
- **Enrolment terms**, **terms digest** — as defined in [Conformance](#enrolment-terms-and-their-digest).

## Request

The subject (`issuer`) sends the request to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Alice rotates her approver to a new laptop, after a step-up from the old one

```json
{
  "id": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5f01",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/enroll/0.1",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T11:00:20Z",
  "threadId": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5fff",
  "payload": {
    "approverDid": "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw",
    "label": "Browser plugin — new laptop",
    "replaces": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "statement": {
      "id": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5f0a",
      "type": "https://trusttasks.org/spec/auth/step-up/approver/attest/0.1",
      "issuer": "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw",
      "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
      "issuedAt": "2026-10-02T11:00:15Z",
      "payload": {
        "purpose": "enrol",
        "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
        "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
        "challenge": "Um90YXRlQXBwcm92ZXJOb25jZTk4NzY1NA",
        "boundTo": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw#z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw",
        "created": "2026-10-02T11:00:15Z",
        "proofPurpose": "authentication",
        "proofValue": "z4mD…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid4:wallet.example:alice#key-1",
    "created": "2026-10-02T11:00:20Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Lg…"
  }
}
```

## Response

The relying party answers with the sub-schema reachable via `$anchor: "response"`: the new binding as stored. Refusals use `trust-task-error`; under the reference design the first answer to an enrolment is a `permissionDenied` carrying `details.stepUpRequest` — not a failure of the enrolment but the request for its authority evidence.

### The new approver is bound and the old one revoked

```json
{
  "id": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5f02",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/enroll/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-02T11:00:21Z",
  "threadId": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5fff",
  "payload": {
    "approver": {
      "approverDid": "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw",
      "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
      "label": "Browser plugin — new laptop",
      "enrolledAt": "2026-10-02T11:00:21Z",
      "enrolledVia": "selfService"
    }
  }
}
```

### A console key tries to enrol an approver

The document was signed by Alice's console signing key, a delegation acting for her, rather than by her own DID. It is refused before any step-up is offered.

```json
{
  "id": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5f03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-02T11:00:01Z",
  "threadId": "urn:uuid:a1d5e8f2-3b6c-4d7e-9f0a-1b2c3d4e5ffe",
  "payload": {
    "code": "auth/step-up/approver/enroll:subjectMismatch",
    "message": "an approver enrolment must be signed by the subject's own DID",
    "retryable": false
  }
}
```

## Security & Privacy

**The subject's own signature, never a delegation.** A console or signing-key delegation is an operational credential held in a browser; if it could start an enrolment, a script that stole it would at least be able to drive the step-up prompt toward its own approver. Requiring the subject's own DID keeps every enrolment — invite, self-service and install alike — attributable to a key only the subject's wallet holds.

**Two pieces of evidence, two keys, one set of terms.** The existing factor's approval and the new approver's statement are both bound to the same terms over the same challenge. A captured approval cannot add a different approver (different terms, different digest), and a captured statement cannot be enrolled under a different approval (different challenge).

**A rotation is one enrolment.** `replaces` is part of the terms, so an approval to add an approver cannot be reused to remove one, and a producer that adds `replaces` after a `tooManyApprovers` refusal has a new enrolment needing fresh evidence.

**Losing every factor is not this task's problem.** A subject with no factor is answered `noFactorHeld` and pointed at an administrator's invite, never at a self-invite: the anchor for a first factor is somebody else's authority.

**Free text.** `label` is bounded at 64 characters, authored by the subject and untrusted; it is retained with the binding and shown to whoever may list or revoke it.

### Data carried

The request carries an approver DID, an optional label, an optional approver to replace, and the new approver's statement naming the subject, the relying party, a challenge and the terms digest. The response echoes the stored binding. No key material moves. A producer **SHOULD NOT** put a person's name or any identifier in `label` beyond what distinguishes one device from another.

### Correlation

The new approver DID becomes a stable handle for the subject's step-ups at this relying party, as the old one was. A producer **SHOULD** generate an approver DID per relying party, so a rotation at one community reveals nothing at another.

### Retention

The binding is kept until revoked, and a replaced approver as a tombstone, so neither can be bound again. The enrolment's audit record is durable.

### Consent/purpose

The approver is bound for one purpose: to be the subject's step-up factor at this relying party. The statement made for its enrolment is exhausted by it and **MUST NOT** be accepted as a step-up.
