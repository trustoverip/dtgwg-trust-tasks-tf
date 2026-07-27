---
slug: auth/step-up/approve-response
version: "0.2"
title: Auth — Step-up Approve Response
summary: An approver's signed answer to a step-up approve-request — the proof on this document is the cryptographic gate the relying party uses to elevate the subject's session.
status: draft
targetFrameworkVersion: "0.2"
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
  requirement: RECOMMENDED
  rationale: Exactly one cryptographic gate MUST back the elevation. For `evidence.kind = did-signed` (the default when `evidence` is absent) the gate IS the framework proof — a signature from the approver's authoritative key (the subject's own key in self step-up, the delegated approver's key in delegated step-up) — so proof is mandatory in that case. For `evidence.kind = webauthn` the gate is the carried WebAuthn assertion over the challenge, and the framework proof MAY be omitted; WebAuthn supplies its own audience binding via rpId/origin. The requirement is therefore RECOMMENDED at the spec level and made conditional in the conformance rules below.
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
    meaning: The echoed `payload.subject` does not equal the session's subject, or the proof's verificationMethod DID does not equal the document's issuer (the signer is not the named approver).
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
    meaning: The document carried neither a verifiable framework proof (did-signed) nor a `webauthn` evidence assertion. There is no cryptographic basis to elevate.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Response** Trust Task is the ratification of an earlier [`auth/step-up/approve-request/0.1`](../../approve-request/0.1/spec.md). The approver echoes the request's `subject`, `sessionId`, and `challenge`, sets `decision` to `approved` or `denied`, and backs the decision with **one of two cryptographic gates**, selected by the optional `payload.evidence` tagged union:

- **`evidence.kind = did-signed`** (the default when `evidence` is absent) — the framework `proof` IS the gate: a Data Integrity signature from a key the subject controls. This is the original, transport-agnostic step-up. Resulting `amr` reflects `"did"`/`"vta"`.
- **`evidence.kind = webauthn`** — the approver carries an `AuthenticatorAssertionResponse` produced by a platform passkey over the step-up `challenge` (the cross-device path: a relying party at AAL 1 in a browser, the user elevating with Face ID / Touch ID / Android biometric on their phone). The assertion is the gate; the relying party verifies it per WebAuthn Level 2 §7.2 exactly as [`auth/passkey/login/finish/0.1`](../../../passkey/login/finish/0.1/spec.md) does. Resulting `amr` reflects `"passkey"`.

Supporting both lets one step-up flow serve a DID-key approver (a VTA ratifying programmatically) and a biometric-bound passkey approver (a phone) without two separate protocols. A relying party advertises which gates it will accept via the request's `acceptableEvidence` ([`approve-request`](../../approve-request/0.1/spec.md)).

**Who signs — self vs delegated.** The document `issuer` is the **approver**, which need not be the subject. In **self** step-up the subject ratifies its own session (`issuer == subject`) — e.g. a wallet holding the subject's key. In **delegated** step-up a distinct, pre-authorized approver ratifies on the subject's behalf (`issuer != subject`) — e.g. an administrator's phone, or a Verifiable-Trust Agent acting under policy ([`auth/step-up/policy`](../../policy/0.2/spec.md) `mode: delegated`, where the approver is the subject's `AclEntry.stepUp.approver`). Either way the gate proves the **approver** signed; the relying party **separately** verifies that approver is authorized to ratify for the subject (see Conformance and Security). This matches the request side, which already addresses the approve-request to that approver as its `recipient`.

A relying party processing an `approved` response elevates the session's `amr`/`acr` per its own policy and replies with the elevated session snapshot. A `denied` response is signed too (did-signed) — it serves as audit evidence that the user explicitly refused.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-response/0.2`, with itself (the **approver**) as `issuer` — the subject in self step-up, or a distinct authorized approver in delegated step-up — and the relying party as `recipient`.
2. Echo `payload.subject`, `payload.sessionId`, and `payload.challenge` verbatim from the matching approve-request.
3. Set `payload.decision` to `approved` or `denied`. When `denied`, populate `payload.deniedReason`.
4. **MAY** declare `payload.grantedAcr` to convey which AAL the approver believes it demonstrated. Relying parties MAY upgrade the session to ≤ this value; MUST NOT exceed it.
5. Provide exactly one cryptographic gate for the elevation:
   - For `evidence.kind = did-signed` (or when `evidence` is omitted): include a framework `proof` whose `verificationMethod` resolves to a key the **approver** controls (the subject's own key in self step-up; the delegated approver's key in delegated step-up) and whose DID equals the document `issuer`. The `proof.proofPurpose` MUST be `assertionMethod`.
   - For `evidence.kind = webauthn`: populate `payload.evidence.assertion` with the unmodified `AuthenticatorAssertionResponse`; binary fields base64url-encoded. The assertion's `clientDataJSON` `challenge` MUST equal `payload.challenge`. A framework `proof` MAY additionally be included (to bind the approver's own identity) but is not the gate.
6. A `denied` decision MUST use `evidence.kind = did-signed` — a refusal is an approver-signed statement (the subject in self mode, the authorized approver in delegated mode), not a possession proof.

A conforming **consumer** (the relying party) **MUST**:

1. Validate the document. Determine the gate from `payload.evidence.kind` (treating an absent `evidence` as `didSigned`).
2. Locate the matching pending step-up via `payload.challenge`. Unknown → `challengeUnknown`. Expired → `challengeExpired`.
3. Verify `payload.challenge` equals the bound challenge bit-for-bit (constant-time comparator).
4. Verify the gate **and** authorize the approver. The signer is the document `issuer` (the approver), which MAY differ from `payload.subject` (delegated step-up). A verified signature is necessary but never sufficient — the relying party MUST also confirm the signer is authorized to ratify for the subject:
   - **`didSigned`** — verify the framework `proof`, then bind both ends:
     - Verify `payload.subject` equals the session's subject (the session located in step 2). Mismatch → `subjectMismatch`.
     - Verify the DID resolved from the proof's `verificationMethod` equals the document's `issuer` — the signature is by the *named* approver, not some third key. Mismatch → `subjectMismatch`.
     - **Authorize the approver.** Confirm `issuer` may ratify step-ups for `payload.subject` per the relying party's own state, according to the effective step-up mode (see [`auth/step-up/policy`](../../policy/0.2/spec.md)): either `issuer == subject` (**self**), or `issuer` is the approver the relying party bound to this step-up at approve-request time — the request's `recipient` (**delegated**; for the VTA, the subject's `AclEntry.stepUp.approver`), or — under **`delegatedAny`** — `issuer` satisfies the relying party's *approver criterion* for the subject (an implementation-defined set of authorized approvers, since `delegatedAny` binds no single `recipient`; for the VTA, an admin whose administered contexts cover the subject's, with a super-admin covering all). None of these → `approverUnauthorized`.
     - The signature is verified under the **`issuer`/approver** key — it is NOT assumed to be the subject's. Missing/invalid proof → `proofInvalid` (or `noGate` if no gate of either kind is present).
   - **`webauthn`** — perform full WebAuthn Level 2 §7.2 assertion verification against the bound challenge: decode `clientDataJSON` (`type === "webauthn.get"`, challenge match, `origin` match); verify `rpIdHash` matches the consumer's RP ID; verify the signature with the stored credential public key; verify the signature counter strictly increased. Any failure → `assertionInvalid` with `details.reason`. Resolve `credential.id` (and `userHandle`) to the **approver**, then authorize that approver for the session's subject exactly as for `didSigned`: the approver is the subject itself (**self**), the bound delegated approver (**delegated**), or — under **`delegatedAny`** — any approver satisfying the relying party's approver criterion. A credential that resolves to no known principal → `assertion_invalid:userHandleMismatch`; a resolved approver authorized by none of those paths → `approverUnauthorized`.
5. When `decision === "approved"`:
   - Apply the session elevation per the consumer's policy: update `session.amr` to include the new factor (`"passkey"` for a webauthn gate, `"vta"`/`"did"` for a did-signed gate), raise `session.acr` to at most `payload.grantedAcr`.
   - If the session's `acr` cannot reach the originally-requested `targetAcr` → `acrUnsatisfied`.
   - Consume the step-up so the same approve-response cannot be replayed. For a webauthn gate, persist the credential counter update.
6. When `decision === "denied"`:
   - Verify the did-signed gate and authorize the approver exactly as in step 4 (a denial MUST be signed by the subject in self mode, or the authorized approver in delegated mode).
   - Consume the step-up.
   - Persist the denied response for audit. Take no further action on the session.

## Definitions

* **Approver.** The party that ratifies the step-up; identified by `issuer` and proven by the gate. In **self** step-up the approver is the subject (`issuer == subject`); in **delegated** step-up the approver is a distinct party the relying party authorized to ratify for the subject (`issuer != subject`) — see [`auth/step-up/policy`](../../policy/0.2/spec.md).
* **Relying party.** The party that initiated the step-up; identified by `recipient`.
* **Subject.** The VID whose session is being elevated. Equals the approver in self step-up; differs from it in delegated step-up.

## Payload

`payload.subject`, `payload.sessionId`, `payload.challenge`, `payload.decision` — REQUIRED.

`payload.deniedReason` — required when decision is `denied`.

`payload.grantedAcr` — optional approver-declared AAL.

`payload.evidence` — optional tagged union selecting the gate: `{ "kind": "didSigned" }` (framework proof is the gate; the default when omitted) or `{ "kind": "webauthn", "assertion": <AuthenticatorAssertionResponse> }` (the assertion over `challenge` is the gate). New kinds may be added in later minor versions; consumers that do not recognise a `kind` MUST reject with `noGate` rather than silently elevate.

`payload.ext` — extension slot.

## Examples

### Approver approves the transfer

```json
{
  "id": "approve-resp-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.2",
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
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.2",
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

The phone received the approve-request via DIDComm (woken by a push notification), showed the `reason`, and the user confirmed with Face ID. The phone's platform passkey signed over the step-up `challenge`; that assertion is the gate. No framework `proof` is required — though one MAY be added to bind the phone's own DID.

```json
{
  "id": "approve-resp-aaaa-bbbb-cccc-ddddeeeeffff",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.2",
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
  }
}
```

### Approver denies

```json
{
  "id": "approve-resp-8901-2345-6789-0abcdef12345",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.2",
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
  "proof": { "…": "…" }
}
```

## Response

The relying party's `#response` confirms whether elevation succeeded.

### Successful elevation

```json
{
  "id": "approve-ack-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.2#response",
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
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.2#response",
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

**Exactly one gate, never zero.** The relying party MUST NOT take any field in this document as authoritative without a verified gate — either a framework `proof` (did-signed) or a verified WebAuthn assertion (webauthn). A bearer-token-style step-up is not safe — the threat model includes a token-stealing attacker who would happily issue their own approve-response. A document presenting neither gate, or an `evidence.kind` the consumer does not recognise, MUST be rejected (`noGate`), never elevated.

**Delegated approver authorization.** In delegated step-up the gate proves the *approver* signed — not the subject. A verified signature is therefore necessary but not sufficient: the relying party MUST independently confirm the `issuer` is an approver it authorized for `payload.subject`. That authority is established from the relying party's own state — for **`delegated`**, the binding made at approve-request time (the request's `recipient`; for the VTA, the subject's `AclEntry.stepUp.approver`); for **`delegatedAny`**, membership in the relying party's approver criterion (no single `recipient` is bound — for the VTA, an admin whose contexts cover the subject's). Either way the authority MUST be read from the relying party's own state, never taken from the document. Without this check, any party that can obtain a step-up challenge could sign its own approve-response and elevate another subject's session — the delegated analogue of the bearer-token attack. An `issuer` authorized by none of self/delegated/delegated-any MUST be rejected with `approverUnauthorized`. A `delegatedAny` criterion MUST remain a bounded, least-privilege set (e.g. context-scoped admins), never "any holder", or it degrades to self-approval. The relying party also applies its normal liveness/assurance policy to the approver (e.g. the approver itself being at a sufficient AAL) before honouring `grantedAcr`.

**WebAuthn challenge binding.** For a webauthn gate the assertion's `clientDataJSON` challenge MUST equal `payload.challenge` — the same nonce the relying party bound server-side at approve-request time. This is what stops an attacker from harvesting a passkey assertion gathered for one ceremony and replaying it into a step-up. The relying party verifies the binding before consulting `subject`; the WebAuthn `rpId`/`origin` checks supply the audience binding that the framework `recipient` would otherwise provide, which is why the framework `proof` MAY be omitted for this kind.

**Gate ↔ amr consistency.** The factor recorded in `session.amr` MUST match the gate actually verified: `"passkey"` only when a WebAuthn assertion verified, `"vta"`/`"did"` only when a subject DID signature verified. A relying party MUST NOT record `"passkey"` on the strength of a `grantedAcr: "aal2"` hint alone — `grantedAcr` is an approver claim, not evidence.

**Echo verification.** All three echo fields (`subject`, `sessionId`, `challenge`) MUST be compared bit-for-bit. An attacker who can re-target a captured approve-response to a different session (by mutating `sessionId`) MUST be defeated by the proof plus the approver-authorization check — but defense-in-depth: comparing all three fields blocks attacks against weak proof implementations. Note that in delegated step-up the echoed `subject` is *not* the signer, so the `subject ↔ approver` authorization binding (above), not the proof's key identity, is what ties the signed response to the right session.

**Replay.** Consuming the challenge on success-or-denial is mandatory. A second approve-response carrying the same challenge MUST fail with `challengeUnknown`.

**Denied responses as audit.** A signed `denied` response is valuable evidence — it proves the user actively refused, not that they were absent. Relying parties SHOULD preserve denied responses with the same retention policy as approvals.

**Wallet UX.** Approvers presenting approve-requests to humans MUST display the request's `reason` and the relying party identity verbatim. Substituting a friendlier summary for an unclear reason is a phishing vector.

The optional `ext` extension is part of the producer's signed surface.
