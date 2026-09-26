---
slug: auth/step-up/approve-response
version: "0.5"
title: Auth — Step-up Approve Response
summary: An approver's signed answer to a step-up approve-request. Every answer carries the approver's assertionMethod proof; passkey evidence is in addition to it, never instead.
status: draft
targetFrameworkVersion: "0.5.0"
category: authentication
keywords:
  - auth
  - step-up
  - aal
  - wallet
  - approval
  - consent
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Approver
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: This document is the approver's attestation of a decision, so it is signed by the approver whatever evidence it carries. The proof is made for `assertionMethod`, by a key listed under the approver's `assertionMethod` relationship, and it is what binds the decision, the echoed challenge and the evidence to one approver. A WebAuthn assertion proves a gesture over the challenge; it does not sign the document, so without the proof an intermediary holding a captured assertion could attach it to a document it wrote. Passkey evidence is therefore in addition to the proof, never a replacement for it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: This document carries a human's approval of one specific elevated operation. A replayed approval spends that decision a second time on an operation they never saw, which is the case SPEC §7.2 item 11 exists for, and item 11 cannot recognise the duplicate outside a bounded window.
sideEffects:
  level: mutating
  rationale: "The signed approval that elevates the subject's session assurance level."
subjectPath: /subject
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/step-up/approve-response:challengeUnknown
    meaning: The relying party has no pending step-up matching the echoed challenge.
    retryable: false
  - code: auth/step-up/approve-response:challengeExpired
    meaning: The matching step-up has expired.
    retryable: false
  - code: auth/step-up/approve-response:subjectMismatch
    meaning: The echoed `payload.subject` does not equal the subject bound to the pending step-up, `payload.sessionId` is present when the step-up has no session (or absent or different when it has one), the proof's verificationMethod DID does not equal the document's issuer (the signer is not the named approver), or in self step-up the approver the proof establishes is not the subject.
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
  - code: auth/step-up/approve-response:noGate
    meaning: The document's `evidence.kind` is one the relying party does not recognise, or names a gate the relying party did not offer for this step-up. A missing framework proof is `proofRequired` and an invalid one `proofInvalid` (SPEC §8.3), never this code.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Response** Trust Task is the ratification of an earlier [`auth/step-up/approve-request/0.3`](../../approve-request/0.3/spec.md). The approver echoes the request's `subject`, `challenge` and — when the request carried one — `sessionId`, sets `decision` to `approved` or `denied`, signs the document with its `assertionMethod` key, and backs the decision with **one of two cryptographic gates**, selected by the optional `payload.evidence` tagged union:

- **`evidence.kind = did-signed`** (the default when `evidence` is absent) — the framework `proof` IS the gate: a Data Integrity signature from a key the subject controls. This is the original, transport-agnostic step-up. Resulting `amr` reflects `"did"`/`"vta"`.
- **`evidence.kind = webauthn`** — the approver carries an `AuthenticatorAssertionResponse` produced by a platform passkey over the step-up `challenge` (the cross-device path: a relying party at AAL 1 in a browser, the user elevating with Face ID / Touch ID / Android biometric on their phone). The assertion is the gate, verified in addition to the framework proof, which the document carries in this case too; the relying party verifies the assertion per WebAuthn Level 2 §7.2 exactly as [`auth/passkey/login/finish/0.1`](../../../passkey/login/finish/0.1/spec.md) does. Resulting `amr` reflects `"passkey"`.

Supporting both lets one step-up flow serve a DID-key approver (a VTA ratifying programmatically) and a biometric-bound passkey approver (a phone) without two separate protocols. A relying party advertises which gates it will accept via the request's `acceptableEvidence` ([`approve-request`](../../approve-request/0.3/spec.md)).

**Who signs — self vs delegated.** The document `issuer` is the **approver**, which need not be the subject. In **self** step-up the subject ratifies its own session (`issuer == subject`) — e.g. a wallet holding the subject's key. In **delegated** step-up a distinct, pre-authorized approver ratifies on the subject's behalf (`issuer != subject`) — e.g. an administrator's phone, or a Verifiable-Trust Agent acting under policy ([`auth/step-up/policy`](../../policy/0.2/spec.md) `mode: delegated`, where the approver is the subject's `AclEntry.stepUp.approver`). Either way the gate proves the **approver** signed; the relying party **separately** verifies that approver is authorized to ratify for the subject (see Conformance and Security). This matches the request side, which already addresses the approve-request to that approver as its `recipient`.

**Every response is signed by the approver.** Whichever gate it uses, the document carries the approver's framework `proof`, made for `assertionMethod` with a key listed under the approver's `assertionMethod` relationship. For `webauthn` the passkey assertion is an additional gate the relying party verifies as well, never a substitute for the proof.

A relying party processing an `approved` response elevates the session's `amr`/`acr` per its own policy and replies with the elevated session snapshot. A `denied` response is signed too (did-signed) — it serves as audit evidence that the user explicitly refused.

**Not every approval elevates a session, and 0.3 exists to say so.** A step-up may be minted to authorize **one operation** rather than to raise an assurance level — the case this was written for is a `persona/disclosure/present` gated by `release: stepUp`, whose approval MUST bind to the `previewId` and not to the session, because binding to the session turns "each time" into "once per login". A relying party applying such an approval changes no session, and the 0.2 acknowledgement had no word for that: `elevated` was untrue and `rejected` was worse, since the approval had in fact been applied. `recorded` is that word.

The version is what forced this. A response document's `type` is the request's `type` plus `#response`, so a relying party cannot reach for a newer minor on its own — the approver's request decides. A relying party that wants to answer `recorded` therefore has to be *asked* in 0.3.

**0.4 lets a bound approval have no session.** In 0.3 every approval echoed a `sessionId`, so a bound approval could only authorize an operation that arrived over a session. An operation that arrives as a signed Trust Task document has none, and [`approve-request/0.3`](../../approve-request/0.3/spec.md) now omits `sessionId` for a step-up bound to such an operation. The echo rules follow: `sessionId` is echoed exactly when the request carried it, and the subject is compared with the subject bound to the pending step-up rather than with a session's. Nothing else changes, and an approval that elevates a session is exactly as it was in 0.3.

**0.5 requires the approver's proof on every response.** In 0.4 a `webauthn` response could omit the framework `proof`. But the document is the approver's attestation of a decision, and a WebAuthn assertion proves a gesture over a challenge without signing the document around it: whoever held a captured assertion could wrap it in a document of their own, naming whatever `decision`, `grantedAcr` or `subject` they liked. In 0.5 every response carries a proof made for `assertionMethod`, by a key listed under the approver's `assertionMethod` relationship, and passkey evidence is in addition to it. The proof is verified, and the signer bound to the approver, before the pending step-up is consulted, so no other party can spend an approver's challenge. Nothing else changes.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-response/0.5`, with itself (the **approver**) as `issuer` — the subject in self step-up, or a distinct authorized approver in delegated step-up — and the relying party as `recipient`.
2. Echo `payload.subject` and `payload.challenge` verbatim from the matching approve-request, and `payload.sessionId` verbatim when that request carried one. When the request carried no `sessionId`, the response **MUST NOT** carry one.
3. Set `payload.decision` to `approved` or `denied`. When `denied`, populate `payload.deniedReason`.
4. **MAY** declare `payload.grantedAcr` to convey which AAL the approver believes it demonstrated. Relying parties MAY upgrade the session to ≤ this value; MUST NOT exceed it.
5. Sign every response, whatever `evidence` it carries, with a framework `proof` ([SPEC §4.7](/SPEC.md#47-proof)) such that:
   - `proof.proofPurpose` is `assertionMethod`;
   - its `verificationMethod` is listed under the `assertionMethod` relationship of the signer's DID document, and that DID equals the document `issuer`;
   - the signer is the **approver**: the subject itself in self step-up, the delegated approver in delegated step-up. Where the approver signs through a console key the relying party records as acting for an administrator, the approver is that administrator.
5a. For `evidence.kind = webauthn`, additionally populate `payload.evidence.assertion` with the unmodified `AuthenticatorAssertionResponse`; binary fields base64url-encoded. The assertion's `clientDataJSON` `challenge` MUST equal `payload.challenge`. The assertion is a second gate, verified in addition to the proof; it never replaces it.
6. A `denied` decision MUST use `evidence.kind = did-signed` — a refusal is an approver-signed statement (the subject in self mode, the authorized approver in delegated mode), not a possession proof.

A conforming **consumer** (the relying party) **MUST**:

1. Validate the document. Determine the gate from `payload.evidence.kind` (treating an absent `evidence` as `didSigned`).
1a. **Verify the approver's proof, for every gate, before anything else is consulted.** Refuse a document with no `proof` with `proofRequired`. Refuse with `proofInvalid` a proof that fails verification, is made for any purpose other than `assertionMethod`, is made by a key not listed under the signer's `assertionMethod` relationship, or is made by a signer whose DID the relying party cannot resolve. Refuse with `subjectMismatch` a proof whose signer DID is not the document `issuer`. The approver the proof establishes is the signer, or the administrator a console key acts for per the relying party's own state; in self step-up it MUST be `payload.subject` (`subjectMismatch` otherwise). These checks run before the pending step-up is looked up, and a document refused by them MUST NOT consume or otherwise change it, so no other party can spend an approver's challenge.
2. Locate the matching pending step-up via `payload.challenge`. Unknown → `challengeUnknown`. Expired → `challengeExpired`.
3. Verify `payload.challenge` equals the bound challenge bit-for-bit (constant-time comparator).
4. Verify the gate **and** authorize the approver. The signer is the document `issuer` (the approver), which MAY differ from `payload.subject` (delegated step-up). A verified signature is necessary but never sufficient — the relying party MUST also confirm the signer is authorized to ratify for the subject:
   - **Echoes, for either gate.** Verify `payload.subject` equals the subject bound to the pending step-up located in step 2 — the session's subject, or for a bound step-up the subject named when it was minted. Verify `payload.sessionId` is present exactly when the pending step-up has a session, and equals it when present. Any mismatch → `subjectMismatch`.
   - **`didSigned`** — verify the framework `proof`, then bind both ends:
     - Verify `payload.subject` equals the subject bound to the pending step-up (above). Mismatch → `subjectMismatch`.
     - Verify the DID resolved from the proof's `verificationMethod` equals the document's `issuer` — the signature is by the *named* approver, not some third key. Mismatch → `subjectMismatch`.
     - **Authorize the approver.** Confirm `issuer` may ratify step-ups for `payload.subject` per the relying party's own state, according to the effective step-up mode (see [`auth/step-up/policy`](../../policy/0.2/spec.md)): either `issuer == subject` (**self**), or `issuer` is the approver the relying party bound to this step-up at approve-request time — the request's `recipient` (**delegated**; for the VTA, the subject's `AclEntry.stepUp.approver`), or — under **`delegatedAny`** — `issuer` satisfies the relying party's *approver criterion* for the subject (an implementation-defined set of authorized approvers, since `delegatedAny` binds no single `recipient`; for the VTA, an admin whose administered contexts cover the subject's, with a super-admin covering all). None of these → `approverUnauthorized`.
     - The signature is verified under the **`issuer`/approver** key — it is NOT assumed to be the subject's. A missing proof → `proofRequired`; an invalid one → `proofInvalid` (step 1a).
   - **`webauthn`** — in addition to the proof verified in step 1a, perform full WebAuthn Level 2 §7.2 assertion verification against the bound challenge: decode `clientDataJSON` (`type === "webauthn.get"`, challenge match, `origin` match); verify `rpIdHash` matches the consumer's RP ID; verify the signature with the stored credential public key; verify the signature counter strictly increased. Any failure → `assertionInvalid` with `details.reason`. Resolve `credential.id` (and `userHandle`) to the **approver**, then authorize that approver for the step-up's subject exactly as for `didSigned`: the approver is the subject itself (**self**), the bound delegated approver (**delegated**), or — under **`delegatedAny`** — any approver satisfying the relying party's approver criterion. A credential that resolves to no known principal → `assertion_invalid:userHandleMismatch`; a resolved approver authorized by none of those paths → `approverUnauthorized`. The approver the credential resolves to MUST be the approver the proof established in step 1a (`subjectMismatch` otherwise): the passkey and the signature are two proofs by one approver, not one proof each by two.
5. When `decision === "approved"` and the step-up is **not bound to an operation**:
   - Apply the session elevation per the consumer's policy: update `session.amr` to include the new factor (`"passkey"` for a webauthn gate, `"vta"`/`"did"` for a did-signed gate), raise `session.acr` to at most `payload.grantedAcr`.
   - If the session's `acr` cannot reach the originally-requested `targetAcr` → `acrUnsatisfied`.
   - Consume the step-up so the same approve-response cannot be replayed. For a webauthn gate, persist the credential counter update.
   - Acknowledge with `status: "elevated"` and the session snapshot.
5a. When `decision === "approved"` and the step-up **is bound to an operation**:
   - Apply the approval to that operation and **MUST NOT** elevate the session. Elevating would let the next gated operation in the assurance window ride an approval taken for a different one, which is the failure binding exists to prevent.
   - Read the bound operation from the consumer's **own state**, established when the step-up was minted. A relying party **MUST NOT** take it from any member of this document: an operation named by the producer would be a bearer instruction, and the approval would authorize whatever the holder of this document nominated.
   - `acrUnsatisfied` does not apply: no `acr` is being raised, so there is nothing for `targetAcr` to be unsatisfied by.
   - Consume the step-up exactly as above.
   - Acknowledge with `status: "recorded"`, `boundTo` naming the operation, and **no** `session`.
   - Applying an approval whose bound operation is no longer there — expired, or already completed — is **not** an error. The relying party changes nothing and still acknowledges `recorded`: the approval was validly given, and the operation it would have authorized has simply gone.
6. When `decision === "denied"`:
   - Verify the did-signed gate and authorize the approver exactly as in step 4 (a denial MUST be signed by the subject in self mode, or the authorized approver in delegated mode).
   - Consume the step-up.
   - Persist the denied response for audit. Take no further action on the session.

## Definitions

* **Bound approval.** An approval that authorizes one operation instead of raising an assurance level. The binding is the relying party's own state, established when the step-up was minted; the operation is named back to the approver in `boundTo`. Answered with `recorded`.
* **Approver.** The party that ratifies the step-up; identified by `issuer` and proven by the gate. In **self** step-up the approver is the subject (`issuer == subject`); in **delegated** step-up the approver is a distinct party the relying party authorized to ratify for the subject (`issuer != subject`) — see [`auth/step-up/policy`](../../policy/0.2/spec.md).
* **Relying party.** The party that initiated the step-up; identified by `recipient`.
* **Subject.** The VID whose session is being elevated. Equals the approver in self step-up; differs from it in delegated step-up.

## Payload

`payload.subject`, `payload.challenge`, `payload.decision` — REQUIRED.

`payload.sessionId` — echoed when the approve-request carried one; absent when it did not.

`payload.deniedReason` — required when decision is `denied`.

`payload.grantedAcr` — optional approver-declared AAL.

`payload.evidence` — optional tagged union selecting the gate: `{ "kind": "didSigned" }` (framework proof is the gate; the default when omitted) or `{ "kind": "webauthn", "assertion": <AuthenticatorAssertionResponse> }` (the assertion over `challenge` is the gate). New kinds may be added in later minor versions; consumers that do not recognise a `kind` MUST reject with `noGate` rather than silently elevate.

`payload.ext` — extension slot.

## Examples

### Approver approves the transfer

```json
{
  "id": "approve-resp-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5",
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

### Delegated approver ratifies another subject's session (did-signed)

An authorized approver (e.g. an administrator's phone holding a `did:key`) ratifies a step-up for a *different* subject. The relying party delegated approval to it via the subject's policy (`mode: delegated`; the approver is the subject's `AclEntry.stepUp.approver`) and addressed the approve-request to it. Here `issuer` is the **approver**, `payload.subject` is the session being elevated, and the gate is signed by the **approver's** key — so the relying party verifies the proof under the issuer key *and* confirms the issuer is the subject's bound approver before elevating.

```json
{
  "id": "approve-resp-1111-2222-3333-444455556666",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5",
  "issuer": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-23T14:00:30Z",
  "payload": {
    "subject": "did:web:carol.example",
    "sessionId": "9c2e1f7a-6b3d-4c8e-9a1b-2d3e4f5a6b7c",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "approved",
    "grantedAcr": "aal2"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2#z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
    "created": "2026-05-23T14:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Approver approves with a passkey on their phone (cross-device AAL2)

The phone received the approve-request via DIDComm (woken by a push notification), showed the `reason`, and the user confirmed with Face ID. The phone's platform passkey signed over the step-up `challenge`, and the phone signed the document with Alice's `assertionMethod` key. The relying party verifies both.

```json
{
  "id": "approve-resp-aaaa-bbbb-cccc-ddddeeeeffff",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5",
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

### Administrator approves one signed operation that has no session

The relying party refused a signed `acl/grant` document and carried an [`approve-request/0.3`](../../approve-request/0.3/spec.md) inline, bound to that grant and with no `sessionId`, because the grant arrived as a signed document rather than over a session. The administrator's agent asserted their passkey over the `challenge` and signed the document with the administrator's `assertionMethod` key; no `sessionId` is echoed, because none was sent. On `recorded` the agent re-sends the grant, which the relying party now executes, once.

```json
{
  "id": "approve-resp-bbbb-cccc-dddd-eeeeffff0000",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5",
  "issuer": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-05-23T14:00:20Z",
  "payload": {
    "subject": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
    "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
    "decision": "approved",
    "evidence": {
      "kind": "webauthn",
      "assertion": {
        "id": "Y3JlZF84ZjJjMWQ0ZQ",
        "rawId": "Y3JlZF84ZjJjMWQ0ZQ",
        "type": "public-key",
        "response": {
          "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiUjNKaGJuUkJaRzFwYms1dmJtTmxXRmxhTVRJek5EVTIifQ",
          "authenticatorData": "Q29tbXVuaXR5QXV0aERhdGE",
          "signature": "U2lnbmF0dXJlT3ZlckdyYW50",
          "userHandle": "YWRtaW5fOGYyYzFkNGU"
        }
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2#z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
    "created": "2026-05-23T14:00:20Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

It is acknowledged `recorded`, naming the operation and carrying no session:

```json
{
  "id": "approve-ack-cccc-dddd-eeee-ffff00001111",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5#response",
  "threadId": "approve-resp-bbbb-cccc-dddd-eeeeffff0000",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
  "issuedAt": "2026-05-23T14:00:21Z",
  "payload": {
    "status": "recorded",
    "boundTo": "uEiD3n4bE1QbQ8nY2l0cGhYc2FsdGVkZGlnZXN0"
  }
}
```

### Approver denies

```json
{
  "id": "approve-resp-8901-2345-6789-0abcdef12345",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5",
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

The relying party's `#response` confirms whether elevation succeeded.

### Successful elevation

```json
{
  "id": "approve-ack-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5#response",
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

### Elevation rejected

```json
{
  "id": "approve-ack-0123-4567-8901-bcdef1234567",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5#response",
  "threadId": "approve-resp-7890-1234-5678-90abcdef1234",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:31Z",
  "payload": {
    "status": "rejected",
    "reason": "challenge expired"
  }
}
```

## Security & Privacy

**A bound approval is not a smaller elevation.** It is a different thing, and a relying party that treated `recorded` as a weak `elevated` — or a consumer that read the absent `session` as "unchanged, therefore fine" — would reintroduce exactly what the binding prevents. The two outcomes are disjoint: an approval either raises an assurance level or authorizes one operation, never both. A relying party MUST NOT emit `recorded` alongside a session change, and a producer receiving `recorded` MUST NOT assume any subsequent request is more privileged than it was before.

**The proof always, and the passkey in addition.** The relying party MUST NOT take any field in this document as authoritative without the approver's verified `assertionMethod` proof, and for a `webauthn` gate without the verified assertion as well. A bearer-token-style step-up is not safe — the threat model includes a token-stealing attacker who would happily issue their own approve-response. Nor is an assertion on its own: it signs the challenge, not the document, so an intermediary holding one could wrap it in a document of its own and choose the `decision`, `grantedAcr` and `subject` it carries. A document without a proof MUST be refused (`proofRequired`), and one whose `evidence.kind` the consumer does not recognise MUST be refused (`noGate`), never elevated.

**Why `assertionMethod`.** An approve-response is an attestation, a statement by the approver that it decided, not an operational message authenticating a session. The proof is therefore made for `assertionMethod`, and the key MUST be listed under the approver's `assertionMethod` relationship: a key the approver lists only for `authentication`, `keyAgreement` or capability relationships does not speak for its decisions.

**Delegated approver authorization.** In delegated step-up the gate proves the *approver* signed — not the subject. A verified signature is therefore necessary but not sufficient: the relying party MUST independently confirm the `issuer` is an approver it authorized for `payload.subject`. That authority is established from the relying party's own state — for **`delegated`**, the binding made at approve-request time (the request's `recipient`; for the VTA, the subject's `AclEntry.stepUp.approver`); for **`delegatedAny`**, membership in the relying party's approver criterion (no single `recipient` is bound — for the VTA, an admin whose contexts cover the subject's). Either way the authority MUST be read from the relying party's own state, never taken from the document. Without this check, any party that can obtain a step-up challenge could sign its own approve-response and elevate another subject's session — the delegated analogue of the bearer-token attack. An `issuer` authorized by none of self/delegated/delegated-any MUST be rejected with `approverUnauthorized`. A `delegatedAny` criterion MUST remain a bounded, least-privilege set (e.g. context-scoped admins), never "any holder", or it degrades to self-approval. The relying party also applies its normal liveness/assurance policy to the approver (e.g. the approver itself being at a sufficient AAL) before honouring `grantedAcr`.

**WebAuthn challenge binding.** For a webauthn gate the assertion's `clientDataJSON` challenge MUST equal `payload.challenge` — the same nonce the relying party bound server-side at approve-request time. This is what stops an attacker from harvesting a passkey assertion gathered for one ceremony and replaying it into a step-up. The relying party verifies the binding before consulting `subject`. The WebAuthn `rpId`/`origin` checks bind the gesture to the relying party; the framework `proof` and `recipient` bind the document. Both are required.

**Gate ↔ amr consistency.** The factor recorded in `session.amr` MUST match the gate actually verified: `"passkey"` only when a WebAuthn assertion verified, `"vta"`/`"did"` only when a subject DID signature verified. A relying party MUST NOT record `"passkey"` on the strength of a `grantedAcr: "aal2"` hint alone — `grantedAcr` is an approver claim, not evidence.

**Echo verification.** Every echo field present (`subject`, `challenge`, and `sessionId` when the step-up has a session) MUST be compared bit-for-bit, and a `sessionId` MUST be refused when the step-up has none — accepting one there would let a bound approval be read as naming a session to elevate. An attacker who can re-target a captured approve-response to a different session (by mutating `sessionId`) MUST be defeated by the proof plus the approver-authorization check — but defense-in-depth: comparing all three fields blocks attacks against weak proof implementations. Note that in delegated step-up the echoed `subject` is *not* the signer, so the `subject ↔ approver` authorization binding (above), not the proof's key identity, is what ties the signed response to the right session.

**Replay.** Consuming the challenge on success-or-denial is mandatory. A second approve-response carrying the same challenge MUST fail with `challengeUnknown`.

**Denied responses as audit.** A signed `denied` response is valuable evidence — it proves the user actively refused, not that they were absent. Relying parties SHOULD preserve denied responses with the same retention policy as approvals.

**Wallet UX.** Approvers presenting approve-requests to humans MUST display the request's `reason` and the relying party identity verbatim. Substituting a friendlier summary for an unclear reason is a phishing vector.

The optional `ext` extension is part of the producer's signed surface.

**Free text.** `deniedReason` is free text, bounded at 500 characters — the
consent-surface figure, because this is prose a human wrote at an approval
prompt and a second human reads at the other end. It is OPTIONAL in the schema
and REQUIRED by the prose only when `decision` is `denied`. It is authored by
the approver (or inferred by their device) rather than by the service that acted
on the denial, so a surface rendering it MUST attribute it to the approver and
MUST NOT present it as the service's own account of the refusal. It is read by
the requesting party and by whoever reviews the step-up audit trail; a consumer
that keeps a record of the denial **retains** it with that record, and one that
does not discards it when the step-up closes. An approver SHOULD say which
condition applied rather than describe their own circumstances — a reason
travels further than the person who wrote it expects.


### Data carried

The request carries the approver's decision, the echoed `subject` and
`challenge` (and `sessionId` when the step-up has a session), an optional
free-text `deniedReason`, and the gate. It carries no
attribute of the subject beyond their identifier, and — importantly for the
bound case — **no description of the operation being approved**. What the
approver was shown travelled in the approve-request; what is signed here is the
decision.

The response carries a status, and either a session snapshot (`elevated`) or an
opaque operation identifier (`recorded`). `boundTo` is deliberately opaque: it
exists so an approver's audit trail can say *which* approval this was, and it
confers nothing, because the approval it names is the relying party's own state
and cannot be spent by presenting the value.

### Correlation

The `challenge` is single-use and unpredictable, so two approve-responses cannot
be linked by it. The `subject` and `sessionId` identify the session being acted
on to anyone who sees the document, which is inherent — the relying party must
locate the session. In delegated step-up the document links an approver to a
subject, which is a relationship the relying party already holds in its own
authorization state and learns nothing new from.

`boundTo` links an approval to one operation for whoever holds both. Where that
operation is a disclosure, the identifier is a short-lived, single-use preview
token that expires with the decision it belongs to, so it is not a durable
correlator.

### Retention

A relying party MUST consume the pending step-up on success or denial, so the
authorization state is short-lived by construction. Signed responses —
approvals and denials alike — SHOULD be retained under the same policy, because
a denial is the evidence that a human actively refused rather than was absent.

A `recorded` acknowledgement SHOULD be retained with the operation it names, so
that a holder reviewing what left their agent can see the approval that
authorized it beside the disclosure itself.

### Consent/purpose

This document *is* a consent artifact: it exists to carry one human's decision
about one specific act, and the gate is what makes that decision attributable.
Its purpose is exhausted by the operation it authorizes.

That is the whole reason for the bound form. An approval taken for one act and
spent as a general elevation is the same decision being reused for a purpose the
human never saw — which is what "each time" was asked for and what binding to
the operation, rather than to the session, delivers.
