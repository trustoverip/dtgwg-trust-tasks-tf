---
slug: acl/list
version: "0.2"
title: ACL — List
summary: A querying party asks an ACL maintainer to enumerate its AclEntry 0.2 records, filtered by role, by context or resource in an explicitly stated direction, and by capability, with paging.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - acl
  - access-control
  - list
  - enumeration
  - revocation
  - capability
parties:
  - role: Querying party
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Most list queries are short-lived and consumed over an authenticated transport; a proof becomes valuable when the list is retained, replayed, or relied upon by a third party — for instance as the evidence that a revocation sweep found everything it removed.
sideEffects:
  level: none
  rationale: "Read-only enumeration of ACL entries."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Discloses the access state — roles, scopes, capabilities and qualifiers — of every entry the filters match and the maintainer's policy lets the querying party see.
retention:
  class: transient
  rationale: A listing is a point-in-time read; the maintainer's entries are the record. A listing kept as evidence of a sweep is the querying party's choice and should carry a proof.
errorCodes:
  - code: acl/list:cursorMismatch
    meaning: The `cursor` was minted under different filters or a different `direction` than this request states. A cursor never resumes under a changed question.
    retryable: false
related:
  - acl/show
  - acl/grant
  - acl/update
  - acl/revoke
---

## Abstract

The **ACL — List** Trust Task lets a *querying party* ask the *ACL maintainer* for the entries in its access-control list. Version 0.2 returns [AclEntry 0.2](../../_shared/0.2/acl-entry.schema.json) records and changes the filters to match: 0.1's `scope` filter becomes `context`, read against the entries' explicit act scope, and its `direction` is now **required** whenever a hierarchical filter is given; new `capability` and `resource` filters find entries by what they may do and where.

The task is **read-only**. It is the enumeration half of the split pair with [`acl/show/0.2`](../../show/0.2/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)). [`acl/list/0.1`](../0.1/spec.md) remains current for maintainers holding AclEntry 0.1.

## Reading a hierarchical filter in two directions

Where a maintainer's contexts (or resource qualifiers) are hierarchical, one identifier raises two opposite questions:

| `direction` | Question | An entry matches when |
|---|---|---|
| `actingIn` | Who may act in this context / on this resource? | its act scope (or qualifier) is an ancestor-or-self of the queried one. An unrestricted act scope (`all`) and an unqualified capability match. |
| `subtree` | What is granted beneath this context / resource? | the queried one is an ancestor-or-self of a context in its act scope (or of its qualifier). An unrestricted act scope and an unqualified capability do **not** match; an entry naming contexts both inside and outside the subtree **does**. |
| `any` | Whose authority touches this subtree? | either holds. |

**There is no default.** 0.1 defaulted to `acting-in` when `direction` was omitted; 0.2 refuses a hierarchical filter without a direction, and a direction without a filter to apply it to. Either default answers a question the caller did not ask, in a form indistinguishable from the one they did, and the caller most likely to be harmed is the one performing a revocation: a sweep asked `actingIn` when it meant `subtree` receives the ancestors — which keep their authority — and none of the leaf-scoped grants it exists to cut. The result is short rather than empty, so it reads as complete.

The two edges in the `subtree` row are deliberate. An unrestricted entry names no context, so it is not a grant *of* the branch, and including it would hand a caller revoking a compromised branch its own super-administrator to delete. An entry straddling the boundary does hold a grant inside the branch, and omitting it would under-report exactly the case the direction exists to surface.

The values were `acting-in` / `subtree` / `any` in 0.1; 0.2 re-cases the first to `actingIn` to follow the registry's lowerCamelCase convention for enumerated values ([SPEC §4.10](/SPEC.md#410-naming-conventions)). A consumer whose contexts are flat **MAY** treat the three alike.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Apply its own policy to decide whether the querying party may enumerate entries, and refuse with `permissionDenied` where it may not. A maintainer **SHOULD** return an entry wherever the querying party could modify it, and additionally wherever the entry's approve scope reaches the querying party.
2. Refuse a `context` or `resource` filter without `direction`, a `direction` with neither, and an unrecognised `direction`, as `malformedRequest`; the refusal's message **SHOULD** name the valid values. It **MUST NOT** substitute a default.
3. Apply the filters conjunctively. `context` is evaluated against each entry's **act** scope, by the table above. `capability` is evaluated against each entry's **effective** capability set — the role's full ceiling for `{"scope": "ceiling"}`, nothing for `none`, and for `listed` the ceiling ∩ the non-additive grants plus the additive grants — so a maintainer **MUST NOT** answer it by searching listed grants alone, which would miss every entry holding its role's full ceiling. `resource` is evaluated against the qualifiers of the entry's effective capabilities, by the same table.
4. Return a filter value it does not recognise (an unknown role, context or capability) as zero matches, not an error.
5. Honor `pageSize` and return a continuation `cursor` if more entries remain. The cursor **MUST** bind the filters and `direction` it was minted under; a request resuming it under any other is refused with `cursorMismatch`.
6. Return entries in the explicit 0.2 form (see [`acl/show/0.2`](../../show/0.2/spec.md) Conformance items 2–3 on redaction).

The context and capability filters read the **act** axis only. An entry whose approve scope reaches a context, or whose `approveCapabilities` names a capability, is not matched by them; a caller sweeping all authority over a context — approvers included — enumerates by role or without the hierarchical filter and inspects `approve` itself.

## Definitions

* **Querying party.** The party initiating the query; `issuer`.
* **ACL maintainer.** The party answering; `recipient`.
* **Effective capability set.** The role's ceiling for `ceiling`; ∅ for `none`; (role ceiling ∩ non-additive grants) ∪ additive grants for `listed` ([CONVENTIONS §6](../../_shared/0.2/CONVENTIONS.md)).
* **Cursor.** An opaque string the maintainer returns to page through a large result; treated as opaque and re-sent verbatim with the same filters.

## Request

The querying party sends the ACL maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json). All members are optional; an empty payload requests the default list, subject to `pageSize`.

### Revocation sweep beneath a context

An operator revoking a compromised branch asks what is granted at or beneath `ctx/payments`.

```json
{
  "id": "urn:uuid:5b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/list/0.2#request",
  "issuer": "did:web:root-admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-10-02T13:00:00Z",
  "payload": {
    "context": "ctx/payments",
    "direction": "subtree",
    "pageSize": 100
  }
}
```

### Who may manage repositories in a namespace

```json
{
  "id": "urn:uuid:6c4d6f3b-2c92-4e4f-8c62-8b4d9a0f4e03",
  "type": "https://trusttasks.org/spec/acl/list/0.2#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T13:05:00Z",
  "payload": {
    "capability": "git.repo.manage",
    "resource": "git-ns:github.com/acme",
    "direction": "actingIn"
  }
}
```

## Response

The ACL maintainer answers with a document of type `https://trusttasks.org/spec/acl/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `entries` — the matching AclEntry records, in maintainer-defined order; **MAY** be empty.
* `truncated` — **REQUIRED**; `true` when more matching entries exist. Consumers **MUST** check it before treating `entries` as exhaustive.
* `cursor` — present only when `truncated` is `true` and the maintainer can continue. `truncated: true` without a `cursor` means the consumer **SHOULD** narrow its filter and re-query.
* `redactedFields` — optional; AclEntry members redacted from every returned entry.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### The sweep's answer

Response to the revocation sweep. The super-administrator (`act: all`) is absent, as `subtree` requires; Erin, scoped to a leaf beneath the branch, and Alice, straddling it, are present.

```json
{
  "id": "urn:uuid:7d5e7a4c-3da3-4f5a-9d73-9c5eab1a5f14",
  "type": "https://trusttasks.org/spec/acl/list/0.2#response",
  "threadId": "urn:uuid:5b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:root-admin.example",
  "issuedAt": "2026-10-02T13:00:01Z",
  "payload": {
    "entries": [
      {
        "subject": "did:web:alice.example",
        "role": "admin",
        "act": { "scope": "contexts", "contexts": ["ctx/payments", "ctx/refunds"] },
        "keys": { "scope": "all" },
        "capabilities": { "scope": "ceiling" },
        "approve": { "scope": "none" },
        "createdAt": "2026-10-02T10:00:01Z",
        "createdBy": "did:web:root-admin.example",
        "delegatedBy": "did:web:root-admin.example"
      },
      {
        "subject": "did:web:erin.example",
        "role": "application",
        "act": { "scope": "contexts", "contexts": ["ctx/payments/settlement"] },
        "keys": { "scope": "listed", "keys": ["key-settlement-1"] },
        "capabilities": { "scope": "ceiling" },
        "createdAt": "2026-10-01T09:00:00Z",
        "createdBy": "did:web:alice.example",
        "delegatedBy": "did:web:alice.example"
      }
    ],
    "truncated": false
  }
}
```

### Refused: a cursor resumed under a different direction

```json
{
  "id": "urn:uuid:8e6f8b5d-4eb4-4a6b-8e84-ad6fbc2b6025",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:9f7a9c6e-5fc5-4b7c-9f95-be7acd3c7136",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:root-admin.example",
  "issuedAt": "2026-10-02T13:10:01Z",
  "payload": {
    "code": "acl/list:cursorMismatch",
    "message": "This cursor was minted for direction subtree.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries filters — a role, a context path, a capability, a resource qualifier, a subject prefix — which themselves reveal what the querying party is looking for. The response carries the access state of every matching entry: subjects, roles, scopes, capabilities with qualifiers, key scopes, expiry and delegation provenance. An ACL listing is the directory of who may do what; maintainers **SHOULD** limit enumeration to parties with a legitimate need (administrators, auditors, approvers for the entries they approve). Redaction is available, but never of `act`, `keys` or `capabilities`, and `approve` / `approveCapabilities` **SHOULD NOT** be redacted because their absence means none.

### Correlation

A listing lets the querying party join every returned subject to its authority in one document, and repeated listings show how the ACL changes over time. `delegatedBy` exposes the delegation graph. An intermediary without transport confidentiality sees the same; confidentiality **SHOULD** be enforced at the transport layer. Response size and paging reveal the approximate size of the ACL even when entries are redacted.

### Retention

A listing is a point-in-time read and **SHOULD NOT** be retained beyond the decision it informs — except where it is the evidence of a revocation sweep, in which case the querying party **SHOULD** require a `proof` so the listing can later be attributed to the maintainer that produced it.

### Consent/purpose

The data is disclosed so that administrators, auditors and approvers can see and manage grants — above all, so that a revocation can find every grant it must remove. It **SHOULD NOT** be reused to profile the listed subjects. Who may enumerate what is the maintainer's policy.
