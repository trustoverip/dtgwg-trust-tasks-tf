# Changelog — `@openvtc/trust-tasks`

All notable changes to the TypeScript bindings package.

This file starts at 0.15.0. Earlier releases are in the git history of
`trust-tasks-ts/`; `trust-tasks-rs/CHANGELOG.md` records the changes the two
libraries shipped together, which is most of them.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

## 0.17.0 — 2026-09-06


### Fixed

- **persona**: A resolved profile entry is not a pool record (#370)

* fix(persona)!: a resolved profile entry is not a pool record

  `persona/profile/get/1.0` types each entry of its `resolved` array as the
  shared `Attribute` — the pool record shape, which requires `attributeId`,
  `version` and `updatedAt`.

  An `inline` profile entry has none of the three. It is a value the holder
  keeps in one profile and nowhere else, so there is no pool attribute to
  have an id, a version or a last-write time; that is the entire reason
  inline exists. The response therefore cannot describe a profile containing
  one, and a maintainer implementing it is left choosing between two
  dishonest answers: synthesise an `attributeId`, which is a false claim
  about where a value lives, or omit the entry, which returns a profile that
  appears to present less than it does — the failure this family is built to
  prevent.

  Found by an implementation driving `profile/get` end to end. It is the
  second defect of this shape in the persona family, after #367, and both
  have the same cause: a *projection* borrowing the shape of the *record* it
  projects from. They are not the same thing, and the difference shows up
  exactly where the projection carries something the record cannot.

  So `resolved` gets its own definition. `ResolvedClaim` requires `type`,
  `valueType` and `provenance` — everything a projection always has — and
  makes the three pool members optional, where their absence now says "this
  value is inline". Their presence stays informative: `version` beside a
  pinned entry is what lets a holder see that a profile is frozen at v3
  while the pool has moved on.

  A new normative requirement says the rest out loud: return every entry
  including inline ones, never omit an entry you cannot fully describe, and
  never synthesise an identifier for a value that has none.

## 0.16.11 — 2026-09-05


### Specifications

- **persona**: Attribute.value is OPTIONAL, so the default listing is representable (#367)

The schema required `value` while its own description said the member is
  absent in two situations. That contradiction made the DEFAULT path of
  persona/attribute/list unrepresentable: `includeValues` defaults to false
  precisely so a picker can render the pool without decrypting every fact in it,
  and a maintainer taking that path had to choose between disclosing every value
  in bulk and emitting a non-conformant response.

  The same contradiction blocked the other documented case — a credential-backed
  value that could not be re-derived, which is returned carrying `stale` so the
  holder learns a claim has stopped being presentable rather than seeing a pool
  that looks smaller than it is.

  Found by a conformance witness in verifiable-trust-infrastructure while wiring
  the family: the witness for the default listing would not round-trip. Which is
  what witnesses are for — the defect was in the schema, not the implementation,
  and nothing else would have caught it before the first non-disclosing read
  went out.

  Both bindings regenerated; conformance checks 403 specs against 403 TypeScript
  and 398 Rust modules.

- **rooms**: The Consent/purpose section six specs shipped without (#365)

The Security & Privacy lint landed 2026-08-26 (#273). These six specs landed
  between 3 and 5 September, each carrying `Data carried`, `Correlation` and
  `Retention` but not `Consent/purpose`. That is not the debt the allowlist is
  for — its own message says it is "only for content predating this lint" — so
  the fix is to write the sections, not to list the specs.

  Each says what the disclosure is for, what records the basis, and where the
  purpose stops. What the six have in common is that the interesting part is the
  limit:

  - `rooms/records/curate` — the `reason` is addressed to the room, not to the
    record's author, who is neither asked nor notified; and no consent withdrawn
    here reaches an export taken before the retraction.
  - `rooms/owner/claim` — the consent authorizing a claim was given in advance by
    the previous owner, and a nomination with no expiry cannot be withdrawn by an
    owner who has stopped, which is definitionally what a dormant owner has done.
    The room's other members are not party to it.
  - `rooms/owner/transfer` — the outgoing owner's consent is contemporaneous; the
    incoming owner's is not recorded at all, and the specification defines no
    member through which they accept or decline a role that carries quota, abuse
    and lifecycle obligations.
  - `rooms/keys/commit` — there is no per-commit decision to make. A member who
    does not apply one has declined nothing; consent operates at the boundaries
    of membership, not on each epoch inside it.
  - `rooms/keys/key-package` — minting is not joining, the expiry is a purpose
    bound rather than housekeeping, and reuse across rooms is a disclosure the
    party never made and cannot detect.
  - `rooms/keys/welcome` — consent reaches the member's own key-holding agent and
    no further, which is why a `private` room's Welcome is not routed through the
    host: accepting an invitation is not consenting to the host learning you are
    in the room.

  Prose only. No front matter, schema, or generated code moves.

  Also turns on TT_STRICT_SECURITY_PRIVACY in the CI build. With these six
  written the non-allowlisted count is zero, so strict mode passes today and
  fires only on a spec added without the section — the allowlist is closed to new
  content by definition. It was six specs in three days that made the case: the
  warning was invisible in a green build, and nothing else would have caught
  them. Negative-tested by removing one section and confirming the build fails.

## 0.16.10 — 2026-09-05


### Added

- **persona**: A holder's own identity, composed once and disclosed under control (#360)

* feat(persona): attribute pool — shared schema and attribute/put

  First slice of the persona family: the holder's own identity attributes,
  agent-scoped above the context boundary.

  The shared schema carries the family's load-bearing definitions — the three
  provenance kinds, the four profile-entry forms, the proof-rung ordering, and
  the two-scope rule that keeps a context-scoped caller from reading the pool.

  attribute/put establishes the pattern: holder-authorized and unscoped,
  value validated against its declared type, credential-backed provenance
  resolved at write time so an attribute cannot read back stale forever, and an
  advisory correlation count returned on the write so a builder can warn while
  the holder is still composing.

- **rooms/owner**: Transfer and claim, so a room outlives one person (#359)

§10 of the data-rooms design was the last part with no specification.
  Ownership there is load-bearing for liveness, not just administration: a
  room's owner is its sole committer, so a room with no reachable owner
  cannot advance an epoch, cannot be renewed, and lapses to read-only. One
  person becoming unreachable ends a shared space.

  Two tasks, not one with a flag. They end the same way and differ in
  everything else - who initiates, what authorizes, and whether the room
  has to have lapsed. Collapsing them would mean either a transfer that
  waits for a lapse or a claim that works while the owner is still
  renewing, and both are wrong.

  There is deliberately no nominate task. A nomination is a credential the
  room issues and the claimant presents, the same shape as an invitation
  and for the same reason: a host keeping a roster of successors would hold
  part of the room's authority structure, and the room could no longer move
  hosts without rebuilding it. The consequence is that 'no successor' and
  'the successor has not claimed yet' are the same observation to a host -
  which is fine, because both resolve identically when retention runs out.

  A claim is an ACT, never an automatic promotion, and that is not
  ceremony. An automatic one is an ownership change nobody performed: no
  actor to audit, no moment to point at, and an owner who returns finds the
  room changed hands with no event to examine.

  Three conditions, all required, each closing a different route to a
  takeover: a nomination the room itself issued naming this claimant; the
  room dormant rather than merely lapsed, so the grace window has passed
  and the owner has had their notice; and the claimant already a member,
  because a successor who cannot commit inherits a room they cannot renew.

  Renewing cancels a pending claim, and that is the property worth
  noticing: the defence against a hostile claim is the same act as ordinary
  use. An owner who was merely away fixes it by doing what they would have
  done anyway - nothing revoked, no dispute raised.

  Transfer requires the incoming owner to be a member for the same reason.
  Handing someone a room they cannot renew looks like success and produces
  a room that lapses on schedule a year later with nobody able to save it.

  Three things the Security sections say plainly. A transfer does NOT
  remove the outgoing owner - they keep whatever their credentials confer,
  usually still admin, so handing over and leaving are two more acts. Every
  credential stays valid because the room's identifier does not change,
  which is the whole reason a room has an identifier of its own rather than
  borrowing its owner's. And the host is not an arbiter: it records what an
  authorized party told it, and ownership of an identifier is settled by
  whoever controls that identifier, not by a service that stores
  ciphertext.



### Documentation

- Base every example on the registered acl/change-role task (#356)

SPEC.md, both READMEs, the loopback example and the framework unit tests
  all illustrated themselves with `kyc-handoff/1.0`, a slug that has never
  been registered. Three consequences, in increasing order of cost:

  - `https://trusttasks.org/spec/kyc-handoff/1.0` is a live URL in the
    Abstract and in five worked examples. It resolves — the registry site
    serves the SPA for any /spec/ path — and rendered `acl/change-role`,
    the alphabetically first entry, because SpecPage fell back to
    `TT_TASKS[0]` for an unresolvable slug. A reader following the URL in
    §1 got a complete, confident page for a different task.
  - No example could be checked against anything. The build validates every
    fenced JSON block against the framework envelope schema; a payload
    naming a slug with no schema was structurally unverifiable.
  - The `trust-task-error` extended-code example was non-conforming: it
    answered an ACL request with a `kyc-handoff:`-namespaced code, which
    §8.5 forbids in the same breath as it defines namespacing.

  Everything now names `acl/change-role/0.1`, whose payloads validate
  against its published schema, whose error codes are the two it declares,
  and whose REQUIRED `proof` the examples honour. Appendix A is reproduced
  from that registry entry rather than invented. The framework parser tests
  that used the slug as an arbitrary string now use real ones —
  `trust-task-discovery` and `trust-task-control` where a single-segment
  slug is the point.

  SpecPage is strict as a result: an unregistered slug, or a known slug at
  an unknown version, renders a not-found page offering the versions that
  do exist, the way BindingSpecPage and CeremonyPage already did. The
  lookup moved into a hook-free wrapper so a route that stops resolving
  unmounts the detail view rather than changing a mounted component's hook
  count.

  SPEC.md is also maintained canonically at trustoverip/dtgwg-trust-tasks-spec;
  the matching change lands there in parallel.



### Fixed

- **rooms/owner**: A host cannot see the MLS group, so stop implying it can (#361)

Both tasks required a host to check that a party is a member of the
  room's group. A host holds no roster and no group state, so neither check
  was implementable as written. Found while starting the implementation,
  which is what implementation is for.

  claim now carries a presentation. The host has exactly one membership
  signal available - the VMC the room itself issued - and the claim payload
  did not ask for it, so condition 3 was unimplementable and the claimant's
  own standing went unverified. It is now required, and it is the same
  presentation every other room task carries.

  The spec is also exact about what that proxy is worth: a party removed
  from the MLS group while still holding an unexpired VMC would pass. That
  gap closes by revoking the credential, which is the room's job, not the
  host's. Better said than implied.

  transfer moves the obligation to the party who can discharge it. The
  incoming owner is not the one making the request, presents nothing, and
  may be someone the host has never seen - so 'MUST refuse if not a member'
  asked a host to judge a third party from no evidence. The requirement is
  now on the transferring owner, who can see the group; a host that can
  independently establish non-membership MAY still refuse, and one that
  cannot MUST NOT invent a check or treat its own ignorance as evidence.

  That last point is the same boundary the rest of the family draws,
  arriving from an unfamiliar direction: a host verifies what is presented
  to it, and a claim about a third party is not that.

## 0.16.9 — 2026-09-04


### Added

- **rooms/keys**: How a group reaches a key-holding agent (#355)

rooms/keys/open assumes something nothing specified: that a key-holding
  agent HAS the room's MLS group. An oracle that opens records cannot open
  anything until a group arrives, and nothing said how one does. Three
  tasks, because there are three distinct steps with three distinct
  authorization stories.

  key-package - the joining side mints. Per room, never reused across
  rooms: a KeyPackage is a stable public identifier, so the same one
  offered to two rooms tells anyone who sees both that one party is in
  both, which is the linkage a private room exists to deny arriving
  through the door rather than the wall. The mint is not free either - the
  recipient retains the private half against a Welcome that may never
  come - so a key-holder should require an invitation and bound how long
  it keeps an unused one.

  welcome - the owner delivers, and the INVITATION is what makes it
  acceptable. A Welcome carries a group's secrets; anyone able to reach a
  key-holder could otherwise push group state into it. Joining a room is
  already a two-party act and the VIC is already the consent artefact, so
  this is where that consent stops being ceremonial. A recipient with no
  matching unconsumed invitation MUST refuse, and MUST consume it on
  success. Joining twice is refused rather than merged: two group states
  for one room is a condition nothing downstream can resolve, and choosing
  wrong returns 'did not open' for a record the member can plainly see.

  commit - the half that is easy to forget and impossible to omit. A
  Welcome gets an agent in once; commits keep it there, and an agent that
  misses one is stuck at its last epoch and can open nothing sealed after
  it - surfacing as 'this record does not open', which reads like
  corruption rather than a missed message. Strictly in order: a replay is
  success with the epoch unchanged (a retry that failed would make every
  unreliable transport a liveness problem), a gap is refused with the
  recipient's actual epoch so the sender resumes rather than guesses. The
  epoch is in the payload rather than parsed out of the commit so a
  recipient can tell replay from gap before doing cryptographic work on a
  message it may not want.

  Two things stated rather than left to be discovered.

  Fan-out is O(n). MLS's logarithmic property is the SIZE of a commit, not
  the number of recipients. On open and attributed a host may carry them;
  on private it must not, and the owner fans out directly - so on that tier
  membership changes need the owner online. That is a real cost of the
  private tier and it belongs in the specification.

  Routing a Welcome discloses membership. Whoever carries it learns that
  this key-holder is joining this room, which is exactly what a private
  room withholds - so on that tier the host is off the path entirely
  rather than trusted not to look. Same rule, same reason, as the
  invitation itself.

  Who may commit is decided inside the group: a recipient verifies against
  the group state it already holds, never from an access-control list of
  its own. And none of this is authorization - holding a group's keys lets
  an agent decrypt; what it may DO comes from the room's authority
  credentials, checked separately.

- **rooms/records/curate**: A member changes a record's standing (#354)

A data room could be written to and read from, and a member who had put
  something in it by mistake had no answer. The rooms family had no way to
  demote, retract, restore or pin a record.

  Separate from rooms/records/put for two reasons, and the second is the
  load-bearing one.

  A record's standing is not its content. On an attributed or private room
  a host cannot read what it stores, so 'replace this record with the same
  content marked deprecated' would make the member re-seal and re-upload a
  body the host already holds, for a change that says nothing about the
  body. Curation carries no content in either direction.

  And curate is its own authority action, deliberately not implied by
  write: deciding what a room's shared knowledge is worth is a different
  grant from the ability to add to it. A community can hand an agent write
  - let it record what it learns - without handing it the standing to
  demote what a person wrote.

  Retraction is a tombstone, not an erasure. A host MUST drop the body and
  MUST keep the key, version and epoch: dropping the body is what the
  member asked for, and keeping the rest is what makes incremental sync
  converge - a caller that never saw the tombstone resurrects the record on
  its next full rebuild, which is why list returns them. active is refused
  for a retracted record rather than reporting a success that restored
  nothing.

  Permanent removal is out of scope on purpose. It breaks convergence for
  every caller that has not synchronised past the tombstone, so it belongs
  to a host's retention lifecycle and not to a member's curation verb.

  Curation assigns a new version. A demotion others are expected to
  converge on is a change like any other, and one that left the version
  alone would be invisible to every sinceVersion watermark in the room.

  pinned is orthogonal to status - 'what matters here' and 'is this still
  current' are different questions, and a room may well want its superseded
  canonical decision kept in view.

  The Security section says plainly that a retraction is not a deletion and
  that a surface presenting it as erasure is making a promise the protocol
  does not keep; and that curate is a censorship surface, which is exactly
  why it should not arrive as a side effect of being able to write.



### Specifications

- **device**: Register roomPresent and roomOpen capabilities (#351)

The rooms/keys oracle pair (#349) says an implementation checks whatever
  authorization the principal granted, and leaves the shape of that grant
  to the implementation. Where it is a device capability, there should be a
  registered value rather than a private one per implementation - that is
  what the shared Capability enum is for, and both values are additive, so
  a consumer that does not recognise one ignores it.

  Two values, not one. Producing a presentation and decrypting a record are
  different powers: an agent that indexes a room should not thereby be able
  to read it.

  Both are separate from sign. An agent that may ask for a scoped,
  audience-bound presentation is not thereby an agent that may sign
  anything at all with its principal's key - and gating either task on the
  generic signing oracle would grant strictly more than it needs, which is
  the opposite of what an oracle is for.

  Each spec's Consent/purpose section now names its value, since that is
  where an implementer reading the task will look.

## 0.16.8

### Added

- **rooms**: `rooms/keys/{open,present}` — the oracle pair that lets an agent use a data room
  without holding its principal's keys or credentials. `open` returns plaintext, `present`
  returns a presentation scoped to one action and audience; neither returns key material.

## 0.16.7

### Added

- **rooms**: the new top-level `rooms/*` family — `create`, `records/{put,get,list}`,
  `epoch/mint` and its `_shared` types. Data rooms are governed by credentials the room
  issues rather than by host state, so a host verifies presentations and keeps no member
  list. The authority chain is presented whole (leaf first, max 8 links, never
  dereferenced), a `private` room presentation must carry a same-subject binding, and reads
  present exactly as writes do.

## 0.16.6

### Added

- **device**: Four `Capability` values the registry was missing —
  `signTrustTask` and `credentialWrite` (already served by the reference
  implementation, never in the schema) and `memoryRead` / `memoryWrite` (new,
  splitting a gate that was previously binary). Added to `device/_shared/0.2`
  as camelCase and to `device/_shared/0.1` as kebab-case, each in its own
  version's convention. Additive: new modules and types, nothing existing removed or reshaped.

## 0.16.5 — 2026-09-01


### Added

- **vta**: Propose vta/credentials/list, so an issuer can see what it issued (#342)
- **vtc**: A task that returns a member's credential bodies (#341)

## 0.16.4 — 2026-09-01


### Added

- **vault**: Specify the vault/credentials family from its implementation (#338)

* feat(vault): specify the vault/credentials family from its implementation

  Eight tasks — receive, query, get, archive, unarchive, delete, restore,
  purge — are dispatched by `vta-sdk` and driven by `pnm cred-vault`, and none
  of them has an entry under `specs/`. The registry generates no bindings for
  what it does not know about, so every TypeScript consumer that wants the
  holder-side credential store has to hand-transcribe the shapes out of the
  Rust: a copy that drifts, silently, in the direction of whatever the reader
  guessed.

  This writes the family down. The shapes are read out of the implementation
  rather than invented, so `status: draft` is honest — the aim is to stop the
  contract being recoverable only by reading someone's service code, not to
  foreclose the working group changing it.

  Three properties the implementation enforces and prose has to carry, because
  a consumer that misses them writes something unsafe that still validates:

  - **Query refuses an unconstrained filter.** An empty filter returns the shape
    of the holder's whole life — every community, every role, every issuer. A
    consumer granted read access to answer one question does not thereby acquire
    the right to ask all of them, and `includeArchived` / `includeDeleted` are
    modifiers that deliberately do not satisfy the ≥1-filter rule.
  - **Descriptors never carry the body.** Query enumerates, get discloses, and
    they are separate tasks so the far narrower act of reading credential
    contents stays separately authorised and separately recorded. `get` is the
    only member of the family declaring `discloses: secret`.
  - **Not-found and not-yours give the same answer.** Distinguishing them lets a
    consumer map another context's vault one identifier at a time.

  Validity and archival lifecycle are documented as orthogonal, because they
  are: a credential can be `valid` and `archived`, or `revoked` and `active`,
  and a consumer that collapses the two axes mis-renders its own wallet.

  Two constraints found while writing these, both worth knowing:

  - A top-level `anyOf` is not usable in this registry. typify renders it as an
    untagged enum the generated example cannot deserialize into, and
    json-schema-to-typescript widens the type to an index signature — which
    would have let TS consumers write payloads that typecheck and then fail
    validation. Query's ≥1-filter rule is therefore stated as a requirement on
    the maintainer with its own error code, and the spec says why rather than
    leaving the absence to look like an oversight. No other spec in the registry
    uses a top-level `anyOf`.
  - `default` on a boolean breaks the generated round-trip test: typify
    materialises the value, and the example no longer matches itself. Defaults
    are stated in prose here, matching the registry's existing specs.

  Every spec carries `payload.invalid-examples.json`, so the negative space is
  tested rather than assumed — including fixtures for the enumeration shapes and
  for a consumer trying to declare a `purpose` that the maintainer derives.

  Found while building a browser-based VTA management console, whose
  credentials pane currently renders an explicit "this waits for the bindings"
  notice rather than hand-copying these shapes.

## 0.16.3 — 2026-08-28


### Added

- **consent**: Specify consent/approve-request/0.1 with proof REQUIRED (#331)

The prompt an agent's home service pushes to a designated approver, asking a
  human whether the agent may act on one conversation. It was already being
  sent — by OpenVTC's VTA, over DIDComm to the approver's device — with no
  published schema on either side and no registry page a second
  implementation could work from.

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
