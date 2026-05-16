---
slug: acl-grant
version: "1.0"
title: ACL — Grant
summary: A granting authority records, in a verifiable form, that a subject has been added to an access-control list with a named role and optional scopes.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - acl
  - access-control
  - authorization
  - role
  - grant
  - admin
authors:
  - DTGWG Governance TF
parties:
  - role: Granting authority
    requirement: REQUIRED
    vidSchemes: [did:web, did:key, x509]
  - role: ACL maintainer
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
proofRequirement:
  requirement: REQUIRED
  rationale: A grant is an evidentiary record that may be replayed by an auditor, used by a downstream service to corroborate authorization decisions, or relied on after the original transport has closed; transport-independent integrity is required.
errorCodes:
  - code: acl-grant:permission_denied
    meaning: The granting authority is not permitted to grant the requested role under the ACL maintainer's policy.
    retryable: false
  - code: acl-grant:role_not_recognized
    meaning: The role string is not part of the ACL maintainer's role vocabulary.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        offendingRole: { type: string }
        knownRoles:
          type: array
          items: { type: string }
  - code: acl-grant:expiry_in_past
    meaning: The supplied expiresAt lies in the past relative to the maintainer's clock.
    retryable: false
  - code: acl-grant:self_grant_prohibited
    meaning: The granting authority and the subject are the same VID; the maintainer's policy forbids self-grants.
    retryable: false
related:
  - acl-revoke
  - acl-change-role
  - acl-list
---

## Abstract

The **ACL — Grant** Trust Task records the addition of a subject to an access-control list. The *granting authority* asserts to the *ACL maintainer* that the *subject* identified in the payload is now entitled to the named `role`, optionally constrained to `scopes` and bounded by `expiresAt`. The maintainer applies its own policy to decide whether to accept the grant; if accepted, the document itself is the evidentiary record of the change.

The task is **idempotent** on `(subject, role, scopes)`: re-emitting an identical grant against an unchanged ACL produces no state change, and the `before` and `after` payload members will be equal. A grant that changes the subject's *role* **MUST NOT** use this task; use [`acl-change-role`](../../acl-change-role/1.0/spec.md) instead. A grant that *narrows* the subject's scopes is a revocation; use [`acl-revoke`](../../acl-revoke/1.0/spec.md).

This specification deliberately leaves the `role` vocabulary and the `scopes` semantics opaque. Each ACL maintainer publishes its own role list and scope conventions; the Trust Task carries the strings but does not interpret them.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels). The schema **MAY** change without notice while the cross-ecosystem ACL pattern stabilizes.

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the granting authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl-grant/1.0`.
2. Identify itself as `issuer`; identify the ACL maintainer as `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the granting authority's key material.
5. Populate `payload.after` with the resulting *AclEntry* and `payload.before` with the prior entry (or `null` if the subject was not previously in the ACL).

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the granting authority's declared verification material.
3. Apply its own policy to decide whether to accept the grant. Where the role string is not recognized, respond with `acl-grant:role_not_recognized`. Where the granting authority is not permitted to assign the requested role, respond with `acl-grant:permission_denied`.
4. On acceptance, persist the document (or a reference to it) as the evidentiary record of the change. On rejection, **SHOULD** return an *error response* per [SPEC.md §8](../../../SPEC.md#8-error-responses).

The maintainer **MUST NOT** apply this task to change a subject's existing role; receipt of an `acl-grant` whose `payload.before.role` differs from `payload.after.role` **MUST** be rejected with `acl-grant:permission_denied` and a `details.reason` indicating that role changes use [`acl-change-role`](../../acl-change-role/1.0/spec.md).

## Definitions

* **Granting authority.** The party invoking the grant; identified by `issuer`. Typically holds an "admin" or equivalent role under the maintainer's policy.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party being granted access; identified by `payload.subject`. Need not be a party in the framework sense, since the subject does not participate in the protocol exchange.
* **AclEntry.** The canonical record of one subject's membership in the ACL. Carried in `payload.before` (prior state) and `payload.after` (resulting state).
* **Role.** A short string interpreted by the ACL maintainer. The framework does not constrain the vocabulary; common examples include `admin`, `member`, `viewer`. Each maintainer publishes its role list as part of its ecosystem governance.
* **Scopes.** An array of opaque strings restricting where the role applies (for example, contexts, domains, resource prefixes). Their interpretation is defined by the maintainer.

## Security & Privacy

A grant document is **evidence**: a captured `acl-grant` Trust Task is sufficient to prove, after the fact, who authorized whom with what role. The `proof` requirement (**REQUIRED**) ensures the granting authority cannot repudiate the grant and that no intermediary can alter the subject, role, scopes, or expiry without invalidating the proof.

The `payload.subject` member identifies the entity being granted access. Where the subject is a natural person and the maintainer's role vocabulary is sensitive (for example, roles that signal membership in a regulated community), producers **SHOULD** apply transport confidentiality appropriate to the underlying privacy regime.

The optional `metadata` extension is **not** signed separately from the rest of the payload; producers **MUST NOT** place data in `metadata` that they would not be comfortable signing.
