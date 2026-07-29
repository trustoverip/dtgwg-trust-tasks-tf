---
slug: task-consent/request
version: "0.1"
title: Task Consent — Request
summary: An executor asks an enrolled approver device to authorize one pending privileged task, presenting the effects it computed by dry-running the real handler against its own prior state.
status: draft
targetFrameworkVersion: "0.2"
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
  - role: Approver device
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The proof is what makes the effects trustworthy. A consent surface renders `effects` as the basis of a human's decision, so an unsigned request would let anyone who can reach the device — including the relying party whose task is being approved — author the prose the human reads while every downstream signature still verified. The approver MUST refuse a request it cannot verify against its own executor.
sideEffects:
  level: none
  rationale: Raising an approval prompt persists no state on the approver device; the pending request lives at the executor.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: Discloses to the approver which task is pending, the subject it acts on, the requesting origin, and the executor-computed effects — descriptive data about a pending operation, but no secret material and no authority exercised.
subjectPath: /subject
errorCodes:
  - code: task-consent/request:untrusted_issuer
    meaning: The request was not signed by an executor this device is enrolled with. The device MUST NOT prompt.
    retryable: false
  - code: task-consent/request:expired
    meaning: The request's `expiresAt` has passed; the device MUST NOT prompt.
    retryable: false
  - code: task-consent/request:not_eligible
    meaning: This device is not a member of the named `approverSet`, or is the `requester` while `excludeRequester` is set.
    retryable: false
  - code: task-consent/request:no_surface
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

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

1. Verify the `proof` and that the `issuer` is an executor it is enrolled with. An unverifiable request → `untrusted_issuer`; the device **MUST NOT** prompt.
2. Render **only** members of this verified document. With the single, explicitly-quarantined exception of `note`, it **MUST NOT** render, and **MUST NOT** allow the requester or the `origin` to contribute, any prose the human reads as the basis of the decision.
3. Render every `effects[].summary` verbatim, including for a `kind` it does not recognise. It **MAY** additionally render structured members of kinds it knows. A surface that silently drops an unrecognised effect misinforms the human precisely where the design is weakest.
4. Where `effects` and `consequences` are **both** empty, tell the approver the consequences could not be determined. It **MUST NOT** present the task as though it had none.
5. Render `note`, when it renders it at all, attributed to `requester` and visually distinct from `effects`. It **MUST NOT** present `note` as a statement of what the task does, and **MUST NOT** let it substitute for, reorder, or obscure any effect. A surface **MAY** drop `note` entirely; it **MUST NOT** drop an effect.
6. Refuse to prompt when `expiresAt` has passed (`expired`), when it is not a member of `approverSet`, or when it is the `requester` and `excludeRequester` is set (`not_eligible`).
7. Return a `#response` with `status: prompted` or `status: refused`. The human's answer is **not** a synchronous reply — it returns as a separate `task-consent/decision`.

A conforming consumer **SHOULD**, for a `sideEffects: destructive` task, require the human to **match** a prefix of `payloadDigest` against the same prefix displayed by the requesting surface, rather than to tap "approve". Only a comparison across two independent screens survives a compromised consent surface; a tap is a reflex, and a reflex is what habituation destroys first.

> **Note (non-normative).** The reference ecosystem signs this document with the `eddsa-jcs-2022` Data Integrity cryptosuite and `proofPurpose: assertionMethod`, as the examples show. This is an implementation profile, not a requirement of this specification: [SPEC.md §4.7](../../../../SPEC.md#47-proof) leaves the choice of cryptosuite open, and any registered suite whose `verificationMethod` resolves to material controlled by the `issuer` satisfies the `proof` requirement.

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
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

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
    "payloadDigest": "3b0c7f1d9e2a5648c1f30b7ae4d2986153ca0f7b8d41e6295af03c8bd71e4a62",
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
