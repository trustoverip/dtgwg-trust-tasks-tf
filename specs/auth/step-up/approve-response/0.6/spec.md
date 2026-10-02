---
slug: auth/step-up/approve-response
version: "0.6"
title: Auth — Step-up Approve Response
summary: An approver's signed answer to a step-up approve-request. Every answer carries the approver's assertionMethod proof; passkey evidence or a bound step-up approver's statement is in addition to it, never instead.
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
parties:
  - role: Approver
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: This document is the approver's attestation of a decision, so it is signed by the approver whatever evidence it carries. The proof is made for `assertionMethod`, by a key listed under the approver's `assertionMethod` relationship, and it is what binds the decision, the echoed challenge and the evidence to one approver. A WebAuthn assertion proves a gesture over the challenge, and a step-up approver's statement proves its key signed the challenge; neither signs this document, so without the proof an intermediary holding a captured assertion or statement could attach it to a document it wrote. Evidence is therefore in addition to the proof, never a replacement for it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: This document carries a human's approval of one specific elevated operation. A replayed approval spends that decision a second time on an operation they never saw, which is the case SPEC §7.2 item 11 exists for, and item 11 cannot recognise the duplicate outside a bounded window.
sideEffects:
  level: mutating
  rationale: "The signed approval that elevates the subject's session assurance level, or authorizes the one operation the step-up was bound to."
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
    meaning: The echoed `payload.subject` does not equal the subject bound to the pending step-up, `payload.sessionId` is present when the step-up has no session (or absent or different when it has one), the proof's verificationMethod DID does not equal the document's issuer (the signer is not the named approver), in self step-up the approver the proof establishes is not the subject, or — for `approverSigned` evidence — the document is not signed by the subject's own DID.
    retryable: false
  - code: auth/step-up/approve-response:approverUnauthorized
    meaning: The document's issuer is neither the subject (self step-up) nor an approver the relying party authorized to ratify step-ups for the subject (delegated step-up).
    retryable: false
  - code: auth/step-up/approve-response:acrUnsatisfied
    meaning: The grantedAcr is below the targetAcr the relying party originally requested.
    retryable: false
  - code: auth/step-up/approve-response:assertionInvalid
    meaning: The WebAuthn assertion carried in `evidence` failed verification. `details.reason` carries a machine-readable hint.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum: ["challenge_mismatch", "origin_mismatch", "rp_id_mismatch", "signature_invalid", "counter_regressed", "credential_unknown", "user_handle_mismatch"]
  - code: auth/step-up/approve-response:statementInvalid
    meaning: "The `approverSigned` statement is not a valid auth/step-up/approver/attest/0.1 document for this step-up: its proof does not verify or is not made for `authentication` by its issuer, its `recipient` or `audience` is not this relying party, its `purpose` is not `stepUp`, its `subject`, `challenge` or `boundTo` differs from the pending step-up's, it was issued outside the step-up's lifetime, or its `id` was already spent."
    retryable: false
  - code: auth/step-up/approve-response:approverNotBound
    meaning: "The `approverSigned` statement is signed by a DID that is not a live step-up approver bound to the subject, was not among the `approvers` the approve-request offered, or is no longer distinct from the subject's DID and signing keys."
    retryable: false
  - code: auth/step-up/approve-response:noGate
    meaning: The document's `evidence.kind` is one the relying party does not recognise, or names a gate the relying party did not offer for this step-up. A missing framework proof is `proofRequired` and an invalid one `proofInvalid` (SPEC §8.3), never this code.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/step-up/approver/attest
  - auth/step-up/approver/list
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Response** Trust Task is the ratification of an earlier [`auth/step-up/approve-request/0.4`](../../approve-request/0.4/spec.md). The approver echoes the request's `subject`, `challenge` and — when the request carried one — `sessionId`, sets `decision` to `approved` or `denied`, signs the document with its `assertionMethod` key, and backs the decision with **one of three cryptographic gates**, selected by the optional `payload.evidence` tagged union:

- **`evidence.kind = didSigned`** (the default when `evidence` is absent) — the framework `proof` IS the gate: a Data Integrity signature from a key the approver controls. Resulting `amr` reflects `"did"`/`"vta"`.
- **`evidence.kind = webauthn`** — the approver carries an `AuthenticatorAssertionResponse` produced by a platform passkey over the step-up `challenge`. The assertion is the gate, verified in addition to the framework proof, exactly as [`auth/passkey/login/finish/0.1`](../../../passkey/login/finish/0.1/spec.md) does. Resulting `amr` reflects `"passkey"`.
- **`evidence.kind = approverSigned`** — the document carries a complete, signed [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) statement with `purpose: stepUp`, made by one of the subject's bound **step-up approvers** over the step-up `challenge` and `boundTo`. The statement is the gate, verified in addition to the framework proof. Resulting `amr` reflects `"approver"`.

A relying party advertises which gates it will accept via the request's `accepts` ([`approve-request/0.4`](../../approve-request/0.4/spec.md)).

**Who signs — self vs delegated.** The document `issuer` is the **approver**, which need not be the subject. In **self** step-up the subject ratifies its own session or operation (`issuer == subject`). In **delegated** step-up a distinct, pre-authorized approver ratifies on the subject's behalf (`issuer != subject`) — see [`auth/step-up/policy`](../../policy/0.2/spec.md). Either way the gate proves the **approver** signed; the relying party **separately** verifies that approver is authorized to ratify for the subject. `approverSigned` is self step-up only (see below).

**Every response is signed by the approver.** Whichever gate it uses, the document carries the approver's framework `proof`, made for `assertionMethod` with a key listed under the approver's `assertionMethod` relationship. For `webauthn` and `approverSigned` the evidence is an additional gate the relying party verifies as well, never a substitute for the proof.

**0.3** added the bound approval — one that authorizes a single operation, acknowledged `recorded`, elevating nothing. **0.4** let a bound approval have no session. **0.5** required the approver's proof on every response.

**0.6 adds `approverSigned`.** A subject who signs in through a wallet acts as a DID the relying party knows no passkey for, and a signature by that DID alone is a proof the subject could make without any additional factor, which a re-authentication must not accept. A **step-up approver** is the missing factor: a `did:key` bound to the subject at this relying party, distinct from every key that can sign the subject's operations, whose key is used only behind a user gesture. With `approverSigned`:

- the document's own proof is **still required**, and **MUST** be by the **subject's own DID** — a verification method in the subject's DID document under `assertionMethod`, never a delegated key such as a console signing key, and never a delegated approver. It establishes *who* is answering;
- the embedded **statement is the factor**. It establishes that the subject's bound approver key — and therefore, by enrolment, the human holding it — attested this one challenge for this one operation at this relying party.

Neither signature suffices alone: the subject's DID proof without the statement is a signature the caller could produce with no additional factor, and the statement without the subject's proof is a factor detached from any answer the subject gave. Nothing else changes from 0.5.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-response/0.6`, with itself (the **approver**) as `issuer` — the subject in self step-up, or a distinct authorized approver in delegated step-up — and the relying party as `recipient`.
2. Echo `payload.subject` and `payload.challenge` verbatim from the matching approve-request, and `payload.sessionId` verbatim when that request carried one. When the request carried no `sessionId`, the response **MUST NOT** carry one.
3. Set `payload.decision` to `approved` or `denied`. When `denied`, populate `payload.deniedReason`.
4. **MAY** declare `payload.grantedAcr` to convey which AAL the approver believes it demonstrated. Relying parties MAY upgrade the session to ≤ this value; MUST NOT exceed it.
5. Sign every response, whatever `evidence` it carries, with a framework `proof` ([SPEC §4.7](/SPEC.md#47-proof)) such that:
   - `proof.proofPurpose` is `assertionMethod`;
   - its `verificationMethod` is listed under the `assertionMethod` relationship of the signer's DID document, and that DID equals the document `issuer`;
   - the signer is the **approver**: the subject itself in self step-up, the delegated approver in delegated step-up. Where the approver signs through a console key the relying party records as acting for an administrator, the approver is that administrator — **except** with `approverSigned` evidence (item 5b).
5a. For `evidence.kind = webauthn`, additionally populate `payload.evidence.assertion` with the unmodified `AuthenticatorAssertionResponse`; binary fields base64url-encoded. The assertion's `clientDataJSON` `challenge` MUST equal `payload.challenge`. The assertion is a second gate, verified in addition to the proof; it never replaces it.
5b. For `evidence.kind = approverSigned`:
   - set `issuer` and `payload.subject` to the **subject**, and sign the document with a verification method in the **subject's own** DID document, listed under its `assertionMethod` relationship. A delegated key (a console or other signing-key delegation acting for the subject), a delegated approver, or the step-up approver's own key **MUST NOT** sign the document;
   - populate `payload.evidence.statement` with a complete, signed [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) document, embedded exactly as the approver produced it, with `purpose: stepUp`, `subject` equal to `payload.subject`, `audience` and `recipient` equal to the relying party, and `challenge` and `boundTo` copied from the approve-request;
   - use only an approver listed in the approve-request's `approvers`.
6. A `denied` decision MUST use `evidence.kind = didSigned` — a refusal is an approver-signed statement (the subject in self mode, the authorized approver in delegated mode), not a possession proof.

A conforming **consumer** (the relying party) **MUST**:

1. Validate the document. Determine the gate from `payload.evidence.kind` (treating an absent `evidence` as `didSigned`). A kind it does not recognise, or one it did not list in `accepts` for this step-up → `noGate`.
1a. **Verify the approver's proof, for every gate, before anything else is consulted.** Refuse a document with no `proof` with `proofRequired`. Refuse with `proofInvalid` a proof that fails verification, is made for any purpose other than `assertionMethod`, is made by a key not listed under the signer's `assertionMethod` relationship, or is made by a signer whose DID the relying party cannot resolve. Refuse with `subjectMismatch` a proof whose signer DID is not the document `issuer`. The approver the proof establishes is the signer, or the administrator a console key acts for per the relying party's own state; in self step-up it MUST be `payload.subject` (`subjectMismatch` otherwise). **For `approverSigned`, the signer MUST be `payload.subject` itself, by a verification method of the subject's own DID document — a console key or any other delegation acting for the subject does not satisfy this, even though it would establish the subject as approver for another gate (`subjectMismatch`).** These checks run before the pending step-up is looked up, and a document refused by them MUST NOT consume or otherwise change it, so no other party can spend an approver's challenge.
2. Locate the matching pending step-up via `payload.challenge`. Unknown → `challengeUnknown`. Expired → `challengeExpired`.
3. Verify `payload.challenge` equals the bound challenge bit-for-bit (constant-time comparator).
4. Verify the gate **and** authorize the approver. A verified signature is necessary but never sufficient:
   - **Echoes, for every gate.** Verify `payload.subject` equals the subject bound to the pending step-up located in step 2. Verify `payload.sessionId` is present exactly when the pending step-up has a session, and equals it when present. Any mismatch → `subjectMismatch`.
   - **`didSigned`** — authorize the approver: confirm `issuer` may ratify step-ups for `payload.subject` per the relying party's own state, according to the effective step-up mode (see [`auth/step-up/policy`](../../policy/0.2/spec.md)): `issuer == subject` (**self**), or `issuer` is the approver bound to this step-up at approve-request time (**delegated**), or `issuer` satisfies the relying party's approver criterion (**`delegatedAny`**). None of these → `approverUnauthorized`.
   - **`webauthn`** — in addition to the proof verified in step 1a, perform full WebAuthn Level 2 §7.2 assertion verification against the bound challenge; any failure → `assertionInvalid` with `details.reason`. Resolve the credential to the **approver** and authorize that approver exactly as for `didSigned`. The approver the credential resolves to MUST be the approver the proof established in step 1a (`subjectMismatch` otherwise).
   - **`approverSigned`** — in addition to the proof verified in step 1a, and in this order:
     1. Verify `payload.evidence.statement` as an [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) document, as that specification requires of its consumer, over the object **exactly as received**: its own proof (`authentication`, by its `issuer`), `recipient` and `payload.audience` equal to this relying party's DID, `payload.purpose` equal to `stepUp`, `payload.subject`, `payload.challenge` and `payload.boundTo` equal to the pending step-up's — `boundTo` read from the relying party's own state, never from this document — `issuedAt` within the step-up's lifetime, and its `id` not already spent. Any failure → `statementInvalid`.
     2. Verify the statement's `issuer` is a live step-up approver bound to `payload.subject`, was listed in the `approvers` of the approve-request the pending step-up was minted with, and is still distinct from the subject's DID, from every verification method in the subject's DID document, and from every key the subject signs operations with at this relying party. Any failure → `approverNotBound`.
     3. Refuse `approverSigned` for a step-up that is not self mode (`approverUnauthorized`): a step-up approver is the subject's own factor and never speaks for anyone else.
5. When `decision === "approved"` and the step-up is **not bound to an operation**:
   - Apply the session elevation per the consumer's policy: update `session.amr` to include the new factor (`"passkey"` for a webauthn gate, `"approver"` for an approverSigned gate, `"vta"`/`"did"` for a did-signed gate), raise `session.acr` to at most `payload.grantedAcr`.
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
7. Record the evidence kind in its audit of the step-up and, for `approverSigned`, the approver DID.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this document exercises is **the approver's standing to ratify a step-up the relying party asked of a subject** — in self step-up, the subject's own; in delegated step-up, the standing the relying party granted the approver for that subject, read from its own state. It is not a standing to choose what is approved: the elevation or operation is the relying party's pending step-up, located by `challenge` and read from its own state.

The proof establishes who answered; the evidence establishes the factor. For `approverSigned` the two are deliberately separate keys: the subject's DID establishes that the subject answered, and the statement — by a key bound to the subject as a step-up factor on an anchor independent of the subject's signing key — establishes that the subject's additional factor was used. Which gates a relying party accepts, and for what, is its own policy; this section describes the evidence each gate is, and imposes no obligation to accept any of them.

## Definitions

* **Bound approval.** An approval that authorizes one operation instead of raising an assurance level. Answered with `recorded`.
* **Approver.** The party that ratifies the step-up; identified by `issuer`.
* **Step-up approver.** A `did:key` bound at the relying party to one subject as their step-up factor; the issuer of the `approverSigned` statement. Not the document's `issuer`.
* **Relying party.** The party that initiated the step-up; identified by `recipient`.
* **Subject.** The VID whose session is being elevated or whose act is being approved.

## Payload

`payload.subject`, `payload.challenge`, `payload.decision` — REQUIRED.

`payload.sessionId` — echoed when the approve-request carried one; absent when it did not.

`payload.deniedReason` — required when decision is `denied`.

`payload.grantedAcr` — optional approver-declared AAL.

`payload.evidence` — optional tagged union selecting the gate: `{ "kind": "didSigned" }` (the default when omitted), `{ "kind": "webauthn", "assertion": … }`, or `{ "kind": "approverSigned", "statement": … }`. Consumers that do not recognise a `kind` MUST reject with `noGate` rather than silently elevate.

`payload.ext` — extension slot.

## Request

The approver (`issuer`) sends the response document to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Approver approves the transfer

```json
{
  "id": "approve-resp-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.6",
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

### A wallet administrator approves one signed grant with a step-up approver

The community refused a signed `acl/grant` with an inline [`approve-request/0.4`](../../approve-request/0.4/spec.md) accepting `approverSigned` and naming Alice's bound approver. The browser plugin recomputed `boundTo` from the grant, showed it, and — after the gesture that unlocks its key — signed the statement. The console then had Alice's wallet sign this document **as Alice's own DID**, embedding the statement unchanged. The community verifies Alice's proof, then the statement, then that its signer is Alice's live approver.

```json
{
  "id": "approve-resp-cccc-dddd-eeee-ffff00002222",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.6",
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

### Approver approves with a passkey on their phone (cross-device AAL2)

```json
{
  "id": "approve-resp-aaaa-bbbb-cccc-ddddeeeeffff",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.6",
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
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.6",
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

The acknowledgement of the approver-signed approval above: it names the operation and carries no session.

```json
{
  "id": "approve-ack-dddd-eeee-ffff-000011112222",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.6#response",
  "threadId": "approve-resp-cccc-dddd-eeee-ffff00002222",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-02T09:00:06Z",
  "payload": {
    "status": "recorded",
    "boundTo": "uEiD3n4bE1QbQ8nY2l0cGhYc2FsdGVkZGlnZXN0"
  }
}
```

### Successful elevation

```json
{
  "id": "approve-ack-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.6#response",
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

### A console key answered an approver-signed step-up

The document was signed by Alice's console signing key — a delegation acting for her — rather than by her own DID. For `approverSigned` that is refused before the step-up is looked up, and the step-up stays pending.

```json
{
  "id": "approve-err-eeee-ffff-0000-111122223333",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "approve-resp-cccc-dddd-eeee-ffff00003333",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-02T09:00:06Z",
  "payload": {
    "code": "auth/step-up/approve-response:subjectMismatch",
    "message": "an approver-signed step-up must be signed by the subject's own DID",
    "retryable": false
  }
}
```

## Security & Privacy

**A bound approval is not a smaller elevation.** The two outcomes are disjoint: an approval either raises an assurance level or authorizes one operation, never both. A relying party MUST NOT emit `recorded` alongside a session change, and a producer receiving `recorded` MUST NOT assume any subsequent request is more privileged than it was before.

**The proof always, and the evidence in addition.** The relying party MUST NOT take any field in this document as authoritative without the approver's verified `assertionMethod` proof, and for a `webauthn` or `approverSigned` gate without the verified evidence as well. An assertion signs the challenge, not the document; so does a statement. An intermediary holding either could otherwise wrap it in a document of its own and choose the `decision`, `grantedAcr` and `subject` it carries.

**Why the subject's own DID, for `approverSigned`.** A console key, or any other delegation, is a key the subject already holds for signing operations — exactly the kind of key a re-authentication must not accept as its factor. Letting one sign an `approverSigned` response would let whoever holds that one key supply the document, while the statement — the factor — could have been captured from any earlier ceremony at a different challenge. Requiring the subject's own DID keeps the answer attributable to the subject; requiring the statement over this challenge keeps the factor fresh; requiring the two to be different keys is what makes it two factors.

**Why the statement is the factor, and only by enrolment.** A step-up approver's signature counts only because its key is not one the caller holds for signing, is held behind a gesture, and was bound to the subject on an anchor independent of the subject's signing key ([`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md)). User verification is not provable from the statement as it is from a WebAuthn assertion's UV flag; a relying party accepting `approverSigned` trusts it from enrolment.

**Approver liveness at use.** Revocation of an approver takes effect at the next statement it signs. A relying party MUST check the binding when the statement arrives (step 4), not only when it listed the approver in the request.

**Why `assertionMethod`.** An approve-response is an attestation, a statement by the approver that it decided. The proof is therefore made for `assertionMethod`, and a key the approver lists only for `authentication`, `keyAgreement` or capability relationships does not speak for its decisions. The embedded statement is made for `authentication`: it proves a key was used, not that a decision was taken.

**Delegated approver authorization.** In delegated step-up the gate proves the *approver* signed — not the subject. The relying party MUST independently confirm the `issuer` is an approver it authorized for `payload.subject`, from its own state, never from the document. A `delegatedAny` criterion MUST remain a bounded, least-privilege set, never "any holder".

**WebAuthn challenge binding.** For a webauthn gate the assertion's `clientDataJSON` challenge MUST equal `payload.challenge`. The WebAuthn `rpId`/`origin` checks bind the gesture to the relying party; the framework `proof` and `recipient` bind the document.

**Gate ↔ amr consistency.** The factor recorded in `session.amr` MUST match the gate actually verified: `"passkey"` only when a WebAuthn assertion verified, `"approver"` only when an approver's statement verified, `"vta"`/`"did"` only when a DID signature verified. `grantedAcr` is an approver claim, not evidence.

**Echo verification.** Every echo field present MUST be compared bit-for-bit, and a `sessionId` MUST be refused when the step-up has none.

**Replay.** Consuming the challenge on success-or-denial is mandatory, and the statement's `id` is recorded against reuse. A second approve-response carrying the same challenge MUST fail with `challengeUnknown`.

**Denied responses as audit.** A signed `denied` response proves the user actively refused. Relying parties SHOULD preserve denied responses with the same retention policy as approvals.

**Wallet UX.** Approvers presenting approve-requests to humans MUST display the request's `reason` and the relying party identity verbatim.

**Free text.** `deniedReason` is free text, bounded at 500 characters, authored by the approver; a surface rendering it MUST attribute it to the approver and MUST NOT present it as the service's own account of the refusal.

The optional `ext` extension is part of the producer's signed surface.

### Data carried

The request carries the approver's decision, the echoed `subject` and `challenge` (and `sessionId` when the step-up has a session), an optional free-text `deniedReason`, and the gate — for `approverSigned`, a statement naming the subject, the relying party, the challenge, the opaque `boundTo` and the approver's `did:key`. It carries no attribute of the subject beyond their identifier and **no description of the operation being approved**.

The response carries a status, and either a session snapshot (`elevated`) or an opaque operation identifier (`recorded`).

### Correlation

The `challenge` is single-use and unpredictable, so two approve-responses cannot be linked by it. The approver DID in an `approverSigned` statement links every step-up the subject answers with it at this relying party — inherent, since the relying party must recognise the factor — and, being bound at this relying party only, nowhere else. In delegated step-up the document links an approver to a subject, a relationship the relying party already holds.

### Retention

A relying party MUST consume the pending step-up on success or denial. Signed responses — approvals and denials alike, with any embedded statement — SHOULD be retained under the same policy, as the evidence of which factor answered. A `recorded` acknowledgement SHOULD be retained with the operation it names.

### Consent/purpose

This document carries one human's decision about one specific act, and the gate is what makes that decision attributable. Its purpose is exhausted by the operation it authorizes. An approver's statement made for it is likewise exhausted: it is bound to one challenge and one operation, and a relying party MUST NOT reuse it as evidence for any other.
