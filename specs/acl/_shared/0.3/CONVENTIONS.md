# acl — category conventions (AclEntry 0.3)

This document records the conventions shared by every specification under the
`acl/` category that references
[`acl-entry.schema.json`](acl-entry.schema.json) **0.3**. It is descriptive;
where it could conflict with a specific spec, **that spec's own front matter and
payload schema are authoritative.** The 0.1 and 0.2 conventions
([`../0.1/CONVENTIONS.md`](../0.1/CONVENTIONS.md),
[`../0.2/CONVENTIONS.md`](../0.2/CONVENTIONS.md)) continue to govern the tasks
on those versions.

**What 0.3 adds to 0.2:** one axis, `purposes` — what the subject may use
content it reads *for* (§12). It is REQUIRED and explicit like `act`, `keys`
and `capabilities`, and nothing else changes. The requirements are the VTI
specification's VTI-ACL-100 – 110.

The one idea behind every change since 0.2: **authority is stated, never inferred
from a shape.** A grant is a statement about authority; whether a list is empty,
absent or omitted by a serializer is a property of an encoder. Where the second
is allowed to imply the first, the authority a deployment holds depends on a
library's configuration.

## 1. Shared schema components

[`acl-entry.schema.json`](acl-entry.schema.json) — `$defs`:

| Definition | What it is |
|---|---|
| `AclEntry` | One access-control entry: subject, role, act scope, approve scope, capabilities, approvable capabilities, key scope, purpose scope, delegation provenance, step-up. |
| `AuthorityScope` | The explicit three-shape scope used for both `act` and `approve`: `{"scope":"all"}`, `{"scope":"none"}`, or `{"scope":"contexts","contexts":[…]}` with at least one path. |
| `KeyScope` | The explicit key scope: `{"scope":"all"}`, `{"scope":"none"}`, or `{"scope":"listed","keys":[…]}` with at least one key. |
| `PurposeScope` | The explicit purpose scope: `{"scope":"any"}` or `{"scope":"listed","purposes":[…]}` with at least one `Purpose`. |
| `Purpose` | A purpose: a lowerCamelCase name registered in the VTI specification's purpose registry, or an absolute IRI. |
| `CapabilityScope` | The explicit capability scope: `{"scope":"ceiling"}`, `{"scope":"none"}`, or `{"scope":"listed","grants":[CapabilityGrant, …]}` with at least one grant. |
| `ApproveCapabilityScope` | The same three shapes for approvable capabilities, with `CapabilityRef` grants. |
| `CapabilityRef` | A capability, optionally qualified by a resource. Names a capability without granting it — role ceilings, `approveCapabilities`. |
| `CapabilityGrant` | A `CapabilityRef` held by an entry, optionally marked `additive`. |
| `Capability`, `ResourceQualifier`, `ContextPath` | The opaque identifier grammars. |

Other families reuse `CapabilityRef` / `CapabilityGrant` by cross-file `$ref`
(the `vtc/roles` role definitions do), so a capability means the same thing on
an entry and on the role that bounds it.

## 2. Recipient party and proof convention

Unchanged from 0.1. Every `acl/` task is addressed to the **ACL maintainer**
(`member: recipient`, `REQUIRED`). Tasks that mutate the list declare
`proofRequirement: REQUIRED`; read-only tasks declare `RECOMMENDED`.

## 3. Cross-slug extended error codes

Unchanged from 0.1 §4: a consumer surfacing a rule declared on a related spec
namespaces the code under the slug it is *processing*
(`acl/update:capabilityOutsideCeiling`, never `acl/grant:…`). Local parts are
lowerCamelCase.

## 4. Every axis is explicit

Six members state authority. Five take exactly one of three explicit shapes,
and `purposes` one of two. `act`, `keys`, `capabilities` and `purposes` are
**REQUIRED**; `approve` and
`approveCapabilities` are optional, and their absence means **none**:

| Member | Unrestricted | Nothing | Narrowed (never empty) |
|---|---|---|---|
| `act` | `{"scope":"all"}` | `{"scope":"none"}` | `{"scope":"contexts","contexts":[…]}` |
| `approve` (absent = none) | `{"scope":"all"}` | `{"scope":"none"}` | `{"scope":"contexts","contexts":[…]}` |
| `keys` | `{"scope":"all"}` — every key `act` reaches | `{"scope":"none"}` | `{"scope":"listed","keys":[…]}` |
| `capabilities` | `{"scope":"ceiling"}` — the role's full ceiling | `{"scope":"none"}` | `{"scope":"listed","grants":[…]}` |
| `approveCapabilities` (absent = none) | `{"scope":"ceiling"}` — the role's approve ceiling | `{"scope":"none"}` | `{"scope":"listed","grants":[…]}` |
| `purposes` | `{"scope":"any"}` — every purpose but `modelTraining` | — (a subject that may read nothing is narrowed by `act` and `capabilities`) | `{"scope":"listed","purposes":[…]}` |

For `act` in particular:

| Value | Meaning |
|---|---|
| `{"scope": "all"}` | Unrestricted — every context the maintainer holds. On an administrative role, a super-administrator. |
| `{"scope": "none"}` | May act nowhere. A least-privilege approver says this. |
| `{"scope": "contexts", "contexts": ["…", …]}` | The named contexts and their descendants, and nowhere else. `contexts` is never empty. |

Rules a consumer applies:

1. **An empty list is invalid** on every axis — `contexts`, `keys`, `grants`, `purposes` —
   not a fourth value. It is not the unrestricted value and it is not `none`;
   the schema rejects it, and a consumer that receives one by a path that
   skipped validation MUST refuse the entry.
2. **An entry without `act`, `keys`, `capabilities` or `purposes` is malformed** and MUST
   be refused. There is no default.
3. **Role and act scope are independent.** A consumer MUST NOT compute either
   from the other, and MUST refuse an entry that omits either.
4. A maintainer **without contexts** — a community node, for instance, that
   narrows administrative authority by resource qualifier (§6) rather than by
   location — states `all` or `none` only, and SHOULD refuse a `contexts`
   value it cannot interpret rather than ignore it.

## 5. Absence derives no authority

0.1 had three members whose meaning depended on absent-vs-empty: `scopes`,
`allowedKeys` and `approve.scopes`. 0.2 has none. Every authority-bearing
member is either REQUIRED and explicit (`act`, `keys`, `capabilities`,
`purposes`) or, where
optional, means **none** when absent (`approve`, `approveCapabilities`).
Nothing a serializer can do — omitting an empty array, dropping a null,
applying a default — turns a narrow grant into a wide one: the result is a
document that fails validation, or one that confers less.

The unrestricted value on each axis is spelt out (`all`, `ceiling`, `any`), so a
reader never has to infer "unrestricted" from a missing list. Implementations in
typed languages SHOULD model each scope as a closed sum type and SHOULD NOT give
any of them a default value.

## 6. Capabilities

A **capability** names one power that can be gated separately from the role
that ordinarily implies it. Capability identifiers and the registry they come
from are the maintainer's; this family keeps them opaque. A capability MAY be
qualified by a **resource** (`git.repo.manage` @ `git-ns:github.com/acme`):
an unqualified capability covers every resource of its kind, and a qualified one
covers its resource and the resources inside it, by the maintainer's
containment rule.

The effective capability set of an entry is:

```
capabilities = ceiling  → effective = role.ceiling
capabilities = none     → effective = ∅
capabilities = listed   → effective = (role.ceiling ∩ grants[non-additive]) ∪ grants[additive]
```

Four rules, each refused **when the grant is written**, never applied by
silently dropping the offending capability:

1. **Outside the ceiling.** A non-additive capability the role's ceiling does
   not include is refused (`capabilityOutsideCeiling`). Refusing by name sends
   the author back to the role, which is the decision that needs revisiting.
2. **Unknown.** A capability the maintainer does not recognise is refused
   (`unknownCapability`), and a consumer reading a stored entry never treats an
   unrecognised capability as granted. A node built before a capability was
   registered has no basis for granting it.
3. **Additive.** An additive capability is one no role implies. It is marked
   `additive: true`, and granting it requires the granter to hold unrestricted
   act authority (`additiveRequiresUnrestricted`). `additive: true` on a
   capability the ceiling already includes is refused (`additiveWithinCeiling`):
   otherwise the flag would let an ordinary capability survive a later role
   change that should have removed it.
4. **Containment of a qualifier.** A grant at a qualifier is bounded by the
   granter's own holding: the granter must hold the capability at a qualifier at
   least as wide (§9).

A qualified capability confers nothing without a live entry for the same
subject. It is a member of the entry, not a right held beside it.

## 7. Act vs approve — two independent axes

| Axis | Location | Capability | Answers |
|---|---|---|---|
| **act** | `act` | `capabilities` | may this subject *do* X? |
| **approve** | `approve` | `approveCapabilities` | may this subject *ratify someone else doing* X? |

They are independent in both directions, and a consumer **MUST** resolve them
separately:

1. **Approve authority is not authority to act.** Answering "may this party do
   X" from `approve` grants an approver the power it was only meant to sign off
   on.
2. **Absent `approve` and absent `approveCapabilities` mean none.** A consumer that has not implemented the
   member confers less than the producer intended. Producers SHOULD state
   `{"scope": "none"}` rather than rely on the omission.
3. **`approveCapabilities` is bounded by `approve`.** With `approve` absent or
   `none`, `approveCapabilities` confers nothing, whatever it lists.
4. **The least-privilege approver is expressible**: `act: {scope: none}`,
   `capabilities: {scope: none}`, an approve scope, and the capabilities it
   may approve. It satisfies an approval inside its approve scope and cannot
   initiate any change.

## 8. Mapping a 0.1 entry onto 0.2

0.1's `scopes` was ambiguous on the wire: at several maintainers an empty or
absent list meant *unrestricted* for an administrative role and *authorized
nowhere* for every other role. The meaning lived in a maintainer convention the
document did not carry, which is why 0.2 replaces it rather than amending it.

A maintainer that serves both versions maps 0.1 → 0.2 as follows, and SHOULD
document the role set it applies rule 2 to:

| 0.1 | 0.2 |
|---|---|
| `scopes` non-empty | `act: {"scope": "contexts", "contexts": scopes}` |
| `scopes` absent or empty, and the maintainer's documented convention reads that as unrestricted **for this role** | `act: {"scope": "all"}` |
| `scopes` absent or empty, any other case | `act: {"scope": "none"}` — the least-authority reading |
| `approve.all: true` | `approve: {"scope": "all"}` |
| `approve.scopes` non-empty (and `all` absent or false) | `approve: {"scope": "contexts", "contexts": approve.scopes}` |
| `approve` absent, `approve.scopes` empty | `approve` absent (none) |
| `allowedKeys` absent (or `null`) | `keys: {"scope": "all"}` |
| `allowedKeys` empty | `keys: {"scope": "none"}` |
| `allowedKeys` non-empty | `keys: {"scope": "listed", "keys": allowedKeys}` |
| `label`, timestamps, `stepUp`, `ext` | unchanged |
| — | `capabilities: {"scope": "ceiling"}` (0.1 had no narrowing) |
| — | `approveCapabilities: {"scope": "ceiling"}` where the entry's 0.1 `approve` conferred anything; absent otherwise |
| — | `delegatedBy` from the maintainer's provenance where it has it; otherwise absent |

A maintainer that cannot determine which convention produced a 0.1 entry MUST
take the least-authority row. Going the other way — rendering a 0.2 entry as
0.1 for a 0.1 client — a maintainer MUST NOT render `act: {scope: none}` as an
empty `scopes` list to a client whose convention would read it as
unrestricted; it SHOULD refuse to render an entry 0.1 cannot express
(`capabilities` other than `ceiling`, `approveCapabilities` other than
`ceiling` or absent, `act: none` on an administrative role) rather than render a lossy approximation.

## 8a. Mapping a 0.2 entry onto 0.3

| 0.2 | 0.3 |
|---|---|
| every member | unchanged |
| — | `purposes: {"scope": "any"}` |

`any` is the authority a 0.2 entry already held, because 0.2 had no purpose
narrowing; writing it makes that authority a statement rather than an inference
(VTI-ACL-101). A maintainer writes it on every entry it holds **before** it
serves a purpose-bound operation under that entry, and never serves one under
an entry that does not state it.

Going the other way — rendering a 0.3 entry as 0.2 for a 0.2 client — a
maintainer MUST NOT drop a `listed` purpose scope silently: a 0.2 client would
read the entry as wider than it is, and might delegate from it. It SHOULD refuse
to render an entry whose `purposes` is not `any`, or render it with a note in
`ext` the client is told to read, rather than render the narrower entry as an
unnarrowed one.

## 9. Bounds on the granter, and delegation

A grant is a delegation of the granter's authority. A consumer evaluates every
create or modify against **the granter's stored entry** — never against a
credential summarising it — and refuses the write of a granter with no live
entry. The resulting entry MUST NOT exceed the granter on any axis:

| Axis | Bound | Code |
|---|---|---|
| role | not above the granter's | `permissionDenied` |
| act scope | within the granter's act scope | `delegationExceedsGranter` |
| capabilities | each held by the granter at a qualifier at least as wide | `delegationExceedsGranter` |
| additive capabilities | granter has `act: {scope: all}` | `additiveRequiresUnrestricted` |
| approve scope / approvable capabilities | within the granter's own approve scope and approvable set | `approveWiderThanGranter` |
| keys | where the granter's `keys` is not `all`, within the granter's keys | `delegationExceedsGranter` |
| expiry | where the granter's entry expires, no later | `delegationExceedsGranter` |
| purposes | where the granter's `purposes` is a list, within it; `modelTraining` only where the granter's own list names it | `purposeOutsideDelegator` |

**Every axis that can be narrowed can be widened again**, so the bound applies
to all of them: replacing a capability list with `ceiling`, a key list with `all`, a purpose list with `any`, lifting
an expiry or widening an approve scope is a grant, and is bounded like one.

The maintainer records the granter in `delegatedBy`. A delegated entry **MUST
NOT retain authority its delegator has since lost**: when the delegator's entry
narrows or is removed, the maintainer re-evaluates the entries it delegated —
withdrawing, narrowing, or queuing them for review under its own policy — and
MUST NOT leave them in force by default. It MUST be able to answer, for any
action, which entry authorized it and who created that entry.

**Re-delegation** — a delegate granting onward — is refused unless the
delegating entry explicitly permits it. This family does not define a
dedicated member for that permission: a maintainer expresses it as a capability
in its own registry (a VTC's `vtc.roles.assign`, for example), so that it is
granted, bounded and audited like every other power. A re-delegated entry MUST
NOT be wider on any axis than the entry it derives from.

## 10. Which task changes what

| Change | Task |
|---|---|
| Add a subject | [`acl/grant/0.3`](../../grant/0.3/spec.md) |
| Move a subject's role | [`acl/change-role/0.3`](../../change-role/0.3/spec.md) — requires the current role (compare-and-swap) |
| Amend act (widening), approve, capabilities, approvable capabilities, keys, purposes, label, expiry, step-up | [`acl/update/0.3`](../../update/0.3/spec.md) |
| Remove an entry, or narrow its act scope | [`acl/revoke/0.3`](../../revoke/0.3/spec.md) |
| Read one / enumerate | [`acl/show/0.3`](../../show/0.3/spec.md) / [`acl/list/0.4`](../../list/0.4/spec.md) |

Two boundaries carry over from 0.1 and are enforced, not advisory:

- **`acl/grant` refuses a role change** on an existing subject.
- **`acl/update` refuses a narrowing of `act`**, directing the caller to
  `acl/revoke/0.3`.

`capabilities`, `approveCapabilities`, `approve`, `keys` and `purposes` are narrowed
through `acl/update/0.3`, because `acl/revoke/0.3` narrows only the act scope. A narrowing on any of them **is a privilege reduction all the
same**, and a consumer MUST audit it distinctly as a reduction and apply it at
the subject's next authorization decision — never deferring it to the expiry of
a session, token or cached decision.

## 11. Granting authority-conferring powers

Granting the ability to grant — approve scope, `approveCapabilities`, or any
capability the maintainer classes as authority-conferring — is the escalation
vector: a subject that can confer it can manufacture an approver, or an
administrator, for an operation it could not itself authorize. Maintainers
**SHOULD** gate such grants more strictly than ordinary ones and **SHOULD**
audit them distinctly. Whether that gate is a step-up, a consent or an N-of-M
approval is the maintainer's policy; no specification in this family declares
it.

## 12. Purpose of use

`purposes` says what the subject may use content it reads **for**
(VTI-ACL-100 – 110). It bounds **purpose-bound operations**: those that
disclose to the caller content held for the principal — a read of agent memory
(`vta/memory/get`, `vta/memory/list` with values), opening a data room's record
for the principal — and any refusal that carries such content, such as a
version conflict that returns the value it conflicted with (VTI-ACL-102). It
does not bound writes that disclose nothing, nor any operation that reads only
metadata.

1. **The request names the purpose.** A request for a purpose-bound operation
   names the one purpose its caller will use the content for, in the task's own
   `purpose` member. Where the entry lists exactly one purpose, a request MAY
   omit it, and that purpose applies. Where the entry states `any`, a request
   that names none is served and its purpose is recorded as unstated
   (VTI-ACL-103).
2. **A purpose the entry does not grant is refused**, and so is a request that
   names none under an entry listing more than one. The refusal is the task's
   `purposeNotGranted`, distinct from its access refusal, and discloses no
   content (VTI-ACL-104).
3. **`any` never grants `modelTraining`** — training, fine-tuning or evaluating
   a machine-learning model on the content. That purpose is granted only by an
   entry whose list names it (VTI-ACL-105), and delegated only by a granter
   whose own list names it (§9).
4. **Every purpose-bound operation is audited with its purpose**, served or
   refused, or with a statement that none was named (VTI-ACL-106). A purpose is
   not personal data.
5. **Purposes are compared by exact string equality.** A purpose is a
   lowerCamelCase name registered in the VTI specification's purpose registry
   (Appendix H: `personalisation`, `taskExecution`, `analytics`,
   `modelTraining`), or an absolute IRI naming one defined elsewhere, such as a
   W3C Data Privacy Vocabulary purpose. A consumer MUST NOT treat a purpose it
   does not recognise as equal to any other — not to a registered name, not to
   another IRI it believes broader (VTI-ACL-109).
6. **Policy may refuse a purpose when the entry is written** (`purposeRefused`,
   naming it), never by writing the entry without it (VTI-ACL-108).
7. **A purpose is a declaration, not a proof.** No maintainer can observe what
   a subject does with content. The value of the member is that it binds the
   subject to what it declared, in an audit record it cannot alter, at the
   moment it declared it.
