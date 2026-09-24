---
slug: auth/step-up/approve-request
version: "0.3"
title: Auth — Step-up Approve Request
summary: A relying party asks a wallet or verifiable-trust agent to ratify an authentication step-up — issuing a challenge that the approver will sign in the follow-up approve-response.
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
  - role: Relying party
    requirement: REQUIRED
    member: issuer
  - role: Approver
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The reason text is shown to the user as the basis of their consent decision. A proof binds the request to the relying party so a downstream attacker cannot intercept the channel and substitute a different reason.
sideEffects:
  level: none
  rationale: "Requests approval for a step-up and issues a challenge; the decision is a separate task."
subjectPath: /subject
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/step-up/approve-request:subjectUnknown
    meaning: The approver does not speak for the named subject.
    retryable: false
  - code: auth/step-up/approve-request:methodUnsupported
    meaning: The approver cannot deliver an approve-response (e.g. the wallet has no key for the subject, or doesn't support the requested AAL).
    retryable: false
  - code: auth/step-up/approve-request:userDeclined
    meaning: The user reviewed the request and declined consent.
    retryable: false
  - code: auth/step-up/approve-request:rateLimited
    meaning: The relying party has exceeded the approver's request budget.
    retryable: true
related:
  - auth/step-up/approve-response
  - auth/step-up/policy
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Request** Trust Task is the first half of an out-of-band step-up flow. The relying party (typically an auth service holding a subject's session at AAL 1) sends this document to an *approver* — usually the subject's wallet or a Verifiable-Trust Agent acting for the subject — asking the approver to ratify an AAL elevation.

The approver SHOULD show the `reason` to a human and obtain consent. If consent is granted, the approver returns an [`auth/step-up/approve-response/0.4`](../../approve-response/0.4/spec.md) signed by the **approver's** key — the subject's own key when the approver *is* the subject (self step-up), or the delegated approver's key when a distinct authorized party ratifies on the subject's behalf (delegated step-up; the approver is this request's `recipient`). That signed response is the cryptographic gate the relying party uses to elevate the session.

This pair (`approve-request` + `approve-response`) is the canonical "trust task instead of a side channel" pattern. The same shape can support transaction-confirmation flows, high-value-operation gates, and admin-takes-over-from-subject ceremonies.

**0.3 lets a step-up be bound to one operation that has no session.** [`approve-response/0.3`](../../approve-response/0.3/spec.md) introduced the *bound approval* — one that authorizes a single operation and elevates nothing — but this request still required a `sessionId`, so the only operations it could bind were ones that arrived over a session. An operation that arrives as a signed Trust Task document has none: its authority is its proof, and binding the human's gesture to some session of theirs after the fact cannot say which, while a window opened on any of them is one a process holding the subject's signing key can spend on acts the subject never saw. In 0.3 `sessionId` is omitted for such a step-up, and `boundTo` names the operation. The same version defines how the request travels when there is no channel to push it on: inline, in the refusal of the operation it is for (see [Inline delivery](#inline-delivery)).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the relying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-request/0.3`, with itself as `issuer` and the approver as `recipient`.
2. Populate `payload.subject` with the VID whose session is being elevated, or — for a step-up bound to one operation — the VID whose act that operation is.
3. Populate `payload.sessionId` with the session id (opaque to the approver but echoed back in the approve-response so the relying party can correlate) — **unless** the step-up is bound to an operation that arrived without a session, in which case `sessionId` **MUST** be omitted. A relying party **MUST NOT** populate `sessionId` with a session it chose after the fact to stand in for one the operation did not have.
4. Generate `payload.challenge` with ≥128 bits of entropy and bind it server-side to `(subject, sessionId, expiresAt)` — or, for a bound step-up, to `(subject, the bound operation, expiresAt)`.
4a. For a step-up bound to one operation, populate `payload.boundTo` with an opaque identifier for that operation, and bind the operation in its own state. Where the operation's content could be recognized from an unsalted identifier (a digest over a short, predictable payload is a confirmation oracle), the identifier **MUST** be salted — for an operation that arrived as a signed document, a digest of its type and payload salted with `payload.challenge`. `boundTo` **MUST** be absent for a step-up that elevates a session.
5. Populate `payload.reason` with a user-meaningful explanation. The approver MAY refuse with `userDeclined` if the reason is empty or generic.
6. **MAY** declare `payload.targetAcr` — the AAL the relying party expects on completion.
7. **MAY** declare `payload.acceptableEvidence` to constrain which approve-response gates it will accept (`didSigned`, `webauthn`, or both). When the relying party wants a passkey-backed elevation (`webauthn`), it **SHOULD** also supply `payload.webauthn` — the `PublicKeyCredentialRequestOptions` the approver feeds to the platform passkey API — whose `challenge` **MUST** equal `payload.challenge`.
8. Include a verified `proof` so the approver can rely on the request's `recipient` as authoritative.

A conforming **consumer** (the approver) **MUST**:

1. Verify the document's `proof`, and verify the DID resolved from the proof's `verificationMethod` equals the document's `issuer` — the signature is by the *named* relying party, not some third key. Mismatch → the framework `proofInvalid` error, per the binding rule of [SPEC.md §4.7](/SPEC.md#47-proof) (this task mints no task-specific code for it).
2. Determine whether it speaks for `payload.subject`. If not → `subjectUnknown`.
3. Decide whether to surface the request to the user (subject of consent) or to ratify it programmatically (policy-bound delegation). The framework leaves this to the approver — but if a human is presented with the request, the `reason` MUST be shown verbatim.
4. Honor `payload.acceptableEvidence` when present: the approve-response it later returns MUST carry an `evidence.kind` in that list. If the approver cannot satisfy any listed kind (e.g. `webauthn` was demanded but the device has no passkey for the subject) → `methodUnsupported`. When `payload.webauthn` is present and the approver will produce `webauthn` evidence, it MUST pass those options to the platform passkey API unchanged and assert over `payload.challenge`.
5. Return a `#response` document carrying `status: accepted` (will return an approve-response asynchronously) or `status: refused` (with a `reason`).

> **Note (non-normative).** The reference ecosystem signs this document with the `eddsa-jcs-2022` Data Integrity cryptosuite and `proofPurpose: assertionMethod`, as the examples show. This is an implementation profile, not a requirement of this specification: [SPEC.md §4.7](/SPEC.md#47-proof) leaves the choice of cryptosuite open, and any registered suite whose `verificationMethod` resolves to material controlled by the `issuer` satisfies the `proof` requirement.

The approve-response document arrives out-of-band — typically via the approver's preferred transport (DIDComm push to the relying party's mediator, or a push channel the relying party registered at request time).

### Inline delivery

A relying party that refuses an operation because it needs a step-up bound to that operation, and has no push channel to the approver, **MAY** carry this task's payload in the refusal instead of sending it as a document. It does so in an *error response* ([SPEC.md §8](/SPEC.md#8-error-responses)) answering the operation, with `details.stepUpRequest` set to a payload valid against this specification's schema, with `boundTo` present and `sessionId` absent.

The producer of the refused operation is then the approver's agent: it surfaces the request to its human, obtains the gesture over `challenge`, returns an [`approve-response/0.4`](../../approve-response/0.4/spec.md) to the relying party, and on a `recorded` acknowledgement re-sends the same operation.

- An inline request carries no `proof` of its own. It is authenticated by the channel it arrived on: it is the relying party's own reply to a request the producer addressed to that relying party. A producer **MUST NOT** surface a `stepUpRequest` it received any other way, and **MUST** check that its `subject` is the party the producer acts for.
- The `reason` in an inline request is still what the human consents to, and the rules in [Security & Privacy](#security--privacy) apply to it unchanged.
- Inline delivery is only for bound step-ups. A session elevation requested inline would let any refusal on any channel raise a session, which is what this document's proof requirement exists to prevent.

## Definitions

* **Relying party.** The party requesting the elevation; identified by `issuer` and verified via `proof`.
* **Approver.** The party authoritative for `payload.subject`; identified by `recipient`. Wallets and VTAs are typical.
* **Subject.** The VID whose session is being elevated.
* **Session.** The session the relying party holds for the subject; the approver does not need to know its contents — only the opaque `sessionId`.
* **Bound step-up.** A step-up that authorizes exactly one operation and elevates no session; the operation is the relying party's own state and is named to the approver by `boundTo`.

## Payload

`payload.subject`, `payload.challenge`, `payload.reason` — REQUIRED.

`payload.sessionId` — REQUIRED for a step-up that elevates a session; absent for a bound step-up whose operation arrived without one.

`payload.boundTo` — present exactly when the step-up is bound to one operation.

`payload.targetAcr`, `payload.ttl` — optional hints.

`payload.acceptableEvidence` — optional list of accepted approve-response gates (`didSigned` / `webauthn`).

`payload.webauthn` — optional `PublicKeyCredentialRequestOptions` for driving a passkey-backed (`webauthn`) elevation; its `challenge` MUST equal `payload.challenge`.

`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Examples

### Relying party asks the user's wallet to confirm a transfer

```json
{
  "id": "step-up-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.3",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "reason": "Confirm transfer of $1,000 to did:web:bob.example",
    "targetAcr": "aal2",
    "ttl": 120
  },
  "proof": { "…": "…" }
}
```

### Relying party requires a passkey-backed elevation on the user's phone

The relying party holds a browser session at AAL 1 and will only accept a `webauthn` gate. It supplies the WebAuthn request options; the approver (the phone) prompts Face ID, asserts the passkey over the shared `challenge`, and returns it inside the approve-response `evidence`.

```json
{
  "id": "step-up-2345-6789-01bc-def123456789",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.3",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "reason": "Confirm transfer of $1,000 to did:web:bob.example",
    "targetAcr": "aal2",
    "acceptableEvidence": ["webauthn"],
    "webauthn": {
      "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
      "rpId": "bank.example",
      "userVerification": "required",
      "allowCredentials": [
        { "type": "public-key", "id": "Y3JlZF8xYTJiM2M" }
      ]
    },
    "ttl": 120
  },
  "proof": { "…": "…" }
}
```

### Relying party refuses a signed operation and asks for a step-up bound to it, inline

An administrator's agent sent a signed `acl/grant` document conferring an administrative role. The relying party's policy asks for a passkey gesture bound to that one grant, so it refuses the document and carries the request in the refusal. There is no session: the operation's authority was its proof. The producer surfaces the `reason`, asserts the passkey over `challenge`, returns an approve-response, and then re-sends the same `acl/grant` document.

```json
{
  "id": "err-5678-9012-3456-7890abcdef12",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "grant-0123-4567-89ab-cdef01234567",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
  "issuedAt": "2026-05-23T14:00:00Z",
  "payload": {
    "code": "permissionDenied",
    "message": "a passkey gesture bound to this grant is required",
    "retryable": false,
    "details": {
      "stepUpRequest": {
        "subject": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
        "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
        "boundTo": "uEiD3n4bE1QbQ8nY2l0cGhYc2FsdGVkZGlnZXN0",
        "reason": "Grant the administrator role to did:key:z6MkhaXg…",
        "acceptableEvidence": ["webauthn"],
        "webauthn": {
          "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
          "rpId": "community.example",
          "userVerification": "required",
          "allowCredentials": [
            { "type": "public-key", "id": "Y3JlZF84ZjJjMWQ0ZQ" }
          ]
        },
        "ttl": 300
      }
    }
  }
}
```

## Response

The `#response` is a synchronous acknowledgement that the approver received the request — NOT the approval itself. The approve-response (signed by the subject) follows out-of-band.

### Approver accepts

```json
{
  "id": "step-up-resp-3456-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.3#response",
  "threadId": "step-up-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:01Z",
  "payload": {
    "status": "accepted"
  }
}
```

### Approver refuses

```json
{
  "id": "step-up-resp-4567-8901-2345-67890abcdef0",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.3#response",
  "threadId": "step-up-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:01Z",
  "payload": {
    "status": "refused",
    "reason": "User declined"
  }
}
```

## Security & Privacy

**Reason integrity.** The `reason` is shown to the user as the basis of their consent. An attacker who can substitute the reason while leaving the rest of the request intact can elicit consent for an action the user did not intend. The framework `proof` binds the request — including the reason — to the relying party's key; consumers MUST verify the proof BEFORE surfacing the reason.

**Challenge entropy.** The approve-response signs over the challenge. ≥128 bits is the minimum; ≥192 bits is RECOMMENDED for high-value flows.

**Out-of-band binding.** The approve-response arrives over a channel the approver chose, not the request channel. Consumers correlating request↔response use the document's `threadId` (which equals the request id) AND verify the embedded `challenge` matches what they sent.

**TTL semantics.** `payload.ttl` is an advisory cap from the relying party. The relying party's server-side state is authoritative: when the relying party's expiry fires, any later approve-response is rejected regardless of the approver's view.

**Privacy of reason.** The reason may carry sensitive information (transfer amounts, account details, beneficiary identities). Consumers MUST require transport confidentiality.

**A bound step-up elevates nothing.** Its approval authorizes the one operation `boundTo` names and is consumed by it. A relying party **MUST NOT** treat the approval as raising any session, and **MUST** read the bound operation from its own state when the approval arrives — never from the approve-response, whose producer could otherwise nominate a different act.

**`boundTo` is not a capability.** It identifies the operation for the approver's audit trail. Presenting it authorizes nothing, because the binding is held by the relying party.

**Salting.** An operation identifier derived from the operation's content, unsalted, lets anyone who sees it test guesses at that content — for an administrative grant the space of plausible payloads is small. The salt is the per-request `challenge`, so the identifier is unlinkable across requests even for identical operations.

### Data carried

The subject's identifier, a single-use challenge, the relying party's `reason`, optional assurance and evidence hints, WebAuthn request options (which name the credential ids the relying party holds for the subject), and — for a session elevation — an opaque session id, or — for a bound step-up — an opaque, salted operation identifier. It carries no attribute of the subject beyond these. The `reason` is the one member that can carry sensitive content, and a relying party SHOULD say what is being approved without restating more of it than the human needs to decide.

### Correlation

The `challenge` and a salted `boundTo` are single-use and unpredictable, so neither links two requests. `allowCredentials` names the subject's credential ids, which do link requests to the same subject — inherent, since the approver must know which credential to use — and SHOULD be limited to the credentials the relying party would accept.

### Retention

A relying party holds the pending step-up — challenge, subject, and the session or bound operation — until the approve-response consumes it or it expires, and SHOULD discard it then. An inline request exists only in the refusal that carried it.

### Consent/purpose

This task asks a human for a decision about one elevation or one operation; the `reason` is the purpose they are consenting to, and it is exhausted by the approve-response that answers it. What authority the relying party requires before asking is its own policy, which this specification does not state.

The optional `ext` extension is part of the signed surface.
