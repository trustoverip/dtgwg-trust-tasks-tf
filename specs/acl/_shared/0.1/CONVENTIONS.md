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

### `allowedKeys` — a narrowing filter on the act axis

`allowedKeys` refines the act axis where the maintainer operates a signing
oracle: it names the key identifiers the subject may invoke it on. It
**intersects with `scopes` and can only narrow, never widen** — a key named in
`allowedKeys` that lies outside the entry's scopes remains unreachable.

Its absent-vs-empty rule is the same fail-closed convention, and it is the one
place where the two spellings deliberately differ:

- **absent** — no per-key filter: every key the entry's `scopes` reach
  (the behaviour of every entry that pre-dates the member);
- **present but empty** — authorized on **no** keys.

Emptiness is never a wildcard. A consumer MUST carry the absent-vs-empty
distinction end to end rather than testing emptiness at a call site.

## 6. Granting approve-authority

Conferring the *ability to confer* is a privilege escalation vector: a subject
that can grant approve-authority can manufacture an approver for an operation
it could not itself authorize. Maintainers **SHOULD** gate the granting of
`approve` more strictly than the granting of `scopes` — typically restricting
it to an unrestricted administrator — and **SHOULD** audit it distinctly from
an ordinary role grant.

## 7. Which task changes what

The family deliberately splits mutation across four tasks rather than offering
one general write, so that each kind of change is separately nameable,
separately gateable and separately auditable:

| Change | Task |
|---|---|
| Add a subject | [`acl/grant`](../../grant/0.1/spec.md) |
| Move a subject's role | [`acl/change-role`](../../change-role/0.1/spec.md) — requires the current role (compare-and-swap) |
| Amend label / scopes / allowed keys / expiry / step-up / approve | [`acl/update`](../../update/0.1/spec.md) |
| Remove authority, wholly or partly | [`acl/revoke`](../../revoke/0.1/spec.md) |

Two boundaries are enforced rather than advisory, and both exist so that a
reduction in authority cannot be performed by a task that does not look like
one:

- **`acl/grant` refuses a role change** on an existing subject.
- **`acl/update` refuses a narrowing of `scopes`**, directing the caller to
  `acl/revoke`.

An implementation that relaxed either would leave an audit trail in which
"withdrew production access" is indistinguishable from "corrected a label".

`allowedKeys` is the one deliberate exception to the second boundary:
`acl/revoke/0.1` cannot express a per-key reduction, so `acl/update` carries
it. A replacement that narrows `allowedKeys` **is a privilege reduction all
the same**, and a consumer MUST give it the same treatment the revocation
doctrine exists for — audit it distinctly as a reduction and apply it to the
subject's live sessions rather than letting the wider set survive until a
credential expires (see [`acl/update`](../../update/0.1/spec.md)).
