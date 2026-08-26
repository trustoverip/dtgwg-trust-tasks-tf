---
slug: vta/contexts/delete
version: "1.0"
title: VTA Contexts — Delete
summary: An administrator deletes a context and everything scoped to it; refused by default while it still holds anything.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - context
  - delete
  - destructive
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The most destructive task in the family — keys and published DIDs go with the context. The VTA must attribute it to a specific administrator independently of the transport, because the audit record is what remains once the contents do not.
sideEffects:
  level: destructive
  rationale: "Destroys the context, the keys and DIDs scoped to it, and the ACL entries that named it. Internally generated keys are unrecoverable."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/delete:notFound
    meaning: No context with this id is reachable by the caller.
    retryable: false
  - code: vta/contexts/delete:notEmpty
    meaning: The context still holds keys, DIDs or templates and `force` was not set.
    retryable: false
related:
  - vta/contexts/preview-delete
  - vta/contexts/create
  - vta/contexts/get
  - vta/contexts/list
---

## Abstract

**VTA Contexts — Delete** removes a [context](../../list/1.0/spec.md) and
everything scoped to it: its keys, the `did:webvh` DIDs it publishes, its DID
templates, and the ACL entries that named it.

By default the VTA refuses to delete a context that still holds anything.
`force` overrides that, and the refusal exists so that the destructive case is
always something the caller asked for in as many words.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** refuse with
`vta/contexts/delete:notEmpty` when the context holds keys, DIDs or templates
and `force` is absent or `false`. It **MUST NOT** partially delete: either the
context and its contents go, or nothing does.

`deleted: false` in a success response means the VTA declined to act. A
consumer **MUST NOT** report the deletion as done on the strength of a
successful response alone; it reads the member.

A conforming **consumer** **MUST** answer `vta/contexts/delete:notFound` for an
id the caller cannot reach, whether or not it exists.

## Authorization

Authority is the **administrator role over the context**.

The required `proof` attributes the deletion. Its role here is unusually
load-bearing: once the contents are gone, the audit record is the only
remaining evidence of what was destroyed and on whose instruction, so a
consumer that accepted a proofless deletion would have no answer to that
question afterwards.

`force` is not a separate authority. A consumer **MUST NOT** require a
different role for it — it is an explicitness flag, not a privilege — and
**MUST NOT** treat its absence as permission to delete contents anyway.

## Request

Refused, because the context still holds keys:

```json
{
  "id": "92a3b4c5-d6e7-4f08-192a-3b4c5d6e7f08",
  "type": "https://trusttasks.org/spec/vta/contexts/delete/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T10:00:00Z",
  "payload": { "id": "personal/banking", "force": false },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T10:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z7QRs..."
  }
}
```

```json
{
  "id": "a3b4c5d6-e7f0-4819-2a3b-4c5d6e7f0819",
  "type": "https://trusttasks.org/spec/trust-task-error/0.2",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T10:00:01Z",
  "threadId": "92a3b4c5-d6e7-4f08-192a-3b4c5d6e7f08",
  "payload": {
    "code": "vta/contexts/delete:notEmpty",
    "message": "context holds 2 keys and 1 published DID"
  }
}
```

## Response

The same request with `force`, and the deletion proceeds:

```json
{
  "id": "b4c5d6e7-f081-492a-3b4c-5d6e7f081920",
  "type": "https://trusttasks.org/spec/vta/contexts/delete/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T10:01:01Z",
  "threadId": "c5d6e7f0-8192-4a3b-4c5d-6e7f08192a3b",
  "payload": { "id": "personal/banking", "deleted": true }
}
```

## Security & Privacy

Call [`vta/contexts/preview-delete`](../../preview-delete/1.0/spec.md) first
and show a human what it returns. This task's failure mode is not a wrong
answer, it is a right answer to a question the caller did not realize they were
asking.

Two consequences outlive the context and cannot be undone by recreating it:

- **Internally generated keys are gone.** A key derived from the VTA's seed can
  be re-derived; one generated from the CSPRNG appears in no backup and no
  export, and nothing it authorized can be re-authorized.
- **Published DIDs stop resolving.** Anything that recorded one as an issuer or
  a subject now points at nothing, and those parties are not notified.

Recreating a context with the same id does not restore either, and a grant that
named the id before the deletion now scopes a different, empty context. Treat
id reuse after a deletion as a distinct decision rather than a continuation.
