---
slug: vta/contexts/update-did
version: "1.0"
title: VTA Contexts — Update DID
summary: An administrator sets the DID a context acts as; existing references to the previous DID are not migrated.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - context
  - did
  - identity
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
  rationale: This changes the identity a context acts as, which downstream parties will see as the author of everything it signs afterwards. The VTA must attribute the change independently of the transport.
sideEffects:
  level: mutating
  rationale: "Changes the DID a context acts as. Reversible as a write, but signatures made under either DID remain attributed to it."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/update-did:notFound
    meaning: No context with this id is reachable by the caller.
    retryable: false
related:
  - vta/contexts/update
  - vta/contexts/get
  - vta/contexts/create
  - vta/contexts/list
---

## Abstract

**VTA Contexts — Update DID** sets the DID a [context](../../list/1.0/spec.md)
acts as: the identity that appears as the issuer of what the context signs and
as the subject of what it is granted.

It exists as its own task, rather than only as a member of
[`vta/contexts/update`](../../update/1.0/spec.md), because changing an identity
is a different kind of change from editing a description — it is worth naming,
auditing and authorizing on its own terms.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** migrate, rewrite or re-issue
anything that referenced the previous DID. Credentials issued to it, ACL
entries naming it, and signatures made under it continue to refer to the
previous DID, and the consumer **MUST NOT** represent this task as having
updated them.

A conforming **consumer** **MUST** answer `vta/contexts/update-did:notFound`
for an id the caller cannot reach, whether or not it exists.

## Authorization

Authority is the **administrator role over the context**, the same role that
[creates](../../create/1.0/spec.md) one — not the super-administrator role that
[`vta/contexts/update`](../../update/1.0/spec.md) requires. The asymmetry is
deliberate: assigning an identity to a scope you already administer is within
that administration, whereas changing what the scope *permits* is not.

The required `proof` attributes the change. It does not establish that the
producer controls the DID being assigned — nothing in this task does. A
consumer that needs that assurance obtains it separately, and **MUST NOT**
infer control of `payload.did` from a valid proof over the request.

## Request

```json
{
  "id": "5e6f7081-92a3-4b4c-d5e6-f708192a3b4c",
  "type": "https://trusttasks.org/spec/vta/contexts/update-did/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:40:00Z",
  "payload": {
    "id": "personal/banking",
    "did": "did:webvh:QmNewScid:example.com"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T09:40:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2LmN..."
  }
}
```

## Response

```json
{
  "id": "6f708192-a3b4-4c5d-e6f7-08192a3b4c5d",
  "type": "https://trusttasks.org/spec/vta/contexts/update-did/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T09:40:01Z",
  "threadId": "5e6f7081-92a3-4b4c-d5e6-f708192a3b4c",
  "payload": {
    "id": "personal/banking",
    "name": "Banking",
    "did": "did:webvh:QmNewScid:example.com",
    "parent": "personal",
    "basePath": "personal/banking",
    "createdAt": "2026-03-11T08:30:00Z",
    "updatedAt": "2026-08-19T09:40:01Z"
  }
}
```

## Security & Privacy

The non-migration rule is the whole risk. After this task succeeds, the context
signs as the new DID while every credential a verifier already holds names the
old one — so a relying party checking "is this the identity I onboarded" will
say no, correctly, until it is told otherwise out of band.

Reassignment is therefore an operation with an audience beyond the VTA, and the
audit record is the only trace of when the switch happened. That is why `proof`
is REQUIRED here even though the task is a single-field write.

**Free text.** The returned record's `name` is free text, bounded at 256
characters — a display name, not prose. It was authored by whichever operator
created or last updated the context, is read by whoever reads this response, and
is **retained** by the VTA for the life of the context. It is operator-facing
only and carries no authorization meaning; re-pointing a context's DID does not
change it, and a caller MUST NOT read the unchanged name as evidence that
nothing moved.

