---
slug: acl/show
version: "1.0"
title: ACL — Show
summary: A querying party asks an ACL maintainer for the entry corresponding to a specific subject.
status: draft
targetFrameworkVersion: "0.1"
category: permission
keywords:
  - acl
  - access-control
  - lookup
  - query
  - show
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Querying party
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: A single-entry lookup is typically short-lived and consumed over an authenticated transport; a proof becomes valuable when the answer is retained, replayed, or relied upon by a third party.
errorCodes:
  - code: acl/show:permission_denied
    meaning: The querying party is not permitted to look up entries in this ACL under the maintainer's policy.
    retryable: false
  - code: acl/show:subject_not_present
    meaning: The subject named in the payload is not currently in the ACL.
    retryable: false
related:
  - acl/list
  - acl/grant
  - acl/revoke
---

## Abstract

The **ACL — Show** Trust Task lets a *querying party* ask the *ACL maintainer* whether a specific subject currently has an entry, and if so, what the entry contains. The response is either the matching *AclEntry* or an `acl/show:subject_not_present` error.

This task is **read-only**: it never mutates the ACL. The response **SHOULD** be a `trust-task-ok` *Trust Task document* once that response type is published (see [SPEC.md §8.6](../../../../SPEC.md#86-reserved-response-type-slugs)); until then, transports define how the entry is conveyed back.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels).

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the querying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/show/1.0`.
2. Identify itself as `issuer`; identify the ACL maintainer as `recipient`.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Apply its own policy to decide whether the querying party is permitted to look up entries. Where the policy denies the query, respond with `acl/show:permission_denied`.
3. If the subject is in the ACL, respond with the *AclEntry* — as a `trust-task-ok` *Trust Task document* once that type is published, or per the transport-binding convention until then.
4. If the subject is not in the ACL, respond with `acl/show:subject_not_present`.

Maintainers **MAY** redact entry fields based on the querying party's role (for example, omitting `metadata` to non-administrators); the response documents which fields were redacted in its own payload.

The maintainer **SHOULD** respect a "self-lookup is always permitted" convention: a querying party whose `issuer` equals the queried `payload.subject` **SHOULD** receive their own entry, even where the broader policy denies general lookups. This lets a subject confirm their own access state without administrative privilege.

## Definitions

* **Querying party.** The party initiating the lookup; identified by `issuer`.
* **ACL maintainer.** The party answering the lookup; identified by `recipient`.
* **Subject.** The party whose entry is being looked up; identified by `payload.subject`. May be the querying party itself (self-lookup).

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/show/1.0` (or `…/1.0#request`), with a payload that validates against the top-level schema in `payload.schema.json`. The producer is the querying party; the recipient is the ACL maintainer.

### Admin looking up a member

```json
{
  "id": "a82a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/show/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-20T08:00:00Z",
  "payload": {
    "subject": "did:web:alice.example"
  }
}
```

The maintainer applies admin-level lookup policy and either returns Alice's current entry (via `trust-task-ok` or a transport-specific response) or responds with `acl/show:subject_not_present`.

### Self-lookup

```json
{
  "id": "b91c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/show/1.0",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-20T08:05:00Z",
  "payload": {
    "subject": "did:web:alice.example"
  }
}
```

`issuer == payload.subject`. Under the "self-lookup is always permitted" convention, the maintainer returns Alice her own entry — letting her confirm her current role, scopes, and expiry without administrative privilege.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/show/1.0#response`, with a payload that validates against the sub-schema reachable via `$anchor: "response"` in `payload.schema.json`. The producer is the ACL maintainer; the recipient is the querying party.

The response payload carries:

* `entry` — the *AclEntry* the maintainer holds for the requested subject.
* `redactedFields` — optional; lists *AclEntry* field names the maintainer omitted from `entry` (for example, `["metadata"]` when the requester does not have administrator privileges).

A subject not being in the ACL is **not** a success: the maintainer returns an `acl/show:subject_not_present` error using `trust-task-error`, not a `#response` document.

### Successful lookup

Response to the "Admin looking up a member" request example:

```json
{
  "id": "ba2a1c44-7b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/show/1.0#response",
  "threadId": "a82a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-20T08:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "label": "Alice — primary admin",
      "createdAt": "2026-05-16T10:00:00Z",
      "createdBy": "did:web:org.example"
    }
  }
}
```

### Successful self-lookup with redaction

Response to the self-lookup request example, where the maintainer's policy redacts `metadata` for non-administrators (including the subject themselves):

```json
{
  "id": "ca1c7b32-7a91-4a91-a3a4-9d61b75e2f02",
  "type": "https://trusttasks.org/spec/acl/show/1.0#response",
  "threadId": "b91c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-06-20T08:05:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "label": "Alice — primary admin",
      "createdAt": "2026-05-16T10:00:00Z",
      "createdBy": "did:web:org.example"
    },
    "redactedFields": ["metadata"]
  }
}
```

### Failure — subject not in the ACL

A querying party asks about a subject that does not exist:

```json
{
  "id": "c91c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/show/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-20T08:10:00Z",
  "payload": {
    "subject": "did:web:nobody.example"
  }
}
```

The maintainer responds with a `trust-task-error` — note the distinct `type`, not the `#response` variant:

```json
{
  "id": "d12c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/trust-task-error/1.0",
  "threadId": "c91c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-20T08:10:01Z",
  "payload": {
    "code": "acl/show:subject_not_present",
    "retryable": false
  }
}
```

## Security & Privacy

A single-entry lookup discloses one subject's access state. Maintainers **SHOULD** apply policy that limits arbitrary lookups to parties who have a legitimate need — administrators, auditors, or the subject themselves. Allowing arbitrary parties to confirm whether a given VID is in the ACL is itself a privacy disclosure.

Where the maintainer responds with a full entry, the response inherits the same sensitivity considerations as the underlying ACL: roles, scopes, and labels **MAY** be sensitive personal or organizational data. Confidentiality **SHOULD** be enforced at the transport layer.

Implementations **SHOULD** populate `issuedAt` and **SHOULD** include a `proof` member where the answer will be retained or forwarded.
