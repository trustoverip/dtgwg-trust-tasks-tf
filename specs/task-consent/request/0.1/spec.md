---
slug: task-consent/request
version: "0.1"
title: Task Consent — Request
summary: An executor asks an enrolled approver device to authorize one pending privileged task, presenting the effects it computed by dry-running the real handler against its own prior state.
status: draft
targetFrameworkVersion: "0.5"
category: consent
keywords:
  - consent
  - delegated-execution
  - approval
  - policy
  - step-up
  - effects
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Executor (Verifiable Trust Agent)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Approver device
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: The proof is what makes the effects trustworthy. A consent surface renders `effects` as the basis of a human's decision, so an unsigned request would let anyone who can reach the device — including the relying party whose task is being approved — author the prose the human reads while every downstream signature still verified. The approver MUST refuse a request it cannot verify against its own executor.
sideEffects:
  level: none
  rationale: Raising an approval prompt persists no state on the approver device; the pending request lives at the executor.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Discloses to the approver which task is pending, the subject it acts on, the requesting origin, and the executor-computed effects — descriptive data about a pending operation, but no secret material and no authority exercised. Inbound, the same document delivers to the approver device a description of another party's activity: `subject`, `requester`, `requesterDeviceId`, `origin`, the requester-authored `note`, and — because `Effect.before` and `Effect.after` are untyped — whatever values sat at the changed path in the executor's authoritative state and whatever will replace them. Where the pending task touches personal data, the diff carries it to the approver's screen."
retention:
  class: exchange
  rationale: "The approver device holds the request only as long as the pending approval it belongs to. `sideEffects` is `none` — raising a prompt persists nothing on the device — and the copy exists so a prompt can be rendered and so `challenge` and `payloadDigest` can be echoed verbatim by the matching `task-consent/decision`. `expiresAt` closes the exchange: after that instant the request lapses and no decision is accepted for it. The executor holds the pending request slightly longer, because the challenge is consumed at execution rather than on receipt of the decision."
subjectPath: /subject
errorCodes:
  - code: task-consent/request:untrustedIssuer
    meaning: The request was not signed by an executor this device is enrolled with. The device MUST NOT prompt.
    retryable: false
  - code: task-consent/request:notEligible
    meaning: This device is not a member of the named `approverSet`, or is the `requester` while `excludeRequester` is set.
    retryable: false
  - code: task-consent/request:noSurface
    meaning: The device has no consent surface available (headless, locked, or backgrounded past its wake budget).
    retryable: true
related:
  - task-consent/decision
  - task-consent/granted
  - policy/evaluate
  - push/wake
---

## Abstract

The **Task Consent — Request** Trust Task carries one pending privileged task to
a human. It is the first half of the delegated-execution consent flow; the
approver's answer returns as a
[`task-consent/decision/0.1`](../../decision/0.1/spec.md).

It exists because of a gap that a payload alone cannot close. When a policy
returns `requireConsent` (see
[`PolicyDecision`](../../../policy/_shared/0.3/policy.schema.json)), *something*
must show a person what they are agreeing to. The obvious candidate — the
payload the requester submitted — is exactly the wrong one: it is authored by
the least trusted party in the system, and it does not contain the task's
consequences.

## The executor is the authority

A payload says what was *asked for*. Only the code about to run knows what will
*happen*, and it knows it only against state the requester cannot see.

Consider a `did:webvh` document update whose payload adds one service endpoint.
The handler's own semantics rotate the DID's update keys and refresh its
pre-rotation commitments as a parallel consequence. A consent surface rendering
a naive diff of the submitted payload shows a one-line service-endpoint
addition and **silently hides a key rotation**. The consequence lives in the
handler, not in the payload's shape, so no amount of schema-driven rendering
recovers it.

Hence the two rules this task exists to enforce:

1. **`effects` MUST be produced by dry-running the real handler** — the same
   code path that will execute — against the executor's own authoritative prior
   state. An executor **MUST NOT** compute `effects` from a parallel
   implementation that describes what the handler does. A second implementation
   drifts, and when it drifts the human is confidently misinformed while every
   signature still verifies. This is `plan` and `apply` sharing one code path,
   except that a human signs the digest of the plan and `apply` refuses to run
   against a plan it did not produce.

2. **`sideEffects` and `exposure` MUST be derived from the compiled handler**,
   never from the registry entry for `taskType`. If the registry decided which
   operations required approval it would be a consent kill-switch, and a
   downgradeable one: publish a `1.1` declaring `sideEffects: none` and consent
   evaporates for every consumer resolving by URI. Registry metadata is for
   rendering; the executing code is for policy. Where the two disagree the
   executor's value wins and the executor SHOULD log the divergence.

## The requester's note

`note` is the one deliberate exception to the rule that every word the human
reads was authored by the executor. It exists because a requester sometimes has
context no executor can compute — "migrating to the new mediator, ticket
OPS-441" — and because the alternative to carrying that context here is a
second, weaker approval family that carries *only* requester prose (this field
absorbs the one legitimate use of the retired
[`confirm/request/0.1`](../../../confirm/request/0.1/spec.md)).

The quarantine rules keep the exception from swallowing the design:

- `note` is **display text, not a statement of effects**. It is authored by the
  least trusted party in the system; nothing in it is verified by anyone.
- The executor carries it **verbatim or not at all** — never edited, never
  summarised — so its signature attests provenance ("the requester said this"),
  never truth.
- A surface renders it **attributed and visually separate** from `effects`, and
  a human decision is based on the effects. A note that contradicts the effects
  is a red flag to display, not a discrepancy to reconcile.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the executor) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/task-consent/request/0.1`, with itself as `issuer` and the approver device as `recipient`, carrying a verifiable `proof`.
2. Validate the pending task's payload against its schema **before** computing effects, so that nothing rides along outside what is rendered. Payload schemas are closed (`additionalProperties: false`); an executor **MUST** reject an unvalidated payload rather than consent to it.
3. Populate `effects` by dry-running the handler it will invoke, against the state identified by `statePin`. Where it has no dry-run for that handler, it **MUST** leave `effects` empty and **SHOULD** populate `consequences` from the task's specification.
4. Populate `sideEffects` and `exposure` from its compiled dispatch table, not from the registry.
5. Compute `payloadDigest` over the canonical payload, the task type, and `challenge` as salt (see *Binding*, below).
6. Generate `challenge` with ≥128 bits of entropy, distinct per request.
7. Populate `origin`, when the task arrived from a relying party, with the origin its **own runtime attested** — never a value the proposing page supplied.
8. Populate `note`, when the requester supplied one, **verbatim** — the executor **MUST NOT** author, edit, or summarise it, and **MAY** omit or truncate it. Signing this document attests that the requester supplied that text, never that it is true (see *The requester's note*, below).
9. Re-evaluate policy **and** the approver's enrolment at execution, not only when this request was minted (see *Time of check*, below).

A conforming **consumer** (the approver device) **MUST**:

1. Verify the `proof` and that the `issuer` is an executor it is enrolled with. An unverifiable request → `untrustedIssuer`; the device **MUST NOT** prompt.
2. Render **only** members of this verified document. With the single, explicitly-quarantined exception of `note`, it **MUST NOT** render, and **MUST NOT** allow the requester or the `origin` to contribute, any prose the human reads as the basis of the decision.
3. Render every `effects[].summary` verbatim, including for a `kind` it does not recognise. It **MAY** additionally render structured members of kinds it knows. A surface that silently drops an unrecognised effect misinforms the human precisely where the design is weakest.
4. Where `effects` and `consequences` are **both** empty, tell the approver the consequences could not be determined. It **MUST NOT** present the task as though it had none.
5. Render `note`, when it renders it at all, attributed to `requester` and visually distinct from `effects`. It **MUST NOT** present `note` as a statement of what the task does, and **MUST NOT** let it substitute for, reorder, or obscure any effect. A surface **MAY** drop `note` entirely; it **MUST NOT** drop an effect.
6. Refuse to prompt when `expiresAt` has passed (`expired`), when it is not a member of `approverSet`, or when it is the `requester` and `excludeRequester` is set (`notEligible`).
7. Return a `#response` with `status: prompted` or `status: refused`. The human's answer is **not** a synchronous reply — it returns as a separate `task-consent/decision`.

A conforming consumer **SHOULD**, for a `sideEffects: destructive` task, require the human to **match** a prefix of `payloadDigest` against the same prefix displayed by the requesting surface, rather than to tap "approve". Only a comparison across two independent screens survives a compromised consent surface; a tap is a reflex, and a reflex is what habituation destroys first.

> **Note (non-normative).** The reference ecosystem signs this document with the `eddsa-jcs-2022` Data Integrity cryptosuite and `proofPurpose: assertionMethod`, as the examples show. This is an implementation profile, not a requirement of this specification: [SPEC.md §4.7](/SPEC.md#47-proof) leaves the choice of cryptosuite open, and any registered suite whose `verificationMethod` resolves to material controlled by the `issuer` satisfies the `proof` requirement.

## Binding — what the human approved is what executes

Five checks, of which only the last survives a compromised approver device.

1. **Salted, type-bound digest.** `payloadDigest` is computed over the canonical (RFC 8785 JCS) payload, the `taskType`, and the `challenge` as salt. JCS so the digest is stable across serializers. The **type** is bound because two tasks whose payloads canonicalize identically would otherwise share a digest, and an approval for a benign task would authorize a destructive one — invisibly, since the approver sees only the digest. The **salt** is there because an unsalted digest over a low-entropy payload ("deactivate `did:webvh:abc…`" has essentially one serialization) is a confirmation oracle for anyone who observes it in transit.
2. **State pinning.** `effects` are computed against the state named in `statePin`, which the executor asserts still holds at execution. A human in the loop makes the approval window minutes wide, so a lost update is a real risk rather than a theoretical one.
3. **Closed payloads.** The pending task's payload is schema-validated before effects are computed, so nothing rides along outside the rendered effects.
4. **Single-use challenge**, consumed at execution rather than at receipt of the decision — a decision authorizes exactly one execution.
5. **Cross-device digest matching** for `destructive` tasks.

Checks 1–4 assume an honest device and defeat a hostile relying party. Only check 5 defeats a hostile *device*, because only it moves the comparison into the human's head across two screens. It is also why `excludeRequester` exists, and why approval routing must be able to target a device other than the one that proposed.

## Time of check, time of use

Policy is evaluated when this request is minted. The task executes after a human
has looked at it — minutes later. An executor **MUST** re-evaluate the policy
decision **and** the approver's enrolment status at execution time. Otherwise
revoking a compromised device does not stop an approval already in flight from
it, and a policy tightened during the window is never applied. The state pin
already does this for the data; nothing else does it for the authorization.

## Payload

`payload.challenge` (REQUIRED) — ≥128-bit nonce; also the digest salt.
`payload.taskType` (REQUIRED) — Type URI of the task awaiting approval.
`payload.payloadDigest` (REQUIRED) — the binding; echoed by the decision.
`payload.sideEffects` (REQUIRED) — authoritative class, from the compiled handler.
`payload.exposure` (REQUIRED) — authoritative exposure class, likewise.
`payload.effects` (REQUIRED, MAY be empty) — executor-authored consequences.
`payload.consequences` (OPTIONAL) — the specification's static fallback text.
`payload.subject` (OPTIONAL) — the identifier the task acts on.
`payload.requester` (REQUIRED) — the DID that submitted the task.
`payload.requesterDeviceId` (OPTIONAL) — the device it was submitted from.
`payload.origin` (OPTIONAL) — runtime-attested origin of the proposing page.
`payload.note` (OPTIONAL) — requester-authored display text, explicitly untrusted; never a statement of effects.
`payload.statePin` (OPTIONAL) — the prior state effects were computed against.
`payload.approverSet` (REQUIRED) — the set named by the policy.
`payload.minApprovals` (REQUIRED) — approvals required.
`payload.excludeRequester` (REQUIRED) — whether the requester may self-approve.
`payload.expiresAt` (REQUIRED) — when the pending request lapses.
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Examples

### A DID document update that also rotates the update keys

The payload adds one service endpoint. The second and third effects are the
point: they are invisible in the payload and would be invisible in any diff the
requester could compute.

## Request

```json
{
  "id": "urn:uuid:9f2c1e70-6f0a-4a1b-9a3f-7c2b1d5e8a44",
  "type": "https://trusttasks.org/spec/task-consent/request/0.1",
  "issuer": "did:key:z6MkExecutorVtaExample",
  "recipient": "did:key:z6MkApproverPhoneExample",
  "issuedAt": "2026-07-13T09:41:00Z",
  "payload": {
    "challenge": "9c1f4b7a2e6d80f35a4c9b1e7d2f6083",
    "taskType": "https://trusttasks.org/spec/webvh/dids/update/1.0",
    "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
    "sideEffects": "mutating",
    "exposure": {
      "discloses": "none",
      "actsAsSubject": false
    },
    "subject": "did:webvh:QmSCIDExample:example.com:acme",
    "requester": "did:key:z6MkRequesterBrowserExample",
    "requesterDeviceId": "dev-8f21c0",
    "origin": "https://control.example.com",
    "note": "Adding the FileStore endpoint for the Q3 files migration (ticket OPS-441).",
    "statePin": {
      "resource": "did:webvh:QmSCIDExample:example.com:acme",
      "version": "3-QmPriorEntryHashExample"
    },
    "effects": [
      {
        "kind": "documentChange",
        "summary": "Adds a FileStore service endpoint at #files. There is no prior value at this path.",
        "path": "/service/0",
        "after": {
          "id": "#files",
          "type": "FileStore",
          "serviceEndpoint": "https://files.example.com/acme"
        }
      },
      {
        "kind": "keyRotation",
        "summary": "Rotates this DID's update key — any document change rotates it. The current update key stops being able to authorize changes.",
        "before": ["z6MkfrQCurrentUpdateKeyExample"],
        "after": ["z6MkpB2NextUpdateKeyExample"]
      },
      {
        "kind": "preRotationRefresh",
        "summary": "Refreshes 2 pre-rotation commitments for the next rotation.",
        "detail": { "commitments": 2 }
      }
    ],
    "approverSet": "operators",
    "minApprovals": 1,
    "excludeRequester": true,
    "expiresAt": "2026-07-13T09:56:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-13T09:41:00Z",
    "verificationMethod": "did:key:z6MkExecutorVtaExample#z6MkExecutorVtaExample",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQmV6ExampleProofValueForTaskConsentRequest"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:1d7e4c05-3b8f-49a2-8c61-0e5f2a9d3b17",
  "type": "https://trusttasks.org/spec/task-consent/request/0.1#response",
  "issuer": "did:key:z6MkApproverPhoneExample",
  "recipient": "did:key:z6MkExecutorVtaExample",
  "issuedAt": "2026-07-13T09:41:02Z",
  "payload": {
    "status": "prompted"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-13T09:41:02Z",
    "verificationMethod": "did:key:z6MkApproverPhoneExample#z6MkApproverPhoneExample",
    "proofPurpose": "assertionMethod",
    "proofValue": "z58aKqExampleProofValueForTaskConsentRequestResponse"
  }
}
```

## Security & Privacy

### Data carried

Nearly every design decision in this specification is about making sure the human
reads something *trustworthy* — see [*The executor is the
authority*](#the-executor-is-the-authority) and [*The requester's
note*](#the-requesters-note). That is an integrity property, and it is worth being
clear that it is not a confidentiality one: the two come apart here, and this
section is about the half the rest of the document does not cover.

What arrives on the approver's device is a rounded description of an operation
somebody else is performing. `subject` names the identifier being acted on —
whose thing is about to change. `requester` and `requesterDeviceId` name who asked
and from which enrolled device. `origin` names the web origin that proposed it, as
the executor's own runtime attested it. `taskType` names the kind of operation.
Together those five members are a fairly complete account of one person's activity,
delivered to a second person because policy routed it there.

The widest members are `effects[]`. `summary` is executor-authored prose, and it is
the only member a surface is obliged to render. `before` and `after`, however, are
**untyped** in the shared schema — they are whatever value sat at `path` in the
executor's authoritative prior state, and whatever will replace it — and `detail` is
an open object. If the task under approval touches personal data, the diff carries
that data to the approver's screen verbatim. This is not a defect to be fixed: a
consent surface that concealed the values would be showing the human less than the
thing they are authorizing, which is the failure mode the whole specification exists
to prevent. But it does mean an approver is routinely shown data belonging to a
subject who is not the approver, and both members are OPTIONAL. An executor
**SHOULD** write `summary` so that an approver who can decide from the summary alone
is not obliged to read the values, and **SHOULD** omit `before` and `after` where the
summary carries the decision. That is where minimisation lives in this task; there is
nothing else in the payload to trim, because the remaining members are the binding.

`note` is the exception to executor authorship and its quarantine is also its
minimisation: it is capped at 500 characters, attributed to `requester`, rendered
visually distinct, and never treated as a statement of effects. A producer
**MUST NOT** use it to carry material the approver would not otherwise be entitled
to see. The executor carries it verbatim and does not read it, so nothing between
the requester and the human's screen will notice if they do.

One member is a deliberate *reduction* in what is carried, and deserves credit as
such: `payloadDigest` is salted with `challenge`. As [*Binding*](#binding--what-the-human-approved-is-what-executes)
explains, an unsalted digest over a low-entropy payload — "deactivate `did:webvh:abc…`"
has essentially one serialization — is a confirmation oracle for anyone who observes
it in transit. The salt is what stops the digest disclosing the payload it commits to.

The response is `status` plus, when `status` is `refused`, a **REQUIRED** free-text
`reason` travelling from the device back to the executor. A device explaining *why*
it will not prompt should say which of the specification's refusal conditions
applied, not describe its own state; "locked" and "the user is asleep" are not the
same disclosure.

Two more members are free text and now carry the same 500-character bound as
`note`, for the same reason: 500 is what a person reads at a prompt.
`effects[].summary` is **REQUIRED** and executor-authored — it is the one member
a surface is obliged to render, so it is trusted exactly as far as the executor's
signature reaches, and an executor MUST write it to be decidable on its own.
`consequences[]` is the specification's own static fallback text, carried by the
executor when it has no dry-run; it describes the task type rather than this
request, and a surface MUST NOT present it as though it had been computed against
real state. Both are read only by the approver, and both are exchange-scoped on
the terms *Retention* sets out below — the bound is what stops a handler with a
long effect list turning one prompt into an unbounded document.

### Correlation

The salient correlation in this task is not between documents — it is the archive one
device accumulates. Every request an approver device receives names a requester, a
device, an origin, a subject, and a time. A device that is a member of a busy
`approverSet` therefore ends up holding a timeline of what a principal has been doing,
from where, and with which of their devices — assembled without any party choosing to
assemble it, and by a device whose only job was to display things.

The binding values are engineered not to contribute to that. `challenge` carries at
least 128 bits of entropy and is distinct per request, so nothing joins on it.
`payloadDigest` is salted with that challenge, which means two executions of an
identical payload produce *different* digests: the digest is a one-request commitment,
not a fingerprint of the operation, and it cannot be used to recognise the same task
recurring. Even the cross-device matching ritual for `destructive` tasks compares a
*prefix* of a salted digest across two screens, which is enough for a human to detect
substitution and not enough to become a durable handle.

What does join, and must, is the identity layer: `subject`, `requester`,
`requesterDeviceId`, `origin`, and `approverSet` are all stable. They have to be —
`excludeRequester` cannot be enforced without comparing the requester against the
approver, and routing to a named set is meaningless without stable device identity.
The specification buys the safety of self-approval prevention with the cost of
linkable device identifiers, and that is the right trade, but it is a trade.

Both parties declare `identifierScope: pairwise`, and given how much of a principal's
activity crosses this channel that is the more important of the two declarations. The
approver verifies that the `issuer` is an executor it is *enrolled with*; the executor
checks that a decision's signer is in the approver set *it* named. Both checks are
internal to one enrolment, and neither is improved by an identifier that a stranger
would recognise. A publicly recognisable identifier on either end would let an
observer of the transport attribute this stream of activity to a known party without
reading a single payload — which is precisely the information the salted digest was
designed to withhold.

### Retention

The approver device's copy is exchange-scoped. `sideEffects` is `none`: raising a
prompt persists nothing. The document is held so that a prompt can be rendered and so
that `challenge` and `payloadDigest` can be echoed verbatim in the matching
[`task-consent/decision`](../../decision/0.1/spec.md), and `expiresAt` closes the
window — after that instant the request has lapsed and no decision is accepted for it.

The consequence for a device implementer is specific: once the prompt is answered,
dismissed, or expired, the rendered content **SHOULD** be discarded. The reason is
`effects[].before` and `effects[].after`. Those values belong to the `subject`, they
were shown for one decision, and the ordinary conveniences of a mobile surface — a
notification history, a "recent approvals" list, a screenshot for support — silently
convert a momentary disclosure into a stored one, on a device the subject may not know
exists. A surface that wants an approval history **SHOULD** keep the fact of the
decision and the `payloadDigest`, not the diff.

The executor holds its pending request a little longer than the prompt does, because,
per [*Time of check, time of use*](#time-of-check-time-of-use), the challenge is
consumed at execution rather than on receipt of the decision. That is a correctness
requirement rather than a retention one, and it ends at execution or at `expiresAt`,
whichever comes first.

### Consent/purpose

The purpose this data is collected for is unusually legible, because the document
exists for nothing else. The payload the requester actually submitted is *not* sent;
the executor computes `effects` specifically so that a person can read them, and every
member here is either something a human needs in order to decide or something that
binds the decision to the thing decided. There is no secondary use built into the
shape.

The limit is the mirror of that. Material rendered so that a human can answer one
question is not material collected about the principal it concerns. An approver device
**SHOULD NOT** index, search, aggregate, or export requests for any purpose other than
answering them, and **MUST NOT** present `note` as a statement of what the task does —
a requester who learns that their prose reaches a human's screen unedited has an
incentive to use it for something other than context.

Whether an approval step exists at all is not this specification's to say. A
[`policy/evaluate`](../../../policy/evaluate/0.3/spec.md) decision of `requireConsent`
is what causes a request to be raised, and per
[SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 13 a *Trust Task
specification* **MUST NOT** declare that a consent or approval step is required. This
document describes the shape of the request a deployment sends *when it has chosen to
have one*, and takes no position on when that choice should be made.
