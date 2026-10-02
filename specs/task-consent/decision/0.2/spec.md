---
slug: task-consent/decision
version: "0.2"
title: Task Consent — Decision
summary: An enrolled approver authorizes or refuses one pending privileged task, bound to the exact payload shown, optionally backed by an extra factor (a passkey assertion or a signed approver statement). The proof by the approver's own DID is the authorization.
status: draft
targetFrameworkVersion: "0.6.0"
category: consent
keywords:
  - consent
  - delegated-execution
  - approval
  - authorization
  - policy
  - passkey
  - step-up
parties:
  - role: Approver
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Executor (Verifiable Trust Agent or Verifiable Trust Community)
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The decision IS the authorization, so the proof is the only thing that carries it, and it is REQUIRED whether or not
    `evidence` is present. The executor takes the approver's identity from the verified proof and never from the
    transport session — a bearer token proves who opened the channel, not who agreed — and only a proof by the approver's
    own DID names an approver: a delegated or console key is never one. `evidence` adds a factor on top of that proof;
    it never stands in for it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The decision authorises one particular Trust Task to proceed. Replayed, it authorises a second execution of that task
    on the strength of a human's single answer, which is exactly the case SPEC §7.2 item 11 is written for.
sideEffects:
  level: mutating
  rationale: >-
    Records an approval against the pending request and, at the threshold, either issues a single-use grant the
    requester's re-submit consumes or — for a parked Verifiable Trust Community action — has the executor run the parked
    operation itself, exactly once. A denial closes the pending request for every approver.
consequences:
  - "At the approval threshold, authorizes the pending task to execute — including, where that task is classified `destructive`, an irreversible effect."
  - "The authorization is single-use and time-boxed; it authorizes exactly one execution of exactly one payload."
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: >-
    `challenge`, `payloadDigest` and `actionId` are echoed from what the executor issued and `decision` is a two-valued
    enum, so the classification turns on `reason` — free text in which a human explains a decision, most often a refusal,
    that travels on to the executor and potentially the requester — and on `evidence`, which carries a WebAuthn assertion
    (a credential identifier and authenticator data about the approver's device) or a signed approver statement. Nothing
    is disclosed back beyond the approval tally.
retention:
  class: durable
  rationale: >-
    The pending request, its challenge and any grant are exchange-scoped and end at execution, denial or expiry. The
    decision document is not: because its proof is the authorization, it is the only evidence that a privileged and
    possibly irreversible operation was agreed to, and by which approver. Evidence carried in it is part of that record —
    it shows which factor backed the agreement.
errorCodes:
  - code: task-consent/decision:noPending
    meaning: No live pending consent exists for the `payloadDigest` — never raised, already decided, or lapsed.
    retryable: false
  - code: task-consent/decision:challengeMismatch
    meaning: The `challenge` does not match the pending request for this digest, or is not the one issued to this approver.
    retryable: false
  - code: task-consent/decision:notAnApprover
    meaning: The proven signer is not a member of the approver set the policy named — including a signer that is a delegated or console key rather than an approver's own DID.
    retryable: false
  - code: task-consent/decision:requesterExcluded
    meaning: The proven signer is the task's requester and the policy set `excludeRequester`.
    retryable: false
  - code: task-consent/decision:actionMismatch
    meaning: The `actionId` names a different action from the pending request that `payloadDigest` and `challenge` resolve to.
    retryable: false
  - code: task-consent/decision:evidenceInvalid
    meaning: "`evidence` was supplied and failed verification: the assertion or statement did not verify, was not bound to this decision's challenge and digest, or was made by a factor not enrolled to the proven signer."
    retryable: false
  - code: task-consent/decision:evidenceRequired
    meaning: The executor's policy for this pending request calls for an evidence kind the decision did not carry.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        acceptedKinds:
          type: array
          items:
            type: string
            enum: [webauthn, approverSigned]
related:
  - task-consent/request
  - task-consent/granted
  - policy/evaluate
  - vtc/admin/actions/list
  - vtc/admin/actions/show
  - auth/step-up/approver/attest
  - auth/step-up/approve-response
---

## Abstract

The **Task Consent — Decision** Trust Task is a human's answer to a pending privileged task — a
[`task-consent/request/0.1`](../../request/0.1/spec.md), or an action parked by a Verifiable
Trust Community and shown through [`vtc/admin/actions/list/0.1`](../../../vtc/admin/actions/list/0.1/spec.md)
— and the authorization an executor consumes before running it.

The invariant it serves is unchanged from 0.1:

> No state-mutating task executes at the executor unless it has verified a
> single-use decision, signed by a **currently-enrolled** approver, whose
> `payloadDigest` equals the digest of the **exact payload it is about to
> execute**, against the **exact prior state** it used to compute the effects it
> showed the human — and unless policy and enrolment **still** permit it at the
> moment of execution.

## What changed from 0.1

Two OPTIONAL members, both additive; a 0.1 decision re-typed as 0.2 is a valid 0.2 decision with the same meaning.

- **`evidence`** — an additional factor backing the decision, tagged on `kind`: a WebAuthn assertion (`webauthn`) or a signed [`auth/step-up/approver/attest/0.1`](../../../auth/step-up/approver/attest/0.1/spec.md) statement (`approverSigned`). It is verified **in addition to** the document's proof, never instead of it.
- **`actionId`** — links the decision to a Verifiable Trust Community action.

## The proof is the authorization

The executor takes the approver's identity from the **verified proof on this document**, and never from the transport session that delivered it. A decision relayed through the requester — the normal case — passes through an untrusted party's hands and must remain sound anyway.

That holds with `evidence` present. The proof is still REQUIRED and is still the authorization, and it **MUST** be made by the approver's **own DID**. A delegated signing key, a console key, or any key that acts *for* an approver is never an approver: an approval it signed is `notAnApprover`. This is what stops a console, a relay, or a compromised delegation from collecting approvals on an approver's behalf — the very thing an approval threshold exists to prevent.

`evidence` answers a different question. The proof says *which approver* agreed; evidence says *how* that approver demonstrated presence at the moment of agreeing — a passkey gesture, or a signature from a step-up approver key held on a separate device. An executor whose policy asks for that assurance verifies it on top of the proof.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/task-consent/decision/0.2`, with itself as `issuer` and the executor as `recipient`, carrying a verifiable `proof` made by the approver's own DID. The `proof.proofPurpose` **MUST** be `assertionMethod`, and its `verificationMethod` **MUST** be listed under the `assertionMethod` verification relationship of the approver's DID document.
2. Echo `challenge` and `payloadDigest` **verbatim** from the pending request it verified — a `task-consent/request`, or a VTC action on which it was shown a `challenge` — and, for a VTC action, set `actionId` to that action's identifier. It **MUST NOT** recompute `payloadDigest` from any payload supplied to it by a party other than the executor.
3. Set `decision` to the human's actual answer. A producer **MUST NOT** synthesise an approval — including on a timeout, a dismissal, or a closed window.
4. When it carries `evidence`:
   - for `webauthn`, pass the UTF-8 bytes of the decision's `challenge` value to the authenticator as the WebAuthn challenge, so that `clientDataJSON.challenge` is their base64url (no padding) encoding, and carry the platform's `AuthenticatorAssertionResponse` unmodified;
   - for `approverSigned`, carry a complete signed `auth/step-up/approver/attest/0.1` document with `purpose: decision`, `subject` equal to the DID that makes this document's proof, `audience` equal to the executor, `challenge` equal to this decision's `challenge`, and `boundTo` equal to this decision's `payloadDigest`, unaltered.

A conforming **consumer** (the executor) **MUST**, on receipt:

1. Verify the `proof` and take the approver's identity from it, refusing a proof whose `proofPurpose` is not `assertionMethod`, whose `verificationMethod` is not listed under the approver's `assertionMethod` relationship, or whose approver DID it cannot resolve.
2. Look up the pending request by `payloadDigest`; absent or lapsed → `noPending`.
3. Assert `challenge` matches that pending request — and, where challenges are per approver, the one issued to this signer → else `challengeMismatch`.
4. When `actionId` is present, assert it names the action the pending request belongs to → else `actionMismatch`.
5. Assert the proven signer is a member of the approver set the policy named, and is an approver's own DID rather than a key delegated to act for one → else `notAnApprover`.
6. Assert the signer is not the requester when `excludeRequester` is set → else `requesterExcluded`.
7. When `evidence` is present, verify it → else `evidenceInvalid`:
   - `webauthn`: verify the assertion per WebAuthn Level 2 §7.2 with itself as relying party, against a credential enrolled to the proven signer, and assert the challenge in `clientDataJSON` is the base64url encoding of this decision's `challenge`. That challenge was issued for exactly one pending request whose `payloadDigest` is salted with it, so the assertion commits to that digest through the executor's own binding.
   - `approverSigned`: verify the statement as a document in its own right — envelope, proof and payload against `auth/step-up/approver/attest/0.1` — over its members exactly as received; assert `purpose` is `decision`, its issuer is a step-up approver enrolled to the proven signer, `subject` is the proven signer, `audience` is itself, `challenge` equals this decision's `challenge`, and `boundTo` equals this decision's `payloadDigest`.
8. When its policy for this pending request calls for an evidence kind the decision does not carry, refuse with `evidenceRequired`, naming the accepted kinds in `details`.
9. On `deny`, close the pending request for every approver. A subsequent submit of the same task starts a fresh one.
10. On `approve`, record the approval idempotently per approver, and at the threshold of distinct approvers either issue a single-use, time-boxed grant (the `task-consent/request` flow) or execute the parked operation itself, exactly once (a VTC action).

and **MUST**, at execution of the task the decision authorizes:

11. Re-derive `payloadDigest` from the payload it is about to execute and refuse on mismatch.
12. Assert the `statePin` (or the VTC's equivalent re-check of the state it acts on) still holds.
13. **Consume the challenge at execution, not on receipt of this decision.** A decision authorizes exactly one execution; consuming it earlier lets the executor's own retry legitimately replay it.
14. **Re-evaluate policy and every approver's enrolment and standing.** An approver revoked during the approval window **MUST NOT** be able to carry a task through it.

An executor **MUST NOT** gate this task behind the very consent mechanism it implements — a decision that itself required a decision would not terminate. The same exemption applies to any step-up ceremony: the factor a decision needs travels *in* the decision, as `evidence`, precisely so that no separate step-up exchange is needed.

## Authorization

The authority is **membership of the approver set the executor's policy named for this pending request, held by the approver's own DID at execution time**. The decision's proof establishes which DID agreed; the executor's own records establish that this DID is an eligible approver for this request — the approver set, the requester exclusion, enrolment, and for a VTC action the administrative standing the action names. The proof is the authorization only in that declared role — attributing agreement to one approver for one payload — and does not extend past it. `evidence`, when present, is an additional factor the executor verifies; it confers no authority of its own, and a valid assertion or statement from a party outside the approver set authorizes nothing.

## Consent fatigue

Designs like this die to habituation, not to cryptography. An executor **SHOULD** apply per-origin approval budgets with escalating friction, and **SHOULD reset the budget on denial rather than on approval**. A VTC action carries `requesterOpenActions` so an approver can see how many requests the same requester has open.

## Definitions

- **`challenge`** (REQUIRED) — echoes the pending request this answers.
- **`payloadDigest`** (REQUIRED) — echoes the digest being authorized.
- **`decision`** (REQUIRED) — `approve` or `deny`.
- **`reason`** (OPTIONAL) — human-facing note, most useful on a denial.
- **`actionId`** (OPTIONAL) — the VTC action this decision answers.
- **`evidence`** (OPTIONAL) — an additional factor, `{kind: "webauthn", assertion}` or `{kind: "approverSigned", statement}`.
- **`ext`** — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

The approver signs a decision to the executor. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Bob approves a VTC action with a passkey gesture

Bob's console showed him the action from `vtc/admin/actions/list`, with his challenge. He approves it, and his authenticator signs over that challenge.

```json
{
  "id": "urn:uuid:c4a90f18-2de6-4b73-9f05-8a1c6b3e27d9",
  "type": "https://trusttasks.org/spec/task-consent/decision/0.2",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T09:16:30Z",
  "threadId": "urn:uuid:c4a90f18-2de6-4b73-9f05-8a1c6b3e27d9",
  "payload": {
    "challenge": "c7f19a3e5b2d48f0a6e1d9c4b8f20a37",
    "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
    "decision": "approve",
    "actionId": "act_7Hq2mZp9Lx4vRk8T",
    "evidence": {
      "kind": "webauthn",
      "assertion": {
        "id": "Vb3qKx0d1mW7cR2tYp9sLg",
        "rawId": "Vb3qKx0d1mW7cR2tYp9sLg",
        "type": "public-key",
        "response": {
          "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiWXpkbU1UbGhNMlUxWWpKa05EaG1NR0UyWlRGa09XTTBZamhtTWpCaE16YyIsIm9yaWdpbiI6Imh0dHBzOi8vY29tbXVuaXR5LmV4YW1wbGUifQ",
          "authenticatorData": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MFAAAAAQ",
          "signature": "MEUCIQDexampleSignatureOverAuthenticatorDataAndClientDataHash"
        },
        "authenticatorAttachment": "platform"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:16:30Z",
    "verificationMethod": "did:web:bob.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2QpLmExampleProofValueForTaskConsentDecision02"
  }
}
```

## Response

The executor acknowledges the recorded decision, per the `$anchor: "response"` sub-schema. Failures use `trust-task-error`.

### Threshold met — the parked operation executes

Bob's was the second of two approvals, so the community runs the parked grant itself.

```json
{
  "id": "urn:uuid:7b3f8d21-5c04-4e19-a6d8-2f9e1b0c4a63",
  "type": "https://trusttasks.org/spec/task-consent/decision/0.2#response",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-02T09:16:31Z",
  "threadId": "urn:uuid:c4a90f18-2de6-4b73-9f05-8a1c6b3e27d9",
  "payload": {
    "status": "granted",
    "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
    "actionId": "act_7Hq2mZp9Lx4vRk8T",
    "approvals": 2
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:16:31Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z6TvNsExampleProofValueForTaskConsentDecision02Response"
  }
}
```

### The executor's policy asked for evidence

```json
{
  "id": "urn:uuid:3e9a1c7f-6b2d-4f80-a5c3-9d7e1b4f2a68",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-02T09:16:31Z",
  "threadId": "urn:uuid:c4a90f18-2de6-4b73-9f05-8a1c6b3e27d9",
  "payload": {
    "code": "task-consent/decision:evidenceRequired",
    "retryable": false,
    "details": { "acceptedKinds": ["webauthn", "approverSigned"] }
  }
}
```

## Security & Privacy

### Data carried

As in 0.1, most of the payload is not the approver's to choose: `challenge`, `payloadDigest` and `actionId` are echoed from what the executor issued, and `decision` is a two-valued enum. `reason` is the free member — capped at 500 characters — and it can reach the requester the approver just refused; a producer **SHOULD** confine it to why *this request* was refused and **MUST NOT** treat it as a private aside to the executor.

`evidence` is new and carries data about the approver's device. A WebAuthn assertion carries a credential identifier, authenticator flags and a signature counter, and `userHandle` where the authenticator returns one; an approver statement carries the step-up approver key's DID. That is the minimum each factor needs to be verified, and it **MUST** be carried unmodified, since the signatures cover it. The response carries `status`, the echoed digest and `actionId`, and the tally — which tells the approver how many colleagues have agreed, information about other people's actions that is carried because an approver who cannot tell whether their approval completed the threshold cannot tell whether to expect anything to happen.

### Correlation

The decision itself joins to little: `challenge` is single-use and per approver, `payloadDigest` is salted with it, and `actionId` names one action. The signer is the joinable thing, unavoidably — every approval and denial an individual makes is signed by them and lands in one place. `evidence` adds one more stable handle: a WebAuthn credential identifier, or an approver key DID, is the same across every decision that approver backs with it, so an executor can link an approver's decisions to one device. Both parties declare `identifierScope: pairwise`, and that declaration is what keeps that history attributable at the executor rather than by anyone who comes into possession of a single decision.

### Retention

The exchange state is short: the pending request closes on `deny`, the grant or execution is single-use, and the challenge is consumed at execution. The decision document is durable, because its proof is the only evidence that a privileged operation was agreed to and by whom; its `evidence` is retained with it as the record of which factor backed that agreement. What an executor **SHOULD NOT** keep alongside it is the rendered request or the parked payload's personal content beyond what its own audit record of the operation already holds — the decision's accountability needs the challenge, the digest, the decision, the proof and the evidence, not the diff.

### Consent/purpose

The data moves so that an executor can establish that a specific, currently-enrolled approver agreed to a specific payload — and, where it asked for one, by a specific factor — and can refuse to act if any part of that fails. The accumulated record is authorization evidence, not performance data: scoring approvers on speed or agreeableness, or routing requests to the approver most likely to agree, are uses no approver assented to, and the second defeats a threshold without any signature failing to verify. Evidence data about an approver's device is for verifying this decision and **MUST NOT** be reused to track the device elsewhere. Per [SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 13, this specification describes what a decision and its evidence *are*, never that either is required; whether a task must be approved, and with what factor, is the executor's policy, expressed through [`policy/evaluate`](../../../policy/evaluate/0.3/spec.md).
