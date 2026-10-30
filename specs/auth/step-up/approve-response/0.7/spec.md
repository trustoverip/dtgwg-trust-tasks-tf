---
slug: auth/step-up/approve-response
version: "0.7"
title: Auth — Step-up Approve Response
summary: An approver's answer to a step-up approve-request, backed by the approver's own evidence — a proof by its own DID, or a WebAuthn assertion by a passkey registered to it over the step-up challenge. A delegated key's proof never counts as the approver's.
status: draft
targetFrameworkVersion: "0.5.0"
category: authentication
keywords:
  - auth
  - step-up
  - aal
  - wallet
  - approval
  - approver
  - consent
  - passkey
parties:
  - role: Approver
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Every approval rests on evidence the approver alone could produce, and there are exactly two kinds. A Data Integrity proof by the approver's own DID, made for `assertionMethod`, signs the whole document, so it binds the decision, the echoed challenge and any carried evidence to one approver; it is REQUIRED for `didSigned`, for `approverSigned`, and for every denial. A WebAuthn assertion by a passkey the relying party has registered to the approver signs the relying party's single-use challenge, which the relying party bound to this one approval in its own state; for `webauthn` it is the evidence, and the proof is OPTIONAL. Nothing a `webauthn` answer could carry beyond what the assertion and the relying party's own state determine is read without a proof. A proof by any other key — a console signing key or other delegation acting for the approver — is never the approver's, and is refused whatever the evidence. The requirement is therefore RECOMMENDED at the specification level and made conditional in the conformance rules below.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: This document carries a human's approval of one specific elevated operation. A replayed approval spends that decision a second time on an operation they never saw, which is the case SPEC §7.2 item 11 exists for, and item 11 cannot recognise the duplicate outside a bounded window.
sideEffects:
  level: mutating
  rationale: "The approval that elevates the subject's session assurance level, or authorizes the one operation the step-up was bound to."
subjectPath: /subject
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries the decision, the echoed subject and challenge, and the evidence — a WebAuthn assertion or an approver's signed statement. The acknowledgement carries a status and, for an elevation, the session snapshot the caller already holds."
errorCodes:
  - code: auth/step-up/approve-response:challengeUnknown
    meaning: The relying party has no pending step-up matching the echoed challenge.
    retryable: false
  - code: auth/step-up/approve-response:challengeExpired
    meaning: The matching step-up has expired.
    retryable: false
  - code: auth/step-up/approve-response:subjectMismatch
    meaning: The echoed `payload.subject` does not equal the subject bound to the pending step-up; `payload.sessionId` is present when the step-up has no session (or absent or different when it has one); the proof's signer is not the document's issuer, or is a delegated key (a console signing key or other delegation) rather than the approver's own DID; the passkey that made a `webauthn` assertion is not registered to the document's issuer; in self step-up the approver is not the subject; or — for `approverSigned` evidence — the document is not signed by the subject's own DID.
    retryable: false
  - code: auth/step-up/approve-response:approverUnauthorized
    meaning: The approver is neither the subject (self step-up) nor an approver the relying party authorized to ratify step-ups for the subject (delegated step-up).
    retryable: false
  - code: auth/step-up/approve-response:acrUnsatisfied
    meaning: The assurance level the approval demonstrates is below the targetAcr the relying party originally requested.
    retryable: false
  - code: auth/step-up/approve-response:assertionInvalid
    meaning: The WebAuthn assertion carried in `evidence` failed verification, or was not made by a credential the relying party accepts as this approver's evidence. `details.reason` carries a machine-readable hint.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum: ["challengeMismatch", "originMismatch", "rpIdMismatch", "signatureInvalid", "counterRegressed", "credentialUnknown", "userHandleMismatch", "userNotVerified", "credentialNotAccepted"]
  - code: auth/step-up/approve-response:statementInvalid
    meaning: "The `approverSigned` statement is not a valid auth/step-up/approver/attest/0.1 document for this step-up: its proof does not verify or is not made for `authentication` by its issuer, its `recipient` or `audience` is not this relying party, its `purpose` is not `stepUp`, its `subject`, `challenge` or `boundTo` differs from the pending step-up's, it was issued outside the step-up's lifetime, or its `id` was already spent."
    retryable: false
  - code: auth/step-up/approve-response:approverNotBound
    meaning: "The `approverSigned` statement is signed by a DID that is not a live step-up approver bound to the subject, was not among the `approvers` the approve-request offered, or is no longer distinct from the subject's DID and signing keys."
    retryable: false
  - code: auth/step-up/approve-response:noGate
    meaning: The document's `evidence.kind` is one the relying party does not recognise, or names a gate the relying party did not offer for this step-up. A missing proof where one is required is `proofRequired` and an invalid one `proofInvalid` (SPEC §8.3), never this code.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/step-up/approver/attest
  - auth/step-up/approver/list
  - auth/passkey/login/finish
  - auth/signing-key/enroll
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Response** Trust Task is the ratification of an earlier [`auth/step-up/approve-request/0.5`](../../approve-request/0.5/spec.md). The approver echoes the request's `subject`, `challenge` and — when the request carried one — `sessionId`, sets `decision` to `approved` or `denied`, and backs the decision with **the approver's own evidence**, selected by the optional `payload.evidence` tagged union:

- **`evidence.kind = didSigned`** (the default when `evidence` is absent) — the framework `proof`, by a key in the approver's own DID document, IS the evidence. Resulting `amr` reflects `"did"`/`"vta"`.
- **`evidence.kind = webauthn`** — the approver carries an `AuthenticatorAssertionResponse` made by a passkey the relying party has registered to the approver, over the step-up `challenge`. The assertion is the evidence; a framework proof is optional, and counts only if it too is the approver's own. Resulting `amr` reflects `"passkey"`.
- **`evidence.kind = approverSigned`** — the document carries a complete, signed [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) statement with `purpose: stepUp`, made by one of the subject's bound **step-up approvers** over the step-up `challenge` and `boundTo`, inside a document proved by the subject's own DID. Resulting `amr` reflects `"approver"`.

A relying party advertises which gates it will accept via the request's `accepts`.

**Who answers — self vs delegated.** The document `issuer` is the **approver**, which need not be the subject. In **self** step-up the subject ratifies its own session or operation (`issuer == subject`). In **delegated** step-up a distinct, pre-authorized approver ratifies on the subject's behalf (`issuer != subject`) — see [`auth/step-up/policy`](../../policy/0.2/spec.md). Either way the evidence proves the **approver** answered; the relying party **separately** verifies that approver is authorized to ratify for the subject. `approverSigned` is self step-up only.

**A delegated key never speaks for the approver.** A key the relying party records as acting for an identity — a console signing key enrolled with [`auth/signing-key/enroll`](../../../signing-key/enroll/0.2/spec.md), or any other delegation — is an operational credential. Its proof is never the approver's evidence, for any gate, and a document it signs is refused.

### Changes from 0.6

0.5 and 0.6 required the approver's framework proof on every answer, and 0.6 counted a proof by a console key the relying party records as acting for an administrator as that administrator's. That left an administrator working from a web console with no way to answer: the console holds only a delegated key, and [`auth/signing-key/enroll/0.2`](../../../signing-key/enroll/0.2/spec.md) item 7 forbids a delegated key's proof on any approve-response. The two specifications contradicted each other. 0.7 resolves the contradiction in enroll's favour and gives the console an answer it can make:

- **A delegated key's proof never counts as the approver's.** 0.6's console-key clause (producer item 5, consumer item 1a) is gone. A proof by a delegated key is `subjectMismatch`, whatever the evidence.
- **A WebAuthn assertion by the approver's own passkey is evidence on its own.** A `webauthn` answer need carry no proof. The relying party verifies the assertion, requires user verification, requires the passkey to be registered to the approver, and reads nothing from the document that the assertion and its own state do not determine. This is 0.4's unsigned passkey answer, now with explicit rules for what such an answer may and may not decide.
- **`proofRequirement`** is `RECOMMENDED`, made conditional below: REQUIRED for `didSigned`, for `approverSigned` and for every denial.
- **`assertionInvalid`'s `details.reason`** values are lowerCamelCase, and add `userNotVerified` and `credentialNotAccepted`.

Breaking on the wire — 0.6 refused an unproved document, which 0.7 accepts for `webauthn` — and released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.6` remains published.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-response/0.7`, with the **approver** as `issuer` — the subject in self step-up, or a distinct authorized approver in delegated step-up — and the relying party as `recipient`. The `issuer` is the approver even when the document is sent by a surface acting for it, such as a console: never the surface's own key.
2. Echo `payload.subject` and `payload.challenge` verbatim from the matching approve-request, and `payload.sessionId` verbatim when that request carried one. When the request carried no `sessionId`, the response **MUST NOT** carry one.
3. Set `payload.decision` to `approved` or `denied`. When `denied`, populate `payload.deniedReason`.
4. **MAY** declare `payload.grantedAcr` to convey which AAL the approver believes it demonstrated, in a document carrying the approver's proof. Relying parties MAY upgrade the session to ≤ this value; MUST NOT exceed it.
5. Carry a framework `proof` ([SPEC §4.7](/SPEC.md#47-proof)) for `didSigned`, for `approverSigned` and for every `denied` decision, and **MAY** carry one for `webauthn`. Whenever a proof is carried:
   - `proof.proofPurpose` is `assertionMethod`;
   - its `verificationMethod` is listed under the `assertionMethod` relationship of the approver's own DID document, and that DID equals the document `issuer`;
   - it is **never** made by a delegated key — a console signing key, or any other key the relying party records as acting for the approver. A producer holding only such a key answers with `webauthn` evidence and no proof.
5a. For `evidence.kind = webauthn`, populate `payload.evidence.assertion` with the unmodified `AuthenticatorAssertionResponse`, binary fields base64url-encoded, from a passkey registered to the approver at the relying party, obtained with user verification. The assertion's `clientDataJSON` `challenge` MUST equal `payload.challenge`.
5b. For `evidence.kind = approverSigned`:
   - set `issuer` and `payload.subject` to the **subject**, and sign the document with a verification method in the **subject's own** DID document, listed under its `assertionMethod` relationship. A delegated key, a delegated approver, or the step-up approver's own key **MUST NOT** sign the document;
   - populate `payload.evidence.statement` with a complete, signed [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) document, embedded exactly as the approver produced it, with `purpose: stepUp`, `subject` equal to `payload.subject`, `audience` and `recipient` equal to the relying party, and `challenge` and `boundTo` copied from the approve-request;
   - use only an approver listed in the approve-request's `approvers`.
6. A `denied` decision MUST use `evidence.kind = didSigned`, proved by the approver's own DID. A refusal is a statement by the approver; an assertion signs only the challenge, and could not say which way the human decided.

A conforming **consumer** (the relying party) **MUST**:

1. Validate the document. Determine the gate from `payload.evidence.kind` (treating an absent `evidence` as `didSigned`). A kind it does not recognise, or one it did not list in `accepts` for this step-up → `noGate`.
1a. **Verify any proof, and that it is the approver's own, before anything else is consulted.** Refuse with `proofRequired` a `didSigned` or `approverSigned` document, or any `denied` one, that carries no `proof`. Refuse with `proofInvalid` a proof that fails verification, is made for any purpose other than `assertionMethod`, is made by a key not listed under the signer's `assertionMethod` relationship, or is made by a signer whose DID the relying party cannot resolve. Refuse with `subjectMismatch` a proof whose signer DID is not the document `issuer`, and a proof by a key the relying party records as a delegation — a console signing key or any other key acting for another identity — whatever the gate. **A delegated key's proof never establishes an approver.** Where a proof is present, the approver is its signer; in self step-up it MUST be `payload.subject` (`subjectMismatch` otherwise). For `approverSigned`, the signer MUST be `payload.subject` itself, by a verification method of the subject's own DID document (`subjectMismatch`). These checks run before the pending step-up is looked up, and a document refused by them MUST NOT consume or otherwise change it, so no other party can spend an approver's challenge.
2. Locate the matching pending step-up via `payload.challenge`, and **take it in one atomic step**, so that of two documents carrying the same challenge exactly one finds it. A document with no proof has an `id` its sender chose freely, so SPEC §7.2 item 11's duplicate record cannot be what stops a second copy of one assertion. Unknown → `challengeUnknown`. Expired → `challengeExpired`.
3. Verify `payload.challenge` equals the bound challenge bit-for-bit (constant-time comparator).
4. Verify the evidence **and** authorize the approver. A verified signature or assertion is necessary but never sufficient:
   - **Echoes, for every gate.** Verify `payload.subject` equals the subject bound to the pending step-up located in step 2. Verify `payload.sessionId` is present exactly when the pending step-up has a session, and equals it when present. Any mismatch → `subjectMismatch`.
   - **Authorizing the approver.** Confirm the approver may ratify step-ups for `payload.subject` per the relying party's own state, according to the effective step-up mode (see [`auth/step-up/policy`](../../policy/0.2/spec.md)): the approver is the subject (**self**), or the approver bound to this step-up at approve-request time (**delegated**), or satisfies the relying party's approver criterion (**`delegatedAny`**). None of these → `approverUnauthorized`.
   - **`didSigned`** — the approver is the signer established in step 1a; authorize it as above.
   - **`webauthn`** — perform full WebAuthn Level 2 §7.2 assertion verification against the bound challenge; any failure → `assertionInvalid` with `details.reason`. Resolve the credential, from the relying party's own registration records, to the party it is registered to: that party is the **approver**, and it MUST be the document `issuer` (`subjectMismatch` otherwise) and, where a proof is present, the signer established in step 1a (`subjectMismatch` otherwise). A credential registered to nobody is `assertionInvalid` (`credentialUnknown`). **Where the document carries no proof**, the assertion MUST carry the user-verified flag (`assertionInvalid`, `userNotVerified`): possession of an authenticator without a gesture is not a decision. A credential the relying party does not accept as this approver's step-up evidence under its own policy is `assertionInvalid` (`credentialNotAccepted`). Authorize the approver as above.
   - **`approverSigned`** — in addition to the proof verified in step 1a, and in this order:
     1. Verify `payload.evidence.statement` as an [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) document, as that specification requires of its consumer, over the object **exactly as received**: its own proof (`authentication`, by its `issuer`), `recipient` and `payload.audience` equal to this relying party's DID, `payload.purpose` equal to `stepUp`, `payload.subject`, `payload.challenge` and `payload.boundTo` equal to the pending step-up's — `boundTo` read from the relying party's own state, never from this document — `issuedAt` within the step-up's lifetime, and its `id` not already spent. Any failure → `statementInvalid`.
     2. Verify the statement's `issuer` is a live step-up approver bound to `payload.subject`, was listed in the `approvers` of the approve-request the pending step-up was minted with, and is still distinct from the subject's DID, from every verification method in the subject's DID document, and from every key the subject signs operations with at this relying party. Any failure → `approverNotBound`.
     3. Refuse `approverSigned` for a step-up that is not self mode (`approverUnauthorized`): a step-up approver is the subject's own factor and never speaks for anyone else.
4a. **What an answer without a proof decides.** A `webauthn` document with no proof is authenticated by its assertion alone, and the assertion signs only the challenge. The relying party **MUST** read from such a document nothing the assertion and its own state do not determine: the decision is `approved` (a denial without a proof is refused in step 1a); the subject, the session and — for a bound step-up — the operation are the pending step-up's own; the assurance level is the one the verified assertion demonstrates, never `payload.grantedAcr`; and `ext` is ignored.
5. When `decision === "approved"` and the step-up is **not bound to an operation**:
   - Apply the session elevation per the consumer's policy: update `session.amr` to include the new factor (`"passkey"` for a webauthn gate, `"approver"` for an approverSigned gate, `"vta"`/`"did"` for a did-signed gate), raise `session.acr` to at most `payload.grantedAcr` — or, for a `webauthn` document with no proof, to the level the assertion demonstrates.
   - If the session's `acr` cannot reach the originally-requested `targetAcr` → `acrUnsatisfied`.
   - Consume the step-up so the same approve-response cannot be replayed. For a webauthn gate, persist the credential counter update; for an approverSigned gate, record the statement's `id` and the approver's last use.
   - Acknowledge with `status: "elevated"` and the session snapshot.
5a. When `decision === "approved"` and the step-up **is bound to an operation**:
   - Apply the approval to that operation and **MUST NOT** elevate the session.
   - Read the bound operation from the consumer's **own state**, established when the step-up was minted. A relying party **MUST NOT** take it from any member of this document, the embedded statement included.
   - `acrUnsatisfied` does not apply.
   - Consume the step-up exactly as above.
   - Acknowledge with `status: "recorded"`, `boundTo` naming the operation, and **no** `session`.
   - Applying an approval whose bound operation is no longer there — expired, or already completed — is **not** an error. The relying party changes nothing and still acknowledges `recorded`.
6. When `decision === "denied"`:
   - Verify the did-signed gate and authorize the approver exactly as in step 4.
   - Consume the step-up.
   - Persist the denied response for audit. Take no further action on the session.
7. Record the evidence kind in its audit of the step-up, whether the document carried a proof, and — for `webauthn` — the credential, and for `approverSigned` the approver DID.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this document exercises is **the approver's standing to ratify a step-up the relying party asked of a subject** — in self step-up, the subject's own; in delegated step-up, the standing the relying party granted the approver for that subject, read from its own state. It is not a standing to choose what is approved: the elevation or operation is the relying party's pending step-up, located by `challenge` and read from its own state.

**Approval evidence is the approver's own, and of exactly two kinds:** a proof by a key in the approver's own DID document, or a user-verified WebAuthn assertion by a passkey the relying party has registered to the approver, over a challenge the relying party bound to this one approval. A key acting *for* the approver — a console signing key, a session key, any delegation — is neither, because it is held by a surface rather than by the approver, and a surface that holds it can sign whatever is put in front of it. Possession of such a key is one factor; an approval exists to require the approver's own act. For `approverSigned` the proof (the subject's own DID) and the factor (the statement by a key bound as a step-up factor) are deliberately two different keys.

Which gates a relying party accepts, and for what, is its own policy; this section describes the evidence each gate is, and imposes no obligation to accept any of them.

## Definitions

* **Bound approval.** An approval that authorizes one operation instead of raising an assurance level. Answered with `recorded`.
* **Approver.** The party that ratifies the step-up; identified by `issuer`, and established by its own proof or by the registration of the passkey that made its assertion.
* **Delegated key.** A key the relying party records as acting for another identity — a console signing key enrolled with [`auth/signing-key/enroll`](../../../signing-key/enroll/0.2/spec.md), for instance. Never the approver's evidence.
* **Step-up approver.** A `did:key` bound at the relying party to one subject as their step-up factor; the issuer of the `approverSigned` statement. Not the document's `issuer`.
* **Relying party.** The party that initiated the step-up; identified by `recipient`.
* **Subject.** The VID whose session is being elevated or whose act is being approved.

## Payload

`payload.subject`, `payload.challenge`, `payload.decision` — REQUIRED.

`payload.sessionId` — echoed when the approve-request carried one; absent when it did not.

`payload.deniedReason` — required when decision is `denied`.

`payload.grantedAcr` — optional approver-declared AAL, read only from a document the approver proved.

`payload.evidence` — optional tagged union selecting the gate: `{ "kind": "didSigned" }` (the default when omitted), `{ "kind": "webauthn", "assertion": … }`, or `{ "kind": "approverSigned", "statement": … }`. Consumers that do not recognise a `kind` MUST reject with `noGate` rather than silently elevate.

`payload.ext` — extension slot. Ignored on a document with no proof.

## Request

The approver (`issuer`) sends the response document to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Approver approves the transfer

```json
{
  "id": "approve-resp-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "approved",
    "grantedAcr": "aal2"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T14:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### A console administrator approves one signed operation with their passkey

The community refused an `acl/grant` signed by Carol's console key with an inline [`approve-request/0.5`](../../approve-request/0.5/spec.md) bound to that grant and accepting `webauthn`. Carol's browser holds her console key — a delegation — and her passkey, and no key of her DID. It runs the ceremony with user verification and sends this document **with no proof**, naming Carol as `issuer`. The community verifies the assertion, finds the passkey registered to Carol, takes the pending step-up, and records the approval against the one grant it was minted for. The console then re-sends the identical grant.

```json
{
  "id": "approve-resp-1111-2222-3333-444455556666",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7",
  "issuer": "did:webvh:QmCarolScid2:carol.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-30T09:00:05Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid2:carol.example",
    "challenge": "R3JhbnRBZG1pbk5vbmNlQ2Fyb2wxMjM0",
    "decision": "approved",
    "evidence": {
      "kind": "webauthn",
      "assertion": {
        "id": "Y3JlZF9jYXJvbF8x",
        "rawId": "Y3JlZF9jYXJvbF8x",
        "type": "public-key",
        "response": {
          "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiUjNKaGJuUkJaRzFwYms1dmJtTmxRMkZ5YjJ3eE1qTTAifQ",
          "authenticatorData": "Q2Fyb2xBdXRoRGF0YVVW",
          "signature": "Q2Fyb2xTaWduYXR1cmU",
          "userHandle": "dXNyX2Nhcm9s"
        }
      }
    }
  }
}
```

### A wallet administrator approves one signed grant with a step-up approver

The community refused a signed `acl/grant` with an inline [`approve-request/0.5`](../../approve-request/0.5/spec.md) accepting `approverSigned` and naming Alice's bound approver. The browser plugin recomputed `boundTo` from the grant, showed it, and — after the gesture that unlocks its key — signed the statement. The console then had Alice's wallet sign this document **as Alice's own DID**, embedding the statement unchanged. The community verifies Alice's proof, then the statement, then that its signer is Alice's live approver.

```json
{
  "id": "approve-resp-cccc-dddd-eeee-ffff00002222",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T09:00:05Z",
  "payload": {
    "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
    "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
    "decision": "approved",
    "evidence": {
      "kind": "approverSigned",
      "statement": {
        "id": "urn:uuid:3a6c1e0b-9d2f-4b7a-8c15-2e4f6a8b0c01",
        "type": "https://trusttasks.org/spec/auth/step-up/approver/attest/0.1",
        "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
        "issuedAt": "2026-10-02T09:00:00Z",
        "payload": {
          "purpose": "stepUp",
          "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
          "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
          "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
          "boundTo": "uEiD3n4bE1QbQ8nY2l0cGhYc2FsdGVkZGlnZXN0"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
          "created": "2026-10-02T09:00:00Z",
          "proofPurpose": "authentication",
          "proofValue": "z3kg…"
        }
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid4:wallet.example:alice#key-1",
    "created": "2026-10-02T09:00:05Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5Lg…"
  }
}
```

### Approver approves with a passkey on their phone, and signs too (cross-device AAL2)

```json
{
  "id": "approve-resp-aaaa-bbbb-cccc-ddddeeeeffff",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "approved",
    "grantedAcr": "aal2",
    "evidence": {
      "kind": "webauthn",
      "assertion": {
        "id": "Y3JlZF8xYTJiM2M",
        "rawId": "Y3JlZF8xYTJiM2M",
        "type": "public-key",
        "response": {
          "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiVkhKaGJuTm1aWEpEYjI1bWFYSnRUbTl1WTJWWVdRIn0",
          "authenticatorData": "TXltSXNUaGVBdXRoRGF0YQ",
          "signature": "U2lnbmF0dXJlVmFsdWVCYXNlNjQ",
          "userHandle": "dXNyXzhmMmMxZDRlOWE3YjMwNTY"
        }
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T14:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Approver denies

```json
{
  "id": "approve-resp-8901-2345-6789-0abcdef12345",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "denied",
    "deniedReason": "User does not recognize this transfer."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T14:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4mD…"
  }
}
```

## Response

The relying party's `#response` confirms whether the approval was applied, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### The bound approval is recorded

The acknowledgement of Carol's passkey approval above: it names the operation and carries no session.

```json
{
  "id": "approve-ack-dddd-eeee-ffff-000011112222",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7#response",
  "threadId": "approve-resp-1111-2222-3333-444455556666",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid2:carol.example",
  "issuedAt": "2026-10-30T09:00:06Z",
  "payload": {
    "status": "recorded",
    "boundTo": "uEiCb5Lr0ZQWm1y7c2FsdGVkZGlnZXN0Q2Fyb2w"
  }
}
```

### Successful elevation

```json
{
  "id": "approve-ack-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.7#response",
  "threadId": "approve-resp-7890-1234-5678-90abcdef1234",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:31Z",
  "payload": {
    "status": "elevated",
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T14:30:31Z",
      "amr": ["did", "vta"],
      "acr": "aal2"
    }
  }
}
```

### A console key signed the answer

The document was signed by Carol's console signing key — a delegation acting for her — rather than by her own DID, and carried her passkey assertion. A delegated key's proof never counts as the approver's, so it is refused before the step-up is looked up, and the step-up stays pending. Sent without the console key's proof, the same assertion would have been accepted.

```json
{
  "id": "approve-err-eeee-ffff-0000-111122223333",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "approve-resp-cccc-dddd-eeee-ffff00003333",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-30T09:00:06Z",
  "payload": {
    "code": "auth/step-up/approve-response:subjectMismatch",
    "message": "a delegated signing key is never the approver's evidence; answer with the passkey assertion alone, or sign with the approver's own DID",
    "retryable": false
  }
}
```

## Security & Privacy

**A bound approval is not a smaller elevation.** The two outcomes are disjoint: an approval either raises an assurance level or authorizes one operation, never both. A relying party MUST NOT emit `recorded` alongside a session change, and a producer receiving `recorded` MUST NOT assume any subsequent request is more privileged than it was before.

**Why a delegated key never counts.** A console key, or any other delegation, is a key a surface holds so that it can sign operations without the approver's hand on each one. An approval exists to require that hand. Counting a delegated key's proof as the approver's would let whatever holds that key — a script in the console's origin, a stolen browser profile — approve the very operations it asked for. 0.6 permitted it for consoles; 0.7 does not, which is what [`auth/signing-key/enroll/0.2`](../../../signing-key/enroll/0.2/spec.md) item 7 already required.

**Why an assertion alone is enough, and only so far.** A WebAuthn assertion by the approver's registered passkey, with user verification, over a single-use challenge the relying party minted and bound to one pending step-up, is the approver's gesture: no surface can produce it without the approver's authenticator and the approver's verification. What it signs is the challenge, not this document. A party holding a captured assertion could therefore wrap it in a document of its own — but every member such a document could vary is either checked against the relying party's own state (subject, session, the bound operation), forced (the decision is `approved`, since a denial needs a proof), derived from the assertion (the assurance level), or ignored (`grantedAcr`, `ext`). The pending step-up is taken atomically and consumed (consumer item 2), so a captured assertion approves at most the one operation the approver was shown, once. That is why consumer item 4a lists what an unproved answer may decide, and why the list must not grow.

**The proof, when present, binds the whole document.** For `didSigned` and `approverSigned`, and whenever a `webauthn` answer also carries the approver's proof, the proof covers the decision, `grantedAcr` and the evidence, and an intermediary cannot rewrap them.

**Why the subject's own DID, for `approverSigned`.** Requiring the subject's own DID keeps the answer attributable to the subject; requiring the statement over this challenge keeps the factor fresh; requiring the two to be different keys is what makes it two factors.

**Why the statement is the factor, and only by enrolment.** A step-up approver's signature counts only because its key is not one the caller holds for signing, is held behind a gesture, and was bound to the subject on an anchor independent of the subject's signing key ([`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md)). User verification is not provable from the statement as it is from a WebAuthn assertion's UV flag; a relying party accepting `approverSigned` trusts it from enrolment.

**Which passkeys count is the relying party's policy.** A relying party may accept, as one approver's step-up evidence, only some of the passkeys registered to it — a dedicated step-up passkey and not a sign-in passkey, say, once the approver has enrolled one — and refuses the others with `credentialNotAccepted`. This specification fixes only that the passkey is the approver's own.

**Approver liveness at use.** Revocation of an approver takes effect at the next statement it signs. A relying party MUST check the binding when the statement arrives (step 4), not only when it listed the approver in the request.

**Why `assertionMethod`.** An approve-response is an attestation, a statement by the approver that it decided. A proof is therefore made for `assertionMethod`, and a key the approver lists only for `authentication`, `keyAgreement` or capability relationships does not speak for its decisions. The embedded statement is made for `authentication`: it proves a key was used, not that a decision was taken.

**Delegated approver authorization.** In delegated step-up the evidence proves the *approver* answered — not the subject. The relying party MUST independently confirm the approver is one it authorized for `payload.subject`, from its own state, never from the document. A `delegatedAny` criterion MUST remain a bounded, least-privilege set, never "any holder".

**WebAuthn challenge binding.** For a webauthn gate the assertion's `clientDataJSON` challenge MUST equal `payload.challenge`. The WebAuthn `rpId`/`origin` checks bind the gesture to the relying party; the challenge, bound in the relying party's state, binds it to one step-up.

**Gate ↔ amr consistency.** The factor recorded in `session.amr` MUST match the gate actually verified: `"passkey"` only when a WebAuthn assertion verified, `"approver"` only when an approver's statement verified, `"vta"`/`"did"` only when a DID signature by the approver verified. `grantedAcr` is an approver claim, not evidence.

**Echo verification.** Every echo field present MUST be compared bit-for-bit, and a `sessionId` MUST be refused when the step-up has none.

**Replay.** Consuming the challenge on success-or-denial is mandatory, and atomic. The statement's `id` is recorded against reuse. A second approve-response carrying the same challenge MUST fail with `challengeUnknown`.

**Denied responses as audit.** A signed `denied` response proves the user actively refused. Relying parties SHOULD preserve denied responses with the same retention policy as approvals.

**Wallet UX.** Approvers presenting approve-requests to humans MUST display the request's `reason` and the relying party identity verbatim.

**Free text.** `deniedReason` is free text, bounded at 500 characters, authored by the approver; a surface rendering it MUST attribute it to the approver and MUST NOT present it as the service's own account of the refusal.

The optional `ext` extension is part of the producer's signed surface, and is ignored on a document with no proof.

### Data carried

The request carries the approver's decision, the echoed `subject` and `challenge` (and `sessionId` when the step-up has a session), an optional free-text `deniedReason`, and the evidence — a WebAuthn assertion, which names a credential and carries the authenticator's signature counter and an opaque user handle, or for `approverSigned` a statement naming the subject, the relying party, the challenge, the opaque `boundTo` and the approver's `did:key`. It carries no attribute of the subject beyond their identifier and **no description of the operation being approved**.

The response carries a status, and either a session snapshot (`elevated`) or an opaque operation identifier (`recorded`).

### Correlation

The `challenge` is single-use and unpredictable, so two approve-responses cannot be linked by it. A passkey's credential id and user handle link every step-up the approver answers with that passkey at this relying party — inherent, since the relying party must recognise its own registration — and WebAuthn scopes both to this relying party. The approver DID in an `approverSigned` statement links every step-up the subject answers with it at this relying party, and nowhere else. In delegated step-up the document links an approver to a subject, a relationship the relying party already holds.

### Retention

A relying party MUST consume the pending step-up on success or denial. Responses — approvals and denials alike, with any embedded assertion or statement — SHOULD be retained under the same policy, as the evidence of which factor answered. A `recorded` acknowledgement SHOULD be retained with the operation it names.

### Consent/purpose

This document carries one human's decision about one specific act, and the evidence is what makes that decision attributable. Its purpose is exhausted by the operation it authorizes. An assertion or an approver's statement made for it is likewise exhausted: it is bound to one challenge and one operation, and a relying party MUST NOT reuse it as evidence for any other.
