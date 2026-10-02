---
slug: acl/show
version: "0.2"
title: ACL — Show
summary: A querying party asks an ACL maintainer for the AclEntry 0.2 it holds for one subject — role, explicit act and approve scopes, and capabilities.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - acl
  - access-control
  - lookup
  - query
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
  rationale: A single-entry lookup is typically short-lived and consumed over an authenticated transport; a proof becomes valuable when the answer is retained, replayed, or relied upon by a third party.
sideEffects:
  level: none
  rationale: "Read-only read of a single ACL entry."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Discloses one subject's access state — role, act and approve scopes, capabilities and their resource qualifiers — to a querying party the maintainer's policy admits.
retention:
  class: transient
  rationale: The answer is a point-in-time read; the maintainer's own entry, not this document, is the record, and the querying party has no reason to keep it beyond the decision it informs.
subjectPath: /subject
errorCodes: []
related:
  - acl/list
  - acl/grant
  - acl/update
  - acl/revoke
---

## Abstract

The **ACL — Show** Trust Task lets a *querying party* ask the *ACL maintainer* whether a specific subject has an entry and, if so, what it contains. Version 0.2 returns an [AclEntry 0.2](../../_shared/0.2/acl-entry.schema.json): the role, the explicit act scope, the approve scope, the capabilities and the approvable capabilities. The response is the entry or `entry: null` — "no such entry" is a successful answer, not an error.

The task is **read-only**. It is the single-record half of the split pair with [`acl/list/0.2`](../../list/0.2/spec.md): a lookup for an unknown subject is a definite `null`, where a list filtered to one subject would return an empty page indistinguishable from "filtered out".

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)). [`acl/show/0.1`](../0.1/spec.md) remains current for maintainers holding AclEntry 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Apply its own policy to decide whether the querying party may read the entry, and refuse with the framework's `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) where it may not. A maintainer **SHOULD** permit a read wherever the querying party could modify the entry, and additionally wherever the entry's approve scope reaches the querying party — an approver needs to see the grants it is asked to bless decisions about, without thereby being able to change them.
2. Return the entry with every member it holds, in the explicit 0.2 form. Every entry carries `act`, `keys` and `capabilities` explicitly; `approve` and `approveCapabilities` are returned whenever they confer anything.
3. Where policy redacts members, list them in `redactedFields`. It cannot redact `act`, `keys` or `capabilities`, which are REQUIRED, and **SHOULD NOT** redact `approve` or `approveCapabilities`: a redacted one is indistinguishable from an absent one, which means none, so the reader would be misinformed. A maintainer that will not disclose an authority member **SHOULD** refuse the read instead.
4. Where a maintainer will not reveal to this querying party whether the subject has an entry, answer `entry: null` rather than `permissionDenied`, so that the error does not itself disclose existence.

The maintainer **SHOULD** permit self-lookup: a querying party whose `issuer` equals `payload.subject` **SHOULD** receive its own entry.

## Definitions

* **Querying party.** The party initiating the lookup; `issuer`.
* **ACL maintainer.** The party answering; `recipient`.
* **Subject.** The party whose entry is looked up; `payload.subject`. May be the querying party.

## Request

The querying party sends the ACL maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An approver looks up the subject of a pending approval

```json
{
  "id": "urn:uuid:a82a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/show/0.2#request",
  "issuer": "did:web:dana.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T12:00:00Z",
  "payload": {
    "subject": "did:web:bob.example"
  }
}
```

## Response

The ACL maintainer answers with a document of type `https://trusttasks.org/spec/acl/show/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `entry` — the AclEntry the maintainer holds for the subject, or `null`.
* `redactedFields` — optional; the AclEntry members omitted from `entry`.

Failures (e.g. `permissionDenied`) use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### The entry, with its label redacted

Dana's approve scope reaches Bob's entry, so the maintainer lets her read it; its policy hides labels from non-administrators.

```json
{
  "id": "urn:uuid:ba2a1c44-7b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/show/0.2#response",
  "threadId": "urn:uuid:a82a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:community.example",
  "recipient": "did:web:dana.example",
  "issuedAt": "2026-10-02T12:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:bob.example",
      "role": "repo-manager",
      "act": { "scope": "all" },
      "keys": { "scope": "none" },
      "capabilities": {
        "scope": "listed",
        "grants": [{ "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" }]
      },
      "approve": { "scope": "all" },
      "approveCapabilities": {
        "scope": "listed",
        "grants": [{ "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" }]
      },
      "createdAt": "2026-10-02T10:05:01Z",
      "createdBy": "did:web:carol.example",
      "delegatedBy": "did:web:carol.example",
      "expiresAt": "2027-04-02T00:00:00Z"
    },
    "redactedFields": ["label"]
  }
}
```

### Subject not in the ACL

```json
{
  "id": "urn:uuid:d12c7b32-7a91-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/show/0.2#response",
  "threadId": "urn:uuid:c91c7b32-7a91-4a91-a3a4-9d61b75e2f00",
  "issuer": "did:web:community.example",
  "recipient": "did:web:dana.example",
  "issuedAt": "2026-10-02T12:10:01Z",
  "payload": {
    "entry": null
  }
}
```

## Security & Privacy

### Data carried

The request carries one subject identifier. The response carries that subject's whole access state: role, act and approve scopes, capability scopes with resource qualifiers, the key scope, expiry, provenance including `delegatedBy`, and any label or extension members policy does not redact. Context paths and qualifiers can reveal organizational structure; a role or capability can reveal a person's position. The smallest response that answers the task is the entry minus members the querying party has no need to see — but never minus an authority member, whose omission would misstate the grant (Conformance items 2–3).

### Correlation

Allowing arbitrary parties to confirm whether a VID is in the ACL is itself a disclosure; maintainers **SHOULD** limit lookups to parties with a legitimate need, and item 4 keeps a refusal from leaking existence. The querying party can correlate the subject's entry across repeated lookups over time (and see its changes). An intermediary without transport confidentiality learns the same; confidentiality **SHOULD** be enforced at the transport layer.

### Retention

The answer is a point-in-time read. The querying party **SHOULD NOT** retain it beyond the decision it informs; the maintainer's entry is the record. Where a querying party does retain an answer as evidence, it **SHOULD** require a `proof` on the response.

### Consent/purpose

The data is disclosed so that a party with a legitimate interest — an administrator, an approver deciding on a pending action, the subject itself — can see a grant. It **SHOULD NOT** be reused to profile the subject. Who may look up whom is the maintainer's policy.
