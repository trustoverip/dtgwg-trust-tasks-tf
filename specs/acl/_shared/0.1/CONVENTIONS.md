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
