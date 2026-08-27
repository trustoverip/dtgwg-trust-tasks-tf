# Changelog — `@openvtc/trust-tasks`

All notable changes to the TypeScript bindings package.

This file starts at 0.15.0. Earlier releases are in the git history of
`trust-tasks-ts/`; `trust-tasks-rs/CHANGELOG.md` records the changes the two
libraries shipped together, which is most of them.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

## 0.16.2 — 2026-08-27


### Fixed

- **ts**: Regenerate the provision/integration/0.3 binding (#326)

#324 corrected the schema — `required` named a `digest` the 0.2→0.3 rename had
  already removed from `properties` — and regenerated the Rust bindings. It did
  not regenerate the TypeScript ones.

  So `trust-tasks-ts` still shipped `"digest"` in `required` for both the request
  and response schemas while `trust-tasks-rs` no longer did. `check-bindings`
  caught exactly that: "the request schema shipped by trust-tasks-ts and the one
  shipped by trust-tasks-rs are not the same document … the two libraries would
  disagree about which payloads conform."

  `npm run build-ts-bindings`, two lines removed, nothing else moved. The TS half
  now carries the same unsatisfiable-schema fix the Rust half got.

  Worth noting for the next schema change: a spec edit needs *both* generators
  run. The Rust one is `cargo run -p trust-tasks-codegen`, the TypeScript one is
  `npm run build-ts-bindings`, and only running one leaves the two libraries
  disagreeing about the wire contract — which is what `check-bindings` exists to
  catch and did.

## 0.16.1 — 2026-08-27


### Other

- Say what identifierScope is, and that it is not a scope taxonomy (#317)

Carries the framework text of trustoverip/dtgwg-trust-tasks-spec#6 into
this repository's copy of SPEC.md, and aligns the meta-schema
description with it.

Addresses trustoverip/dtgwg-trust-tasks-spec#5: the framework's
identifierScope and the DTG credentials specification's proposed
declared correlation scope (trustoverip/dtgwg-cred-spec#22) read as two
vocabularies for one axis, with `community` and `linked` unmappable
here. They are not competing, and neither document said so.

identifierScope is a machine-readable restatement of §10.5 item 1,
which is binary: relationship-scoped, or reused across relationships
and justified. It is not a taxonomy of correlation width. It is
declared by a specification about a party role, not by a holder about
an identifier it controls, so the two are assertions about different
subjects and neither overrides the other. An identifier recognisable
within a bounded set is, for item 1's purposes, cross-relationship, so
it is declared `public` and carries the justification obligation in
full.

The meta-schema change is to the `description` string only. The enum
stays at three values, so all 56 existing declarations remain valid,
and spec.meta.schema.json is excluded from both binding generators by
name — scripts/build-ts-bindings.mjs via NOT_PAYLOAD_SCHEMAS, and
trust-tasks-codegen, which walks payload.schema.json only. No generated
binding, crate, or package is affected, and no version bump is needed.

npm run validate: 349 specs, no errors. Registry build clean.

Signed-off-by: Glenn Gore <glenn.gore@gmail.com>

## 0.16.0 — 2026-08-27


### Added

- **specs**: Re-target every live specification to framework 0.5 (#306)

* feat(specs): target framework 0.5 in the framework-reserved specs

  The five framework-reserved families -- trust-task-error,
  trust-task-next-step, trust-task-discovery, trust-task-control and
  trust-ceremony-receipt -- are always compiled and are the ones every
  other spec composes with, so they move first.

  SPEC 7.2 item 1 selects the envelope schema by the specification's
  declared target framework version, so this is the member that decides
  which envelope a consumer validates these documents against. Framework
  0.5.0 has been published under specs/_framework/0.5/ since it merged
  canonically, but nothing targeted it, so none of its requirements bound
  anything.

  The 0.5 envelope is byte-identical to 0.4's apart from $id, title and
  descriptions, and is a strict superset of 0.1-0.3 (same required set,
  additionalProperties true, adding only constraints on ceremony and
  parentThreadId). No document that conformed before stops conforming.

- **specs**: Require issuedAt on every live consequential spec, and make the §7.3 item 17 floor fatal (#302)

* spec: make the §7.3 item 17 freshness floor a hard failure

- **rs**: Make SPEC §7.3 item 17's issuedAt MUST expressible (#300)

Framework 0.5.0's §7.3 item 17 says a Trust Task specification defining a
  consequential Trust Task MUST require the `issuedAt` member, raising §4.2's
  SHOULD to a MUST for documents conforming to it. There was no way to declare
  that: the meta-schema had `proofRequirement` for the `proof` envelope member
  and nothing equivalent for `issuedAt`, so the MUST was unexpressible and none
  of the 209 consequential specs in the registry could comply.

  Adds `issuedAtRequirement` to the front-matter meta-schema, modelled on
  `proofRequirement` — same two forms (a single `requirement`, or a per-variant
  `request`/`response` pair), same enum, same `rationale`. It governs an envelope
  member, admits a per-variant reading because request and response are relied on
  differently, and reading one declaration now teaches the other.

  Declared rather than derived, with the derived floor checked and reported:

    * `sideEffects` and `exposure` are DESCRIPTIVE (§7.3 items 13/14). Deriving
      `issuedAt`-REQUIRED from them would mean that correcting `sideEffects.level`
      from `none` to `mutating` for accuracy silently changed which documents
      every consumer of that spec must reject.
    * §2 makes the *handler* authoritative for consequentiality, not the front
      matter — so a value computed from the front matter would enforce something
      the front matter does not authoritatively state.
    * A non-consequential spec may legitimately require `issuedAt` anyway; a
      derived-only mechanism cannot express that.
    * Deriving would flip 209 specs' wire behaviour in one commit, invisibly.

  So the build derives the floor instead of substituting for the declaration.
  `npm run validate` prints a `Freshness floor (§7.3 item 17)` line counting the
  consequential specs that have not declared it (currently 0/209), *fails* on a
  spec that declares something weaker than REQUIRED, and escalates the undeclared
  case to a hard error under `TT_STRICT_ISSUED_AT=1` — the ratchet that becomes
  the default once the registry has caught up.

  Enforcement reaches consumers the same way `IS_PROOF_REQUIRED` does:

    * `Payload::IS_ISSUED_AT_REQUIRED` (Rust) and `SpecPolicy.isIssuedAtRequired`
      (TypeScript), emitted per variant by both generators from the front matter.
    * `TrustTask::enforce_spec_policy` / `enforceSpecPolicy` reject a document
      with no `issuedAt` as `malformedRequest`. Not `expired`: §8.3 defines no
      dedicated code, `expired` names a document that was once acceptable, and
      §7.2 item 13 already uses `malformedRequest` for the other freshness
      rejections.
    * The trait constant has a default of `false`, so hand-written impls — the
      crate's own `trust-task-error`, hand-modelled in `error.rs` because it is
      in the codegen's SKIP_SLUGS, and every downstream impl — keep compiling.
    * `check-bindings` compares the new key across front matter, Rust and
      TypeScript, so the two languages cannot drift on it.

  Distinct from `FreshnessPolicy::require_issued_at`, which is a *consumer's*
  own posture applied to every document it sees. This is the *specification's*
  requirement, published in the registry and not the consumer's to relax.

  No specification declares `issuedAtRequirement` yet — that is a follow-up,
  spec by spec. The generated Rust tree is therefore byte-identical (the const is
  emitted only on override) and the generated TypeScript gains one
  `isIssuedAtRequired: false` line per policy object. Nothing that was accepted
  before is rejected now.

  Also documents the declaration in CONTRIBUTING-SPECS.md, and teaches
  `npm run new-spec` to emit it on a scaffold whose declared classes make the
  task consequential.



### Specifications

- Bound the remaining free-text payload members and state who reads them (#301)

SPEC.md §7.3 (framework 0.5.0) item 19 has three parts. PR #296 closed most
  of part 1 for eight member names (`reason`, `note`, `detail`, `description`,
  `message`, `text`, `label`, `comment`), 92 members in all. Sweeping the whole
  corpus rather than a name list finds **48 more** unbounded free-text members
  across 27 draft specifications and 10 shared schemas, under names the earlier
  sweep did not look for: `purpose`, `deviceLabel`, `displayName`, `explanation`,
  `summary`, `title`, `name`, `blurb`, `filename`, `trace`, `error`, `lastError`,
  `deniedReason`, `displayHint`, `consequences`, `rpName`, `userName`,
  `userDisplayName`.

  The one the earlier review named specifically is here: `task-consent`'s
  `Effect.summary` — the member its own schema calls "the ONLY member a consent
  surface is guaranteed to be able to render" — was unbounded on the surface a
  human reads to approve an action. It is now 500, the same figure as the `note`
  beside it.

  Bounds are drawn from the vocabulary the corpus already uses and sized per
  member, not applied uniformly:

      64   WebAuthn entity names (`rp.name`, `user.name`, `user.displayName`,
           `rpName`, `userName`, `userDisplayName`) — the length WebAuthn tells
           authenticators to truncate to, so a longer value would be silently
           cut on the device rather than carried.
     128   `purpose` where it names a decision slot or a declared intent. This
           is consistency repair, not a new judgement: policy/activate/0.1 and
           policy/active/0.1 already declared `maxLength: 128` on the request
           member while the response that echoes it declared none.
     256   display labels and names — `deviceLabel`, `displayName`,
           `displayHint`, `label`, `title`, `name`, `blurb`, `filename`;
           matches #296's reasoning that a display name is not prose.
     500   consent-surface prose a human reads while deciding — `Effect.summary`,
           `consequences[]`, credential-exchange `purpose`, `deniedReason`;
           matches task-consent/request/0.1 `note`.
    1024   operator or service diagnostic prose — `explanation`, `trace[]`,
           `error`, `lastError`, and vta/credentials/issue `purpose`.

  **Part 3 of the MUST is the half nobody had done.** Item 19 also requires each
  specification to state who reads a free-text value, whether the recipient is
  expected to retain it, and whether it is trusted. Prose is added to the
  `## Security & Privacy` section of all **37** draft specifications that reach a
  newly bounded member — the 27 whose own payload schema moved, plus 10 that
  reach one through a shared `$def`. `vta/credentials/issue/0.{1,2}` had no such
  section at all and now carry one containing the item 19 statement; both remain
  on the Security & Privacy debt allowlist, because neither yet has the four
  sub-headings the repo lint wants. The other 35 sections were extended in place.

  Deliberately untouched, matching #296's own carve-outs:

    * `messaging/_shared/0.1` `AuditEntry.detail` and `did-management/_shared/0.1`
      `DomainEntry.label` — shared `$defs` reachable from a retired specification,
      so bounding them would move a frozen specification's wire contract. These
      are the only two shared schemas in this change's blast radius that a
      retired spec reaches; the ten this PR edits are draft-only.
    * Members item 19's own last paragraph excludes: identifiers, opaque handles,
      digests and references; and values carried verbatim from an external
      vocabulary — DNS domains, did:webvh agent-name local parts, DID logs, Rego
      source, RFC 6901 pointers, HTTP header and cookie names, APNs topics, IANA
      media types, BCP 47 language tags, and maintainer- or community-defined
      role and kind vocabularies.

  Two members were bounded where the schema's own description calls the value an
  undeclared closed set (`nature`, `wired` in vtc/ceremonies/list) — deliberately
  left alone, because the right fix there is the closed enumeration item 19
  prefers, not a length.

  All 48 members sit in `status: draft` specifications and are amended in place
  per SPEC §5.2. `npm run validate` re-checks all 533 fenced example documents
  against the amended schemas; none is rejected by a new bound.

  **Still open, and deliberately not in this PR:** item 19's "SHOULD be OPTIONAL"
  half. 18 of the 48 members bounded here are REQUIRED — `CredentialCreationOptions`
  `rp.name`/`user.name`/`user.displayName`, credential-exchange `purpose` (both
  copies), `Effect.summary`, `ContextRecord.name` and vta/contexts/create `name`,
  the three `enroll-challenge` WebAuthn names, `CommunityProfile.name`,
  `CommunityProfileView.name`, `CommunityProfileSnapshot.name`, and the four
  decision-slot `purpose` members. Added to the 13 #296 found, that is 31.
  Making any of them optional is a wire-compatibility change and belongs in its
  own PR; several (purpose binding, `Effect.summary`) are required by design and
  the specification should say so rather than change.

  This is **breaking**, on the same terms #296 judged itself so: typify emits a
  bounded member as a validating newtype, so `Option<String>` becomes
  `Option<PayloadDeviceLabel>`, `Option<PayloadDeniedReason>`,
  `Option<AttachmentRefFilename>` and so on, and the newtype rejects in
  `Deserialize` a document the library previously accepted. `@openvtc/trust-tasks`
  generates unchanged *types* — TypeScript has no length refinement — but its
  embedded schemas now reject over-long values at runtime. No version is bumped
  and no changelog is written: release-plz computes both from this title.

- **trust-task-ok**: Retire the registry entry, deprecated by framework 0.5.0 (#292)

Framework 0.5.0 lets a fire-and-forget specification's consumer return that
  task's own <type>#response with payload {} as a courtesy acknowledgement.
  That removes the only reason this slug existed: the previous text forbade a
  response-less specification from emitting a #response, so 'received and
  performed' needed a document of its own.

  - specs/trust-task-ok/0.1/spec.md: status draft -> retired (SPEC 5.3
    permitted transition 3), a Deprecation and retirement section carrying the
    three normative consequences from framework 0.5.0, and a rewritten Status
    of this Document. No supersededBy: the replacement is a framework
    mechanism, not another Trust Task specification, so there is no successor
    slug and none is invented. SPEC 7.3 item 11 RECOMMENDS one rather than
    requiring it, and the build warns rather than fails.
  - SPEC.md 6.1 and 8.6: adopt the framework 0.5.0 wording for the
    deprecation, including that the slug and its Type URI remain RESERVED
    permanently, plus a clearly marked registry note recording the retirement.

  Retirement is a status, not a deletion. The entry stays published, stays in
  the codegen (it is not in SKIP_SLUGS), and stays on the website; the reserved
  slug allowlists in specs/spec.meta.schema.json and trust-tasks-rs/src/type_uri.rs
  are untouched, so the name is not freed for anyone else. Neither generator
  reads front-matter status, so regenerating both bindings produced no diff and
  no library version bump is needed.

- Bound every free-text payload member with a maxLength (§7.3) (#296)

* spec: bound every free-text payload member with a maxLength

  SPEC.md §7.3 (framework 0.5.0) requires that any member holding free
  text declare a `maxLength`. 92 free-text string members across 83 draft
  schemas carried none, leaving the wire contract unbounded and every
  consumer to invent its own ceiling — or none, which is what §10.3
  (schema-validation DoS) exists to prevent.

  Bounds are chosen per member from the vocabulary the registry already
  uses rather than applied uniformly:

    256   `label`, `comment` — a display name or an OpenSSH key comment;
          matches the existing 256 on provision/integration `label` and
          the `name` members alongside it.
    500   requester-authored prose that a surface renders to a human who
          is deciding something; matches task-consent/request/0.1 `note`,
          the registry's considered consent-surface bound.
    1024  `reason`, `description`, `message` — operator or service prose
          recorded for audit or returned as a diagnostic; matches the six
          existing `reason: 1024` and the `description: 1024` in policy/
          and vtc/endorsement-type.
    16384 chat/message `text` — the task's actual content rather than
          metadata about it; matches the corpus's long-form bound on
          vault `secureNotes`.

  All amended specifications are `status: draft`, so the change is made in
  place per SPEC §5.2. Deliberately untouched:

    * 17 members in `retired` specifications, frozen by SPEC §6.4.
    * messaging/_shared/0.1 `AuditEntry.detail` and did-management/
      _shared/0.1 `DomainEntry.label` — shared $defs reachable from a
      retired specification, so bounding them would change a frozen
      specification's effective wire contract.
    * vault/_shared/{0.1,0.2,0.3} `TspMessageEnvelope.message` — opaque
      base64url TSP bytes, not free text.

  The `label` description in vault/_shared/*/vault-entry.schema.json said
  the wire spec enforced no maximum length. It now does, so the sentence
  is corrected rather than left contradicting the schema it annotates.

  `npm run validate` re-checks all 533 fenced example documents against
  the amended schemas; none is rejected by a new bound.

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
