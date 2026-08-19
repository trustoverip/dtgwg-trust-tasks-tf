---
slug: vta/contexts/update
version: "1.0"
title: VTA Contexts — Update
summary: A super-administrator edits a context's name, description, DID or policy; the id is fixed because ACL scopes name it.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - context
  - update
  - policy
  - scope
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Super-administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A policy edit changes what the context permits for everyone inside it, so the VTA must attribute the change to a specific administrator independently of the transport.
sideEffects:
  level: mutating
  rationale: "Edits a context's metadata and, where supplied, replaces its policy."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/update:notFound
    meaning: No context with this id is reachable by the caller.
    retryable: false
related:
  - vta/contexts/create
  - vta/contexts/get
  - vta/contexts/update-did
  - vta/contexts/list
---

## Abstract

**VTA Contexts — Update** edits a [context](../../list/1.0/spec.md): its name,
description, DID, or the `ContextPolicy` that restricts what may be presented,
signed and exported within it.

The id is not editable. ACL scopes name contexts by id, so renaming one would
silently detach every grant that referenced it — a caller wanting a different
id creates a new context and moves what belongs in it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

Members other than `id` are optional, and an omitted member **MUST** leave that
field unchanged. This is a patch.

`contextPolicy` is the exception and is supplied **whole**: a consumer **MUST**
replace the stored policy with the one supplied rather than merging them. The
distinction matters because policy members mean *unrestricted* by absence — a
merge would make it impossible to lift a restriction, since the member's
absence would be read as "leave it as it was" rather than "remove it".

A conforming **consumer** **MUST** answer `vta/contexts/update:notFound` for an
id the caller cannot reach, whether or not it exists.

## Authorization

Authority is the **super-administrator role**. It is deliberately higher than
the administrator role that [creates](../../create/1.0/spec.md) a context: a
policy edit changes what every holder of the context may do inside it, so it
sits with the authority that governs the VTA rather than the one that governs
the context.

The required `proof` attributes the edit for audit. It is not the
authorization; the role check is, and it follows signature verification
([SPEC §7.2 item 10](../../../../../SPEC.md#72-consumer-requirements)).

## Request

Replacing a policy so the context may only present to two named verifiers, and
may not be exported:

```json
{
  "id": "3c4d5e6f-7081-492a-b3c4-d5e6f7081920",
  "type": "https://trusttasks.org/spec/vta/contexts/update/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:30:00Z",
  "payload": {
    "id": "personal/banking",
    "description": "Accounts, payment credentials and mandates",
    "contextPolicy": {
      "trustedVerifiers": ["did:web:bank.example", "did:web:broker.example"],
      "exportAllowed": false
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T09:30:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z58Gh..."
  }
}
```

Because the policy is replaced whole, the request above also **removes** any
`presentableTypes` and `signableKeys` restrictions the context previously had:
they are absent from the new policy, and absent means unrestricted.

## Response

```json
{
  "id": "4d5e6f70-8192-4a3b-c4d5-e6f708192a3b",
  "type": "https://trusttasks.org/spec/vta/contexts/update/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-08-19T09:30:01Z",
  "threadId": "3c4d5e6f-7081-492a-b3c4-d5e6f7081920",
  "payload": {
    "id": "personal/banking",
    "name": "Banking",
    "description": "Accounts, payment credentials and mandates",
    "parent": "personal",
    "basePath": "personal/banking",
    "createdAt": "2026-03-11T08:30:00Z",
    "updatedAt": "2026-08-19T09:30:01Z"
  }
}
```

The response is the context record, which does **not** carry the policy. A
caller that needs to confirm the policy it just set reads it back with
[`vta/contexts/get`](../../get/1.0/spec.md) rather than inferring success from
this response.

## Security & Privacy

The whole-replacement rule for `contextPolicy` is the sharp edge of this task.
A caller that reads a policy, edits one member, and writes back only that
member has removed every other restriction — and the response will not tell it
so. A conforming producer sends the complete intended policy every time.

Lifting `exportAllowed` is not retroactive: material already exported under a
permissive policy stays exported. Setting it to `false` constrains what happens
next, and nothing more.
