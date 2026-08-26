---
slug: vta/contexts/preview-delete
version: "1.0"
title: VTA Contexts — Preview Delete
summary: An administrator asks what deleting a context would destroy, and is told, without anything being destroyed.
status: draft
targetFrameworkVersion: "0.5"
category: did-management
keywords:
  - vta
  - context
  - delete
  - preview
  - dry-run
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
  requirement: OPTIONAL
  rationale: The task changes nothing. It does enumerate what a context holds, so a maintainer whose audit retains the response MAY require a proof.
sideEffects:
  level: none
  rationale: "Computes what a deletion would affect; nothing is written."
subjectPath: /id
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/preview-delete:notFound
    meaning: No context with this id is reachable by the caller.
    retryable: false
related:
  - vta/contexts/delete
  - vta/contexts/get
  - vta/contexts/list
---

## Abstract

**VTA Contexts — Preview Delete** answers the question
[`vta/contexts/delete`](../../delete/1.0/spec.md) does not give you a second
chance to ask: what would be destroyed.

It returns the keys, published DIDs and DID templates the context holds, and
the ACL entries that would be removed or narrowed as a consequence. Nothing is
written, and the caller may decide not to proceed.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** modify any state in the course
of answering, including reservations, locks or tombstones. A caller may call
this repeatedly, and a caller that never follows through leaves no trace.

The response is a snapshot and **MUST NOT** be treated as a guarantee: the
context can change between the preview and the deletion. A consumer **MAY**
re-derive the effects at deletion time and act on those instead.

A conforming **consumer** **MUST** distinguish `aclEntriesRemoved` from
`aclEntriesUpdated`: the first names subjects whose ACL entry disappears
entirely because this context was its only scope, and the second names subjects
who keep an entry with this scope removed. Collapsing the two hides the fact
that a subject is about to lose all of its authority.

## Authorization

Authority is the **administrator role over the context** — the same as the
deletion it previews. The preview is not a lesser capability granted more
widely: it enumerates a context's contents, and a caller who may not delete a
context may not inventory it either.

## Request

```json
{
  "id": "708192a3-b4c5-4d6e-f708-192a3b4c5d6e",
  "type": "https://trusttasks.org/spec/vta/contexts/preview-delete/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:50:00Z",
  "payload": { "id": "personal/banking" }
}
```

## Response

```json
{
  "id": "8192a3b4-c5d6-4e7f-0819-2a3b4c5d6e7f",
  "type": "https://trusttasks.org/spec/vta/contexts/preview-delete/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T09:50:01Z",
  "threadId": "708192a3-b4c5-4d6e-f708-192a3b4c5d6e",
  "payload": {
    "id": "personal/banking",
    "keys": ["signing-1", "agreement-1"],
    "webvhDids": ["did:webvh:QmScid:example.com"],
    "aclEntriesRemoved": ["did:key:z6MkBankBot"],
    "aclEntriesUpdated": ["did:key:z6MkOperator"],
    "didTemplates": ["bank-persona"]
  }
}
```

Read as: deleting this context destroys two keys and stops serving one DID;
`did:key:z6MkBankBot` loses its ACL entry outright, and `did:key:z6MkOperator`
keeps its entry with this scope removed.

## Security & Privacy

This task is an inventory of a context, which is why its authorization is the
same as the deletion's rather than the read's. A caller who may
[list](../../list/1.0/spec.md) contexts learns that this one exists; only an
administrator learns what is inside it.

The response says nothing about *recoverability*, and a consumer rendering it
for a human should not imply any. A key derived from the VTA's seed can be
re-derived; an internally generated one cannot be recovered by any means, and
`keys` does not distinguish them — resolve each key before presenting the
choice.
