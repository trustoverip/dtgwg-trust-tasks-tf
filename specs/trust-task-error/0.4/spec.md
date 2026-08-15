---
slug: trust-task-error
version: "0.4"
title: Trust Task Error
summary: The framework-defined response type a consumer returns when it cannot or will not act upon a received Trust Task document.
status: draft
targetFrameworkVersion: "0.4"
category: framework
keywords:
  - error
  - response
  - failure
  - framework
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Reporting consumer
    requirement: REQUIRED
    member: issuer
  - role: Original producer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: An error response is typically short-lived and consumed over the same transport that delivered the failed request, but a proof becomes necessary when the failure will be retained as evidence (for example, a compliance refusal) or replayed by intermediaries.
sideEffects:
  level: none
  rationale: "A failure report returned in place of a result; changes no recipient state."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related: []
---

## Abstract

The **Trust Task Error** is the framework-defined response a *consumer* returns when it cannot or will not act upon a received *Trust Task document*. It is itself a *Trust Task document*, signed and validated by the same pipeline that handles successful tasks, so an implementation does not need a parallel "error path".

This specification is the registry publication of the type reserved at [SPEC.md §8](../../../SPEC.md#8-error-responses). It defines the canonical `payload` shape carried by every framework-conformant error response, enumerates the standard error codes consumers **MUST** recognize, and describes how individual *Trust Task specifications* extend the code set ([SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications)).

A `trust-task-error` document is itself a *response*. It does not have a `#response` variant of its own.

**New in 0.4:** the `idConflict` standard code, reporting a document whose `id` matches one the consumer has already accepted but whose content differs. It exists to keep that case distinguishable from a retry: [SPEC.md §8.4](../../../SPEC.md#84-retry-semantics) defines a retry as re-sending the bit-for-bit identical document, and [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) item 11 requires a consumer to absorb such a resend without executing a consequential effect twice. A document that reuses an `id` over *different* bytes is not that, and silently treating it as a retry would suppress an execution the producer may well have intended. See [Standard codes](#standard-codes).

**New in 0.3:** the optional `inResponseTo` member, which names the document the error reports on. Without it an error response is correlated only by `threadId`, which is meaningful to a party that saw the originating request and to nobody else — so a retained error names neither the task it terminated nor the instance. See [Identifying what failed](#identifying-what-failed).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the consumer of the original task, now reporting failure) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-error/0.4`, with itself as `issuer` and the original task's producer as `recipient`.
2. Set the document's `threadId` to the originating document's `threadId` if one was carried, or to the originating document's `id` otherwise, per [SPEC.md §4.9](../../../SPEC.md#49-the-threadid-member). The error document's own `id` **MUST NOT** reuse the originating document's `id`.
3. Populate `payload.code` with one of the standard codes below or a slug-namespaced extension per [SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications).
4. Populate `payload.retryable` with a boolean reflecting whether retrying the original document is expected to succeed.
5. Include a `proof` member where the failure is intended to be retained or replayed beyond the original transport.

A conforming **consumer** (the original producer, receiving the failure) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Apply retry semantics per [SPEC.md §8.4](../../../SPEC.md#84-retry-semantics): **MUST NOT** re-send the original document when `retryable` is `false`; **SHOULD** wait until any `retryAfter` value and apply transport-appropriate backoff when `retryable` is `true`.
3. Recognize every standard code listed below. Unrecognized extended codes **SHOULD** be treated as `taskFailed`, but `retryable` and `retryAfter` **MUST** still be honored.

A failure that **arises while handling** a `trust-task-error` document (for example, an error document with a malformed payload) is itself reported with another `trust-task-error` whose `code` is `malformedRequest`; this is not recursion in practice because the inner document is small and well-formed by definition once it reaches the consumer.

## Definitions

* **Reporting consumer.** The party emitting the error response; identified by `issuer`. Typically the consumer of the original task that could not be acted upon.
* **Original producer.** The party that emitted the failed task; identified by `recipient`. Receives the error.

## Standard codes

The codes below are normative; every conforming consumer **MUST** recognize them. The "Default `retryable`" column gives the value an emitter **SHOULD** use unless task-specific knowledge dictates otherwise; the value actually carried on a given document is authoritative.

| Code | Meaning | Default `retryable` |
|---|---|---|
| `malformedRequest` | The document did not validate against the framework schema or the task-specific payload schema. | `false` |
| `unsupportedType` | The consumer does not recognize the `type` URI. | `false` |
| `unsupportedVersion` | The `type` URI was recognized but its `MAJOR.MINOR` version is not supported. | `false` |
| `expired` | The document's `expiresAt` was in the past at the time of evaluation. | `false` |
| `proofRequired` | A `proof` was required (by the *Trust Task specification* or consumer policy) and was missing. | `false` |
| `proofInvalid` | A `proof` was present but failed verification. | `false` |
| `permissionDenied` | The requesting party is not authorized to invoke this task. | `false` |
| `wrongRecipient` | The document's `recipient` does not identify the receiving consumer. | `false` |
| `identityMismatch` | An in-band `issuer` or `recipient` value is inconsistent with the corresponding transport-authenticated identity. | `false` |
| `idConflict` | The document's `id` matches one the consumer has already accepted, but its content differs. | `false` |
| `taskFailed` | The consumer attempted the task and could not complete it; further detail **SHOULD** appear in `details`. | varies |
| `unavailable` | The consumer is temporarily unable to process the task. | `true` |
| `internalError` | The consumer encountered an unexpected internal failure. | `true` |

See [SPEC.md §8.3](../../../SPEC.md#83-standard-error-codes) for the authoritative version of this table.

## Identifying what failed

An *error response* correlates back to the document it reports on by `threadId` ([SPEC.md §4.9](../../../SPEC.md#49-the-threadid-member)). That is sufficient between the two parties to an exchange: the *producer* knows what it sent.

It is not sufficient for anyone else. A `threadId` is opaque, and the error payload names neither the *Trust Task specification* the failure occurred under nor the document instance that triggered it. A party handed a retained error — a verifier evaluating it as evidence, an auditor reconstructing a sequence, an operator reading a log — sees `{"code": "taskFailed", "retryable": false}` and cannot tell what failed. For an extended code the *slug* namespace ([SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications)) hints at the family, but for any of the standard codes in [§8.3](../../../SPEC.md#83-standard-error-codes) there is no signal at all.

`inResponseTo` closes that. A *reporting consumer*:

- **SHOULD** include `inResponseTo.typeUri`, carrying the `type` of the document being reported on, **including any `#request` or `#response` fragment it carried**. This is what tells a consumer which specification's semantics apply to an extended `code`, and which specification declared the requirement that was breached.
- **SHOULD** include `inResponseTo.id`, carrying that document's `id`. Because [§4.3](../../../SPEC.md#43-the-id-member) makes an `id` globally unique and never reused, it names one instance where `threadId` names an exchange.
- **MUST** include both where the error is intended to be retained, replayed, or relied upon by parties beyond the immediate exchange — the same condition under which [§4.7.1](../../../SPEC.md#471-when-to-include-a-proof) makes a `proof` mandatory. An attributable-but-unidentifiable error is not evidence of anything.

Both members are optional in this version so that a `0.2` consumer's output remains valid `0.3`. That is a migration allowance, not a design position: a future **major** version is expected to require them, and `0.1` and `0.2` to be retired once consumers have moved.

### Disclosure

`inResponseTo` echoes values the *original producer* already chose and sent, so returning them to that producer discloses nothing new. Two cases need care:

- Under `identityMismatch` the response is addressed to the transport-authenticated sender rather than the in-band `issuer` ([SPEC.md §8.1](../../../SPEC.md#81-the-trust-task-error-specification)). That party is not necessarily the one that composed the document, so a *consumer* **SHOULD** omit `inResponseTo.id` in that case; the `typeUri` alone is not identifying.
- Where an error is forwarded beyond the original producer, the `typeUri` reveals which task was attempted. A *consumer* that treats the attempted task as sensitive **MAY** omit the member entirely rather than emit a partly-populated one.

## Extending the code set

An individual *Trust Task specification* **MAY** define additional codes specific to its task. Extended codes **MUST** be namespaced with the specification's `<slug>` followed by a colon and a snake_case local name — for example, `acl/grant:roleNotRecognized`. Extended codes **MUST NOT** shadow any code in the standard table above. See [SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications).

A consumer that does not recognize an extended code **SHOULD** treat the failure as if its code were `taskFailed`, and **MUST** still honor the `retryable` and `retryAfter` members.

## Examples

### A basic failure

The consumer rejects a document that has expired:

```json
{
  "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.4",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-05-16T14:22:00Z",
  "payload": {
    "code": "expired",
    "message": "Task expired at 2026-04-12T09:31:00Z.",
    "retryable": false
  }
}
```

### Naming the document that failed

```json
{
  "id": "urn:uuid:1c9b64de-7f0a-4a2e-9c31-8b5d0f2a6e41",
  "type": "https://trusttasks.org/spec/trust-task-error/0.4",
  "threadId": "urn:uuid:4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-08-08T14:22:00Z",
  "payload": {
    "code": "proofRequired",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/acl/grant/0.1",
      "id": "urn:uuid:4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2"
    },
    "message": "specification declares proof REQUIRED and the document carried none",
    "retryable": false
  }
}
```

A party holding only this document knows which specification was attempted, which instance failed, and why — none of which the same error carries without `inResponseTo`.

### Retryable, with `retryAfter`

The consumer is temporarily unable to process the task and asks the producer to wait:

```json
{
  "id": "1c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.4",
  "threadId": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-06-01T09:00:00Z",
  "payload": {
    "code": "unavailable",
    "message": "Scheduled maintenance window; retry after the timestamp below.",
    "retryable": true,
    "retryAfter": "2026-06-01T11:00:00Z"
  }
}
```

### Task-specific extended code with `details`

A KYC-related ACL grant fails because a breeder document used in the underlying verification was revoked after the fact. The maintainer reports this with a slug-namespaced extended code and a `details` object whose shape is defined by the originating specification:

```json
{
  "id": "c4d2f713-9a8e-4d04-b29c-2f1b0b4cbe71",
  "type": "https://trusttasks.org/spec/trust-task-error/0.4",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-05-16T14:30:00Z",
  "payload": {
    "code": "kyc-handoff:documentRevoked",
    "message": "Passport used in verification was revoked by the issuing authority on 2026-05-10.",
    "retryable": false,
    "details": {
      "documentRef": "urn:passport:NL:XYZ123456",
      "revokedAt": "2026-05-10T08:00:00Z"
    }
  }
}
```

A consumer that does not implement `kyc-handoff` would treat this as `taskFailed` and still honor `retryable: false`.

### Carrying a proof for retention

Where the failure will be retained as evidence (for example, a compliance refusal that needs to survive the original transport), the maintainer signs the error document:

```json
{
  "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.4",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-05-16T14:22:00Z",
  "payload": {
    "code": "permissionDenied",
    "message": "Sender is not on this institution's accepted-verifier list.",
    "retryable": false
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:bank.example#key-1",
    "created": "2026-05-16T14:22:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z58D..."
  }
}
```

## Security & Privacy

An error document **MAY** carry sensitive context in `message` or `details` — for example, the name of an internal service, the identifier of a revoked credential, or the reason a particular party was denied access. Producers **SHOULD** apply the principle of least disclosure: include only what the original producer needs to understand the failure and take next-step action.

Implementations **SHOULD** include a `proof` member where the error will be retained, replayed, or relied upon by parties beyond the immediate exchange. Without a proof, a retained error document cannot be attributed to the reporting consumer after the fact and offers no integrity guarantee against tampering by intermediaries.

`retryAfter` is advisory, not enforcement. A producer that ignores `retryAfter` will simply receive another `trust-task-error` (typically with `code: unavailable`) — consumers **SHOULD NOT** rely on `retryAfter` as a rate limit.
