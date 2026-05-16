---
slug: trust-task-error
version: "0.1"
title: Trust Task Error
summary: The framework-defined response type a consumer returns when it cannot or will not act upon a received Trust Task document.
status: draft
targetFrameworkVersion: "0.1"
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
  - role: Original producer
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: An error response is typically short-lived and consumed over the same transport that delivered the failed request, but a proof becomes necessary when the failure will be retained as evidence (for example, a compliance refusal) or replayed by intermediaries.
errorCodes: []
related: []
---

## Abstract

The **Trust Task Error** is the framework-defined response a *consumer* returns when it cannot or will not act upon a received *Trust Task document*. It is itself a *Trust Task document*, signed and validated by the same pipeline that handles successful tasks, so an implementation does not need a parallel "error path".

This specification is the registry publication of the type reserved at [SPEC.md §8](../../../SPEC.md#8-error-responses). It defines the canonical `payload` shape carried by every framework-conformant error response, enumerates the standard error codes consumers **MUST** recognize, and describes how individual *Trust Task specifications* extend the code set ([SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications)).

A `trust-task-error` document is itself a *response*. It does not have a `#response` variant of its own.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the consumer of the original task, now reporting failure) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-error/0.1`, with itself as `issuer` and the original task's producer as `recipient`.
2. Set the document's `threadId` to the originating document's `threadId` if one was carried, or to the originating document's `id` otherwise, per [SPEC.md §4.9](../../../SPEC.md#49-the-threadid-member). The error document's own `id` **MUST NOT** reuse the originating document's `id`.
3. Populate `payload.code` with one of the standard codes below or a slug-namespaced extension per [SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications).
4. Populate `payload.retryable` with a boolean reflecting whether retrying the original document is expected to succeed.
5. Include a `proof` member where the failure is intended to be retained or replayed beyond the original transport.

A conforming **consumer** (the original producer, receiving the failure) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Apply retry semantics per [SPEC.md §8.4](../../../SPEC.md#84-retry-semantics): **MUST NOT** re-send the original document when `retryable` is `false`; **SHOULD** wait until any `retryAfter` value and apply transport-appropriate backoff when `retryable` is `true`.
3. Recognize every standard code listed below. Unrecognized extended codes **SHOULD** be treated as `task_failed`, but `retryable` and `retryAfter` **MUST** still be honored.

A failure that **arises while handling** a `trust-task-error` document (for example, an error document with a malformed payload) is itself reported with another `trust-task-error` whose `code` is `malformed_request`; this is not recursion in practice because the inner document is small and well-formed by definition once it reaches the consumer.

## Definitions

* **Reporting consumer.** The party emitting the error response; identified by `issuer`. Typically the consumer of the original task that could not be acted upon.
* **Original producer.** The party that emitted the failed task; identified by `recipient`. Receives the error.

## Standard codes

The codes below are normative; every conforming consumer **MUST** recognize them. The "Default `retryable`" column gives the value an emitter **SHOULD** use unless task-specific knowledge dictates otherwise; the value actually carried on a given document is authoritative.

| Code | Meaning | Default `retryable` |
|---|---|---|
| `malformed_request` | The document did not validate against the framework schema or the task-specific payload schema. | `false` |
| `unsupported_type` | The consumer does not recognize the `type` URI. | `false` |
| `unsupported_version` | The `type` URI was recognized but its `MAJOR.MINOR` version is not supported. | `false` |
| `expired` | The document's `expiresAt` was in the past at the time of evaluation. | `false` |
| `proof_required` | A `proof` was required (by the *Trust Task specification* or consumer policy) and was missing. | `false` |
| `proof_invalid` | A `proof` was present but failed verification. | `false` |
| `permission_denied` | The requesting party is not authorized to invoke this task. | `false` |
| `wrong_recipient` | The document's `recipient` does not identify the receiving consumer. | `false` |
| `identity_mismatch` | An in-band `issuer` or `recipient` value is inconsistent with the corresponding transport-authenticated identity. | `false` |
| `task_failed` | The consumer attempted the task and could not complete it; further detail **SHOULD** appear in `details`. | varies |
| `unavailable` | The consumer is temporarily unable to process the task. | `true` |
| `internal_error` | The consumer encountered an unexpected internal failure. | `true` |

See [SPEC.md §8.3](../../../SPEC.md#83-standard-error-codes) for the authoritative version of this table.

## Extending the code set

An individual *Trust Task specification* **MAY** define additional codes specific to its task. Extended codes **MUST** be namespaced with the specification's `<slug>` followed by a colon and a snake_case local name — for example, `acl/grant:role_not_recognized`. Extended codes **MUST NOT** shadow any code in the standard table above. See [SPEC.md §8.5](../../../SPEC.md#85-extension-by-individual-trust-task-specifications).

A consumer that does not recognize an extended code **SHOULD** treat the failure as if its code were `task_failed`, and **MUST** still honor the `retryable` and `retryAfter` members.

## Examples

### A basic failure

The consumer rejects a document that has expired:

```json
{
  "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
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

### Retryable, with `retryAfter`

The consumer is temporarily unable to process the task and asks the producer to wait:

```json
{
  "id": "1c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
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
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-05-16T14:30:00Z",
  "payload": {
    "code": "kyc-handoff:document_revoked",
    "message": "Passport used in verification was revoked by the issuing authority on 2026-05-10.",
    "retryable": false,
    "details": {
      "documentRef": "urn:passport:NL:XYZ123456",
      "revokedAt": "2026-05-10T08:00:00Z"
    }
  }
}
```

A consumer that does not implement `kyc-handoff` would treat this as `task_failed` and still honor `retryable: false`.

### Carrying a proof for retention

Where the failure will be retained as evidence (for example, a compliance refusal that needs to survive the original transport), the maintainer signs the error document:

```json
{
  "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-05-16T14:22:00Z",
  "payload": {
    "code": "permission_denied",
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
