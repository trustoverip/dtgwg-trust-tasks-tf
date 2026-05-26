---
slug: acl/show
version: "0.1"
title: ACL — Show
summary: A querying party asks an ACL maintainer for the entry corresponding to a specific subject.
status: draft
targetFrameworkVersion: "0.1"
category: access-control
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
errorCodes: []
related:
  - acl/list
  - acl/grant
  - acl/revoke
---

## Abstract

The **ACL — Show** Trust Task lets a *querying party* ask the *ACL maintainer* whether a specific subject currently has an entry, and if so, what the entry contains. The response is either the matching *AclEntry* or `entry: null` if the subject is not in the ACL — "no such entry" is a successful answer, not an error.

This task is **read-only**: it never mutates the ACL.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the querying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/show/0.1`, with itself as `issuer` and the ACL maintainer as `recipient`.
2. Populate `payload.subject` with the VID of the entry being looked up.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Apply its own policy to decide whether the querying party is permitted to look up entries. Where the policy denies the query, respond with the framework's `permission_denied` (see [SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)).
3. Respond with the `#response` variant: `entry` is the AclEntry if the subject is present, or `null` if the subject is not in the ACL.

The maintainer **SHOULD** respect a "self-lookup is always permitted" convention: a querying party whose `issuer` equals `payload.subject` **SHOULD** receive their own entry, even where the broader policy denies general lookups.

## Definitions

* **Querying party.** The party initiating the lookup; identified by `issuer`.
* **ACL maintainer.** The party answering the lookup; identified by `recipient`.
* **Subject.** The party whose entry is being looked up; identified by `payload.subject`. May be the querying party itself.

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/show/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Admin looking up a member

```json
{
  "id": "a82a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/show/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-20T08:00:00Z",
  "payload": {
    "subject": "did:web:alice.example"
  }
}
```

### Self-lookup

```json
{
  "id": "b91c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/show/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-20T08:05:00Z",
  "payload": {
    "subject": "did:web:alice.example"
  }
}
```

`issuer == payload.subject`. Under the "self-lookup is always permitted" convention, the maintainer returns Alice her own entry.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/show/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload carries:

* `entry` — the *AclEntry* the maintainer holds for the subject, or `null` if the subject is not in the ACL.
* `redactedFields` — optional; lists *AclEntry* field names the maintainer omitted from `entry`.

Failures (e.g. `permission_denied`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful lookup

Response to the "Admin looking up a member" request example:

```json
{
  "id": "ba2a1c44-7b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/show/0.1#response",
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
      "createdBy": "did:web:org.example",
      "ext": { "vnd.example.hr": { "department": "compliance" } }
    }
  }
}
```

### Subject not in the ACL

Response to a lookup for a subject the maintainer doesn't have an entry for — note `entry: null` rather than a `trust-task-error`:

```json
{
  "id": "d12c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/show/0.1#response",
  "threadId": "c91c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-20T08:10:01Z",
  "payload": {
    "entry": null
  }
}
```

### Self-lookup with redaction

A subject queries their own entry; the maintainer's policy redacts the `ext.vnd.example.hr` namespace for non-administrators. The visible `entry` lacks that namespace, and `redactedFields` makes the redaction explicit:

```json
{
  "id": "ca1c7b32-7a91-4a91-a3a4-9d61b75e2f02",
  "type": "https://trusttasks.org/spec/acl/show/0.1#response",
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
    "redactedFields": ["ext.vnd.example.hr"]
  }
}
```

Compare with the admin lookup above — Alice's `ext.vnd.example.hr.department` was present there but is omitted here.

## Security & Privacy

A single-entry lookup discloses one subject's access state. Maintainers **SHOULD** limit arbitrary lookups to parties with a legitimate need; allowing arbitrary parties to confirm whether a given VID is in the ACL is itself a privacy disclosure.

Where the maintainer responds with a full entry, the response carries the same sensitivity as the underlying ACL. Confidentiality **SHOULD** be enforced at the transport layer.

Implementations **SHOULD** include a `proof` member where the answer will be retained or forwarded.
