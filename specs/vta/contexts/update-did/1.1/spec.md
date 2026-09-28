---
slug: vta/contexts/update-did
version: "1.1"
title: VTA Contexts — Update DID
summary: An administrator sets or clears the DID a context acts as; existing references to the previous DID are not migrated.
status: draft
targetFrameworkVersion: "0.5.0"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Repointing a context's DID changes which identity the agent acts as inside it. Replayed out of order it restores a DID the operator has already moved the context off, typically one whose keys were being retired.
sideEffects:
  level: mutating
  rationale: "Changes the DID a context acts as, or leaves it with none. Reversible as a write, but signatures made under either DID remain attributed to it."
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

`payload.did` MAY be `null`, which leaves the context with **no DID of its
own** — the same state as a context created without one. That is the only way
to retire the last DID a context acts as: a VTA refuses to delete a DID while a
context still acts as it, and without `null` the only way past that refusal was
to assign a different DID the operator may not want.

### Changes from 1.0

- `did` accepts `null`, meaning "no DID".
- A string `did` MUST now be a syntactically valid DID (DID Core §3.1). 1.0
  accepted any non-empty string, so `"did:"` or `"hello"` validated and was
  stored as the context's identity. Every conforming 1.0 document that carried
  a real DID is also a conforming 1.1 document.

A consumer implementing 1.1 **MUST** also accept 1.0 documents, per
[SPEC.md §5.2](/SPEC.md#52-compatibility-rules).

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

When `payload.did` is `null` the consumer **MUST** remove the context's DID, so
that the returned record carries no `did` member — absent, per `ContextRecord`,
never `null` and never an empty string. Clearing a context that already has no
DID **MUST** succeed and change nothing but `updatedAt`. Clearing a DID does
not delete it: the DID, its keys and its log are untouched, and remain
available to be assigned again or deleted.

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
  "type": "https://trusttasks.org/spec/vta/contexts/update-did/1.1",
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
    "proofPurpose": "authentication",
    "proofValue": "z2LmN..."
  }
}
```

Clearing the DID is the same request with `did: null`:

```json
"payload": {
  "id": "personal/banking",
  "did": null
}
```

and its response is the record with no `did` member.

## Response

```json
{
  "id": "6f708192-a3b4-4c5d-e6f7-08192a3b4c5d",
  "type": "https://trusttasks.org/spec/vta/contexts/update-did/1.1#response",
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
signs as the new DID — or, cleared, as nothing — while every credential a
verifier already holds names the old one, so a relying party checking "is this
the identity I onboarded" will say no, correctly, until it is told otherwise out
of band. Reassignment and clearing are therefore operations with an audience
beyond the VTA, and the audit record is the only trace of when the switch
happened. That is why `proof` is REQUIRED here even though the task is a
single-field write.

**Clearing.** A context with no DID presents no identity: anything the VTA
would sign as the context has no issuer to sign as, and a consumer **MUST**
refuse such an operation rather than fall back to another DID it holds. A
relying party that onboarded the cleared DID keeps it, exactly as after a
reassignment.

### Data carried

The request carries a context id and a DID, or `null`. Both are identifiers
rather than personal data, though a context id is operator-chosen text and may
name a person or customer (`personal/banking`); producers **SHOULD** choose ids
that identify the scope rather than whoever it belongs to.

The response is the whole `ContextRecord`. Its `name` is free text, bounded at
256 characters — a display name, not prose. It was authored by whichever
operator created or last updated the context, is read by whoever reads this
response, and is operator-facing only, carrying no authorization meaning.
Re-pointing or clearing a context's DID does not change it, and a caller
**MUST NOT** read the unchanged name as evidence that nothing moved.

### Correlation

This task exists to change which identifier a context presents, which is a
correlation event by construction: an observer that saw the context act as the
previous DID and later as the new one learns, from timing alone, that they are
likely the same scope. Nothing in this task can hide that. Conversely, clearing
does not *unlink* anything — every signature and credential made under the
previous DID still names it, and an observer who already joined the two keeps
the join.

### Retention

The VTA retains the context record, including `name`, for the life of the
context, and the audit record of each assignment and clearing for as long as its
audit log is kept. The audit record is the only evidence of when a context
stopped acting as a DID, so a consumer **SHOULD NOT** retain it for less time
than it retains anything signed under that DID.

### Consent/purpose

The DID and context id are carried to identify what is being changed, and the
audit record exists to attribute the change. Neither is collected for any other
purpose, and this task implies nothing about the consent of whoever the previous
DID identified to other parties.
