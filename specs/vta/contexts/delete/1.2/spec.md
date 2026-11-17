---
slug: vta/contexts/delete
version: "1.2"
title: VTA Contexts — Delete
summary: An administrator deletes a context and everything scoped to it, every DID of every method included; refused by default while it still holds anything, or while a hosting server cannot confirm what deleting one of its DIDs requires there.
status: draft
targetFrameworkVersion: "0.6.0"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Deleting a context destroys everything scoped inside it. Replayed after a context was re-created under the same name, it destroys the new one and its contents with it.
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
  - code: vta/contexts/delete:hostCleanupFailed
    meaning: What deleting at least one DID in the subtree requires on its hosting server — removing a did:webvh log, publishing a did:webs deactivation and removing its slot — could not be confirmed, and `abandonHostCopies` was not set. Nothing was deleted locally. Retry once the host is back, or retry with `abandonHostCopies` to complete the deletion and take on the named DIDs.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [hostCleanupFailures]
      properties:
        hostCleanupFailures:
          type: array
          minItems: 1
          items:
            type: object
            additionalProperties: false
            required: [did, reason]
            properties:
              did: { type: string, minLength: 1 }
              server: { type: string, minLength: 1 }
              reason: { type: string, minLength: 1 }
related:
  - vta/contexts/preview-delete
  - vta/contexts/create
  - vta/contexts/get
  - vta/contexts/list
  - vta/dids/list
  - vta/webvh/dids/delete
  - vta/webs/dids/delete
---

## Abstract

**VTA Contexts — Delete** removes a [context](../../list/1.0/spec.md) and
everything scoped to it: its keys, every DID it holds — of every method, as
[`vta/dids/list`](../../../dids/list/0.1/spec.md) summarises them — its DID
templates, and the ACL entries that named it.

It also removes the context's **sub-contexts, and everything scoped to those**,
to any depth. A context is a folder as well as a scope, and there is no state
in which a sub-context outlives the parent whose path it is part of.

By default the VTA refuses to delete a context that still holds anything —
sub-contexts included. `force` overrides that, and the refusal exists so that
the destructive case is always something the caller asked for in as many words.

Deleting a context's DIDs reaches outside the VTA, to the servers that host
them, and those servers can be gone. By default the VTA refuses a
deletion it cannot finish there, and `abandonHostCopies` is how the caller
completes it anyway, taking on what was left behind.

### Changes from 1.1

1.1 named only `did:webvh`: it settled their logs on their hosts and deleted
them as `vta/webvh/dids/delete` does. A context holding a `did:webs` DID had its
record removed and nothing else — the key event log stayed published and the
DID kept resolving as live, with the records and keys that could end it gone,
and neither the refusal nor `daemonCleanupErrors` mentioned it. That is the
deletion this task forbade for `did:webvh` in as many words.

1.2 covers **every identity the VTA holds in the subtree**, whatever its method:
each is deleted as its method's own delete task deletes it, and whatever that
requires on a hosting server is settled first, under the same refusal and the
same `abandonHostCopies` choice. The payload schema is unchanged except for the
descriptions that named `did:webvh` only.

1.2 supersedes 1.1, and a producer **SHOULD** send 1.2.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** refuse with
`vta/contexts/delete:notEmpty` when the context holds keys, DIDs, templates
**or sub-contexts** and `force` is absent or `false`. It **MUST NOT** partially
delete: either the context and its contents go, or nothing does — and
"contents" is the whole subtree, so a refusal that can only be discovered part
way through the cascade **MUST** be discovered before any of it is performed.

Every identity the consumer holds in the subtree — every one
[`vta/dids/list`](../../../dids/list/0.1/spec.md) would summarise there
([CONVENTIONS.md §20.1](../../../dids/_shared/0.2/CONVENTIONS.md#201-which-identities-are-summarised)),
deactivated ones included — **MUST** be deleted as its method's own delete task
deletes it, including revocation of credentials issued to it and removal of the
authority that named it:

| Method | Deleted as | On its hosting server |
|---|---|---|
| `did:webvh`, `did:scid:vh:1` | [`vta/webvh/dids/delete/1.0`](../../../webvh/dids/delete/1.0/spec.md) | the published log removed |
| `did:webs` | [`vta/webs/dids/delete/0.1`](../../../webs/dids/delete/0.1/spec.md) | the deactivation published, then the slot removed |
| `did:key` | its key record removed | nothing: it is published nowhere |

Removing only the consumer's own record of a DID is **not** a deletion of it:
what is published keeps resolving for every party except its owner, and the
records that could end or remove it are the ones just destroyed.

**Host copies are settled first.** Before it removes anything of its own, the
consumer **MUST** attempt, for every DID in the subtree, what the table requires
on its hosting server. A host that answers that it does not hold the log or
slot has removed it; a `did:webs` already deactivated needs only its slot
removed. A copy is **unconfirmed** when the host refuses, does not answer, or
cannot be reached, and when the consumer no longer knows the host the DID was
published on.

- When any copy is unconfirmed and `abandonHostCopies` is absent or `false`,
  the consumer **MUST** refuse with `vta/contexts/delete:hostCleanupFailed`,
  listing every unconfirmed DID in `details.hostCleanupFailures`, and **MUST
  NOT** remove any local record. What *was* settled stays settled — a log
  removed, a `did:webs` deactivated: the records that remain are what lets a
  retry complete, and the retry finds that work already done.
- When `abandonHostCopies` is `true`, the consumer **MUST** complete the
  deletion and **MUST** report every unconfirmed DID in `daemonCleanupErrors`
  rather than presenting the deletion as complete.

A host that never answers must not hold the deletion past the requester's
patience. The consumer **SHOULD** bound the time it spends on hosts — together,
not per DID — so that it answers within the requester's timeout, and treat a
host that has not answered by then as unconfirmed.

`deleted: false` in a success response means the VTA declined to act. A
consumer **MUST NOT** report the deletion as done on the strength of a
successful response alone; it reads the member.

A conforming **consumer** **MUST** answer `vta/contexts/delete:notFound` for an
id the caller cannot reach, whether or not it exists.

A conforming **producer** that receives `vta/contexts/delete:hostCleanupFailed`
**SHOULD** show the listed DIDs to a human and ask, before retrying with
`abandonHostCopies`, whether to proceed or wait for the host. It **MUST NOT**
set `abandonHostCopies` on a first attempt on the human's behalf without being
told to.

## Authorization

Authority is the **administrator role over the context**.

The required `proof` attributes the deletion. Its role here is unusually
load-bearing: once the contents are gone, the audit record is the only
remaining evidence of what was destroyed and on whose instruction, so a
consumer that accepted a proofless deletion would have no answer to that
question afterwards.

`force` and `abandonHostCopies` are not separate authorities. A consumer **MUST
NOT** require a different role for either — they are explicitness flags, not
privileges — and **MUST NOT** treat their absence as permission to delete
contents, or to abandon a host copy, anyway.

## Request

Refused, because a hosting server did not answer:

```json
{
  "id": "92a3b4c5-d6e7-4f08-192a-3b4c5d6e7f08",
  "type": "https://trusttasks.org/spec/vta/contexts/delete/1.2",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T10:00:00Z",
  "payload": { "id": "personal/banking", "force": true, "abandonHostCopies": false },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T10:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z7QRs..."
  }
}
```

```json
{
  "id": "a3b4c5d6-e7f0-4819-2a3b-4c5d6e7f0819",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T10:00:16Z",
  "threadId": "92a3b4c5-d6e7-4f08-192a-3b4c5d6e7f08",
  "payload": {
    "code": "vta/contexts/delete:hostCleanupFailed",
    "message": "1 published DID could not be removed from its hosting server; nothing was deleted",
    "details": {
      "hostCleanupFailures": [
        {
          "did": "did:webvh:QmScid:host.example:banking",
          "server": "host-1",
          "reason": "the hosting server did not answer within 15s"
        }
      ]
    }
  }
}
```

## Response

The same request with `abandonHostCopies`, and the deletion completes:

```json
{
  "id": "b4c5d6e7-f081-492a-3b4c-5d6e7f081920",
  "type": "https://trusttasks.org/spec/vta/contexts/delete/1.2#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T10:01:16Z",
  "threadId": "c5d6e7f0-8192-4a3b-4c5d-6e7f08192a3b",
  "payload": {
    "id": "personal/banking",
    "deleted": true,
    "daemonCleanupErrors": [
      "did:webvh:QmScid:host.example:banking: the hosting server did not answer within 15s"
    ]
  }
}
```

## Security & Privacy

Call [`vta/contexts/preview-delete`](../../preview-delete/2.0/spec.md) first
and show a human what it returns. This task's failure mode is not a wrong
answer, it is a right answer to a question the caller did not realize they were
asking.

Two consequences outlive the context and cannot be undone by recreating it:

- **Internally generated keys are gone.** A key derived from the VTA's seed can
  be re-derived; one generated from the CSPRNG appears in no backup and no
  export, and nothing it authorized can be re-authorized.
- **Published DIDs stop resolving, or resolve as deactivated** — unless their
  host copy was abandoned.
  Anything that recorded one as an issuer or a subject now points at nothing,
  and those parties are not notified.

**An abandoned host copy is a live identity nobody controls.** Its log, or its
key event log, keeps resolving — for a `did:webs` whose deactivation never
reached the host, as a live identity — and with the VTA's records gone nobody can update, deactivate or
remove it through the VTA again; credentials the VTA issued to it are revoked,
but a party that never checks status still accepts them. `abandonHostCopies`
exists because a host can be gone for good and the context must still be
deletable, not as a shortcut past a host that is merely slow — which is why it
is never the default and a producer asks before setting it.

Recreating a context with the same id does not restore either, and a grant that
named the id before the deletion now scopes a different, empty context. Treat
id reuse after a deletion as a distinct decision rather than a continuation.

### Data carried

The request carries a context id and two flags. A context id is operator-chosen
text and may name a person or customer (`personal/banking`); producers
**SHOULD** choose ids that identify the scope rather than whoever it belongs to.

The `hostCleanupFailed` refusal and `daemonCleanupErrors` name DIDs in the
subtree, the VTA's identifier for their hosting servers, and a human-readable
reason that may quote the host's own error. They are disclosed only to a caller
already authorized to delete the context, which the preview would have shown
the same DIDs.

### Correlation

The DIDs a refusal names are the subtree's own, already disclosed to this
caller by `vta/contexts/preview-delete`; naming them again links nothing new.
A refusal does reveal which hosting servers were unreachable at the time of the
request.

### Retention

Nothing is retained on a refusal beyond the audit record. On success the VTA
retains no record of the context or its DIDs other than the audit record of the
deletion, which **SHOULD** name each abandoned host copy: it is the only
remaining evidence of which logs were left resolving and on whose instruction.

### Consent/purpose

The flags record the administrator's decision, and the DIDs and servers are
carried so that the administrator can make it and later act on it. They are not
used for any other purpose.
