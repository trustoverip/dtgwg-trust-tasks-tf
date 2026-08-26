# Changelog — `@openvtc/trust-tasks`

All notable changes to the TypeScript bindings package.

This file starts at 0.15.0. Earlier releases are in the git history of
`trust-tasks-ts/`; `trust-tasks-rs/CHANGELOG.md` records the changes the two
libraries shipped together, which is most of them.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

## [0.15.0] - 2026-08-26

### Changed

- **BREAKING. Cross-file schema definitions are declared once, in
  `_shared/components.ts`, instead of being copied into every module that
  references them.** The counter-suffixed duplicates the copying produced —
  `Ext1`, `Ext2`, `Ext3`, `Vid1`, `Vid2`, `SecretKind1`, `DigestMultibase1` and
  the rest — are gone. 200 exported names disappear; **every one of them ends
  in a digit**, and every unsuffixed name a module exported before it still
  exports now.

  Before, `Ext` — the framework's `ext` object, SPEC §4.5.1 — was declared 481
  times across 341 modules, and which of `Ext`, `Ext1` or `Ext2` you got
  depended on declaration order inside a generated file. There was no way to
  write a signature over "the framework extension object". Now there is one
  `Ext`.

  **Migration.** Replace the suffix with the bare name:

  | Before | After |
  |---|---|
  | `Ext1`, `Ext2`, `Ext3` | `Ext` |
  | `Vid1`, `Vid2` | `Vid` |
  | `DigestMultibase1` | `DigestMultibase` |
  | `SecretKind1` | `SecretKind` |
  | `AclEntry1`, `AccountType1`, `MediatorAcl1`, `QueueLimits1`, `KeyCustody1`, `KeyType1`, `KeyStatus1`, `SignAlgorithm1`, `WakeTriggerPolicy1`, `Namespace1`, `Version1`, `Version2`, `CredentialId1`, `ConsentSubject1`, `PersonhoodGovernance1` | the same name without the digit |

  The rule is mechanical: `NameN` → `Name`, imported from the same module as
  before. Nothing else moves. Because the suffixed and unsuffixed forms were
  structurally identical, the replacement is type-safe — TypeScript is
  structurally typed, so the two were already mutually assignable and no value
  changes type.

  The same hoist was **declined for the Rust bindings** (#283) and the
  asymmetry is deliberate: `upsert::v0_3::VaultEntry` and
  `delete::v0_1::VaultEntry` are distinct *nominal* types in Rust, so merging
  them is an E0119 coherence break for any consumer holding a trait impl on
  each. TypeScript has no coherence rule, so the same merge changes names and
  nothing else.

- **A definition name that denotes more than one shape is now qualified.**
  Grouping is by structure, not by name: `VaultEntry` exists in three
  structurally different versions, `Scope` in two unrelated ones (consent's and
  vta's), and 22 names in all cover more than one shape. In
  `_shared/components.ts` these are `VaultEntry_VaultV0_1`,
  `VaultEntry_VaultV0_2`, `VaultEntry_VaultV0_3`, `Scope_ConsentV0_1`,
  `Scope_VtaV0_1` and so on — every shape qualified, including the oldest,
  because there is no canonical one. **Spec modules are unaffected**: each
  re-exports what it uses under the name it used before, so
  `vault/get/0.3/payload.js` still exports `VaultEntry`.

### Added

- **`_shared/components.ts`**, exported from the barrel as `SharedComponents`.
  Import a definition once and use it across specs:
  `import { SharedComponents } from "@openvtc/trust-tasks"` then
  `SharedComponents.Ext`, or reach it directly at
  `@openvtc/trust-tasks/_shared/components.js`.

### Fixed

- **The `_shared/` and `_framework/` modules publish their definitions.** They
  are generated from schemas whose root declares no `type` and no `properties`,
  so the compiler emitted a lone `[k: string]: unknown` interface and dropped
  every `$def` as unreachable: `VaultEntryShared_v0_1.VaultEntry` in the barrel
  named nothing at all, and the generator's own header comment claimed
  otherwise. Each now re-exports the definitions it owns.

### Notes for the next person adding a framework standard error code

`StandardCode` stays a **closed** union — `(typeof STANDARD_CODES)[number]` —
and this is a deliberate choice, re-taken here rather than inherited.

It means the two SDKs treat a new SPEC §8.3 code differently. In
`trust-tasks-rs` `StandardCode` is `#[non_exhaustive]` (since 0.7.0), so adding
one is **additive**: downstream `match` expressions already carry a wildcard
arm. Here the union is exhaustive by construction, so adding one is
**breaking**: a `switch` that covers every member stops being exhaustive and
`never`-typed default arms start erroring. `@openvtc/trust-tasks` went to 0.7.0
for precisely that, alongside `trust-tasks-rs` 0.7.0 which took the
`#[non_exhaustive]` break once and was done.

So budget a **minor bump on this package** for the next standard code, and
expect only a patch on the Rust side.

The alternative — widening to `StandardCode | (string & {})` — was considered
and rejected. It would make every `StandardCode`-typed position accept any
string, so a misspelled `"proofRequred"` would compile everywhere the union is
used, including `RejectReason.code`, which decides what error document the
runtime emits. That cost is paid on every line of every consumer, forever, to
soften a break that arrives once per framework minor and arrives as a compile
error naming the exact sites to fix.

Consumers who want to be immune should narrow rather than switch exhaustively:
`isStandardCode(code)` (exported from the root) is a type guard from `string`,
it normalizes the frozen framework 0.1 snake_case spellings on the way, and a
`switch` over its narrowed result with a `default` arm survives any addition.
That pattern is documented on `StandardCode` itself.
