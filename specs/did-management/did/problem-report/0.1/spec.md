---
slug: did-management/did/problem-report
version: "0.1"
title: DID Management — Problem Report
summary: A hosting service emits a problem report to an owner when an async background operation on the owner's slot fails after the original transport has closed.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, problem-report, error, async]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID hosting service
    requirement: REQUIRED
    member: issuer
  - role: DID owner
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A problem report is informational rather than evidentiary; the consumer's session usually authenticates the receiver, but a proof becomes valuable when the report is retained for ticket-tracking.
errorCodes: []
related: [did-management/did/publish]
---

## Abstract

The **DID Management — Problem Report** Trust Task is fire-and-forget. The hosting service emits a problem report to the slot's owner when a background operation (witness validation, registry fanout, sweep eligibility) fails or is rejected after the original request transport has already closed. The owner is expected to action the issue out-of-band; the report carries enough context (slot identifier, framework error code, free-text message, optional structured context) to triage.

Because this task is fire-and-forget, there is no `#response` document — the schema MUST NOT declare a `Response` `$def`.

## Status of this Document

Draft.

## Conformance

The producer (the hosting service) emits `type: https://trusttasks.org/spec/did-management/did/problem-report/0.1` with at minimum `payload.mnemonic`, `payload.code`, and `payload.message`. The consumer (the owner) SHOULD ack receipt at the transport layer but does not return a `#response` Trust Task document.

## Definitions

* **Slot identifier.** `payload.mnemonic` (+ optional `payload.domain`) locates the slot on the producer.
* **Framework error code.** A code drawn from [SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes) OR a `did-management/...` extended code; consumers treat unknown codes as informational.

## Request

```json
{ "id": "pr-1", "type": "https://trusttasks.org/spec/did-management/did/problem-report/0.1",
  "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-05T14:00:00Z",
  "payload": { "mnemonic": "alice", "code": "did-management/did/publish:invalid_log",
    "message": "Background witness validation failed after async retry: signature did not verify against parameters.updateKeys[0].",
    "ctx": { "witnessVersionId": "1-abc", "retries": 3 } } }
```

## Security & Privacy

Problem reports carry diagnostic detail that MAY leak operational metadata to the owner. The hosting service SHOULD NOT include data sourced from other tenants or any other owner's slot in `ctx`.
