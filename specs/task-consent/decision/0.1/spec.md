---
slug: task-consent/decision
version: "0.1"
title: Task Consent — Decision
summary: An enrolled approver authorizes or refuses one pending privileged task, bound to the exact payload they were shown. The proof on the decision — not the session that carried it — is the authorization.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - delegated-execution
  - approval
  - authorization
  - policy
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Approver device
    requirement: REQUIRED
    member: issuer
  - role: Executor (Verifiable Trust Agent)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The decision IS the authorization, so the proof is the only thing that carries it. The executor takes the approver's identity from the verified proof and never from the transport session — a bearer token proves who opened the channel, not who agreed. Without a proof an attacker holding any authenticated session to the executor could approve their own pending task.
sideEffects:
  level: mutating
  rationale: Records an approval against the pending request and, at the threshold, issues a single-use grant the requester's re-submit consumes. A denial deletes the pending request.
consequences:
  - "At the approval threshold, authorizes the pending task to execute — including, where that task is classified `destructive`, an irreversible effect."
  - "The grant is single-use and time-boxed; it authorizes exactly one execution of exactly one payload."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: task-consent/decision:no_pending
    meaning: No live pending consent exists for the `payloadDigest` — never raised, already decided, or lapsed.
    retryable: false
  - code: task-consent/decision:challenge_mismatch
    meaning: The `challenge` does not match the pending request for this digest.
    retryable: false
  - code: task-consent/decision:not_an_approver
    meaning: The proven signer is not a member of the approver set the policy named.
    retryable: false
  - code: task-consent/decision:requester_excluded
    meaning: The proven signer is the task's requester and the policy set `excludeRequester`.
    retryable: false
related:
  - task-consent/request
  - policy/evaluate
---

## Abstract

The **Task Consent — Decision** Trust Task is a human's answer to a
[`task-consent/request/0.1`](../../request/0.1/spec.md), and the authorization
an executor consumes before running a privileged task.

The invariant it serves:

> No state-mutating task executes at the executor unless it has verified a
> single-use decision, signed by a **currently-enrolled** approver, whose
> `payloadDigest` equals the digest of the **exact payload it is about to
> execute**, against the **exact prior state** it used to compute the effects it
> showed the human — and unless policy and enrolment **still** permit it at the
> moment of execution.

Four properties fall out. The relying party cannot authorize anything. A
compromised device cannot self-approve a destructive task when policy names an
approver set excluding it. A compromised device cannot swap the payload after
approval — the digest binding fails. And revoking a device stops approvals
already in flight from it.

## The proof is the authorization

The executor takes the approver's identity from the **verified proof on this
document**, and never from the transport session that delivered it. A bearer
token proves who opened the channel; it does not prove who agreed. The
distinction is the whole design: a decision relayed through the requester —
which is the normal case, since the requester is the party holding the rejected
task — passes through an untrusted party's hands, and must remain sound anyway.

This is what lets approval be **decoupled from the request channel**. The device
that proposes need not be the device that approves; the decision is a
free-standing, verifiable object.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver device) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/task-consent/decision/0.1`, with itself as `issuer` and the executor as `recipient`, carrying a verifiable `proof`.
2. Echo `challenge` and `payloadDigest` **verbatim** from the `task-consent/request` it verified. It **MUST NOT** recompute `payloadDigest` from any payload supplied to it by a party other than the executor, and it **MUST NOT** emit a decision for a request whose proof it could not verify.
3. Set `decision` to the human's actual answer. A device **MUST NOT** synthesise an approval — including on a timeout, a dismissal, or a closed window, all of which are denials or silence, never assent.

A conforming **consumer** (the executor) **MUST**, on receipt:

1. Verify the `proof` and take the approver's identity from it.
2. Look up the pending request by `payloadDigest`; absent or lapsed → `no_pending`.
3. Assert `challenge` matches that pending request → else `challenge_mismatch`.
4. Assert the proven signer is a member of the approver set the policy named → else `not_an_approver`.
5. Assert the signer is not the requester when `excludeRequester` is set → else `requester_excluded`.
6. On `deny`, delete the pending request. A subsequent submit of the same task starts a fresh one.
7. On `approve`, record the approval idempotently per approver, and at `minApprovals` distinct approvers issue a single-use, time-boxed grant.

and **MUST**, at execution of the task the grant authorizes:

8. Re-derive `payloadDigest` from the payload it is about to execute and refuse on mismatch.
9. Assert the `statePin` still holds.
10. **Consume the challenge at execution, not on receipt of this decision.** A decision authorizes exactly one execution; consuming it earlier lets the executor's own retry legitimately replay it.
11. **Re-evaluate policy and the approver's enrolment.** A device revoked during the approval window **MUST NOT** be able to carry a task through it.

An executor **MUST NOT** gate this task behind the very consent mechanism it implements — a decision that itself required a decision would not terminate. The same exemption applies to any step-up ceremony.

## Consent fatigue

Designs like this die to habituation, not to cryptography. An executor
**SHOULD** apply per-origin approval budgets with escalating friction, and
**SHOULD reset the budget on denial rather than on approval** — so that spamming
a human makes the next prompt harder rather than clearing the counter. The
inverse, which is the intuitive implementation, rewards the attacker for
persistence.

## Payload

`payload.challenge` (REQUIRED) — echoes the request this answers.
`payload.payloadDigest` (REQUIRED) — echoes the digest being authorized.
`payload.decision` (REQUIRED) — `approve` or `deny`.
`payload.reason` (OPTIONAL) — human-facing note, most useful on a denial.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### An operator approves the pending DID document update

## Request

```json
{
  "id": "urn:uuid:c4a90f18-2de6-4b73-9f05-8a1c6b3e27d9",
  "type": "https://trusttasks.org/spec/task-consent/decision/0.1",
  "issuer": "did:key:z6MkApproverPhoneExample",
  "recipient": "did:key:z6MkExecutorVtaExample",
  "issuedAt": "2026-07-13T09:43:18Z",
  "payload": {
    "challenge": "9c1f4b7a2e6d80f35a4c9b1e7d2f6083",
    "payloadDigest": "3b0c7f1d9e2a5648c1f30b7ae4d2986153ca0f7b8d41e6295af03c8bd71e4a62",
    "decision": "approve"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-13T09:43:18Z",
    "verificationMethod": "did:key:z6MkApproverPhoneExample#z6MkApproverPhoneExample",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2QpLmExampleProofValueForTaskConsentDecision"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:7b3f8d21-5c04-4e19-a6d8-2f9e1b0c4a63",
  "type": "https://trusttasks.org/spec/task-consent/decision/0.1#response",
  "issuer": "did:key:z6MkExecutorVtaExample",
  "recipient": "did:key:z6MkApproverPhoneExample",
  "issuedAt": "2026-07-13T09:43:19Z",
  "payload": {
    "status": "granted",
    "payloadDigest": "3b0c7f1d9e2a5648c1f30b7ae4d2986153ca0f7b8d41e6295af03c8bd71e4a62",
    "approvals": 1
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-13T09:43:19Z",
    "verificationMethod": "did:key:z6MkExecutorVtaExample#z6MkExecutorVtaExample",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6TvNsExampleProofValueForTaskConsentDecisionResponse"
  }
}
```
