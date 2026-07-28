# acl — category conventions

This document records the conventions shared by every specification under the
`acl/` category. It is descriptive; where it could conflict with a specific
spec, **that spec's own front matter and payload schema are authoritative.**

## 1. Shared schema component

ACL payloads reference one shared component
([SPEC.md §6.6](../../../../SPEC.md#66-shared-schema-components)):

[`acl-entry.schema.json`](acl-entry.schema.json) — `$defs`: `AclEntry` (the
subject, role, scopes, and step-up requirement of one access-control entry).

## 2. Recipient party

Every `acl/` task is addressed to the **ACL maintainer** — the party tagged
`member: recipient`, declared `REQUIRED`. The producer is the
granting/revoking/querying authority. Per
[SPEC.md §7.2 item 5](../../../../SPEC.md#72-consumer-requirements) the
`recipient` is enforced in-band.

## 3. Proof convention

Tasks that **mutate the access-control list** declare `proofRequirement:
REQUIRED` (`acl/grant`, `acl/revoke`, `acl/change-role`, `acl/swap-key`):
an ACL change is evidentiary and may be relied on after delivery (see
[SPEC.md §4.7.1](../../../../SPEC.md#471-when-to-include-a-proof)). Read-only
tasks declare `RECOMMENDED` (`acl/list`, `acl/show`). The per-spec declaration
is authoritative.

## 4. Cross-slug extended error codes

A consumer that surfaces a rule canonically declared on a *related* acl spec
**MUST** namespace the extended `code` under the slug it is *processing*, not
the related slug — e.g. `acl/change-role:lastAuthorityProtected`, never
`acl/revoke:lastAuthorityProtected` (see
[SPEC.md §8.5](../../../../SPEC.md#85-extension-by-individual-trust-task-specifications)
and [CONTRIBUTING-SPECS.md](../../../../CONTRIBUTING-SPECS.md)). Extended-code
local parts are lowerCamelCase ([SPEC.md §4.10](../../../../SPEC.md#410-naming-conventions)).

## 5. Act vs confer — two independent authority axes

`AclEntry` carries two authority axes that a consumer **MUST** resolve
separately:

- **`scopes`** — what the subject may **exercise itself**.
- **`approve`** — what the subject may **confer on others** by ratifying an
  approval (a step-up ratification, a task-consent delegation).

They are independent in both directions. A subject may hold approve-authority
over scopes it cannot act in — that is the least-privilege approver, and it is
the reason the axes are separate rather than one list. It may equally act in
scopes it cannot confer.

Two consumer rules follow, and both have been got wrong in practice:

1. **Approve-authority is not authority to act.** A consumer that answers "may
   this party do X" by consulting `approve` grants an approver the ability to
   perform what it was only meant to sign off on.
2. **Omission confers nothing.** An absent `approve`, an absent `approve.all`,
   and an empty `approve.scopes` all mean "may ratify nothing". A consumer that
   has not implemented the member therefore confers *less* than the producer
   intended — the direction a missed member has to fail in.

**Do not infer either axis from an empty list.** An empty `scopes` and an empty
`approve.scopes` mean "nothing", never "everything"; the wildcard is
`approve.all`, and there is deliberately no `scopes` equivalent of it — a
maintainer that wants unrestricted authority expresses it through a role, not
through an absent scope list. Reading emptiness as unrestricted is a known
source of privilege-escalation bugs in ACL implementations.

## 6. Granting approve-authority

Conferring the *ability to confer* is a privilege escalation vector: a subject
that can grant approve-authority can manufacture an approver for an operation
it could not itself authorize. Maintainers **SHOULD** gate the granting of
`approve` more strictly than the granting of `scopes` — typically restricting
it to an unrestricted administrator — and **SHOULD** audit it distinctly from
an ordinary role grant.
