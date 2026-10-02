---
slug: auth/step-up/approve-request
version: "0.4"
title: Auth — Step-up Approve Request
summary: A relying party asks a wallet or verifiable-trust agent to ratify an authentication step-up — issuing a challenge, and stating which kinds of evidence it will accept, including a statement by one of the subject's bound step-up approvers.
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
  - role: Relying party
    requirement: REQUIRED
    member: issuer
  - role: Approver
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The reason text is shown to the user as the basis of their consent decision. A proof binds the request to the relying party so a downstream attacker cannot intercept the channel and substitute a different reason, a different set of accepted evidence kinds, or a different list of approvers to ask.
sideEffects:
  level: none
  rationale: "Requests approval for a step-up and issues a challenge; the decision is a separate task."
subjectPath: /subject
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names the subject, the evidence kinds the relying party accepts and — when it accepts an approver's statement — the subject's bound approver DIDs. The acknowledgement discloses nothing."
errorCodes:
  - code: auth/step-up/approve-request:subjectUnknown
    meaning: The approver does not speak for the named subject.
    retryable: false
  - code: auth/step-up/approve-request:methodUnsupported
    meaning: The approver cannot deliver an approve-response carrying any of the evidence kinds in `accepts` (e.g. the wallet has no passkey for the subject and holds none of the listed approvers, or does not support the requested AAL).
    retryable: false
  - code: auth/step-up/approve-request:userDeclined
    meaning: The user reviewed the request and declined consent.
    retryable: false
  - code: auth/step-up/approve-request:rateLimited
    meaning: The relying party has exceeded the approver's request budget.
    retryable: true
related:
  - auth/step-up/approve-response
  - auth/step-up/approver/attest
  - auth/step-up/approver/list
  - auth/step-up/policy
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Request** Trust Task is the first half of an out-of-band step-up flow. The relying party (typically an auth service holding a subject's session at AAL 1) sends this document to an *approver* — usually the subject's wallet or a Verifiable-Trust Agent acting for the subject — asking the approver to ratify an AAL elevation, or — bound to one operation — to approve that one act.

The approver SHOULD show the `reason` to a human and obtain consent. If consent is granted, the approver returns an [`auth/step-up/approve-response/0.6`](../../approve-response/0.6/spec.md) signed by the **approver's** key, carrying the evidence the relying party asked for. That signed response is the cryptographic gate the relying party uses to elevate the session or authorize the operation.

**0.3 let a step-up be bound to one operation that has no session**, with `boundTo` naming the operation and `sessionId` omitted, and defined how the request travels inline, in the refusal of the operation it is for (see [Inline delivery](#inline-delivery)).

**0.4 lets the relying party ask for a statement by a step-up approver.** A subject who signs in through a wallet acts as a DID the relying party knows no passkey for, so a passkey gesture can never be asked of them. A [step-up approver](../../approver/attest/0.1/spec.md) — a `did:key` bound to that subject as their step-up factor, whose key is unlocked only by a user gesture — can answer instead. To let the approver's agent choose, 0.4:

- replaces 0.3's optional `acceptableEvidence` with a **required** `accepts`, which also admits `approverSigned`, so the approver never has to guess what the relying party will take;
- adds `approvers`, the subject's bound approver DIDs, present exactly when `accepts` lists `approverSigned`, so the agent can tell which of the approvers it holds will be accepted;
- makes the WebAuthn request options present exactly when `accepts` lists `webauthn`, rather than merely permitted.

Everything else is 0.3 unchanged.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the relying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-request/0.4`, with itself as `issuer` and the approver as `recipient`.
2. Populate `payload.subject` with the VID whose session is being elevated, or — for a step-up bound to one operation — the VID whose act that operation is.
3. Populate `payload.sessionId` with the session id (opaque to the approver but echoed back in the approve-response so the relying party can correlate) — **unless** the step-up is bound to an operation that arrived without a session, in which case `sessionId` **MUST** be omitted. A relying party **MUST NOT** populate `sessionId` with a session it chose after the fact to stand in for one the operation did not have.
4. Generate `payload.challenge` with ≥128 bits of entropy and bind it server-side to `(subject, sessionId, expiresAt)` — or, for a bound step-up, to `(subject, the bound operation, expiresAt)`.
5. For a step-up bound to one operation, populate `payload.boundTo` with an opaque identifier for that operation, and bind the operation in its own state. Where the operation's content could be recognized from an unsalted identifier (a digest over a short, predictable payload is a confirmation oracle), the identifier **MUST** be salted — for an operation that arrived as a signed document, a digest of its type and payload salted with `payload.challenge`. `boundTo` **MUST** be absent for a step-up that elevates a session.
6. Populate `payload.reason` with a user-meaningful explanation. The approver MAY refuse with `userDeclined` if the reason is empty or generic.
7. **MAY** declare `payload.targetAcr` — the AAL the relying party expects on completion.
8. Populate `payload.accepts` with every evidence kind it will accept for this step-up, and no other.
   - When it lists `webauthn`, populate `payload.webauthn` with the `PublicKeyCredentialRequestOptions` the approver feeds to the platform passkey API, whose `challenge` **MUST** equal `payload.challenge`. When it does not, `payload.webauthn` **MUST** be absent.
   - When it lists `approverSigned`, populate `payload.approvers` with the step-up approvers bound to `payload.subject` that it will accept a statement from — at least one. When it does not, `payload.approvers` **MUST** be absent. A relying party **MUST NOT** list `approverSigned` for a subject that has no live approver.
   - `approverSigned` is evidence for a step-up whose approver is the subject itself (self mode). A relying party **MUST NOT** list it for a delegated step-up, where the approve-response would be signed by a party other than the subject.
9. Include a verified `proof` so the approver can rely on the request's `recipient` as authoritative.

A conforming **consumer** (the approver) **MUST**:

1. Verify the document's `proof`, and verify the DID resolved from the proof's `verificationMethod` equals the document's `issuer` — the signature is by the *named* relying party, not some third key. Mismatch → the framework `proofInvalid` error, per the binding rule of [SPEC.md §4.7](/SPEC.md#47-proof) (this task mints no task-specific code for it).
2. Determine whether it speaks for `payload.subject`. If not → `subjectUnknown`.
3. Decide whether to surface the request to the user (subject of consent) or to ratify it programmatically (policy-bound delegation). The framework leaves this to the approver — but if a human is presented with the request, the `reason` MUST be shown verbatim.
4. Return an approve-response whose `evidence.kind` is listed in `payload.accepts`. If it can satisfy none of them → `methodUnsupported`. In particular:
   - for `webauthn`, pass `payload.webauthn` to the platform passkey API unchanged and assert over `payload.challenge`;
   - for `approverSigned`, have one of the approvers in `payload.approvers` — and no other — sign an [`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md) statement with `purpose: stepUp`, copying `subject`, `challenge` and `boundTo` from this request and naming the relying party as `audience`.
5. Return a `#response` document carrying `status: accepted` (will return an approve-response asynchronously) or `status: refused` (with a `reason`).

> **Note (non-normative).** The reference ecosystem signs this document with the `eddsa-jcs-2022` Data Integrity cryptosuite and `proofPurpose: authentication`, as the examples show. This is an implementation profile, not a requirement of this specification: [SPEC.md §4.7](/SPEC.md#47-proof) leaves the choice of cryptosuite open, and any registered suite whose `verificationMethod` resolves to material controlled by the `issuer` satisfies the `proof` requirement.

The approve-response document arrives out-of-band — typically via the approver's preferred transport (DIDComm push to the relying party's mediator, or a push channel the relying party registered at request time).

### Inline delivery

A relying party that refuses an operation because it needs a step-up bound to that operation, and has no push channel to the approver, **MAY** carry this task's payload in the refusal instead of sending it as a document. It does so in an *error response* ([SPEC.md §8](/SPEC.md#8-error-responses)) answering the operation, with `details.stepUpRequest` set to a payload valid against this specification's schema, with `boundTo` present and `sessionId` absent.

The producer of the refused operation is then the approver's agent: it surfaces the request to its human, obtains the evidence — a passkey gesture over `challenge`, or an approver's statement over it — returns an [`approve-response/0.6`](../../approve-response/0.6/spec.md) to the relying party, and on a `recorded` acknowledgement re-sends the same operation.

- An inline request carries no `proof` of its own. It is authenticated by the channel it arrived on: it is the relying party's own reply to a request the producer addressed to that relying party. A producer **MUST NOT** surface a `stepUpRequest` it received any other way, and **MUST** check that its `subject` is the party the producer acts for.
- The `reason` in an inline request is still what the human consents to, and the rules in [Security & Privacy](#security--privacy) apply to it unchanged.
- Inline delivery is only for bound step-ups. A session elevation requested inline would let any refusal on any channel raise a session, which is what this document's proof requirement exists to prevent.

## Definitions

* **Relying party.** The party requesting the elevation; identified by `issuer` and verified via `proof`.
* **Approver.** The party authoritative for `payload.subject`; identified by `recipient`. Wallets and VTAs are typical. (Not to be confused with a *step-up approver*, below, which is a factor and not a party to this exchange.)
* **Step-up approver.** A `did:key` bound at the relying party to exactly one subject as that subject's step-up factor ([`auth/step-up/approver/attest/0.1`](../../approver/attest/0.1/spec.md)). It confers nothing; its statement over `challenge` is the subject's own additional factor.
* **Subject.** The VID whose session is being elevated.
* **Session.** The session the relying party holds for the subject; the approver does not need to know its contents — only the opaque `sessionId`.
* **Bound step-up.** A step-up that authorizes exactly one operation and elevates no session; the operation is the relying party's own state and is named to the approver by `boundTo`.

## Payload

`payload.subject`, `payload.challenge`, `payload.reason`, `payload.accepts` — REQUIRED.

`payload.sessionId` — REQUIRED for a step-up that elevates a session; absent for a bound step-up whose operation arrived without one.

`payload.boundTo` — present exactly when the step-up is bound to one operation.

`payload.webauthn` — present exactly when `accepts` lists `webauthn`: `PublicKeyCredentialRequestOptions` whose `challenge` MUST equal `payload.challenge`.

`payload.approvers` — present exactly when `accepts` lists `approverSigned`: the subject's bound step-up approver DIDs the relying party will accept a statement from.

`payload.targetAcr`, `payload.ttl` — optional hints.

`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

The relying party (`issuer`) sends the request to the approver (`recipient`), or carries its payload inline in a refusal. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Relying party asks the user's wallet to confirm a transfer

```json
{
  "id": "step-up-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.4",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "reason": "Confirm transfer of $1,000 to did:web:bob.example",
    "accepts": ["didSigned"],
    "targetAcr": "aal2",
    "ttl": 120
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-05-23T14:00:00Z",
    "verificationMethod": "did:web:bank.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKiMU5gSwwFdcjtPar"
  }
}
```

### Relying party requires a passkey-backed elevation on the user's phone

```json
{
  "id": "step-up-2345-6789-01bc-def123456789",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.4",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "reason": "Confirm transfer of $1,000 to did:web:bob.example",
    "targetAcr": "aal2",
    "accepts": ["webauthn"],
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
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-05-23T14:00:00Z",
    "verificationMethod": "did:web:bank.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKiMU5gSwwFdcjtPar"
  }
}
```

### A community refuses a wallet administrator's signed grant and asks for an approver's statement, inline

A wallet-signed-in administrator sent a signed `acl/grant` document conferring an administrative role. The community's policy asks for a step-up bound to that one grant. The administrator has no passkey at the community, but has a step-up approver bound — the browser plugin's — so the refusal accepts `approverSigned` and names it. The console hands the request and the operation to the plugin, which recomputes `boundTo`, shows the grant, and has the approver sign after the user's gesture; the console returns an approve-response signed by the administrator's own DID and re-sends the grant.

```json
{
  "id": "err-6789-0123-4567-890abcdef123",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "grant-1234-5678-9abc-def012345678",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-02T08:59:30Z",
  "payload": {
    "code": "permissionDenied",
    "message": "a step-up bound to this grant is required",
    "retryable": false,
    "details": {
      "stepUpRequest": {
        "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
        "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
        "boundTo": "uEiD3n4bE1QbQ8nY2l0cGhYc2FsdGVkZGlnZXN0",
        "reason": "Grant the administrator role to did:key:z6MkhaXg…",
        "accepts": ["approverSigned"],
        "approvers": ["did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"],
        "ttl": 300
      }
    }
  }
}
```

## Response

The `#response` is a synchronous acknowledgement that the approver received the request — NOT the approval itself. The approve-response follows out-of-band. Its payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`.

### Approver accepts

```json
{
  "id": "step-up-resp-3456-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.4#response",
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
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.4#response",
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

**`boundTo` is not a capability.** It identifies the operation for the approver's audit trail and for the step-up approver's statement. Presenting it authorizes nothing, because the binding is held by the relying party.

**Salting.** An operation identifier derived from the operation's content, unsalted, lets anyone who sees it test guesses at that content — for an administrative grant the space of plausible payloads is small. The salt is the per-request `challenge`, so the identifier is unlinkable across requests even for identical operations.

**`accepts` is the relying party's whole offer.** A relying party verifies the approve-response's evidence against the kinds it listed for *this* step-up, from its own state, and refuses any other (`approve-response:noGate`). An approver that could pick an unlisted kind — the weakest the relying party supports somewhere — would let the weakest gate answer every step-up. Listing `approverSigned` with an `approvers` list is likewise an offer read back from the relying party's own state, never from the response.

**`approvers` is not a list of authorities.** A step-up approver confers nothing; naming it tells the subject's agent which of its factors will be accepted. The relying party re-checks, when the statement arrives, that its signer is still a live approver bound to the subject — a binding revoked between request and response does not count because it was listed.

### Data carried

The subject's identifier, a single-use challenge, the relying party's `reason`, the accepted evidence kinds, optional assurance hints, WebAuthn request options (which name the credential ids the relying party holds for the subject) when a passkey is accepted, the subject's bound approver DIDs when an approver's statement is accepted, and — for a session elevation — an opaque session id, or — for a bound step-up — an opaque, salted operation identifier. It carries no attribute of the subject beyond these. The `reason` is the one member that can carry sensitive content, and a relying party SHOULD say what is being approved without restating more of it than the human needs to decide.

### Correlation

The `challenge` and a salted `boundTo` are single-use and unpredictable, so neither links two requests. `allowCredentials` and `approvers` name the subject's factors, which do link requests to the same subject — inherent, since the approver must know which factor to use — and SHOULD be limited to the factors the relying party would accept. Because an approver DID is used only at the relying party that bound it, `approvers` links requests at that relying party and nowhere else. Inline delivery discloses `approvers` only to the producer of the refused operation, which signed that operation as, or for, the subject.

### Retention

A relying party holds the pending step-up — challenge, subject, accepted kinds and approvers, and the session or bound operation — until the approve-response consumes it or it expires, and SHOULD discard it then. An inline request exists only in the refusal that carried it.

### Consent/purpose

This task asks a human for a decision about one elevation or one operation; the `reason` is the purpose they are consenting to, and it is exhausted by the approve-response that answers it. What authority the relying party requires before asking, and which evidence it accepts, is its own policy, which this specification does not state.

The optional `ext` extension is part of the signed surface.
