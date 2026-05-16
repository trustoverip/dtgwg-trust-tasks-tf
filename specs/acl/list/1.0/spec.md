---
slug: acl/list
version: "1.0"
title: ACL — List
summary: A querying party asks an ACL maintainer to enumerate the entries currently in its access-control list, with optional filters.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - acl
  - access-control
  - list
  - enumeration
  - query
authors:
  - DTGWG Governance TF
parties:
  - role: Querying party
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: Most list queries are short-lived and consumed over an authenticated transport; a proof becomes valuable when the list is retained, replayed, or relied upon by a third party. Where the listed roles are themselves sensitive, an in-band proof is preferred.
errorCodes:
  - code: acl/list:permission_denied
    meaning: The querying party is not permitted to list this ACL under the maintainer's policy.
    retryable: false
  - code: acl/list:scope_unknown
    meaning: A scope filter was provided that the maintainer does not recognize.
    retryable: false
related:
  - acl/show
  - acl/grant
  - acl/revoke
---

## Abstract

The **ACL — List** Trust Task lets a *querying party* ask the *ACL maintainer* for the set of entries currently in its access-control list. The query supports optional filters by `role`, `scope`, and `subjectPrefix`, plus a paging cursor for large lists.

This task is **read-only**: it never mutates the ACL. The response from the maintainer carries the entry list and (where the list spans multiple pages) a continuation cursor. The response **SHOULD** be a `trust-task-ok` *Trust Task document* once that response type is published (see [SPEC.md §8.6](../../../../SPEC.md#86-reserved-response-type-slugs)); until then, transports define how the list is conveyed back.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels).

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the querying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/list/1.0`.
2. Identify itself as `issuer`; identify the ACL maintainer as `recipient`.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Apply its own policy to decide whether the querying party is permitted to enumerate the ACL. Where the policy denies the query, respond with `acl/list:permission_denied`.
3. Apply any provided filters (`role`, `scope`, `subjectPrefix`) and return only the matching entries.
4. Honor `pageSize` (default and maximum values are at the maintainer's discretion) and return a continuation `cursor` if more entries remain.
5. Respond with the resulting list — as a `trust-task-ok` *Trust Task document* once that type is published, or per the transport-binding convention until then.

Maintainers **MAY** redact entry fields based on the querying party's role (for example, omitting `metadata` to non-administrators); the response documents which fields were redacted in its own payload.

## Definitions

* **Querying party.** The party initiating the query; identified by `issuer`.
* **ACL maintainer.** The party answering the query; identified by `recipient`.
* **Filter.** A constraint that narrows the returned set. The framework defines three optional filters: `role`, `scope`, `subjectPrefix`. Filters are conjunctive: an entry matches only if all provided filters match.
* **Cursor.** An opaque string the maintainer returns to allow paging through large result sets. Consumers **MUST** treat the cursor as opaque and re-send it verbatim on the follow-up query.

## Examples

### List everyone

```json
{
  "id": "2e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-15T10:00:00Z",
  "payload": {}
}
```

The maintainer returns its default page of entries (using its own default `pageSize`), optionally followed by a `cursor`.

### Filter by role, with paging

```json
{
  "id": "5b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/1.0",
  "issuer": "did:web:auditor.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-15T10:05:00Z",
  "payload": {
    "role": "admin",
    "pageSize": 50
  }
}
```

Returns the first 50 entries with `role == "admin"`.

### Continuation page

```json
{
  "id": "7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/1.0",
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

The `cursor` value is opaque — the maintainer returned it verbatim with the prior page; the querying party re-sends it to fetch the next page.

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

## Security & Privacy

An ACL listing is the directory of who has what access. Maintainers **SHOULD** apply policy that limits enumeration to parties who already have a legitimate need to see the directory — for example, administrators and auditors. Public enumeration of an ACL is rarely appropriate.

Where the maintainer does respond with full entries, the response inherits the same sensitivity considerations as the underlying ACL: roles, scopes, and labels **MAY** be sensitive personal or organizational data. Confidentiality **SHOULD** be enforced at the transport layer (mutually-authenticated TLS, signed DIDComm envelope, etc.), even though the request payload itself contains no PII.

Implementations **SHOULD** populate `issuedAt` and **SHOULD** include a `proof` member where the list will be retained or forwarded; without the proof, a retained list cannot be attributed to its maintainer after the fact.
