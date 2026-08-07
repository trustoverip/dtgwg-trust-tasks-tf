# Changelog

All notable changes to `trust-tasks-rs` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to a `MAJOR.MINOR` versioning scheme that tracks
the corresponding `SPEC.md` framework version.

## [0.2.57] — 2026-08-07

### Added
- **`Payload::family_code(namespace, local)`** — mints an extended error code
  under a *family namespace*, the proper-path-prefix form that SPEC §8.5 now
  permits alongside a specification's own slug.

  26 specifications in the registry declare `did-management:unknown_domain`,
  a condition defined once for the whole family in
  `did-management/_shared/0.1/CONVENTIONS.md`. There was no drift-safe way to
  emit it: `extended_code` derives the namespace from `TYPE_URI`, so a
  `did-management/did/delete` handler produced
  `did-management/did/delete:unknown_domain` — not the code the registry
  advertises. A consumer matching the declared code missed it and fell through
  §8.5's unrecognized-code rule to `taskFailed`, losing the specific meaning
  silently. The alternative was `TrustTaskCode::new_extended` with a
  hand-written namespace, which reintroduces exactly the drift `extended_code`
  exists to prevent.

  `family_code` checks `namespace` against the slug derived from `TYPE_URI`
  rather than taking it on trust, so the §8.5 prefix rule holds by
  construction. A sibling's slug is rejected: it shares a prefix but is not
  itself one, and that is precisely the confusion §8.5 forbids.

### Fixed
- **The registry build now enforces the §8.5 namespacing rule.** Neither the
  original rule (namespace **MUST** equal the slug) nor the relaxed one was
  ever checked — `spec.meta.schema.json` states it in prose and validates only
  the grammar, because JSON Schema cannot compare `errorCodes[].code` against
  `slug`. `checkErrorCodeNamespaces()` compares them.

## [0.2.56] — 2026-08-05

### Security
- **Twelve specifications that act with the subject's authority, release secret
  material, or destroy state now declare `proof` REQUIRED.** `keys/sign`,
  `keys/derive-and-sign`, `keys/derive-and-sign-document`, `audit/list`,
  `credential-exchange/{issue,present,request}`, `vtc/auth/admin-session`,
  `vtc/members/{personhood/assert,renew,rotate-challenge}` and
  `webvh/sync/delete` previously declared `OPTIONAL` or `RECOMMENDED`, so
  `IS_PROOF_REQUIRED` generated as `false` and `consume_inbound` accepted a
  proofless document against a signing oracle, a session mint, or an
  irreversible delete.

  SPEC §7.3 item 8 already said a declaration **MUST NOT** be weaker than the
  §4.7.1 default, but that could never be checked: §4.7.1's default is a
  function of the *transport*, which a specification does not know when it is
  authored. `scripts/build-registry.mjs` now derives the floor from the
  declarations a spec does make — `sideEffects.level: destructive`,
  `exposure.discloses: secret`, or `exposure.actsAsSubject: true` require
  `proofRequirement.requirement: REQUIRED` — and fails the build otherwise.
  This does not conflict with the side-effect and exposure classes being
  "descriptive, not prescriptive": that rule forbids deriving a *consent*
  requirement from them, and an integrity floor constrains how a document is
  authenticated, not whether a human must approve it.

  **This is a behavioural change for consumers of those twelve specs.** A
  document previously accepted without a `proof` is now rejected with
  `proofRequired`. All twelve are `draft`, where SPEC §5.2 permits schema and
  prose to change in place without notice, so no new version folders were
  minted. Producers must attach a proof, and because §7.2 item 8 then applies
  they must also carry an in-band `recipient` — none of the twelve is a bearer
  specification.

### Fixed
- Corrected the `Payload::extended_code` grammar doc, which described the local
  part of an extended error code as lowercase-only. `validate_local` accepts
  interior uppercase, as it must: SPEC §4.10 item 4 prefers lowerCamelCase
  locals and only the first character is required to be lowercase.
- Merged the two changelog entries that both claimed `0.2.54`, and removed the
  `0.2.55` entry that had been duplicated verbatim above them — the residue of
  the #170/#171 version collision described below.

## [0.2.55] — 2026-08-01

### Fixed
- **Republish of 0.2.54's content, which never shipped.** #170 and #171 both
  claimed version `0.2.54`; #170 merged first and published it, so when #171
  merged the publish workflow found `0.2.54` already on crates.io and skipped —
  correctly, and silently, reporting success. The result was a registry whose
  `main` carried `vta/webvh/servers/domains/0.1` while the published crate did
  not, so a consumer binding the task got an unresolved module path rather than
  any signal that the spec was missing.

  No spec content changes here. This is the version bump that lets the workflow
  ship what is already on `main`.

  **Claiming a library version at authoring time is what makes this possible.**
  Two PRs in flight take the same number, and the second one's content
  disappears at merge with a green check. The version is only safe to set when a
  PR is next to merge.

## [0.2.54] — 2026-07-31

### Changed
- **`credentialId` is now the shared `CredentialId` newtype across the credential
  families.** `credentials/_shared/0.1/credentials` hoists the identifier both
  receipts are keyed by into its own `$def`, and `vta/credentials/{issue,revoke}`
  reference it instead of restating the field. **Source-breaking for Rust
  consumers** (the wire is unchanged): in `vtc/endorsements/*` the generated type
  is renamed `IssuedCredentialCredentialId` → `CredentialId`, and in
  `vta/credentials/*` `credential_id` moves from a plain `String` to the
  validated newtype. Construct via `.parse()` / `try_from`, or by deserializing
  the payload whole. Only the identifier was hoisted — `expiresAt`, `revokedAt`
  and the opaque `credential` object keep their natural generated types on
  purpose, since wrapping them would deduplicate a description at the cost of
  every consumer's ergonomics.
- **`vta/credentials/*` `credentialId` is now constrained non-empty**, inherited
  from the shared definition. Every real payload already satisfied it.

### Added
- **`vault/_shared/0.2` gains `StepUpChallenge` and `EnvelopeMismatch`** — the
  `details` shapes behind `stepUpRequired` and `envelopeUnsupported`, previously
  restated verbatim across `vault/release`, `vault/proxy-login` and
  `vault/sign-trust-task`. An `errorCodes[].detailsSchema` may now be a `$ref`
  into a shared schema; the registry build resolves and inlines it, so
  `registry.json` consumers see the full fragment as before. No generated-code
  effect — the code generators do not read `errorCodes`.
- **`vta/webvh/servers/domains/0.1`** — an agent relays a hosting server's
  caller-scoped domain view. The response items `$ref` the existing
  `did-management/_shared/0.1/domain-entry#DomainEntry` rather than restating
  it: this is one object crossing two hops (operator → agent → hosting server),
  and an operator comparing the agent's answer against the server's must not
  have to reconcile two spellings of the same domain.

  The request differs from [`did-management/me/domains`](https://trusttasks.org/spec/did-management/me/domains/0.1)
  by exactly one member, `serverId`, and that is the whole reason it is a
  separate task: "me" is unambiguous when addressed to a server, and meaningless
  when addressed to an agent that knows several. Hosting servers do not share a
  domain namespace, so a merged answer would carry entries no consumer could
  attribute.

  Conformance requires the agent to relay **unfiltered** and to preserve
  `createdAt`. Both are under-reporting hazards: an agent that narrows the
  server's list reports fewer domains than the producer may actually use, and
  one that drops members turns "the agent did not tell me" into "the server does
  not know".

## [0.2.53] — 2026-07-31

### Added
- **`keys/create/0.1` gains an optional `mnemonic`.** Additive, so the version
  folder is edited in place per SPEC §5.2. A producer may ask the custodian to
  derive from a supplied BIP-39 phrase rather than from the custodian's own seed
  — an import of externally-generated seed material wearing create's clothes.

  The member is **secret-bearing in a way the rest of the payload is not**: the
  phrase reconstitutes the key anywhere. Consumers MUST refuse it on any
  transport that is not end-to-end confidential — the same rule `keys/import`
  applies to its cleartext carrier — and MUST NOT log or echo it.

  Found by conformance rather than by review: an implementation was already
  sending `mnemonic` on create, and the first version of this spec had no member
  for it, so binding the task would have silently dropped the capability.

## [0.2.52] — 2026-07-31

### Added
- **`keys/*` — a new nine-task family for key custody and the signing oracle.**
  `keys/create`, `keys/import`, `keys/show`, `keys/list`, `keys/rename`,
  `keys/revoke`, `keys/sign`, `keys/derive-and-sign` and
  `keys/derive-and-sign-document`, all at `0.1`. Authored top-level rather than
  under a vendor namespace: holding keys on a producer's behalf and signing
  without exporting them is generic to any agent, not specific to one
  implementation.
- **`keys/_shared/0.1/key-record`** — the shared `KeyRecord`, plus the `KeyType`,
  `KeyStatus` and `KeyOrigin` enumerations every task in the family references.
  `origin` distinguishes a **derived** key (reproducible from a seed the
  custodian holds) from an **imported** one (stored material only, and gone if
  that storage is lost) — a distinction operators reasoning about disaster
  recovery get wrong without it.
- **`keys/_shared/0.1/sign-algorithm`** — the shared, closed `SignAlgorithm`
  enumeration (`EdDSA`, `ES256`), referenced by both signing tasks so they cannot
  drift apart on what a custodian will accept.
- **`key-management` category** — added to `specs/spec.meta.schema.json` and
  `website/assets/data.js` together, per the taxonomy rule. Distinct from
  `credentials`, which covers stored secrets rather than the keys that sign.

Three design points worth noting for implementers:

- **`keys/list` requires `total`.** A rotation sweep that reads one short page
  and stops has silently skipped every key past the boundary, and without `total`
  nothing in the response distinguishes "all of them" from "the first twenty".
- **`keys/revoke` is `destructive` and retains the record.** Revocation is not
  deletion: signatures made before it remain verifiable artefacts, and a deleted
  record would leave them unattributable. Reactivation is forbidden.
- **`keys/import` names the carrier as a confidentiality decision.** The cleartext
  `privateKeyMultibase` member is admissible only where the transport is
  end-to-end confidential; a custodian behind a TLS-terminating intermediary MUST
  refuse it.

## [0.2.51] — 2026-07-29

### Added
- **`AclEntry.allowedKeys`** (`acl/_shared/0.1/acl-entry`) — optional actor-scoped
  key-id filter for maintainers that operate a signing oracle. Intersects with —
  never widens — `scopes`. Absent (`None`) = every key in scope (prior
  behaviour); **present-but-empty = authorized on no keys**. Generated as
  `Option<Vec<String>>` so the absent-vs-empty distinction survives the binding.
- **`allowedKeys` on the `acl/update/0.1` payload** — replacement-not-merge;
  explicit `null` clears the filter (a privilege increase), an empty array sets
  it to no keys, and a narrowing replacement is a privilege reduction the
  consumer must audit and apply to live sessions
  (OpenVTC/verifiable-trust-infrastructure#818).

## [0.2.50] — 2026-07-29

### Added
- **`did-management/agent-name/update/0.1`** — declarative binding state
  (`active` | `parked`) replacing the set / enable / disable verb trio.
  `agent-name/remove` deliberately stays a separate destructive task.
- **`did-management/did/set-state/0.1`** — `active` | `suspended`, replacing
  the `did/enable` + `did/disable` pair.
- **`did-management/domain/set-state/0.1`** — `active` | `disabled`, replacing
  the `domain/enable` + `domain/disable` pair.
- **`did-management/agent-name/check/0.1`** and
  **`did-management/agent-name/list/0.1`** — specs for the previously
  implemented-but-unspecced availability probe and owner-scoped name listing.

### Changed
- **Retired** `agent-name/{set,enable,disable}`, `did/{enable,disable}`,
  `domain/{enable,disable}` (superseded by the state-enum tasks above) and
  `did/publish` (superseded by `did/register`, whose owner-update rule covers
  the reserved-slot flow). Modules remain generated for auditability of
  previously-issued documents.

(affinidi/affinidi-webvh-service#143 consolidation.)

## [0.2.49] — 2026-07-29

### Added
- **`vta/did-templates/{create,delete,get,list,render,update}/2.0`** — the
  global and context-scoped DID-template families merged into one six-task
  family behind an optional `contextId` (absent = global scope, super-admin
  gated; present = that context, context-admin gated). `render/2.0` documents
  the ambient `CONTEXT_ID`/`CONTEXT_DID` variables injected for scoped
  renders. Proof levels re-derived per task: the pure reads (`get`, `list`,
  `render`) are now RECOMMENDED; mutations stay REQUIRED.
- **`vta/_shared/0.1/did-template`** — shared `DidTemplate` /
  `DidTemplateRecord` / `Scope` definitions, previously duplicated inline in
  all twelve 1.0 payload schemas.

### Retired
- The twelve 1.0 specs (`vta/did-templates/*/1.0` and
  `vta/contexts/did-templates/*/1.0`), each superseded by the corresponding
  `vta/did-templates/*/2.0` task
  (OpenVTC/verifiable-trust-infrastructure#851).

## [0.2.48] — 2026-07-29

### Added
- **`vtc/join-requests/decide/0.1`** — an administrator decides a pending join
  request with one payload: `{ id, decision: approved|rejected, reason? }`.
  Supersedes the `approve`/`reject` pair (near-identical payloads, same admin
  gate, same pending→notPending lifecycle check, same REQUIRED-proof posture),
  following the enum-variant pattern of `provision/integration` and
  `auth/passkey/login/start`.
- **`vtc/members/vmc/0.1` gains optional `requestId`** (additive, in-place —
  the spec is `draft`). When present and naming an approved join request whose
  applicant is the delivering member, the delivery also closes that request,
  recording the credential as the reciprocal half of the join; the receipt
  echoes `requestId`. New error codes `requestNotFound`, `requestNotApproved`,
  `requestApplicantMismatch`.

### Retired
- **`vtc/join-requests/approve/0.1`** and **`vtc/join-requests/reject/0.1`** —
  superseded by `vtc/join-requests/decide`.
- **`vtc/join-requests/accept/0.1`** — superseded by `vtc/members/vmc` with
  `requestId`; one credential-delivery path instead of two.

(OpenVTC/verifiable-trust-infrastructure#853.)

## [0.2.47] — 2026-07-29

### Added
- **`registry/record/put/0.1`** — create-or-replace at the four-part record
  key, with an optional `expectedExisting` assertion recovering the strict
  create-only / update-only semantics (vault/upsert precedent).
- **`registry/record/query/0.1`** — optional four-part key: fully keyed is an
  exact fetch (`notFound` on a miss); partial is a filtered enumeration with
  `cursor`/`limit` pagination, fixing `registry/record/list`'s pagination gap.
- **`registry/did/rotate/0.1`** — rotate the registry's own agent-managed
  `did:webvh` keys in place; documents the pre-existing deployed wire contract
  from `affinidi-trust-registry-rs`.

### Retired
- **`registry/record/{create,update}/0.1`** → superseded by
  `registry/record/put/0.1`; **`registry/record/{read,list}/0.1`** → superseded
  by `registry/record/query/0.1`. The generated modules remain so
  already-issued documents keep validating.

(affinidi/affinidi-trust-registry-rs#120 registry/record consolidation.)

## [0.2.46] — 2026-07-29

### Added
- **`messaging/account/update/0.1`** — one partial-update task for a served
  account's role, capabilities, and queue limits, mirroring
  `messaging/account/add`'s payload (`did`, `accountType?`, `acl?`,
  `queueLimits?`; an omitted member is unchanged). Per-member guards:
  `rootAdminRequired`, `selfChangeDenied`.
- **`messaging/access-list/update/0.1`** — `{ did, clear?, add?, remove? }`,
  applied in that fixed order, replacing the three single-verb access-list
  writers.
- **`accountType` role filter on `messaging/account/list/0.1`** — subsumes
  `messaging/admin/list`.
- **`entries` membership filter on `messaging/access-list/list/0.1`** —
  subsumes `messaging/access-list/get`.

### Deprecated
- Twelve `messaging/*` tasks are now `retired` with `supersededBy`
  (affinidi/affinidi-tdk-rs#667; 19 → 9 active tasks):
  `account/change-type`, `account/change-queue-limits`, `acl/set`,
  `admin/add`, `admin/strip` → `messaging/account/update`;
  `admin/list` → `messaging/account/list` (+ role filter);
  `access-list/add`, `access-list/remove`, `access-list/clear` →
  `messaging/access-list/update`;
  `access-list/get` → `messaging/access-list/list` (+ membership filter);
  `admin/audit-log` → the generic `audit/list`;
  `admin/config` → the generic `config/show`.
  The retired modules remain generated so existing consumers keep compiling;
  producers should stop emitting them.

### Fixed
- `messaging/admin/add` (and `strip`) duplicated the `admin` keyword;
  `messaging/admin/list` listed itself in `related`.

## [0.2.45] — 2026-07-29

### Added
- **`GovernancePolicyCredential` claims profile on `vta/credentials/issue/0.1`**
  (draft-additive) — a domain's governance policy issued as a Verifiable
  Credential: `GovernancePolicyClaims` (`domain`, `policy`, `policyHash`,
  optional `contextId` / `policyMediaType`), a single-active supersession rule
  surfaced as the new optional `supersedes` response field, a mandatory
  published `credentialStatus`, and the `profileViolation` error code.
- **`statusListIndex` on the `vta/credentials/revoke/0.1` response**
  (optional) — confirms the published status-list bit flipped when revoking a
  profile credential.

Design for OpenVTC/verifiable-trust-infrastructure#804 (governance policy as a
credential); distribution reuses `credential-exchange/query`/`present`, so no
new task is introduced.

## [0.2.44] — 2026-07-29

### Added
- **`task-consent/granted/0.1`** — the previously unspecified fire-and-forget
  notice an executor sends the requester when a pending task reaches its
  approval threshold and a single-use grant is waiting. Specced from the wire
  shape VTI already ships (`{status: "granted", payloadDigest, taskType}`).
- **`task-consent/request/0.1` gains an optional `note`** — explicitly-untrusted,
  requester-authored display text, rendered attributed and never as a statement
  of effects. Absorbs the one legitimate use of the retired confirm family.

### Retired
- **`confirm/request/0.1`** → superseded by `task-consent/request`.
- **`confirm/response/0.1`** → superseded by `task-consent/decision`.
  A confirm is a task-consent with empty `effects` and `minApprovals: 1`; the
  requester-authored-`reason` trust model it required is the one task-consent
  exists to reject (OpenVTC/verifiable-trust-infrastructure#852).

## [0.2.43] — 2026-07-28

### Added
- **`acl/update/0.1`** — amend the non-role attributes of an existing entry:
  label, scopes, expiry, step-up requirement, approve-authority.
- **`AclEntry` scope direction on `acl/list`** — a `direction` filter
  (`acting-in` | `subtree` | `any`, default `acting-in`).

Both close gaps found while folding a maintainer's private `acl/*` surface onto
this family (OpenVTC/verifiable-trust-infrastructure#840 phase A). Neither is
implementation-specific.

**`acl/update` fills a hole between the existing verbs.** `acl/grant` creates
and explicitly refuses role changes; `acl/change-role` moves the role and
nothing else; `acl/revoke` removes. Nothing could say "same role, different
step-up approver". A maintainer needing that had to model it as
revoke-then-grant — a window in which the subject holds nothing — or invent a
private task.

Two refusals in it are enforced rather than advisory, and both exist so a
reduction in authority cannot be performed by a task that does not look like
one:

- **A role member is rejected** (`roleChangeNotPermitted`). `change-role` owns
  that transition because it requires the current role as a compare-and-swap;
  role is the one attribute where a lost update between concurrent writers is a
  privilege change rather than a cosmetic one.
- **Narrowing `scopes` is rejected** (`narrowingNotPermitted`), directing the
  caller to `acl/revoke`. Removing authority is what an auditor most needs to
  find, and it should appear in exactly one place. Were narrowing expressible
  here, "withdrew production access" and "corrected a label" would be
  indistinguishable in the trail.

Members replace rather than merge, which keeps *omitted* (leave alone),
*explicit null* (clear) and *empty array* (set to nothing) distinguishable —
three intentions a merge cannot tell apart, and under which removal is not
expressible at all.

**`direction` on `acl/list`** matters only where scopes are hierarchical, which
is the case where one scope identifier raises two different questions: who may
act *in* it, and what is granted *beneath* it. The default preserves existing
behaviour. It is called out because getting it wrong is quiet: a revocation
sweep using the `acting-in` reading returns exactly the entries that are **not**
the answer — the ancestors keeping their authority — while omitting every
leaf-scoped grant underneath. Short, not empty, so it reads as complete.

`_shared/0.1/CONVENTIONS.md` gains a table of which task changes what, and
records both enforced boundaries.

## [0.2.42] — 2026-07-28

### Added
- **`AclEntry.approve`** — the approve-vs-act authority axis, on the shared
  `acl/_shared/0.1/acl-entry` component, so all six `acl/*` specs gain it at
  once. `{ all?: boolean, scopes?: string[] }`: what a subject may **confer on
  others** by ratifying an approval, as distinct from `scopes`, which is what
  it may **exercise itself**.

The two axes are independent in both directions, and that is the point — it is
what lets a maintainer configure a **least-privilege approver**: a party that
can authorize an operation in a scope it has no authority to perform. Modelling
that with one list cannot express it.

Additive and safe to ignore, by construction. An absent `approve`, an absent
`all`, and an empty `scopes` all mean "may ratify nothing", so a consumer that
has not implemented the member confers *less* than the producer intended. That
is the direction a missed member has to fail in, and it is why this is a
first-class member rather than an `ext` namespace — `ext` is explicitly
ignorable, which for an authority member would mean silently dropping a
restriction a producer relied on.

`_shared/0.1/CONVENTIONS.md` gains two sections stating the consumer rules,
both of which have been got wrong in real implementations: approve-authority is
**not** authority to act (resolving "may this party do X" against `approve`
hands an approver the ability to perform what it was only meant to sign off
on), and **emptiness is never a wildcard** — an empty `scopes` or
`approve.scopes` means "nothing"; the wildcard is `approve.all`, and there is
deliberately no `scopes` equivalent.

Also records that granting `approve` is itself an escalation vector — a subject
that can grant approve-authority can manufacture an approver for an operation
it could not authorize — so maintainers SHOULD gate and audit it more strictly
than an ordinary role grant.

Unblocks folding `vta/acl/{create,get,list,update,delete}/1.0` onto the
canonical `acl/*` family (OpenVTC/verifiable-trust-infrastructure#840 phase A);
it was the one member of the VTA's ACL body with no canonical home.

## [0.2.41] — 2026-07-27

### Added
- **`credential-exchange/{offer,request,issue,query,present}/0.1`** — the
  issuance and presentation legs. The Trust Task is the transport,
  authentication, threading and relayer envelope; the body is OID4VCI
  (issuance) or OID4VP + DCQL (presentation), carried **verbatim** and
  deliberately not re-specified — re-stating a foreign specification here
  would create a second source of truth that drifts the moment it revises.
  These five are one-way messages on a thread, so they declare no `Response`
  (same shape as `chat/message` and `sync/event`).
- **`credential-exchange/pending/{list,approve,deny}/0.1`** — the holder
  operator's out-of-band surface over presentation requests the agent
  deferred for consent. A verifier the holder has not pre-trusted gets
  `consentRequired` rather than an answer or a refusal; these three are how
  the decision is then made.
- **`credential-exchange/_shared/0.1/deferred-presentation`** —
  `DeferredPresentation` and `RequestedCredential`, the approver-facing view.

Three properties are load-bearing enough to call out, because each is a rule
a plausible implementation gets wrong by default:

- **`purpose` on a query is REQUIRED and non-empty.** The holder's decision is
  a consent decision, and consent to an unstated use is not consent. Optional
  would make the well-behaved verifier indistinguishable from the one that
  declines to say, exactly when it matters.
- **No wallet enumeration.** Candidates are gathered *only* through the type
  index named by the query's `meta` discriminator. A query with no
  discriminator contributes **no** candidates — it does not mean "everything".
- **A denial carries no reason field, and unknown members are rejected.**
  "I don't hold that" and "I hold it and won't show you" must stay
  indistinguishable to the verifier; a reason string is where that leaks.

`pending/approve` is `sideEffects: destructive` — not because it deletes much,
but because the disclosure it causes cannot be walked back. It is bound to the
**original** query and nonce, and an expired deferral MUST be refused rather
than re-nonced, or the holder presents against a request the verifier has
forgotten making.

These supersede eight non-conformant URIs that `vta-sdk` binds today
(OpenVTC/verifiable-trust-infrastructure#821). The five exchange specs existed
only as files in that repo while claiming a `trusttasks.org` ID no consumer
could resolve; the three `pending-*` had no spec anywhere. They also renumber
`1.0` → `0.1` and nest `pending-list` → `pending/list`, matching how the rest
of the registry versions and groups.

## [0.2.40] — 2026-07-27

### Added
- **`vtc/config/export/0.1`** — export a community's portable configuration (its
  profile plus its stored configuration overrides) as one self-describing
  document. Takes no parameters: a document carrying an arbitrary subset of keys
  is not portable, because importing it would silently leave the unselected keys
  at whatever the target already had.
- **`vtc/config/import/0.1`** — apply that document, or preview what applying it
  would change. `confirm` defaults to `false`, so the default outcome of a
  forgotten member is a preview rather than an overwritten community. `rejected`
  is populated on the preview too — a rejection discovered *after* writing the
  accepted half leaves a community matching neither configuration.
- **`vtc/_shared/0.1/config-portability`** — `ConfigExportDocument`,
  `CommunityProfileSnapshot`, and `ConfigFieldChange`, shared by the two tasks
  above.

### Fixed
- **The TS binding generator now fails on a duplicate export alias** instead of
  emitting an `index.ts` that breaks `tsc`. Shared-module aliases are derived
  from basename + version and do **not** distinguish families, so
  `config/_shared/0.1/config` and a `vtc/_shared/0.1/config` both want
  `ConfigShared_v0_1`. That surfaced as a `TS2300` in generated code — a
  diagnostic pointing at the symptom that names neither colliding spec. The
  generator now raises at generation time with both paths. (This is why the
  shared schema above is `config-portability` rather than `config`; a
  family-qualified alias scheme would rename every existing shared export and
  break TS consumers, so the collision is avoided by name instead.)

`CommunityProfileSnapshot` is deliberately *not* `vtc/_shared/0.1/community`'s
`CommunityProfile`. That one is the update-facing view and omits `communityDid`
so a patch cannot re-point a community's identity; the snapshot **requires** it,
because the DID is what lets an import refuse a document taken from a different
community. The import reads it for that comparison and never writes it.

These supersede the non-conformant `openvtc/vtc/admin/config/{export,import}/1.0`
that `vtc-service` still binds (VTI #710) — the last two bindings on that retired
authority. They are VTC-slugged rather than folded into the generic `config/*`
family: `communityProfile` and its diff are roughly half the import's payload, so
a generic task with those pushed into `ext` would be a hollow shell in its only
real use. Same reasoning that put `vtc/backup/{export,import}` under `vtc/`.

## [0.2.39] — 2026-07-27

### Added
- **`auth/passkey/list/0.1`** — enumerate the passkeys an auth service holds for
  a subject. The credential-management counterpart to `auth/sessions/list`:
  sessions answers "where am I signed in?", this answers "what can sign me in?".
  Proof-required with no subject filter — the subject comes from the proof, so a
  filter parameter cannot disagree with the authorization check. Deliberately
  omits public key, signature counter and AAGUID: none of them help a human pick
  which authenticator to revoke.

- **`auth/passkey/revoke/{start,finish}/0.1`** — remove a passkey behind a fresh
  WebAuthn user-verification ceremony. Two legs, mirroring `enroll/*`: `start`
  names the target and returns the UV challenge, `finish` submits the assertion
  and the credential is unbound.

  - The **target is bound to the `revocationId` server-side**, so `finish`
    carries no credential of its own. Accepting one would let an intercepted
    ceremony be redirected — the user verifies one credential and a different
    one is destroyed.
  - The **last-credential refusal is normative**, and is checked at *both* legs.
    Start's check fails fast; finish's check is the one that holds the invariant,
    because ceremonies can complete in the gap between the two.
  - `user_verification_failed` covers bad signature, wrong challenge, wrong
    origin and clear UV flag alike — separating them hands an attacker the map
    of which control stopped them.

- **`RegisteredCredential`** in `auth/_shared/0.1/webauthn.schema.json` — the
  management view of an enrolled passkey, shared by list and revoke. Additive:
  no existing `$def` changed. `lastUsedAt` is a plain optional rather than a
  nullable: the first draft distinguished "tracked and never used" from "not
  tracked", and the generated-binding round-trip test rejected it, because absent
  and null map onto the same `Option`. A distinction the reference implementation
  cannot represent is not one a conforming consumer could rely on, so it is gone.

### Changed
- **`auth/passkey/enroll/{start,finish}` → 0.2.** Adds optional `uvOptions` to
  the start response and `uvCredential` to the finish request, so a consumer can
  require an **existing** authenticator to authorize adding a new one. 0.1
  modelled enrollment as a single ceremony authorized by the `proof`, which is
  right for a first passkey and wrong for a subsequent one: silently enrolling an
  attacker-controlled authenticator is durable access that outlives the token
  used to obtain it. `uvOptions` is optional because a first enrollment has
  nothing to verify against. Framework target moves to 0.2.

  0.1 remains published and unchanged. A 0.1 producer against a 0.2 consumer that
  issues `uvOptions` fails at finish — intended, rather than the consumer
  silently dropping a control it decided it needed.

### Fixed
- Relative links in the new `auth/passkey/*` specs resolve from the version
  directory. The 0.1 enroll/login specs they were modelled on carry a
  depth-by-one error in their `SPEC.md`, `_shared/` and sibling-leg links; the
  0.2 copies correct it. The same class of broken link exists in ~69 spec
  directories repo-wide and is left for a separate sweep.

## [0.2.38] — 2026-07-26

### Added
- **`vtc/*` — the 22 remaining VTC tasks.** Completes the VTC surface in the
  registry: every task `vtc-service` binds now has a spec here, closing the
  residual that kept those URIs on the non-conformant
  `trusttasks.org/openvtc/vtc/...` authority
  (OpenVTC/verifiable-trust-infrastructure#710).

  - `vtc/backup/{export,import}` — encrypted full-state envelope; import is
    two-phase, previewing row counts unless `confirm` is set.
  - `vtc/admin/invites/{list,create,revoke}` and
    `vtc/invitations/{issue,list,revoke}` — two mounts that had served
    several operations under one URI, split per method. "Enumerate" and
    "mint a bearer credential" are different contracts with different
    exposure; one URI could not state both.
  - `vtc/members/{solicit-vmc,request-vmc,vmc}` — the reciprocal-membership
    exchange as three tasks, one per party pair. The first two would have
    collided on a single slug once the parsing-artifact `spec/` segment was
    dropped from the old URIs.
  - `vtc/members/{purge,removed,self-remove-receipt}`,
    `vtc/auth/{admin-session,recognise/challenge}`, `vtc/ceremonies/list`,
    `vtc/directory/query`, `vtc/recognition/check`,
    `vtc/relationships/graph`, `vtc/policies/test`,
    `vtc/join-requests/submit-receipt`.

- **`vtc/_shared/0.1`** gains `backup`, `invite`, and `invitation` schemas —
  types two tasks in a family both need. A `$ref` across task boundaries does
  not resolve under per-file `$id`, and the build only *warns*, so shared
  types have to live here to be real.

### Notes
- Several specs make previously-implicit contracts normative where getting
  them wrong is silent: `vtc/members/solicit-vmc` states that `requested` means
  *dispatched*, not delivered; `vtc/invitations/revoke` keeps the
  `newlyRevoked` discriminator and returns the *original* `revokedAt` on a
  repeat call; `vtc/directory/query` requires a withheld field to be **absent**
  rather than present-and-null, and `notFound` rather than an empty projection,
  so the directory cannot be used as a membership oracle.

## [0.2.37] — 2026-07-22

### Added
- `credentials/_shared/0.1` — registry-wide `IssuedCredential` and
  `RevocationReceipt` definitions: the credential-minting core shared by every
  issuing family. `RevocationReceipt` states the registry-wide contract that a
  consumer MUST report `alreadyRevoked` / `already_revoked` on re-revocation
  rather than returning a second receipt silently.
- `vtc/endorsements/{issue,show,revoke,list}` — the community's Verifiable
  Endorsement Credential family, embedding the shared issuance/revocation
  receipts and adding the VTC-specific parts: endorsement-type registry gating
  and a published status-list slot (`statusListIndex`) so foreign verifiers can
  check revocation without contacting the issuing community. `revoke` adopts
  the canonical `alreadyRevoked` contract, correcting the superseded VTC
  behaviour of returning `200 OK` silently on re-revocation.
- `vtc/_shared/0.1/endorsement.schema.json` — the shared `Endorsement` row.

## [0.2.36] — 2026-07-21

### Added
- `vtc/community/profile/show` and `vtc/community/profile/update` — the read /
  write halves of the former `community/profile/manage` operation, sharing a
  new `vtc/_shared/0.1/community.schema.json#CommunityProfile`. `update` is a
  partial patch; `registryStatus` is read-only.
- `vtc/auth/recognise` — mint a scoped `xc-` cross-community session from a
  foreign community's endorsement (`vec`) + membership (`vmc`) credentials,
  mapping the foreign role to a local one via `cross_community_roles` policy.
- `vtc/registry/diagnostics` — registry-reconciler telemetry (queue depth,
  RTBF-batched / failed counts, oldest-pending age, last success/failure).
  Replaces the non-conformant `health/diagnostics` naming.

## [0.2.27] — 2026-07-21

### Added

- `vtc/members/{list,show,update}` 0.1 + `vtc/_shared/0.1` `MemberResponse`
  — the first of the OpenVTC community-service tasks migrating from the
  non-conformant `openvtc/vtc/*` authority to canonical `spec/vtc/*`. New
  top-level `vtc` namespace. `list` pages community members (each joined
  with its ACL role); `show` fetches one by DID; `update` changes a
  member's role/metadata and refuses promotion to `admin` (a separate
  gated flow). The shared `MemberResponse` replaces a cross-file relative
  `$ref` that does not survive the registry `$id` scheme. `list`/`show`
  read-only; `update` mutating + proof-REQUIRED. The subject DID is
  carried in the payload (not only the REST path) so these dispatch over
  DIDComm too and expose a `subjectPath` for policy evaluation.

## [0.2.26] — 2026-07-21

### Added

- `config/reload` + `config/restart` 0.1 — the operational half of the
  config family. `reload` re-applies hot-reloadable keys live without a
  restart (`{ keysReloaded }`, idempotent); `restart` requests a
  graceful supervisor-driven bounce so restart-gated keys take effect
  (`{ supervisor, drainTimeoutSeconds }`) and refuses with
  `supervisorRequired` when no supervisor is detected, so a restart can
  never become an indefinite outage. Both mutating, proof-REQUIRED (a
  reconfiguration and especially an availability interruption must be
  attributable); `restart` audits before signalling shutdown. `supervisor`
  is an open maintainer-defined label, matching the `source` decision in
  0.2.25.

## [0.2.25] — 2026-07-21

### Added

- `config/show` + `config/patch` 0.1 + `config/_shared/0.1` — new
  top-level `config` family for runtime configuration. `show` reads the
  effective value of each key (`ConfigField { key, value, source,
  requiresRestart }`; `source` is an open maintainer-defined label, not a
  fixed enum); `patch` writes per-key overrides and sorts each into
  `applied` / `pendingRestart` / `rejected` (partial-success, reported as
  data). Overrides ride under a single `overrides` object so the payload
  keeps a fixed additionalProperties:false envelope. `show` read-only /
  RECOMMENDED proof; `patch` mutating / REQUIRED proof. Both mandate
  redaction of secret-bearing key values. Splits what OpenVTC carried as
  one two-method admin/config/manage task into the read/write pair its own
  spec planned.

## [0.2.24] — 2026-07-21

### Added

- `audit/list` 0.1 + `audit/_shared/0.1` `AuditEnvelope` — page through a
  maintainer's append-only audit log, newest first, with optional filters
  (`from`/`to`, `action`, `actor`, `outcome`, `contextId`) and an opaque
  continuation cursor (stable under concurrent appends; SHOULD be signed).
  `AuditEnvelope` is the generic common denominator across services —
  `eventId`/`recordedAt`/`action` universal, everything else optional
  (plaintext principal DIDs nullable after RTBF, `prevHash`/`entryHash`
  present only on chained logs, `detail` for event-specific payload,
  service-specifics in `ext`). Read-only, audit-read-gated; `exposure`
  `secret` because envelopes carry plaintext DIDs and full payloads.
  Companion to `audit/verify` 0.1.

## [0.2.23] — 2026-07-21

### Added

- `audit/verify` 0.1 — new top-level `audit` family. Walks a
  maintainer's append-only audit hash chain and reports whether it is
  internally consistent, locating the first break (`tamperedEntry` /
  `brokenLink`) if not. Response accounts for every envelope
  (`entriesExamined` / `entriesVerified` / `legacySkipped` /
  `unparseableSkipped`) so a `verified: true` over a log full of skips
  cannot read as clean. Read-only, audit-read-gated. Spec is explicit
  that this proves consistency, not authenticity — an unkeyed digest is
  forgeable by an adversary with store write access; it detects accident
  and careless tampering, not a determined adversary.

## [0.2.22] — 2026-07-21

### Added

- `policy/active` 0.1 — read the active-policy bindings that
  `policy/activate` writes: given a `purpose`, the single policy
  authoritative for that slot (empty when none is active), or every
  active binding when `purpose` is omitted. Its own task rather than a
  `policy/list` filter because a purpose is a binding, not a property of a
  policy module in the relational model — `list` enumerates modules,
  `active` enumerates bindings. Read-only, `PolicyAdmin`-gated; each
  binding carries the full `PolicyModule` so no follow-up `policy/get` is
  needed.

## [0.2.21] — 2026-07-21

### Added

- `policy/activate` 0.1 — make one policy the single active policy for a
  named decision slot (`purpose`), atomically deactivating whatever was
  active before and returning the displaced `previousPolicyId`. A
  different selection model from `appliesTo`/`priority` layering (exactly
  one active per `(contextId, purpose)`), and relational rather than
  intrinsic: a policy is bound to a purpose here, not born tied to one, so
  no `_shared`/PolicyModule change is needed. Mutating, proof-REQUIRED,
  `PolicyAdmin`-gated; refuses a no-op re-activation with `alreadyActive`.

## [0.2.20] — 2026-07-21

### Added

- `policy/get` 0.1 — fetch one Rego policy module by `id`, including its
  source. The read-one companion to `policy/list` (which has no `id`
  filter): returns the full `PolicyModule`, or `notFound` for an unknown
  id where a filtered list would return an empty page. Read-only,
  `PolicyAdmin`-gated.

## [0.2.19] — 2026-07-21

### Added

- `did-management/agent-name/*` 0.1 spec family — binding human-memorable
  agent names (`/@alice`) to hosted DIDs: `set` and `enable` (mutating;
  `didData` MUST claim the name via `alsoKnownAs`), `disable` (mutating,
  step-up-required; keeps the host-side reservation) and `remove`
  (destructive, step-up-required; frees the name). Each task carries a new
  signed DID document whose `alsoKnownAs` the host verifies against the
  intended post-state so the document update and name binding land together.

## [0.2.18] — 2026-07-17

### Added

- `git-trust/*` 0.1 spec family — the first capability-module task
  family (used with `governance/capability/*`): `grant` and `revoke`
  (both proof-required) record/withdraw a member DID's commit-signing
  authority as the TRQP tuple `{subject, community authority,
  git.commit.sign, resource}` that CI verifiers query anonymously.

## [0.2.17] — 2026-07-17

### Added

- `governance/capability/*` 0.1 spec family — pluggable community
  capability management: `enable` (proof-required; supports built-in and
  DID-delegated companion capabilities via `delegate` + `manifest`),
  `disable` (proof-required; disable-not-delete), and `list` (read;
  manifest-level view for management surfaces). Shared
  `CapabilityManifest` definition in `governance/_shared/0.1` describing
  a capability's spec families, trust-registry vocabulary, roles,
  membership lifecycle hooks, consent classes, external adapters, and
  config schema.

## [0.2.16] — 2026-07-14

### Fixed

- **`vault/delete` documents `force`.** The reference implementation has always
  accepted it: it skips the grace window and hard-deletes, zeroising the secret
  bytes. The spec did not mention it, and its `consequences` promised that "the
  entry is retained only for a grace period" — which is false when `force` is set.
  A consent surface with no dry-run renders exactly that text, so it would have
  told a human they had a recovery window they did not have.

  `force` is now specified, the consequences describe both cases, and the prose
  says normatively that a consumer gating this task on human approval MUST compute
  per-request effects rather than rely on the static text: `consequences` are
  per-task, and `force` is per-request.

- **`vault/list` documents `status`** (`active` | `archived` | `deleted` | `all`).
  The archival view selector the implementation has always accepted.

Both were found by turning on payload-schema validation in a consumer — which is
the point of turning it on.

## [0.2.15] — 2026-07-14

### Added

- **`vta/webvh/dids/update/1.0`.** The task a caller uses to ask an agent to
  publish a `did:webvh` log entry it holds the update key for. Classified
  `destructive`: supplying `document` rotates the DID's update key, which SPEC
  §7.3 item 13 calls authority-shifting — and the rotation happens whether or not
  the caller asked for it.

- **`schema_index::schema_for(type_uri)`** (feature `validate`). Type URI →
  payload schema, generated from the specs.

  Without it a consumer that dispatches on a Type URI could not *find* the schema
  for the payload it was about to run: `ValidatedPayload::SCHEMA_JSON` is a
  per-type associated const, and a generic gate has no type to name. It could only
  validate by hand-writing a match arm per task and remembering to add one with
  every new task — which is to say, it would validate whatever somebody
  remembered.

## [0.2.14] — 2026-07-13

### Added

- **`task-consent/*` Trust Task family.** New `task-consent/request/0.1` and
  `task-consent/decision/0.1`, plus the shared `task-consent/_shared/0.1`
  (`Effect`, `StatePin`, `Exposure`, `Decision`).

  These are the documents a `PolicyDecision.requireConsent` (added to
  `policy/_shared/0.3` in #111) has been referring to without them existing: it
  requires "a signed consent decision from the named set, bound to the request's
  `payloadDigest`", and no such document was specified. `task-consent/decision`
  is that document.

  `task-consent/request` is the other half — the executor asks an enrolled
  approver to authorize one pending privileged task, and carries the `effects`
  it computed by **dry-running the real handler** against its own prior state.
  That matters because a payload says what was asked for, while only the code
  about to run knows what will happen: a `did:webvh` document update whose
  payload adds one service endpoint also rotates the DID's update keys, and no
  diff of the payload can show that.

  Distinct from the existing `consent/*` family, which gates an inbound
  *messaging conversation* for an AI agent (`ConsentSubject` is a
  platform/conversation pair). Additive — `consent/*` 1.0 is untouched.
  Consumers pick up `0.2.14` via `cargo update -p trust-tasks-rs`.

## [0.2.13] — 2026-07-09

### Added

- **`registry/*` Trust Task family.** New `registry/recognition/0.1` and
  `registry/authorization/0.1` (TRQP v2.0 recognition / authorization queries),
  plus `registry/record/{create,update,delete,read,list}/0.1` for Trust Registry
  record CRUD. Generated from the `registry/*` specs added in #108; writes carry
  `IS_PROOF_REQUIRED`. Additive — no change to existing tasks. Consumers pick up
  `0.2.13` via `cargo update -p trust-tasks-rs`.

  (The `registry/*` source was merged in #108 without a version bump, so it never
  reached crates.io; this release publishes it.)

## [0.2.12] — 2026-06-24

### Added

- **`vta/memory/*` Trust Task family.** New `vta/memory/put/0.1`,
  `vta/memory/list/0.1`, and `vta/memory/delete/0.1` — a generic per-context
  key/value store for AI-agent memory (cross-session recall, context-isolated),
  regenerated from the registry. Consumers pick up `0.2.12` via
  `cargo update -p trust-tasks-rs`.

## [0.2.11] — 2026-06-24

### Added

- **`messaging/admin/*` Trust Task family** — the admin-management surface, mirroring
  the messaging mediator's admin protocol: `admin/add` and `admin/strip` (grant /
  revoke admin rights), `admin/list` (page the admin accounts), `admin/audit-log`
  (page the privileged-change log, newest-first), and `admin/config` (read the
  mediator's version + configuration). Adds the shared `AdminAccount`, `AuditEntry`,
  and `AuditAction` `$def`s to `messaging/_shared`. Additive — no change to existing
  tasks.

## [0.2.10] — 2026-06-24

### Added

- **`vta/credentials/*` Trust Task family.** New `vta/credentials/issue/0.1`
  (issue a scoped, time-boxed Verifiable Credential to a holder, gated by
  operator step-up) and `vta/credentials/revoke/0.1` (withdraw an issued
  credential), regenerated from the registry. Consumers pick up `0.2.10` via
  `cargo update -p trust-tasks-rs`.

## [0.2.9] — 2026-06-23

### Fixed

- **Publish the `messaging` Trust Task family.** The messaging tasks (`ping`,
  `account/*`, `acl/*`, `access-list/*`) were added in #96 but landed without a
  version bump, so they never reached crates.io — the published 0.2.8 predates
  them. This patch republishes with `specs::messaging` included. No source change
  beyond the version; the specs are exactly as merged.

## [0.2.8] — 2026-06-22

Additive new `messaging/*` Trust Task family, regenerated from the registry.
Consumers pick up `0.2.8` via `cargo update -p trust-tasks-rs`.

### Added

- **`messaging/*` family** — generated payload modules for the new
  messaging-infrastructure Trust Tasks (`specs::messaging::*`): `ping`,
  `acl::{get,set}`, `access_list::{add,remove,clear,get,list}`, and
  `account::{add,get,list,remove,change_type,change_queue_limits}`. These
  re-express mediator account / ACL / queue administration and liveness as
  transport-agnostic Trust Tasks, sharing the `messaging/_shared` `Account`,
  `MediatorAcl`, `QueueLimits`, `AccountType`, and `Vid` definitions. No change
  to existing modules.

## [0.2.7] — 2026-06-18

Additive `chat/message` routing flags, regenerated from the registry.
Consumers pick up `0.2.7` via `cargo update -p trust-tasks-rs`.

### Added

- **`chat/message` `isGroup` / `isMention`** — optional booleans on the payload
  (`specs::chat::message::v0_1`). `isGroup` records group/channel vs 1:1 DM;
  `isMention` records whether an inbound message addresses the agent (an
  @-mention of the agent, or any DM) — distinct from `mentions`, which lists the
  participants referenced in the body. Both are signed routing context so the
  audit chain captures where a message was sent. Generated as `Option<bool>`
  (omitted when absent), so a `false` flag can be omitted for byte-lean DMs.

## [0.2.6] — 2026-06-18

`chat/message` renumbered `1.0` → **`0.1`** (aligning with the registry's `0.x`
draft convention) and extended with @-mentions, regenerated from the registry.
Consumers pick up `0.2.6` via `cargo update -p trust-tasks-rs`.

### Changed

- **`specs::chat::message::v0_1`** replaces `v1_0` — the type URI is now
  `https://trusttasks.org/spec/chat/message/0.1`. The `chat/message` task was
  the lone `1.0` outlier among `0.x` drafts; renumbered while still `draft`.

### Added

- **`chat/message` `mentions`** — an optional, ordered array of platform-neutral
  @-mentions on the payload. Each `Mention` references the mentioned party by an
  **opaque `participant` handle** (never a raw address — same model as
  `conversationId`) with an optional `displayName` hint and advisory
  `start`/`length` offsets. The body carries one `U+FFFC` sentinel per mention;
  the Nth sentinel binds positionally to the Nth entry.

## [0.2.3] — 2026-06-17

Additive `vta/did-templates/*` and `vta/contexts/did-templates/*` Trust Task families, regenerated from the registry — the previously-implemented-but-unspecced DID-template management endpoints (global + context scope), now published. Additive; consumers pick up `0.2.3` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::vta::did_templates::{list,get,create,update,delete,render}::v1_0`** — `vta/did-templates/<op>/1.0`: manage **global** DID templates on a VTA. `create`/`update`/`delete` are super-admin gated; `list`/`get`/`render` are open to any authenticated caller. `create`/`get`/`update` return the persisted `DidTemplateRecord`; `list` returns `{ templates }`; `delete` returns `{ name, deleted }`; `render` returns `{ document }`.
- **`specs::vta::contexts::did_templates::{list,get,create,update,delete,render}::v1_0`** — `vta/contexts/did-templates/<op>/1.0`: the context-scoped counterparts, gated on the context's admin (or super-admin) for writes and context access for reads. Each request carries `contextId`.

All twelve are in the `did-management` category, proof REQUIRED (`IS_PROOF_REQUIRED`, `IS_RECIPIENT_REQUIRED`), with member names in lowerCamelCase per SPEC §4.10.

## [0.2.2] — 2026-06-16

New `chat` Trust Task category, regenerated from dtgwg PR #85. Additive; consumers pick up `0.2.2` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::chat::message::v1_0`** — `chat/message/1.0`: a conversational message between an AI agent and a messaging-platform bridge, signed by its author (`eddsa-jcs-2022`, proof REQUIRED) and hash-linked (`prev`) into a verifiable per-conversation chain for audit and dispute resolution. Conversations are referenced by opaque bridge-issued handles. Fire-and-forget (no response document).

## [0.2.1] — 2026-06-07

Additive `vta/passkey-vms/*` Trust Task family, regenerated from dtgwg PR #81 — the previously-implemented-but-unspecced passkey-as-verificationMethod endpoints, now published in the registry. Additive; consumers pick up `0.2.1` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::vta::passkey_vms::enroll_challenge::v0_1`** — `vta/passkey-vms/enroll-challenge/0.1`: a DID administrator requests a WebAuthn registration challenge for a VTA-managed DID. Payload `{ did, label? }`; response carries the challenge + relying-party / user parameters.
- **`specs::vta::passkey_vms::enroll_submit::v0_1`** — `vta/passkey-vms/enroll-submit/0.1`: the administrator submits the WebAuthn registration result; the VTA re-derives the public key from the attestation and, on success, publishes the verificationMethod via a WebVH log entry. Response `{ verificationMethod, webvhVersion }`.
- **`specs::vta::passkey_vms::list::v0_1`** — `vta/passkey-vms/list/0.1`: enumerate the passkey verificationMethods on a DID. Response `{ verificationMethods }`.
- **`specs::vta::passkey_vms::revoke::v0_1`** — `vta/passkey-vms/revoke/0.1`: remove a passkey verificationMethod by fragment via a WebVH log entry. Empty success body.
- **`PasskeyVerificationMethod`** shared shape (`vta/_shared/0.1/passkey-vm`) — a WebAuthn passkey published as a `Multikey` verificationMethod (purpose `authentication`); reused by the `enroll-submit` and `list` responses.

These are admin-gated (`IS_PROOF_REQUIRED`, `IS_RECIPIENT_REQUIRED`), in the `authentication` category.

## [0.1.8] — 2026-06-04

`did-management/did/check-name/0.1` gains an **auto-assign** mode and the shared `DidRecord` gains a `didUrl` locator. Additive; consumers pick up `0.1.8` via `cargo update -p trust-tasks-rs`.

### Added

- **`DidRecord.didUrl`** (optional) — the resolvable URL of the DID's log document (e.g. `https://did.example.com/alice/did.jsonl`), stable from the initial reservation (`versionCount: 0`). Propagates to every `did-management/did/*` response that carries a `record` (`check-name`, `register`, `publish`, `info`, `list`, `change-owner`, `enable`, `disable`, `rollback`, `delete`).

### Changed

- **`check-name/0.1` request `path` is now optional.** Omitting `path` with `reserve: true` requests an **auto-assign** reservation: the host generates a fresh server-side mnemonic, reserves it under the caller, and returns `available: true, reserved: true, record` (with the generated `mnemonic` + `didUrl`). A path-less request without `reserve: true` remains invalid — that conditional is stated in the spec's §Conformance and enforced by the consumer, because the Rust codegen (typify) cannot model JSON-Schema `if/then/else`.

## [0.1.7] — 2026-06-03

Additive `push/*` Trust Task family, regenerated from dtgwg PR #72 (the push-gateway control plane modeled as Trust Tasks). Additive; consumers pick up `0.1.7` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::push::register::v0_1`** — `push/register/0.1`: a device registers its platform push token with a gateway and receives an opaque `WakeHandle` (response). Payload `{ registration, controllerVtaDid }`.
- **`specs::push::provision::v0_1`** — `push/provision/0.1`: the controller VTA sets a handle's trigger allowlist. Payload `{ handle, policy: WakeTriggerPolicy }`.
- **`specs::push::wake::v0_1`** — `push/wake/0.1`: a trigger requests a contentless wake. Payload `{ handle, v, mediator?, count?, urgency? }`; response `{ status }`.

These are addressed to the push gateway and reuse `WakeHandle` / `WakeTriggerPolicy` / `PushRegistration` from `device/_shared`. New `notifications` spec category.

## [0.1.6] — 2026-06-02

Additive push wake-up support, regenerated from the spec changes in dtgwg PR #68 (the push-gateway / VTA-owned-trigger-allowlist model). The change set is additive; existing consumers pick up `0.1.6` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::device::set_wake::v0_1`** — the new `device/set-wake/0.1` Trust Task: a device conveys its opaque `WakeHandle` (`{ gateway, handle }`) to its VTA so the VTA can own the trigger allowlist and provision the push gateway. Payload carries `wake_handle` (absent = clear), optional `push_platform` and `suggested_triggers`; the `#response` carries the effective `WakeTriggerPolicy` and `push_capable`. The raw platform push token never appears — only the opaque handle.
- **`WakeHandle`** and **`WakeTriggerPolicy`** shapes (from `device/_shared/0.1/device-binding`) — the opaque gateway handle and the VTA-owned allowlist of DIDs permitted to trigger a wake.

### Changed

- **`DeviceBinding.pushCapable`** doc — clarified that the push token is held by the gateway alone; the VTA holds only the opaque handle and the allowlist (reflected in `device/register` and `device/list`).

## [0.1.5] — 2026-06-01

Additive step-up policy support, regenerated from the spec changes in this PR (`auth/step-up/policy/0.1` + the `AclEntry.stepUp` field). The wire change set is additive; existing consumers pick up `0.1.5` via `cargo update -p trust-tasks-rs`.

### Added

- **`specs::auth::step_up::policy::v0_1`** — the new `auth/step-up/policy/0.1` Trust Task: a relying party's per-operation-class step-up *floor* (`Floor { operation, mode, allowAal1IfNonEscalating }`, `FloorMode` = `none`/`self`/`delegated`/`delegated-any`) plus the `enabled` master switch, with a `#response` carrying the effective policy.
- **`AclEntry.stepUp`** (`AclEntryStepUp { approver, require }`) on the shared `acl/_shared` `AclEntry`, surfaced in every `specs::acl::*` binding. Per-entry, additive-only step-up override: names the subject's approver VID and an optional minimum mode that may raise — never lower — the maintainer's system-wide floor.

### Changed

- Regenerated the `specs::*` modules. `AclEntry` gains the optional `step_up` field; Rust consumers constructing `AclEntry` with a struct literal must add `step_up: None` (deserialization is unaffected — the field is optional and absent ⇒ `None`).

## [0.1.4] — 2026-06-01

Dependency-maintenance release; no spec changes and no behavioural changes. Existing consumers pick up `0.1.4` via `cargo update -p trust-tasks-rs`.

### Changed

- Regenerated the `specs::*` modules with `typify` 0.6, which no longer emits the redundant reflexive `impl From<&T> for T` (a `value.clone()` no-op) on generated types. These auto-derived conversions are extremely unlikely to be referenced directly; the typed payloads and their fields, the `FromStr` / `Display` / `TryFrom` impls, and `validate`-feature validators are all unchanged.
- Bumped the internal `regress` dependency 0.10 → 0.11. `regress` is used only inside the codegen-emitted pattern validators (it does not appear in any public signature), so this is not an observable API change.

## [0.1.3] — 2026-05-30

Additive cross-device step-up + push-wake-up support, regenerated from the spec changes in PRs #61 and #62. The change set is purely additive, so existing consumers pick up `0.1.3` via `cargo update -p trust-tasks-rs` without code changes.

### Changed — existing specs

- **`auth/step-up/approve-response/0.1`**: adds optional `evidence` — a tagged union (`did-signed` | `webauthn`) selecting the elevation gate. The `webauthn` variant carries an `AuthenticatorAssertionResponse` over the step-up challenge, enabling cross-device AAL2 (a browser session elevated by a passkey on the phone) alongside the existing DID-signed gate. Adds the `assertion_invalid` and `no_gate` error codes. Generated as the `Evidence` enum.
- **`auth/step-up/approve-request/0.1`**: adds optional `acceptableEvidence` (which evidence gates the relying party will accept) and `webauthn` (`PublicKeyCredentialRequestOptions`) so a relying party can drive a passkey-backed elevation.
- **`device/_shared/0.1/device-binding`** (`DeviceBinding`): adds the non-secret `pushCapable` flag for `device/list` visibility. The push token itself is held by the mediator (per the push wake-up binding), never by the maintainer/VTA.

## [0.1.2] — 2026-05-27

This is a roll-up release covering everything merged since 0.1.1 (PRs #40–#56). The change set is overwhelmingly additive — new spec families regenerated into `specs::*` — so existing consumers should pick up `0.1.2` via `cargo update -p trust-tasks-rs` without code changes.

### Added — new spec families

- **`did-management/`** (24 specs) — full DID hosting lifecycle: `did/{check-name, register, publish, info, list, delete, disable, enable, change-owner, rollback, problem-report}`, `domain/{create, update, disable, enable, purge, set-default, assign, unassign}`, `me/domains`, `server/{register, health, stats-sync}`, `registry/{admin-register, deregister}`. Shared schemas for `DidRecord`, `DomainEntry`, `ServiceInstance`, and the webvh method extension.
- **`webvh/`** (3 specs) — did:webvh-protocol-internal mechanics: `witness/publish`, `sync/update`, `sync/delete`.
- **`vault/`** (11 specs) — credential manager surface: `list, get, upsert, delete, sync, proxy-login, release, sign-trust-task, usage` and shared schemas (`VaultEntry`, `VaultSecret`, `SessionBlob`, `SealedEnvelope`, `ConsumerContext`).
- **`device/`** (5 specs) — Companion / Service lifecycle: `register, list, disable, wipe, heartbeat`.
- **`policy/`** (4 specs) — Rego policy CRUD: `list, upsert, delete, evaluate`.
- **`sync/`** — `sync/event/0.1` server-push event envelope.
- **`provision/integration/0.1`** — generic provisioning Trust Task for template-driven integration bootstrap.
- **`auth/`** (15 specs) — full session lifecycle: `challenge, authenticate, refresh, revoke-session, whoami, sessions/list, step-up/{approve-request, approve-response}, passkey/{enroll, login}/{start, finish, invite}`.
- **`acl/swap-key/0.1`** — atomic ACL key rotation for the swap-then-rotate enrolment pattern.
- **`confirm/{request, response}/0.1`** — generic confirmation envelope.

### Changed — existing specs

- `did-management/did/register/0.1`: adds `did-management/did/register:invalid_path` error code, mirroring `did/check-name`'s identical code for the atomic register flow.
- `did-management/server/register/0.1`: adds optional `enabledMethods[]` and `protocolVersion` request fields so hosting servers can declare their capabilities. Both default cleanly when omitted.
- `provision/integration/0.1`: makes `contextId` optional with inference rules from the template's declared scope.
- `vault/proxy-login/0.1`: adds optional `nonce` and `ttlSecondsHint` fields for RP-issued challenges and caller-hinted TTLs.
- `vault/_shared/0.1/vault-entry`: adds optional `principalDid` metadata so vault entries can carry the DID they would act AS in a proxy-login.
- `vault/_shared/0.1/vault-secret`: `SealedSecret`/`SealedSessionBlob` reshape into pluggable envelopes; `VaultSecret::Password` gains a `PasswordLoginConfig` for site-specific form quirks.

### Added — framework

- `category` taxonomy is now an enforced enum at the spec.meta level — invalid categories fail validation.

## [0.1.1] — 2026-05-19

### Changed — consumer-pipeline hardening (SPEC §7.2 items 6 + 7)

- **BREAKING**: `consume_inbound`'s handler signature changes from
  `FnOnce(TrustTask<P>) -> Future<Result<TrustTask<R>, RejectReason>>`
  to `FnOnce(TrustTask<P>, ResolvedParties) -> Future<Result<TrustTask<R>, ErrorResponse>>`.
  Handlers now receive the SPEC §4.8.1-resolved parties (no need to call
  `transport.resolve_parties` themselves) and return a fully-routed
  `ErrorResponse` on refusal, freeing them to mint extended codes
  (SPEC §8.5), attach task-specific `details`, and apply spec-specific
  routing without being constrained to the framework's `RejectReason`
  vocabulary. The docstring spells out that handler-built errors are
  passed through verbatim — handlers that reject for identity-style
  reasons MUST use `reject_with_recipient` or `TransportHandler::reject`
  to preserve §8.1 routing.
- **BREAKING**: `consume_inbound`'s `verifier: Option<&V>` parameter is
  replaced by `policy: ProofPolicy<'_, V>` with three explicit variants:
  `Verify(&V)`, `RejectIfPresent`, and `AcceptUnverified`. Forces the
  security tradeoff to be a deliberate, audit-able choice at the call
  site instead of an `Option::None` whose meaning was ambiguous. The
  `AcceptUnverified` variant is the documented opt-out for transports
  whose integrity guarantees live outside the in-band proof (signed
  DIDComm envelopes, mTLS-bound HTTPS).
- `consume_inbound` now reads `Payload::IS_PROOF_REQUIRED`
  authoritatively for the SPEC §7.2 item 7 proof-required check,
  replacing the `verifier.is_some() && !P::IS_BEARER` heuristic. Per-
  spec proof contracts are enforced regardless of the chosen policy.
- **SECURITY**: under `ProofPolicy::RejectIfPresent`, `consume_inbound`
  rejects documents carrying an in-band proof with `malformed_request`.
  Silently dropping a producer-supplied proof previously misled the
  producer about the integrity guarantees of the exchange. The wire-
  exposed `message` is a neutral constant — it cites the spec section
  but does not name the consumer's configuration, so an unauthenticated
  probe cannot fingerprint deployments by verifier coverage.

### Added

- `Payload::IS_PROOF_REQUIRED` (default `false`). Codegen emits an
  explicit `const IS_PROOF_REQUIRED: bool = true;` override when a spec's
  front matter declares `proofRequirement.requirement: REQUIRED`. Mirrors
  the existing `IS_BEARER` plumbing.
- `Payload::extended_code(local)` convenience trait method — builds a
  `TrustTaskCode::Extended` under the payload's own slug (sourced from
  `Self::TYPE_URI`). Eliminates slug-literal drift in handler code and
  makes the SPEC §8.5 namespace rule enforceable by construction.
  `TrustTaskCode::new_extended(slug, local) -> Result<Self, ParseCodeError>`
  is the runtime-input-safe constructor.
- `DynProofVerifier` trait + `ErasedVerifier<V>` adapter + `erase_verifier`
  helper. Object-safe wrapper around [`ProofVerifier`] for transport
  bindings that need to store a verifier behind `Arc<dyn …>` on shared
  state (the generic method on `ProofVerifier::verify` is not
  object-safe). Reusable across bindings (HTTPS, future DIDComm, …).
- `PROOF_NOT_ACCEPTED_BY_POLICY` constant — the wire-safe message
  shared by `consume_inbound` and transport bindings for the
  proof-without-verifier rejection. Sanitised: no mention of the
  consumer's configuration that could be used as a probe oracle.

## [0.1.0] — initial pre-release, tracks `SPEC.md` 0.1

### Added — framework primitives

- `TrustTask<P>` document envelope (SPEC §4.2) with serde round-trip,
  forward-compatible extra members, and JSON-LD `@context` support.
- `TypeUri` (SPEC §4.4, §6.1) — parses
  `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` with `#request` /
  `#response` fragments, accepts private-authority variants from
  SPEC §6.5, rejects reserved-namespace slugs. Rejects `http://`
  scheme per the tightened SPEC §6.1 (HTTPS only).
- `Proof` data structure (SPEC §4.7) with forward-compat extras for
  future cryptosuites.
- `ErrorPayload`, `StandardCode`, `TrustTaskCode` — the
  `trust-task-error/0.1` payload (SPEC §8.2) with standard codes
  (§8.3) and namespaced extension codes (§8.5).
- `RejectReason` enum modelling the SPEC §7.2 rejection conditions;
  impls `std::error::Error` and converts to `ErrorPayload` via `From`.
- `ErrorResponse` type alias and `impl Error for TrustTask<ErrorPayload>`
  so error responses `?`-propagate.
- `TrustTask::reject_with` / `respond_with` / `validate_basic` — wire
  the §4.4.1 success and failure response paths and apply §7.2 items 4
  and 5. `is_expired_at` / `validate_basic` use `now ≥ expiresAt`
  (inclusive bound), matching SPEC §4.2 / §7.2 item 4.
- `TransportHandler` trait (SPEC §9.2) with the §4.8.1 in-band-wins
  precedence baked into a default `resolve_parties` method. Reference
  implementations: `NoopHandler`, `InMemoryHandler`.
- `Payload` trait + `TrustTask::for_payload` for typed per-spec payloads.
- `specs::*` module tree — per-spec `Payload` / `Response` structs
  generated by `trust-tasks-codegen` from
  `specs/<slug>/<version>/payload.schema.json`.
- `#[cfg(test)] mod conformance` inside each generated module —
  round-trip tests harvested from each spec's `spec.md`.

### Added — audience binding + identity-mismatch routing (SPEC §4.8.2, §8.1)

- `TrustTask::enforce_audience_binding()` checks `proof.is_some() &&
  recipient.is_none() && !P::IS_BEARER` and rejects with
  `MalformedRequest` per SPEC §7.2 item 8.
- `Payload::IS_BEARER: bool = false` — codegen emits an override when a
  spec opts in via `bearer: true` front-matter.
- `TrustTask::reject_with_recipient` for explicit recipient override.
  Used by `TransportHandler::reject` to apply SPEC §8.1 routing —
  identity-mismatch rejections go to the transport-authenticated peer,
  never the contested in-band issuer.
- `RejectReason::wire_message()` returns sanitised strings for
  identity-bearing rejections; `From<RejectReason> for ErrorPayload`
  uses it so the consumer's expected VID isn't leaked over the wire.

### Added — opt-in validation, proof verification trait

- `validate` Cargo feature — runtime JSON Schema validation against
  the embedded `payload.schema.json` files via the `jsonschema` crate
  (Draft 2020-12). `ValidatedPayload` trait emitted by the codegen on
  every request payload.
- `ProofVerifier` trait (async via `async-trait`) + `VerificationError`
  enum — the seam where cryptosuite crates plug in. No suites
  implemented in this crate; companion crates live elsewhere
  (`trust-tasks-proof` with the `affinidi` feature).
- `Dispatcher<R>` keys its routes on `TypeUri::for_routing()` so the
  `#request`-fragmented and bare forms route together, per SPEC
  §4.4.1 item 1.

### Added — discovery (SPEC §11)

- `discovery` module with `match_slug` / `query_matches` primitives and a
  `DiscoveryRegistry` builder. `respond_to(&query)` consumes a typed
  `trust-task-discovery/0.1` request and emits the matching subset of
  registered Type URIs.
- `DiscoveryRegistry` implements `FromIterator<impl Into<String>>` so
  routing tables (e.g. `HttpsServer`'s) can `.collect()` directly into
  a registry.
- Generated `specs::trust_task_discovery::v0_1::{Payload, Response}`
  via the codegen, with the spec.md request/response examples wired
  into the standard `#[cfg(test)] mod conformance`.
- `TypeUri` parser accepts `trust-task-discovery` as a framework-defined
  slug per the SPEC §6.1 reserved-slug list.

[0.1.4]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.4
[0.1.3]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.3
[0.1.1]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-rs-v0.1.1
[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
