---
slug: acl/update
version: "0.1"
title: ACL — Update
summary: Amend the non-role attributes of an existing access-control entry — its label, scopes, expiry, step-up requirement, or approve-authority — leaving role changes to acl/change-role.
status: draft
targetFrameworkVersion: "0.1"
category: access-control
keywords:
  - acl
  - access-control
  - authorization
  - update
  - amend
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Granting authority
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: An amendment changes what a subject may do or who may vouch for it, and may be relied on after the transport has closed. Same evidentiary standing as the grant it modifies.
sideEffects:
  level: mutating
  rationale: "Replaces attributes of an existing entry; recoverable by amending again, or by acl/revoke."
consequences:
  - The subject's scopes, expiry, step-up requirement or approve-authority change immediately.
  - Clearing `expiresAt` makes a time-boxed grant permanent.
  - Widening `approve` lets the subject vouch for operations it could not previously authorize.
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: acl/update:notFound
    meaning: No entry exists for `subject`. This task amends; use acl/grant to create.
    retryable: false
  - code: acl/update:roleChangeNotPermitted
    meaning: The payload attempted a role change. Use acl/change-role, which requires the current role.
    retryable: false
  - code: acl/update:narrowingNotPermitted
    meaning: The replacement scopes remove authority the entry currently holds. Use acl/revoke.
    retryable: false
related:
  - acl/grant
  - acl/change-role
  - acl/revoke
---

## Abstract

The **ACL — Update** Trust Task amends an entry that already exists: its label, scopes, expiry, step-up requirement, or approve-authority.

It exists because the `acl/*` family had a gap. [`acl/grant`](../../grant/0.1/) creates and is explicitly *not* a role-change path; [`acl/change-role`](../../change-role/0.1/) moves the role and nothing else; [`acl/revoke`](../../revoke/0.1/) removes. None of them can say "this entry keeps its role but its step-up approver has changed" — a maintainer with that need had either to model it as revoke-then-grant, which is a window in which the subject holds nothing, or to invent a private task.

## What it deliberately cannot do

**It cannot change a role.** That transition belongs to `acl/change-role`, which requires the current role as a compare-and-swap. Two callers amending the same entry concurrently would otherwise silently overwrite one another on the single attribute where a lost update is a privilege change rather than a cosmetic one. A payload naming a role is rejected with `roleChangeNotPermitted`, not quietly ignored.

**It cannot narrow scopes.** `scopes` replaces the set wholesale, and a consumer MUST refuse a replacement that removes authority the entry currently holds, directing the caller to `acl/revoke` with `narrowingNotPermitted`.

That looks like an inconvenience and is the point: removing authority is the operation an auditor most needs to find, and it should appear in exactly one place. If narrowing were expressible here, a revocation could be performed by a task whose name and error vocabulary say "amend", and the audit trail would no longer distinguish "we adjusted a label" from "we withdrew access to production".

## Replacement, not merge

Every member replaces rather than merges. A caller adding one scope sends the full intended set.

Merge semantics would make the payload ambiguous in the direction that matters: with a merge, there is no way to express removal at all, and the absence of a member and the presence of an empty one come to mean the same thing. Replacement keeps *omitted* (leave alone) distinct from *explicitly null* (clear) and from *empty array* (set to nothing) — three different intentions a maintainer must be able to tell apart.

## Conformance

Producer: send `subject` plus only the members being changed. Omit what should stay. Send explicit `null` to clear a nullable member.

Consumer: reject an unknown subject with `notFound` — this task does not create. Reject any role member with `roleChangeNotPermitted`. Compare a replacement `scopes` against what the entry holds and reject a narrowing with `narrowingNotPermitted`. Apply the additive-only rule to `stepUp` exactly as for a grant: a per-entry setting may raise the required assurance above the system floor, never lower it. Record the amendment with the caller's identity and any `reason`.

## Security & Privacy

Two members are privilege *increases* and warrant gating beyond the maintainer's ordinary amend permission:

- **Clearing `expiresAt`** converts a time-boxed grant into a permanent one. A short-lived grant is a common way to bound blast radius, and removing the bound is not a lesser act than issuing the grant.
- **Widening `approve`** lets the subject confer authority on others. A subject able to grant approve-authority can manufacture an approver for an operation it could not itself authorize, so a maintainer SHOULD restrict this to an unrestricted administrator and audit it distinctly (see `_shared/0.1/CONVENTIONS.md` §6).

`exposure.discloses` is `none`: the payload names a subject and the attributes being set, and asserts nothing about anyone else.
