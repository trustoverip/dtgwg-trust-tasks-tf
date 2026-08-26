---
slug: task-consent/decision
version: "0.1"
title: Task Consent — Decision
summary: An enrolled approver authorizes or refuses one pending privileged task, bound to the exact payload they were shown. The proof on the decision — not the session that carried it — is the authorization.
status: draft
targetFrameworkVersion: "0.5"
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
    identifierScope: pairwise
  - role: Executor (Verifiable Trust Agent)
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: The decision IS the authorization, so the proof is the only thing that carries it. The executor takes the approver's identity from the verified proof and never from the transport session — a bearer token proves who opened the channel, not who agreed. Without a proof an attacker holding any authenticated session to the executor could approve their own pending task.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The decision authorises one particular Trust Task to proceed. Replayed, it authorises a second execution of that task on the strength of a human's single answer, which is exactly the case SPEC §7.2 item 11 is written for.
sideEffects:
  level: mutating
  rationale: Records an approval against the pending request and, at the threshold, issues a single-use grant the requester's re-submit consumes. A denial deletes the pending request.
consequences:
  - "At the approval threshold, authorizes the pending task to execute — including, where that task is classified `destructive`, an irreversible effect."
  - "The grant is single-use and time-boxed; it authorizes exactly one execution of exactly one payload."
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "Three of the four payload members are not the approver's to author — `challenge` and `payloadDigest` are echoed verbatim from the request, `decision` is a two-valued enum — so the classification turns entirely on `reason`. That member is unbounded free text in which a human explains a decision they have just taken, most often a refusal, and it travels onward to the executor and potentially to the requester. Nothing is disclosed back to the producer beyond the approval tally, so `discloses` stays `none`."
retention:
  class: durable
  rationale: "The pending request, the single-use `challenge`, and the time-boxed grant are all exchange-scoped and end at execution, at denial, or at the request's `expiresAt`. The decision document is not: because the proof on it — rather than the session that carried it — is the authorization, it is the only evidence that a privileged and possibly irreversible operation was agreed to, and by which enrolled approver. An executor that discards it cannot afterwards show that a `destructive` task it ran was authorized at all. What it keeps in exchange is a signed, attributed record of an individual's decisions; see Security & Privacy → Retention for what an executor should therefore not keep alongside it."
errorCodes:
  - code: task-consent/decision:noPending
    meaning: No live pending consent exists for the `payloadDigest` — never raised, already decided, or lapsed.
    retryable: false
  - code: task-consent/decision:challengeMismatch
    meaning: The `challenge` does not match the pending request for this digest.
    retryable: false
  - code: task-consent/decision:notAnApprover
    meaning: The proven signer is not a member of the approver set the policy named.
    retryable: false
  - code: task-consent/decision:requesterExcluded
    meaning: The proven signer is the task's requester and the policy set `excludeRequester`.
    retryable: false
related:
  - task-consent/request
  - task-consent/granted
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver device) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/task-consent/decision/0.1`, with itself as `issuer` and the executor as `recipient`, carrying a verifiable `proof`.
2. Echo `challenge` and `payloadDigest` **verbatim** from the `task-consent/request` it verified. It **MUST NOT** recompute `payloadDigest` from any payload supplied to it by a party other than the executor, and it **MUST NOT** emit a decision for a request whose proof it could not verify.
3. Set `decision` to the human's actual answer. A device **MUST NOT** synthesise an approval — including on a timeout, a dismissal, or a closed window, all of which are denials or silence, never assent.

A conforming **consumer** (the executor) **MUST**, on receipt:

1. Verify the `proof` and take the approver's identity from it.
2. Look up the pending request by `payloadDigest`; absent or lapsed → `noPending`.
3. Assert `challenge` matches that pending request → else `challengeMismatch`.
4. Assert the proven signer is a member of the approver set the policy named → else `notAnApprover`.
5. Assert the signer is not the requester when `excludeRequester` is set → else `requesterExcluded`.
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
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

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
    "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
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
    "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
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

## Security & Privacy

### Data carried

The payload is four members and three of them are not the approver's to choose.
`challenge` and `payloadDigest` are echoed **verbatim** from the
[`task-consent/request`](../../request/0.1/spec.md) the device verified — a producer
**MUST NOT** recompute the digest from any payload handed to it by a party other than
the executor — and `decision` is a two-valued enum. That leaves exactly one free
member, and it is worth putting next to its counterpart in the request: the
requester's `note` is capped at 500 characters precisely because it is untrusted prose
crossing a trust boundary, while `reason` has **no** length bound at all and crosses
the same boundary in the opposite direction.

The asymmetry matters, because `reason` is a channel the person writing it may not
realise is one. The schema says it is most useful on a `deny`, which is exactly the
moment a human is most likely to write something explanatory and personal — where they
are, what they suspect, who they think is behind the request. It reaches the executor,
and an executor that relays denials back to the requester is doing something entirely
reasonable ([`task-consent/granted`](../../granted/0.1/spec.md) exists so requesters do
not have to poll) while delivering that text to the very party the human just refused.
A producer **SHOULD** confine `reason` to why *this request* was refused, in terms of
the request, and **MUST NOT** treat it as a private aside to the executor.

The rest of the document is engineered to say as little as possible. `payloadDigest`
is echoed in the encoding the request carried it and is salted with `challenge`, so it
commits to the payload the human approved without disclosing anything about it.

The response carries `status`, the echoed `payloadDigest`, and — where relevant —
`approvals` and `needed`. Those last two are a small, easily-overlooked disclosure:
they tell the approver how many colleagues have already agreed and how many are
required, which is information about other people's actions and about the
organisation's approval policy that neither the approver nor those colleagues chose to
share. They are carried because an approver who cannot tell whether their approval
completed the threshold cannot tell whether to expect anything to happen.

### Correlation

Almost nothing in this document joins to anything else, and that is a deliberate
result rather than a happy accident. `challenge` is per-request and single-use;
`payloadDigest` is salted with that challenge, so approving the same operation twice
produces two unrelated digests; `decision` is an enum; the remaining member is prose.
As a binding, this is close to the minimum that could work, and the minimum is also
the least correlatable.

The joinable thing is the signer, unavoidably. The whole design rests on the executor
taking the approver's identity from the **verified proof on this document** and never
from the transport session — see [*The proof is the
authorization*](#the-proof-is-the-authorization) — which means every approval and every
denial an individual ever makes is signed by them, non-repudiably, and lands in one
place. The executor therefore accumulates an attributed history of one person's
judgements. That is the correct trade and it is what makes a relayed decision sound
when it passes through the requester's untrusted hands, but it should be recognised for
what it is rather than discovered later.

Both parties declare `identifierScope: pairwise`, and here that declaration does real
work. Every check in the *Conformance* pipeline is internal to one enrolment: the
executor asserts the proven signer is a member of *the approver set it named*, and the
device answers only an executor *it is enrolled with*. Neither check improves if the
identifiers are recognisable to strangers. Given that this document is a durable,
signed statement about a named human's decisions, a publicly recognisable approver
identifier would make that history attributable by anyone who came into possession of a
single decision — so pairwise identifiers are what keep the non-repudiability pointed
at the executor rather than at the world.

### Retention

Two lifetimes, and the split is the whole story.

The **exchange state is short**. On `deny` the executor deletes the pending request
outright, and a subsequent submit of the same task starts a fresh one. The grant issued
at the threshold is single-use and time-boxed. The `challenge` is consumed **at
execution rather than on receipt of this decision**, which is a correctness rule — it
lets an executor's own retry legitimately replay — and it also means the window closes
at execution or at the request's `expiresAt`, whichever comes first.

The **decision document is durable**, which is what the front matter declares and why.
Because the proof on it is the authorization, it is the only evidence that a privileged
operation — including, where the task was classified `destructive`, an irreversible one
— was agreed to at all, and by which enrolled approver. An executor that discards it
has performed an unaccountable act. Note that re-evaluating enrolment at execution does
not change this: revoking a device stops its *future* approvals from being honoured and
leaves its past ones standing in the record, because revocation is a change of
authority and not a retraction of history.

What an executor **SHOULD NOT** keep alongside it is the request it rendered. The
decision carries a digest; the request carries `effects[].before` and
`effects[].after`, which are the `subject`'s data and were shown for one decision. The
durable record that makes an authorization accountable needs the challenge, the digest,
the decision, and the proof — not the diff. Keeping the two together turns an audit
trail into a data store, which is the failure this separation exists to avoid.

### Consent/purpose

The data moves for one narrow reason: so that an executor can establish that a
specific, currently-enrolled human agreed to a specific payload, and can refuse to act
if any part of that fails. Every member serves it — the echoes bind the answer to the
question, the enum is the answer, and the proof attributes it. Nothing is collected
here that is not consumed by one of the eleven checks in *Conformance*.

The limit is on what the accumulated record may then be used for. It is authorization
evidence, not performance data. Scoring approvers on how fast they respond or how often
they agree, and — worse — using that history to choose *which* approver to route a
request to in order to obtain an approval, are uses no approver assented to when they
answered a prompt. The second is not merely a privacy problem but an attack on the
mechanism itself: routing for agreeableness is how `minApprovals` and
`excludeRequester` are defeated in practice, without a single signature failing to
verify. It is the same reasoning that puts [*Consent fatigue*](#consent-fatigue) in
this specification — the thing being protected is a human's genuine assent, and every
optimisation that makes assent easier to obtain erodes it.

Finally, and per [SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 13: this
specification describes what a decision *is*, never that one is required. Whether a
given task must be approved is a consumer's policy decision, expressed through
[`policy/evaluate`](../../../policy/evaluate/0.3/spec.md). This specification does
constrain the mechanism in one direction only — an executor **MUST NOT** gate this task
behind the very consent mechanism it implements, because a decision that itself
required a decision would not terminate.
