---
slug: acl/revoke
version: "0.2"
title: ACL — Revoke
summary: A revoking party removes a subject's AclEntry 0.2 from an access-control list, or narrows its act scope to a stated, explicit narrower scope.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - acl
  - access-control
  - authorization
  - revocation
  - remove
  - leave
parties:
  - role: Revoking party
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A revocation is the evidentiary counterpart to a grant; the maintainer, the former subject, and any downstream party that retained the grant need to verify, after the fact, who withdrew the authority and when.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A revoke replayed after the subject has legitimately been granted again removes them a second time, and nothing in the payload distinguishes the stale copy from a fresh instruction to revoke.
sideEffects:
  level: mutating
  rationale: "Removes an entry, or narrows its act scope; recoverable via acl/grant or acl/update."
consequences:
  - The subject loses the revoked authority at its next authorization decision — not when a session or token expires.
  - A removal also ends every approval authority the entry held, and leaves entries delegated from it for the maintainer to re-evaluate.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The request names one subject and the authority to withdraw; the response returns the narrowed entry, or null, to the revoking party.
retention:
  class: durable
  rationale: The accepted document is the record that authority ended — the counterpart an auditor reads beside the grant it cancels.
subjectPath: /subject
errorCodes:
  - code: acl/revoke:subjectNotPresent
    meaning: The subject named in the payload is not currently in the ACL.
    retryable: false
  - code: acl/revoke:lastAuthorityProtected
    meaning: The revocation would leave the ACL with no party able to perform a privileged operation; the maintainer's policy forbids it.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        protectedRole: { type: string }
        remainingHolders:
          type: array
          items: { type: string }
  - code: acl/revoke:notNarrowing
    meaning: The stated act scope is not strictly narrower than the entry's stored act scope. Widening, or restating, is acl/update/0.2.
    retryable: false
  - code: acl/revoke:invalidActScope
    meaning: The stated act scope names a context the maintainer does not hold, or uses the `contexts` shape at a maintainer without contexts.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        offendingContexts:
          type: array
          items: { type: string }
related:
  - acl/grant
  - acl/update
  - acl/change-role
  - acl/list
---

## Abstract

The **ACL — Revoke** Trust Task removes a subject's entry from an access-control list, or narrows the entry's act scope. Version 0.2 operates on [AclEntry 0.2](../../_shared/0.2/acl-entry.schema.json) and makes the revocation itself explicit:

1. **Removal** — `revocation: {"kind": "entry"}`. The entry is gone.
2. **Act-scope narrowing** — `revocation: {"kind": "act", "act": <AuthorityScope>}`. The entry stays with the stated, strictly narrower act scope — a subset of its contexts, or `{"scope": "none"}` (which leaves, for example, a least-privilege approver).
3. **Self-revocation** — either of the above with `issuer == payload.subject`, under the maintainer's self-revoke policy.

0.1 selected full removal by *omitting* `scopes` and expressed a narrowing as the list of scopes to remove. 0.2 states the kind of revocation, and for a narrowing states the **resulting** act scope rather than a difference, so a dropped member cannot turn a narrowing into a removal and producer and maintainer cannot disagree about what remains.

Reductions of the other axes — capabilities, approvable capabilities, approve scope, keys — are not revocations in this task's vocabulary; they are carried by [`acl/update/0.2`](../../update/0.2/spec.md), which treats each as a privilege reduction with the same audit and immediacy obligations.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)). [`acl/revoke/0.1`](../0.1/spec.md) remains current for maintainers holding AclEntry 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Confirm the revoking party is the subject (self-revocation) or a party entitled to modify the subject's entry — at a maintainer with hierarchical contexts, one whose act scope covers every context in the entry's act scope and approve scope; overlap is not enough. Otherwise respond with `permissionDenied`.
2. Respond with `subjectNotPresent` where the subject has no entry.
3. For a narrowing, refuse an act scope that is not strictly narrower than the stored one with `notNarrowing`, and one the maintainer cannot hold with `invalidActScope`.
4. Refuse a revocation that would leave the ACL with no holder of a privileged role the maintainer's policy protects with `lastAuthorityProtected`. At a maintainer with unrestricted administrators, that count is of unrestricted administrators, and it covers removal and narrowing alike.
5. Apply the revocation at the subject's **next authorization decision**. It **MUST NOT** defer the effect to the expiry of an existing session, token or cached decision.
6. Re-evaluate the entries whose `delegatedBy` is the subject, since a delegated entry must not retain authority its delegator has lost ([CONVENTIONS §9](../../_shared/0.2/CONVENTIONS.md)).
7. On acceptance, persist the document as the evidentiary record and return the resulting state.

A revocation that is one step of a sweep over a context is enumerated with [`acl/list/0.2`](../../list/0.2/spec.md) in the `subtree` direction; a sweep that cannot remove every entry it found reports itself incomplete and names the entries that remain, rather than reporting success.

## Authorization

**Authority: standing to modify the subject's entry, or being the subject.** A revocation is a modification of the entry, so what entitles the revoking party is that its own stored entry covers all of the entry it is revoking — every context the entry may act or approve in — or that it is the subject withdrawing its own authority. Conformance item 1 enforces this. Narrowing is bounded only from above: a party entitled to modify an entry may narrow it to anything narrower.

`lastAuthorityProtected` is **not** an authorization check. It fires after the caller has been authorized, and protects the resulting state — the ACL must keep a holder of a privileged role — not who asked.

Removing or narrowing another subject's **unrestricted act scope**, or an entry holding an **authority-conferring capability**, is the act an administrator could use to seize a community or node from its peers. Some maintainers therefore hold it for the consent of a party other than both the requester and the subject, and where no such party exists, require the requester's re-authentication, notify the subject and audit the removal at the highest severity (VTI-APV-019 of the VTI specification). That is maintainer policy, described here because it shapes how a revocation is processed; this specification does not require it.

The `proof` identifies the revoking party so its stored entry can be read and the revocation attributed; it is not the authorization.

## Definitions

* **Revoking party.** The party invoking the revocation; `issuer`. An entitled administrator or the subject.
* **ACL maintainer.** The party that holds and enforces the list; `recipient`.
* **Subject.** The party being removed or narrowed; `payload.subject`.
* **Strictly narrower.** An act scope that reaches a proper subset of what the stored one reaches: a proper subset of its contexts (by the maintainer's ancestry predicate), or `none` from anything else. `all` is narrower than nothing.
* **Self-revocation.** `issuer == payload.subject`. Consumers recognise it explicitly and apply their self-revoke policy.

## Request

The revoking party sends the ACL maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Removal by an administrator

```json
{
  "id": "urn:uuid:9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/revoke/0.2#request",
  "issuer": "did:web:root-admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-10-02T18:00:00Z",
  "payload": {
    "subject": "did:web:erin.example",
    "revocation": { "kind": "entry" },
    "reason": "Settlement integration decommissioned."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:root-admin.example#key-1",
    "created": "2026-10-02T18:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4abExampleProofValue"
  }
}
```

### Act-scope narrowing

Alice keeps `ctx/payments` and loses `ctx/refunds`. The payload states what remains.

```json
{
  "id": "urn:uuid:7a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/revoke/0.2#request",
  "issuer": "did:web:root-admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-10-02T18:05:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "revocation": {
      "kind": "act",
      "act": { "scope": "contexts", "contexts": ["ctx/payments"] }
    },
    "reason": "Refunds moved to another team."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:root-admin.example#key-1",
    "created": "2026-10-02T18:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5bcExampleProofValue"
  }
}
```

## Response

The ACL maintainer answers with a document of type `https://trusttasks.org/spec/acl/revoke/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `entry` — `null` after a removal; the resulting AclEntry after a narrowing.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### Removal

```json
{
  "id": "urn:uuid:ae2a1c44-7b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/revoke/0.2#response",
  "threadId": "urn:uuid:9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:root-admin.example",
  "issuedAt": "2026-10-02T18:00:01Z",
  "payload": {
    "entry": null
  }
}
```

### Narrowing

```json
{
  "id": "urn:uuid:8a91c7b3-2e62-4a91-a3a4-9d61b75e2f02",
  "type": "https://trusttasks.org/spec/acl/revoke/0.2#response",
  "threadId": "urn:uuid:7a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:root-admin.example",
  "issuedAt": "2026-10-02T18:05:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "act": { "scope": "contexts", "contexts": ["ctx/payments"] },
      "keys": { "scope": "all" },
      "capabilities": { "scope": "ceiling" },
      "approve": { "scope": "none" },
      "createdAt": "2026-10-02T10:00:01Z",
      "createdBy": "did:web:root-admin.example",
      "delegatedBy": "did:web:root-admin.example",
      "updatedAt": "2026-10-02T18:05:01Z",
      "updatedBy": "did:web:root-admin.example"
    }
  }
}
```

## Security & Privacy

### Data carried

The request names one subject, the kind of revocation and, for a narrowing, the resulting act scope; the response returns the narrowed entry or `null`. Context paths can reveal organizational structure, and a revocation of a natural person's access can itself be sensitive. A producer **MUST NOT** put personal data in `reason` or `ext` beyond what the audit trail needs.

### Correlation

The maintainer learns who revoked whom, and when; an intermediary without transport confidentiality learns the same. A revocation is correlatable by design with the grant it cancels — that is its evidentiary value. `threadId` joins request and response.

### Retention

The accepted document is **durable**: maintainers **SHOULD** keep it alongside the grant it cancels. Where retention is bounded by privacy regulation, they **SHOULD** keep at least `id`, `threadId`, `issuer`, `issuedAt` and `payload.subject`, so the audit trail stays intact even if other fields are trimmed.

### Consent/purpose

The data is collected to withdraw and record the withdrawal of authority. It **SHOULD NOT** be reused for other purposes. Whether a particular revocation — notably of another unrestricted administrator — must first be consented to by a third party, or preceded by a step-up, is the maintainer's policy and is not declared here.
