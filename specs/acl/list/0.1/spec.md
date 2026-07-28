---
slug: acl/list
version: "0.1"
title: ACL — List
summary: A querying party asks an ACL maintainer to enumerate the entries currently in its access-control list, with optional filters and paging.
status: draft
targetFrameworkVersion: "0.1"
category: access-control
keywords:
  - acl
  - access-control
  - list
  - enumeration
  - query
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Querying party
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Most list queries are short-lived and consumed over an authenticated transport; a proof becomes valuable when the list is retained, replayed, or relied upon by a third party. Where the listed roles are themselves sensitive, an in-band proof is preferred.
sideEffects:
  level: none
  rationale: "Read-only enumeration of ACL entries."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - acl/show
  - acl/grant
  - acl/revoke
---

## Abstract

The **ACL — List** Trust Task lets a *querying party* ask the *ACL maintainer* for the set of entries currently in its access-control list. The query supports optional filters by `role`, `scope`, and `subjectPrefix`, plus a paging cursor for large lists.

This task is **read-only**: it never mutates the ACL. The response carries the entry list and (where the list spans multiple pages) a continuation cursor.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Reading a scope filter in two directions

Where a maintainer's scopes are hierarchical, a single scope identifier raises
two different questions, and `scope` alone cannot distinguish them:

- **Who may act in this scope?** Entries scoped to it *or to an ancestor of it*
  — an ancestor's authority reaches down. This is `acting-in`, and it is what a
  `scope` filter means when `direction` is omitted.
- **What is granted beneath this scope?** Entries holding a grant *at or below*
  it. This is `subtree`.

`any` is the union, which is the auditor's question.

The distinction is easy to get wrong and fails quietly. A revocation sweep is
the clearest case: asking `acting-in` when you meant `subtree` returns precisely
the entries that are **not** the answer — the ancestors, which keep their
authority — while omitting every leaf-scoped grant underneath, which is what you
were trying to find. The result is short rather than empty, so it reads as a
complete answer to a question nobody asked.

A consumer whose scopes are flat MAY treat all three values alike, since the
three questions collapse into one.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the querying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/list/0.1`, with itself as `issuer` and the ACL maintainer as `recipient`.
2. Populate `payload` with any subset of the filter and paging members defined by the schema.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Apply its own policy to decide whether the querying party is permitted to enumerate the ACL. Where the policy denies the query, respond with the framework's `permission_denied` (see [SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)).
3. Apply any provided filters (conjunctively) and return only the matching entries. Filter strings the maintainer does not recognize **MUST** simply produce zero matches; they are not an error.
4. Honor `pageSize` (default and maximum at the maintainer's discretion) and return a continuation `cursor` if more entries remain.
5. Respond via the `#response` variant defined below.

Maintainers **MAY** redact entry fields based on the querying party's role and **SHOULD** declare any blanket redactions in `payload.redactedFields` of the response.

## Definitions

* **Querying party.** The party initiating the query; identified by `issuer`.
* **ACL maintainer.** The party answering the query; identified by `recipient`.
* **Cursor.** An opaque string the maintainer returns to allow paging through large result sets. Consumers **MUST** treat the cursor as opaque and re-send it verbatim on the follow-up query.

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/list/0.1` with a payload that validates against the top-level schema in `payload.schema.json`. All payload members are optional; an empty payload requests the default list.

### List everyone

```json
{
  "id": "2e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-15T10:00:00Z",
  "payload": {}
}
```

> A request with no filters is still subject to the maintainer's `pageSize` ceiling. Implementations **MUST** assume large ACLs will be truncated; check `payload.truncated` on the response (and, where present, follow `payload.cursor`) before treating the returned `entries` as the complete set.

### Filter by role, with paging

```json
{
  "id": "5b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/0.1",
  "issuer": "did:web:auditor.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-15T10:05:00Z",
  "payload": {
    "role": "admin",
    "pageSize": 50
  }
}
```

### Continuation page

```json
{
  "id": "7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/0.1",
  "issuer": "did:web:auditor.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-15T10:06:00Z",
  "payload": {
    "role": "admin",
    "pageSize": 50,
    "cursor": "eyJvZmZzZXQiOjUwfQ"
  }
}
```

### Compound filter

```json
{
  "payload": {
    "role": "member",
    "scope": "context:project-alpha",
    "subjectPrefix": "did:web:"
  }
}
```

Returns entries whose role is `member`, whose `scopes` array contains `context:project-alpha`, and whose `subject` VID begins with `did:web:`. Filters are conjunctive.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/list/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload carries:

* `entries` — an array of *AclEntry* items matching the filters, in maintainer-defined order. **MAY** be empty.
* `truncated` — **REQUIRED** boolean. `true` when more matching entries exist beyond `entries`; `false` when the response is the complete result. Consumers **MUST** check this before treating `entries` as exhaustive.
* `cursor` — present only when `truncated` is `true` **and** the maintainer supports pagination from this point. Opaque to the consumer; re-send verbatim to fetch the next page. A response with `truncated: true` but no `cursor` means the maintainer cut the result short and cannot continue (for example, an enforced maximum total result size); the consumer **SHOULD** narrow its filter and re-query.
* `redactedFields` — optional; lists *AclEntry* field names the maintainer redacted from every returned entry.

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### A page of admin entries

Response to the "Filter by role, with paging" request example:

```json
{
  "id": "6c3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/list/0.1#response",
  "threadId": "5b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:auditor.example",
  "issuedAt": "2026-06-15T10:05:01Z",
  "payload": {
    "entries": [
      {
        "subject": "did:web:alice.example",
        "role": "admin",
        "label": "Alice — primary admin",
        "createdAt": "2026-05-16T10:00:00Z",
        "createdBy": "did:web:org.example"
      },
      {
        "subject": "did:web:carol.example",
        "role": "admin",
        "createdAt": "2026-05-18T08:30:00Z",
        "createdBy": "did:web:alice.example"
      }
    ],
    "truncated": true,
    "cursor": "eyJvZmZzZXQiOjUwfQ"
  }
}
```

`truncated` is `true` and `cursor` is present, so the auditor sends a continuation request to fetch the next page.

### A page with redactions

A non-administrator queries; the maintainer returns entries but blanket-redacts the `label` field and any `ext.vnd.example.hr` namespace from each entry:

```json
{
  "id": "7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/list/0.1#response",
  "threadId": "5b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:auditor.example",
  "issuedAt": "2026-06-15T10:06:00Z",
  "payload": {
    "entries": [
      {
        "subject": "did:web:alice.example",
        "role": "admin",
        "createdAt": "2026-05-16T10:00:00Z",
        "createdBy": "did:web:org.example"
      }
    ],
    "truncated": false,
    "redactedFields": ["label", "ext.vnd.example.hr"]
  }
}
```

`truncated: false` confirms the auditor has the complete result; `label` and any `ext.vnd.example.hr` namespace are absent from each entry because the maintainer's policy redacts them for this querying party.

## Security & Privacy

An ACL listing is the directory of who has what access. Maintainers **SHOULD** limit enumeration to parties with a legitimate need (administrators, auditors). Public enumeration of an ACL is rarely appropriate.

Roles, scopes, and labels in returned entries **MAY** be sensitive personal or organizational data. Confidentiality **SHOULD** be enforced at the transport layer (mutually-authenticated TLS, signed DIDComm envelope, etc.).

Implementations **SHOULD** include a `proof` member where the list will be retained or forwarded; without it, a retained list cannot be attributed to its maintainer after the fact.
