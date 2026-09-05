# Changelog

All notable changes to `trust-tasks-rs` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Versioning follows [Semantic Versioning](https://semver.org/) over this
library's own API — not the framework's.** A breaking change to what a consumer
compiles or links against bumps the leading non-zero component (`0.2.x` →
`0.3.0` while below 1.0); anything additive is a patch. That includes changes
that alter *behaviour* against an unchanged wire format: when a specification
starts declaring `proof` REQUIRED, documents a consumer previously accepted are
now rejected, and the library version has to say so.

This scheme is deliberately **decoupled from the `SPEC.md` framework version**,
which the crate version once claimed to track. The two move for different
reasons — a framework revision can change the spec-authoring contract without
touching a single generated type, and the library can break its own API without
any framework change — so tying them meant one number trying to answer two
questions. A document's framework version is carried by each specification's
`targetFrameworkVersion` declaration (SPEC §7.3 item 3), which is where a
consumer should read it.

> **Bumping the leading component is a workspace event.** `trust-tasks-https`,
> `-didcomm`, `-proof`, `-tsp`, `-capability-client` and `-didcomm-v1` each
> depend on `trust-tasks-rs` with a pinned `version` requirement. Moving the
> leading component means updating all six requirements and releasing those
> crates too, in the dependency order `publish.yml` uses. Plan it as one change
> rather than discovering it mid-bump. (`trust-tasks-ceremony` does not depend
> on this crate and is not part of the set.)

## [Unreleased]

## [0.17.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.8...trust-tasks-rs-v0.17.9) — 2026-09-05


### Added

- **persona**: A holder's own identity, composed once and disclosed under control ([#360](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/360))

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

- **rooms/owner**: Transfer and claim, so a room outlives one person ([#359](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/359))

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

- Base every example on the registered acl/change-role task ([#356](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/356))

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

- **rooms/owner**: A host cannot see the MLS group, so stop implying it can ([#361](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/361))

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



## [0.17.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.7...trust-tasks-rs-v0.17.8) — 2026-09-04


### Added

- **rooms/keys**: How a group reaches a key-holding agent ([#355](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/355))

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

- **rooms/records/curate**: A member changes a record's standing ([#354](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/354))

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

- **device**: Register roomPresent and roomOpen capabilities ([#351](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/351))

The rooms/keys oracle pair ([#349](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/349)) says an implementation checks whatever
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



## [0.17.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.6...trust-tasks-rs-v0.17.7)


### Added

- **rooms**: `rooms/keys/{open,present}` — the oracle pair that lets an AI agent use a data
  room without holding its principal's keys or credentials.

  `open` decrypts one sealed record and returns the plaintext; `present` produces a
  presentation for one operation. **Neither returns key material or credentials**, and that
  is the whole design rather than a detail: an agent runs on a general-purpose machine, and
  a key-release call would leave it holding — indefinitely — material belonging to every
  other member of the room.

  Two properties follow that key release cannot offer. Revoking the agent actually revokes
  it, because there is nothing else for it to keep. And the blast radius of a compromised
  agent is bounded by time rather than by the lifetime of a key it was given.

  `present` requires `action`, so a presentation is scoped to what it was asked for; an
  implementation minting one that covers every action has handed the caller its principal's
  whole standing, which is what attenuation exists to prevent.

> Additive: new modules and types, nothing removed or reshaped, so a patch bump.


## [0.17.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.5...trust-tasks-rs-v0.17.6)


### Added

- **rooms**: A new top-level `rooms/*` family — `create`, `records/{put,get,list}`,
  `epoch/mint`, and a `_shared` schema carrying the visibility ladder, the authority
  presentation and the sealed-record envelope.

  A **data room** is a shared space whose access is governed by credentials the room itself
  issues. The property distinguishing this family from every other stored-data task here is
  that **a host never consults a member list of its own** — which is why the family is
  top-level rather than under a service prefix. Any host that speaks it can host any room,
  and a room moves between hosts without a credential being reissued.

  Three rules are wire commitments from this first version, because retrofitting any of them
  would break every verifier:

  - **The whole authority chain is presented, leaf first, capped at 8.** A host must never
    dereference a link's `parent`: resolving over the network would make verification depend
    on availability, turn an identifier into a request the host can be induced to make
    against an address the *producer* chooses, and signal credential use to whoever hosts it.
  - **A `private` room presentation must carry `subjectBinding`** — proof that the membership
    credential and the authority chain's leaf describe the same subject. Without it two
    parties pool credentials and verify as one party holding both.
  - **Reads present exactly as writes do**, and a `private` room requires no host session.
    Authorizing reads by session would log a member identifier on every access, and a period
    of such logs reconstructs the membership the tier exists to withhold.

> **Patch, not a leading-component bump.** New spec families add modules and
> types; the generated tree carries no enum over slugs or families for a consumer
> to match exhaustively, so nothing existing stops compiling. A leading-component
> bump here would be a workspace event — seven crates pin `trust-tasks-rs = "0.17"`
> and would each need moving and releasing — for a change that breaks nobody.

- **device**: Four `Capability` values the registry was missing. `sign-trust-task`
  and `credential-write` were already served by the reference implementation
  without ever reaching the schema; `memory-read` and `memory-write` are new,
  and split a gate that was previously binary — before them, any consumer that
  could reach a trust context could also rewrite every memory in it, so there
  was no way to grant an agent read-only access to a person's memory. Added to
  both published shapes in each one's own convention: `device/_shared/0.1`
  (kebab-case) and `device/_shared/0.2` (camelCase).

> **Additive, and still a call-site break.** The generated `Capability` is a
> plain Rust enum, so four new variants make every exhaustive `match` on it
> non-exhaustive. Consumers that match with a wildcard arm are unaffected;
> consumers that enumerate need one edit each. Patch bump per this crate's
> versioning rule (the wire format is unchanged and older documents still
> parse), but worth reading before upgrading.
>
> The schema description now also states the rule consumers should already have
> been following: an unrecognised capability value is **ignored**, never treated
> as conferring anything, and never a reason to reject the whole binding.


## [0.17.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.4...trust-tasks-rs-v0.17.5)


### Added

- **device**: Four `Capability` values the registry was missing. `sign-trust-task`
  and `credential-write` were already served by the reference implementation
  without ever reaching the schema; `memory-read` and `memory-write` are new,
  and split a gate that was previously binary — before them, any consumer that
  could reach a trust context could also rewrite every memory in it, so there
  was no way to grant an agent read-only access to a person's memory. Added to
  both published shapes in each one's own convention: `device/_shared/0.1`
  (kebab-case) and `device/_shared/0.2` (camelCase).

> **Additive, and still a call-site break.** The generated `Capability` is a
> plain Rust enum, so four new variants make every exhaustive `match` on it
> non-exhaustive. Consumers that match with a wildcard arm are unaffected;
> consumers that enumerate need one edit each. Patch bump per this crate's
> versioning rule (the wire format is unchanged and older documents still
> parse), but worth reading before upgrading.
>
> The schema description now also states the rule consumers should already have
> been following: an unrecognised capability value is **ignored**, never treated
> as conferring anything, and never a reason to reject the whole binding.

## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.3...trust-tasks-rs-v0.17.4) — 2026-09-01


### Added

- **vta**: Propose vta/credentials/list, so an issuer can see what it issued ([#342](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/342))
- **vtc**: A task that returns a member's credential bodies ([#341](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/341))
- **vault**: Specify the vault/credentials family from its implementation ([#338](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/338))

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



## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.2...trust-tasks-rs-v0.17.3) — 2026-08-28


### Added

- **consent**: Specify consent/approve-request/0.1 with proof REQUIRED ([#331](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/331))

The prompt an agent's home service pushes to a designated approver, asking a
  human whether the agent may act on one conversation. It was already being
  sent — by OpenVTC's VTA, over DIDComm to the approver's device — with no
  published schema on either side and no registry page a second
  implementation could work from.



## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.1...trust-tasks-rs-v0.17.2) — 2026-08-28


### Fixed

- **codegen**: Carry unevaluatedProperties strictness into the generated type ([#327](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/327))

* fix(codegen): carry unevaluatedProperties strictness into the generated type

  A specification that reuses a shared definition and adds members of its own
  has exactly one way to close the result: `unevaluatedProperties` at the
  outer level. `additionalProperties` cannot do it — both keywords are
  evaluated against the whole instance from within the subschema that declares
  them, so one inside either the shared definition or the composing schema
  rejects the other's members. That is why `credentials/_shared/0.2`'s
  `IssuedCredentialBase` is deliberately left open and
  `vta/credentials/issue/0.2` closes over it.

  typify does not model `unevaluatedProperties`. It flattens the `allOf` into
  a single struct carrying every member — the right shape — but emits no
  `deny_unknown_fields`, because it maps only `additionalProperties: false`.
  The generated type is then more permissive than the schema it came from: a
  validator rejects an unknown member, the Rust type silently accepts it.

  `issue/0.1` closes with `additionalProperties` and generates
  `#[serde(deny_unknown_fields)]`. `issue/0.2` says the same thing the only
  way a composition can, and generated nothing. That is a silent weakening
  across a version bump whose whole purpose was to stop the two definitions
  drifting.

  This flattens the composition before typify sees it, so the strictness
  survives. The struct is unchanged; only the attribute is added.

  Two things it deliberately does not do:

  - It runs after `raw` is captured, so `PAYLOAD_SCHEMA` still carries the
    published `allOf` + `unevaluatedProperties` text. Runtime validation is
    against the real schema, not this rewrite.
  - It declines any composition whose members are not plain object schemas —
    a nested combinator, an `if`/`then`, a member that closes itself, an
    unresolvable `$ref`. The merge would not be sound there, and a wrong
    `deny_unknown_fields` rejects valid documents, which is worse than the
    permissiveness it would fix.

  Only `vta/credentials/issue/0.2` uses the pattern today, so the regenerated
  output is one file. It is the composition style the registry is moving
  toward, which is why it is worth fixing at the generator.



## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.17.0...trust-tasks-rs-v0.17.1) — 2026-08-27


### Added

- **rs**: Index the consumer policy by Type URI, not just the schema ([#321](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/321))

`schema_index` exists, in its own words, "for consumers that dispatch on the
  URI". Such a consumer holds a `TrustTask<serde_json::Value>` and has no `P`, so
  it cannot reach `TrustTask::enforce_spec_policy` — the one API that applies the
  flag-driven §7.2 checks. The index handed it the schema and stopped there, so
  the only way to enforce recipient-REQUIRED, proof-REQUIRED, audience binding or
  §7.3 item 17's issuedAt-REQUIRED was to hand-maintain a URI → payload-type
  table. That is a second source of truth for data this file already generates,
  and it goes stale the first time a spec is added.

  `spec_policy_for(type_uri) -> Option<SpecPolicy>` closes that, emitted from the
  same loop as `schema_for` so the two cannot drift.

  `SpecPolicy` is a value, not four public constants, because the point is to
  share the *checks* rather than re-apply them. `TrustTask::enforce_spec_policy`
  and `enforce_audience_binding` now delegate to `SpecPolicy::enforce`, so the
  typed path and the URI-keyed path are one implementation. A new flag-driven
  rule added to `enforce` reaches both; there is no second copy to forget.



### Fixed

- **specs**: Provision/integration/0.3's response could never validate ([#324](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/324))

The 0.2 → 0.3 change renamed the bundle digest from a bare-hex `digest` to a
  multibase `digestMultibase`. It moved the member in `properties` and left
  `digest` behind in `required`.

  The object also declares `additionalProperties: false`, so the two rules
  contradict: a document MUST carry `digest`, and `digest` is not a member it may
  carry. No response can satisfy the schema, which makes the whole 0.3 response
  side unusable — and 0.3 is the version a consumer moves to for the multibase
  digest in the first place.

  The codegen then compounded it. A `required` entry with no property schema has
  no type to read, so `trust-tasks-rs` emitted

      pub digest: ::serde_json::Value,

  a required, untyped field. The generated Rust type demanded a member the schema
  forbade, so the two halves of the same release disagreed about the wire form.
  Both are correct now: `digest` is gone from `required`, and the regenerated type
  carries `digestMultibase` as the optional member the spec describes.

  Nothing caught it because every existing shape check reads one keyword at a
  time — `required` is checked for presence, `properties` for casing and bounds,
  `additionalProperties` for being declared at all. The contradiction only exists
  between them.

  So the guard is added at that level: for any object closed by
  `additionalProperties: false`, every name in `required` must appear in
  `properties`. It runs over the whole registry — 349 specs — and this was the
  only instance. Restricted to `additionalProperties: false` deliberately: under
  `unevaluatedProperties` an `allOf` branch may legitimately supply the member,
  and failing those would punish exactly the composition the registry is moving
  towards.



## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-rs-v0.14.0...trust-tasks-rs-v0.17.0) — 2026-08-27


### Added

- **specs**: Require issuedAt on every live consequential spec, and make the §7.3 item 17 floor fatal ([#302](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/302))

* spec: make the §7.3 item 17 freshness floor a hard failure

- **rs**: Make SPEC §7.3 item 17's issuedAt MUST expressible ([#300](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/300))

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



### Changed

- **versioning**: Release the trust-tasks-rs-exposing crates in lockstep ([#315](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/315))


### Fixed

- **rs**: Enforce both halves of the SPEC §6.1 slug reservation ([#293](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/293))

* fix(rs): enforce both halves of the SPEC 6.1 slug reservation

  TypeUri rejected a published framework specification and accepted a reserved
  one, in the same function pair:

    REJECTED  trust-task-control     published since 0.1; its Type URI would not parse
    PARSES    trust-ceremony-evil    reserved by 6.1, accepted from any party

  is_reserved_namespace checked only the trust-task half of
  ^trust-(task|ceremony)($|-|/), so the ceremony half of the reservation — which
  exists precisely so the ceremony layer has a namespace no other party can claim
  first — protected nothing. And is_allowed_framework_slug listed five slugs
  where specs/spec.meta.schema.json lists seven, so two published specs sat
  inside the reservation with no permission to be there.

  The allowlist is one of two hand-maintained copies. The new test reads the
  meta-schema's copy and fails naming the missing slug if they drift again;
  verified by removing trust-task-control and watching it fail.

  Found while retiring trust-task-ok ([#292](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/292)), which swept for references to the
  slug and noticed the two lists disagreed.



### Specifications

- Bound the remaining free-text payload members and state who reads them ([#301](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/301))

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

- Bound every free-text payload member with a maxLength (§7.3) ([#296](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/296))

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



**No version has been taken for this entry yet, deliberately.** The change below
is breaking under the rules at the top of this file, so releasing it moves the
leading component and drags the six dependent crates with it — the workspace
event the callout above says to plan rather than discover. Three specification
PRs are in flight against the same registry; whoever sequences that cascade
assigns the number and dates this heading.

### Changed

- **BREAKING.** 92 free-text payload members across 83 draft schemas now declare
  a `maxLength`, which SPEC.md §7.3 requires of any member holding free text.
  Two consequences, both observable by a consumer:

  1. **Type change.** A `String` member that gains a `maxLength` is generated by
     typify as a validating newtype, so `Option<String>` becomes
     `Option<PayloadReason>`, `Option<AclEntryLabel>`, and so on. Reads through
     `Deref` are unaffected; construction and assignment are not.
  2. **Behavioural change.** The newtype enforces its bound in `Deserialize`, so
     a document carrying an over-long value — one this library previously
     accepted — is now rejected. This is the "alters behaviour against an
     unchanged wire format" case named at the top of this file.

  #### Migration

  A builder setter takes the newtype rather than a `String`:

  ```rust
  // before
  let entry = grant::AclEntry::builder()
      .subject("did:web:carol.example")
      .role("moderator")
      .label("Carol — content moderation".to_string())
      .try_into()?;

  // after
  let entry = grant::AclEntry::builder()
      .subject("did:web:carol.example")
      .role("moderator")
      .label(grant::AclEntryLabel::try_from("Carol — content moderation")?)
      .try_into()?;
  ```

  The conversion is fallible precisely because the bound is real: a value longer
  than the schema permits now fails where it is authored rather than at the far
  end of the wire. Reading a member still derefs to `&String`, so
  `label.as_deref().map(String::as_str)` recovers an `Option<&str>`.

  Bounds were chosen per member from the vocabulary the registry already uses —
  256 for a display `label` or an OpenSSH key `comment`, 500 for requester
  prose a surface renders to a human deciding something (matching
  `task-consent/request/0.1` `note`), 1024 for `reason` / `description` /
  `message`, and 16384 for a `chat/message` `text` body. Members in `retired`
  specifications are untouched, frozen by SPEC §6.4.

## [0.14.0] - 2026-08-26

### Changed

- **BREAKING, and taken once so that no additive spec change is breaking
  again.** Every generated payload type is now `#[non_exhaustive]` and carries
  a builder. Cross-crate construction goes through the builder or
  `Default::default()` instead of a struct literal, and a `match` on a
  generated enum needs a wildcard arm.

  #### Migration

  A minimal `acl/grant` request was a struct literal naming thirteen members
  that carry no information — twelve `None`s and an empty `vec![]`:

  ```rust
  let payload = grant::Payload {
      entry: grant::AclEntry {
          subject: "did:web:alice.example".into(),
          role: "admin".into(),
          allowed_keys: None,
          scopes: vec![],
          label: None,
          created_at: None,
          created_by: None,
          updated_at: None,
          updated_by: None,
          expires_at: None,
          approve: None,
          step_up: None,
          ext: None,
      },
      reason: None,
      ext: None,
  };
  ```

  It is now the two members the specification actually requires:

  ```rust
  let entry: grant::AclEntry = grant::AclEntry::builder()
      .subject("did:web:alice.example")
      .role("admin")
      .try_into()?;
  let payload: grant::Payload = grant::Payload::builder()
      .entry(entry)
      .try_into()?;
  ```

  A struct whose members are all optional also has `Default`, so
  `list::v0_1::Payload::default()` replaces a literal of seven `None`s.

  **This is the last time an additive spec change breaks a construction site.**
  The tax this CHANGELOG has recorded repeatedly — "consumers constructing
  `AclEntry` with a struct literal must add `step_up: None`", "source-breaking
  for Rust consumers (the wire is unchanged)" — was the direct consequence of
  the generated structs being plain, exhaustive and builderless. A member added
  to a schema after this release lands in the builder as one more optional
  setter and leaves every existing call site compiling. The same now holds for
  a value added to a schema `enum`, on the precedent `StandardCode` set in
  0.7.0.

  Reading a payload is unchanged: the fields are still `pub`, so every
  `doc.payload.entry.role` keeps working. Only construction moved.

  The builder is typify's, not this crate's: `X::builder()` returns a
  `builder::X` with one setter per member, and `TryFrom<builder::X> for X`
  reports a missing required member as a `Result` rather than a panic. A setter
  takes anything that `TryInto`s the member's type, so `.role("admin")` works
  where the member is a `String` and `.label("…".to_string())` works where it
  is an `Option<String>`.

- **BREAKING: `HttpsClient::send` in `trust-tasks-https` takes one type
  parameter.** See that crate's CHANGELOG; the enabling change is here.

### Added

- **`RequestPayload`, pairing a request payload with its response type**
  (SPEC §4.4.1). The codegen emits
  `impl RequestPayload for Payload { type Response = Response; }` for every
  specification that defines a `$defs.Response`, so a transport can infer the
  response type instead of being told it — which is what removes the second
  type parameter from `HttpsClient::send` and, with it, the class of bug where
  `send::<grant::Payload, revoke::Response>(req)` compiled and failed against a
  live server.

  It is a **separate trait**, not an associated type on `Payload`, because
  associated type defaults are still unstable in Rust
  ([rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)): a
  required `type Response` on `Payload` would break every hand-written
  `Payload` impl, including this crate's own hand-modelled `trust-task-error`
  payload. Implementing `Payload` is exactly as cheap as it was.

  A specification defining no `$defs.Response` gets **no** `RequestPayload`
  impl. There is no response document to name, and the absence says so rather
  than a stand-in doing it badly — `()` is not a `Payload` and would let
  `send()` compile for an exchange that returns no document, and
  `trust_task_ok::v0_1::Payload` is being retired by framework 0.5.0 in favour
  of a specification declaring an empty `#response` of its own. When a
  specification adopts that empty `#response`, the codegen sees the
  `$defs.Response`, emits the type, and emits the impl — no change needed here.

### Notes

- **`ext` still deserialises as `Option<Ext>`.** Mapping it to
  `#[serde(default)] ext: Ext` so it never has to be spelled was considered and
  rejected: `Ext` carries `minProperties: 1` in the framework schema, so an
  empty `Ext` would serialise as `{}` and fail validation where the member is
  today simply absent. Changing the wire is not on the table for an ergonomics
  release, and the builder removes the need anyway — an unset `ext` is never
  spelled.

## [0.13.4] - 2026-08-26

### Added

- **One Cargo feature per top-level spec family, so a consumer can compile the
  three tasks it uses instead of all 344.** `specs/` is 15 MB of generated
  types and every consumer paid for the whole registry; the largest consumer in
  the ecosystem keeps 172 hand-maintained URI string constants rather than
  depend on these types at all, and compile weight is part of why.

  Each of the 26 non-framework families — `acl`, `audit`, `auth`, `chat`,
  `config`, `confirm`, `consent`, `credential-exchange`, `device`,
  `did-management`, `git-trust`, `governance`, `keys`, `messaging`, `policy`,
  `provision`, `push`, `registry`, `sync`, `task-consent`, `vault`, `vrc`,
  `vta`, `vtc`, `webvh`, `witness` — now has a feature of the same name, and
  `all-specs` enables all of them.

  Measured on one machine, dependencies cached, `CARGO_INCREMENTAL=0`,
  `cargo clean -p trust-tasks-rs` before each run:

  | selection | before | after |
  |---|---|---|
  | default features | 21.7 s | 22.4 s |
  | `--features validate` | 21.3 s | 22.2 s |
  | `--no-default-features --features vault` | n/a | **4.0 s** |
  | `--no-default-features --features vault,validate` | n/a | **4.0 s** |
  | `--no-default-features --features acl` | n/a | **1.2 s** |

  The default column is deliberately flat: it compiles the same 344 modules it
  always did, and the difference between the two figures is run-to-run
  variance. The saving is entirely in what a consumer can now decline. The
  generated tree is still 15 MB on disk — this changes what is compiled, not
  what ships.

  ("before" was measured at `cd170a1`, i.e. 0.13.2; 0.13.3 added hand-written
  code only and does not move the generated tree.)

  The feature table and the `#[cfg]` gates are written by
  `trust-tasks-codegen` from what is on disk under `specs/`, and
  `tests/spec_feature_manifest.rs` re-derives the family set from `specs/`
  and fails if the two disagree. `CLAUDE.md` documents three separate
  incidents of a hand-maintained list drifting from the tree; this list is
  not one of them.

  The five framework-reserved families of SPEC §6.1
  (`trust-task-control`, `trust-task-discovery`, `trust-task-next-step`,
  `trust-task-ok`, `trust-ceremony-receipt`) are **not** gated. The crate's own
  framework machinery depends on them unconditionally — `discovery.rs` does
  `use crate::specs::trust_task_discovery::v0_1` at module scope — and they are
  1.4% of the generated tree, so gating them would cost every consumer the
  framework surface and save nothing.

### Changed

- `schema_index::schema_for` returns `None` for a Type URI whose family was not
  selected. That is the documented meaning of `None` ("this build knows no spec
  for it") and is unreachable under `default`, which enables every family.

> **Non-breaking under `default`, with one caveat.** `default = ["all-specs"]`,
> so a consumer who configures nothing compiles exactly the module tree it
> compiled in 0.13.3. The exception is a consumer that already wrote
> `default-features = false` — a no-op against 0.13.3's empty `default`, but
> from 0.13.4 it deselects every family. Such a consumer adds `"all-specs"`
> (or the families it uses) to `features`.

## [0.13.3] - 2026-08-26

### Added

- **`AsyncDispatcher<Ctx, R>` — async, context-carrying dispatch.**
  `Dispatcher::on` takes `Fn(TrustTask<P>) -> R`: synchronous, with nowhere to
  put request-scoped state. A handler that must `await` a database, a DID
  resolution or an approval prompt cannot be written against it, so every
  receiver in the wild hand-rolls its own router instead — and none of them
  answer `unsupportedVersion` for a known slug at an unknown `MAJOR.MINOR`
  (SPEC §5.2 / §8.3); they answer `unsupportedType`, telling a producer its spec
  is unimplemented when the real fix is to downgrade.

  `AsyncDispatcher` is the async sibling: `.on_async::<P, _, _>(|req, ctx| async
  move { … })`, `.dispatch(doc, ctx).await`, `.dispatch_or_reject(doc, ctx,
  error_id).await`. It keeps the routing table `Dispatcher` already had, so the
  `unsupportedType` / `unsupportedVersion` split falls out by construction, and
  it downcasts `Value → P` once per message rather than once per arm.

  After the downcast — the first point at which the codegen-emitted `Payload`
  flags are reachable — it applies `TrustTask::enforce_spec_policy` to request
  documents, covering §7.2 items 5b (`recipient` REQUIRED), 7A (`proof`
  REQUIRED) and 8 (audience binding). That is the same method `consume_inbound`
  and the HTTPS server call, so the three cannot drift. `#response`-variant
  documents route straight through, since those items govern what a consumer
  demands of a request.

  `Dispatcher` is untouched and remains the right tool for a synchronous match.

## [0.13.2] - 2026-08-26

### Changed

- **Regenerated after the registry re-cased 28 references to the §8.3 *standard*
  error codes** (SPEC §4.10 rule 2). Where 0.13.1 covered rule 4's SHOULD over a
  specification's own extended codes, this covers rule 2's MUST over the
  framework's own — `permission_denied` where §8.3 says `permissionDenied`.

  Only one spec's schema named a standard code in a `description`
  (`trust-task-discovery/0.1`, `requiredExt`), so the movement is confined to
  the doc comments and the embedded `PAYLOAD_SCHEMA` literal of
  `specs::trust_task_discovery::v0_1`. Descriptions do not participate in
  validation: no generated type, constant or behaviour changes, so this is a
  patch.

  `trust-task-error/0.1` was deliberately **not** re-cased and no generated
  module moved on its account. Its snake_case table is the published
  framework-0.1 error vocabulary, `trust-task-error/0.2` exists solely to carry
  the re-cased set and declares `wireCompatibleWith: "0.1"`, and the Rust side
  of that slug is hand-modelled in `error.rs` rather than generated.

## [0.13.1] - 2026-08-26

### Changed

- **Regenerated after the registry re-cased 200 extended error codes to
  lowerCamelCase** (SPEC §4.10 rule 4). The only movement is `description` text
  inside embedded `PAYLOAD_SCHEMA` literals, where 21 schemas referenced a code
  by name. Descriptions do not participate in validation, so no generated type,
  constant or behaviour changes — this is a patch.

  The rename itself is wire-visible for *services*: an emitter sending the old
  spelling no longer matches what the registry declares. That migration is
  documented on the pull request, not here, because it does not reach this
  library's API.

## [0.13.0] - 2026-08-26

### Changed

- **`vta/memory/list/0.1` now requires a `proof` on its response.** Correcting
  that task's `exposure.discloses` from `metadata` to `secret` — its response
  returns every free-text memory `value` in a context — engages the registry's
  proof floor, which obliges a secret-disclosing response to be attributable.
  The generated `Response` impl's `IS_PROOF_REQUIRED` therefore flips to `true`,
  and a consumer now rejects an unproofed response it previously accepted.

  Behaviour changed against an unchanged wire format, which under the rules at
  the top of this file the library version has to say so. Ten other `exposure`
  corrections in the same change move no constant.

- **`trust-task-discovery/0.1` bounds `patterns` at 16 items** and raises its
  proof requirement to RECOMMENDED. Neither moves a generated constant
  (`IS_PROOF_REQUIRED` flips only on REQUIRED); the schema bound is reflected in
  the regenerated modules.

## [0.12.1] - 2026-08-26

### Added

- **`keys/create/0.1` gains `keyId`.**

  0.11.17 added `internal`, and an internal key exposed the gap: it derives
  from no seed, so it has no `derivationPath` for the maintainer to name it
  after, and the request had no other way to name it. A maintainer offering
  internal keys therefore had to invent an identifier or refuse every request
  — and that identifier is the consumer's handle for every later
  `keys/{show,sign,rename,revoke}` call, so inventing one is a poor answer.

  Optional, because for an ordinary derived key a maintainer MAY default it to
  `derivationPath`, which is what implementations already do. The specification
  says a maintainer MUST reject a colliding `keyId` rather than overwrite,
  since silently replacing a signing key is the worst available outcome.

## [0.12.0] - 2026-08-26

> ### ⚠ Breaking, and released as a workspace event
>
> Under the rules at the top of this file this change earns **`0.12.0`**: it
> adds a required argument to `consume_inbound`, adds variants to two public
> enums, and — most importantly — makes a consumer reject documents it used to
> accept. Moving the leading component is a workspace event, so this release
> also carries the six crates that depend on this one: each has had its
> `trust-tasks-rs` requirement moved to `version = "0.12"` and its own leading
> component bumped. Publish in the dependency order `publish.yml` uses.

### Added

- **`ReplayGuard` — the duplicate-execution record of SPEC §7.2 item 11.**

  Item 11 is normative and unconditional for a *consequential Trust Task*: a
  document already accepted under a given `id` **MUST NOT** cause the effect a
  second time, and a *different* document under the same `id` **MUST** be
  rejected with `idConflict`. §8.4 is the same rule seen from the producer's
  end — a retry is a bit-for-bit resend, and it is safe *precisely because*
  item 11 obliges the consumer to absorb it. §10.1 names it as the defence
  against replay by the original recipient.

  No runtime implemented it. Every transport binding delegates it: `https/0.2`
  §5 says "Freshness / replay: None", and `didcomm/0.2` §6, `didcomm-v1/0.2` §6
  and `tsp/0.1` §7 say the same. There was no id-keyed store anywhere, and
  `StandardCode::IdConflict` was emitted by nothing. A captured envelope or a
  bearer-authenticated body, re-sent, executed `acl/grant` or `vault/release`
  again — and an ordinary mediator retry did it by accident.

  The seam is an object-safe async trait so a replicated deployment can back it
  with a shared store; `InMemoryReplayGuard` is the batteries-included LRU
  default and is documented as **not** correct behind a load balancer.

  The record is keyed on the document `id` — which §7.2 requires, to the
  exclusion of transport message ids and execution handles — and holds a
  SHA-256 digest of the canonical serialization of the whole document. Not the
  received octets: a re-indented body, or a member order chosen by an
  intermediary, would make a legitimate retry look like a different document,
  and the consumer would answer it with `idConflict` or execute it. The digest
  covers `proof`, which is a deliberate difference from the §4.9.3 *task
  digest* over `document ∖ proof`; §4.9.3 spells the distinction out, and §8.4
  requires a re-signed `proof` over identical content to be `idConflict` rather
  than an absorbed retry.

- **`FreshnessPolicy` — the acceptance window of SPEC §4.2 / §7.2 item 4.**

  `validate_basic` checked `expiresAt` and nothing else; `issuedAt` was parsed
  and looked at by nobody. So a document stamped a year in the future was
  accepted — and accepted again for the whole of that year — and one whose
  `expiresAt` sat at or before its own `issuedAt` was accepted whenever the
  clock happened to sit before the expiry.

  It is also what makes item 11 implementable. §7.2 (*Bounding the record*)
  makes the acceptance window and the replay record's retention **the same
  bound**; without a window the record would have to be kept forever, and a
  document the consumer can place in no window at all is one §7.2 forbids
  executing a consequential task on. `consume_inbound` refuses that case rather
  than pretending to guard it.

- `TrustTask::validate_freshness`, `ConsumeChecks`, `ReplayPolicy`,
  `document_digest`, `DocumentDigest`, `ReplayVerdict`, `ReplayGuardError`,
  `canonical_json`, `sha256_hex`, and `RejectReason::malformed_from_serde`.
- `trust_task_error_type_uri()` is now `pub` — the single source of truth for
  the emitted `trust-task-error` version, as `CLAUDE.md` already assumed it
  was. Three answers were in circulation (`0.5` here and in the TS runtime,
  `0.2` in the HTTPS server, `0.1` in the READMEs).
- Wire-message constants: `PROOF_INVALID_WIRE_MESSAGE`,
  `IDENTITY_MISMATCH_WIRE_MESSAGE`, `WRONG_RECIPIENT_WIRE_MESSAGE`,
  `STALE_WIRE_MESSAGE`, `UNAVAILABLE_WIRE_MESSAGE`.

### Changed — behavioural

- **`consume_inbound` takes a fourth argument, `ConsumeChecks`.** Required, not
  defaulted, for the reason `PayloadPolicy` is: whether the task a consumer
  implements is *consequential* is a decision only that consumer can make, and
  the failure mode of getting it wrong silently is an ACL grant applied twice
  by a mediator retry. Use `ConsumeChecks::consequential(&guard)` or
  `ConsumeChecks::not_consequential()`.

- **`consume_inbound` now enforces freshness.** Under any policy it rejects an
  `issuedAt` beyond the skew tolerance (`malformedRequest`) and an `expiresAt`
  at or before `issuedAt` (`malformedRequest`). Neither is emitted by a
  conforming producer, but a deployment whose producers are not conforming will
  see documents refused that previously passed.

- **`ConsumeOutcome` gains `Duplicate { prior_response, in_flight }`.** It is
  **not** an error: §7.2 (*Disposition of a duplicate*) — "In no case is a
  duplicate reported as `taskFailed`; the task did not fail, it already
  happened." Callers matching exhaustively must handle it.

- **`RejectReason` gains `IdConflict` and `Stale`.** Callers matching
  exhaustively must add arms. `RejectReason::code()` maps `Stale` to `expired`:
  §8.3 defines no separate code, and the window is the consumer's own bound on
  when it is still willing to act.

- **`consume_inbound` requires `R: Serialize`**, so a completed execution's
  response can be retained for the duplicate that follows it.

### Fixed — security

- **`proofInvalid` no longer carries verifier internals on the wire.**
  `wire_message` sanitised only `IdentityMismatch` and `WrongRecipient`, so
  `ProofInvalid { reason }` passed the verifier's own text through — strings
  such as `resolve did:web:x: <error>`, `verificationMethod … not present in DID
  document`, and `verificationMethod is controlled by {vm_did}, not the
  document issuer {issuer}`. The recipient of that message is by construction
  *unauthenticated* — the proof did not verify — so it was a
  resolver-reachability and DID-document oracle, answered at the sender's
  chosen rate. SPEC §10.4 makes the rule explicit for `identityMismatch` and
  generalises it to every code. The detail remains on the `Display` impl, which
  is what the operator logs.

  `Unavailable` and `Stale` are sanitised on the same reasoning: the first
  would otherwise name a backing store, the second the size of the consumer's
  acceptance window.

- **`RejectReason::malformed_from_serde` categorises deserializer failures**
  instead of echoing `serde_json`'s member path and byte offset, which describe
  the consumer's internal type layout to anyone willing to send malformed JSON.

### Documentation

- README: the `consume_inbound` example took 7 arguments and the function took
  8 (`payload_policy` was added later); it now takes 9 and compiles. The
  `trust-task-error/0.1` claims and the "`0.1.0` — tracks `SPEC.md` version
  `0.1`" status line were both years stale.

### Not in this release

The binding crates still do not apply either check to the documents they
consume themselves — see the PR body. They are owned by concurrent work and
were deliberately left untouched.

## [0.11.17] - 2026-08-26

### Added

- **`keys/create/0.1` gains `internal`, and `KeyOrigin` gains `internal`.**

  A maintainer may hold keys that are generated from a CSPRNG rather than
  derived from a seed — reproducible from nothing, recoverable by no means once
  the maintainer's storage is gone. The value of such a key is precisely that
  it cannot be exported. Neither the request nor the response vocabulary could
  express it: `keys/create` is `additionalProperties: false`, so a consumer
  asking for one was rejected outright, and `KeyOrigin` offered only `derived`
  and `imported`, so a maintainer that minted one could not describe it.

  The response side matters more than it looks. A consumer asks for an
  unexportable key because it needs that property; a maintainer that ignored an
  unrecognised member and returned a derived key would hand back something that
  looks identical and is not. `origin` is `internal` iff the request was
  honoured, which makes the difference detectable rather than a matter of
  trust — and the specification now says a maintainer that cannot mint one MUST
  reject rather than silently downgrade.

  `derivationPath` and `internal: true` are documented as contradictory: an
  internal key derives from no seed and records no path.

  `internal` declares no JSON Schema `default`, for the reason 0.11.16 records
  for `VaultEntry.status`: a declared default is materialised by the generated
  bindings, so an absent member reappears as an explicit `false` and breaks
  round-trip idempotence. `keys/create`'s own request example caught it.

  Additive: both members are optional, and `KeyOrigin` gains a value rather
  than changing one, so no previously-valid document becomes invalid.

## [0.11.16] - 2026-08-26

### Added

- **`vault/_shared/0.1/vault-entry`'s `VaultEntry` gains `status`,
  `archivedAt`, `deletedAt` and `graceUntil`.**

  `vault/list/0.1` already specifies a `status` request filter whose `all`
  value "lists every entry regardless of lifecycle state". That view was not
  usable: `VaultEntry` is `additionalProperties: false` and carried no
  lifecycle member, so a consumer that asked for every state received a mixed
  list it had no way to partition. A specification that offers a filter and
  then withholds the member the filter selects on is inconsistent with itself,
  in the same way `CredentialCreationOptions` without `extensions` was in
  0.11.15.

  `graceUntil` is the member that matters most in practice. A `deleted` entry
  is a tombstone, not an absence, and without a deadline a consumer cannot
  distinguish "still restorable" from "gone" — which is the difference between
  a holder recovering their credential and losing it.

  All four are optional, and an absent `status` means `active`, so a maintainer
  that models no archival lifecycle omits them and every existing document
  stays valid. Additive: five specifications that reference the shared
  component (`vault/{get,list,sync,upsert}` and `sync/event`) pick the members
  up, and no previously-valid document becomes invalid.

  `status` deliberately declares no JSON Schema `default`. A declared default
  is materialised by the generated bindings, so an absent `status` would
  reappear as an explicit `"active"` on re-serialisation — which broke
  round-trip idempotence for `vault/list`'s own response example on the first
  attempt at this change.

## [0.11.15] - 2026-08-26

### Added

- **`auth/_shared/0.1/webauthn`'s `CredentialCreationOptions` and
  `CredentialRequestOptions` gain `extensions`.**

  Both components state that they mirror the WebAuthn Level 2
  `PublicKeyCredentialCreationOptions` / `PublicKeyCredentialRequestOptions`
  dictionaries. Those dictionaries define `extensions`; these did not, while
  also being `additionalProperties: false` — so the two claims contradicted
  each other, and **a server emitting standard WebAuthn options could not
  conform**. The widely-used server libraries emit the member by default, so
  the practical effect was that conforming required stripping a standard field
  on the way out.

  Structure is deliberately unconstrained. The set of extensions is open and
  registered outside this framework; enumerating them here would date the
  schema against a registry it does not own, and a closed list would reproduce
  the original defect one revision later.

  Additive, so a patch bump. Affects `auth/passkey/enroll/start/{0.1,0.2}`,
  `auth/passkey/login/start/0.2` and `auth/passkey/revoke/start/0.1`, which
  all reference these components.

## [0.11.14] - 2026-08-26

### Added

- **`vtc/relationships/graph/0.2`** — an edge is a **pair**, not a credential.

  `0.1` called each credential an edge, which made a DTG edge inexpressible:
  the two directed halves between the same pair of identifiers are one
  relationship, and a flat list left every consumer to re-derive that — sort
  the DIDs, group by pair, decide for itself what "complete" means. Two
  implementations doing that independently disagree at the margins, which is
  the reasoning a schema exists to settle once.

  `endpoints` carries the two DIDs sorted, `halves` every credential between
  them, and `complete` whether both parties have asserted. An edge asserted by
  one party is a claim; an edge asserted by both is a relationship, and only
  the model can say which. `GraphHalf.personaDid` makes deliberate correlation
  readable: two pairwise halves carrying the same persona are the same party,
  said so by that party.

- **`vtc/relationships/list/0.2`** — `vrcSha256` becomes
  `vrcDigestMultibase`.

  A bare hex digest names neither its hash function nor its encoding, so two
  parties comparing one must agree on both out of band — and the member name
  was the only record of the function, which stops being true the moment
  anything but SHA-256 is wanted. `relationships/publish/0.2` already takes a
  multibase digest on the way in; `0.1` read back a different encoding from
  the one its sibling accepts.

### Changed

- **`vtc/relationships/graph/0.1`** and **`list/0.1`** are now `retired`,
  declaring `supersededBy` their `0.2`.

  Both breaking, both `draft`, so released as MINOR increments under
  SPEC §5.2.

## [0.11.13] - 2026-08-25

### Added

- **`schema_index::schema_for` now indexes response schemas**, under the same
  Type URI suffixed `#response` — 332 of them.

  The index answered only for requests, so a consumer could validate what it
  *received* and not what it *sent back*. A producer's own output is the half
  it can actually fix, and leaving it unindexed meant every implementation
  that wanted to check its emissions had to hand-write a match arm per task —
  which this function's own documentation already identifies as "validate
  whatever somebody remembered, which is not validation".

  `#response` is not a convention invented for the index: it is what each
  generated `Response` type already declares as its `TYPE_URI`. A task with no
  response side has no `#response` entry, and `None` remains a real answer.

## [0.11.12] - 2026-08-25

### Added

- **`vtc/auth/recognise/0.2`** — takes a holder-signed Verifiable Presentation
  instead of a bare `{vec, vmc}` credential pair.

  `0.1`'s payload was a **replayable impersonation token**. Both credentials
  are bearer artifacts, so anyone who obtained the pair — a relayed join, an
  audit log, a backup, a compromised device — held everything the payload
  required, and the recognising community could not tell the subject from
  someone holding a copy. No proof of key possession, no freshness, no
  audience binding, so one captured pair worked at every community that
  recognised the issuer, indefinitely.

  `0.2` requires a presentation holder-signed with `proofPurpose:
  authentication`, committing to the single-use nonce from
  `vtc/auth/recognise/challenge` and naming the recognising community's DID as
  `domain`. Three properties, none redundant: the holder signature proves
  possession of the subject key, the nonce defeats replay, and `domain` stops
  a presentation minted for one community being spent at another. Consumers
  MUST also refuse unless the holder is the credentials' subject.

- **`vtc/join-requests/submit/0.2`** — returns a `verdict` instead of
  `status: "pending"`.

  A submission has four outcomes; `0.1`'s `const: "pending"` could express
  one. A policy that admitted outright, refused outright, or needed more
  evidence had to be reported as pending or not at all.

  The distinction that matters most is `refer` versus `requestMore`: both mean
  "not decided", and they place the next action with different parties —
  `refer` waits on the community, `requestMore` waits on the applicant and
  names what it needs. An applicant told "pending" cannot tell whether to wait
  or to act.

- **`vtc/_shared/0.1/ceremony`** — new shared component defining `Verdict`,
  `VerdictEffect` and `VerdictWith`, because the four outcomes are the same
  wherever a policy decides something about an applicant.

  `VerdictWith` is one flat object whose members are effect-dependent by prose
  rather than by `if`/`then` per effect — a deliberate trade of schema
  strictness for a generated type that does not need a discriminated union per
  ceremony family. Consumers MUST branch on `effect`, not on which members
  happen to be present.

### Changed

- **`vtc/auth/recognise/0.1`** and **`vtc/join-requests/submit/0.1`** are now
  `retired`, declaring `supersededBy` their `0.2`.

  Both breaking — a required member replaced — but released as MINOR
  increments under SPEC §5.2, which permits that for `draft` artifacts.

## [0.11.11] - 2026-08-25

### Added

- **`vtc/install/claim/start/0.2`** and **`vtc/install/claim/finish/0.2`** —
  the first-admin enrolment pair without the DID-binding challenge and
  signature.

  `0.1` required the installer to sign a 32-byte server challenge with the
  candidate `did:key` and submit the signature at `finish`. Two things are
  wrong with that. The admin DID is *derived from* the passkey's public key,
  so a signature by that key proves nothing the WebAuthn attestation has not
  already proven — it is one key asserting control of itself, twice. And it
  cannot be produced: WebAuthn never exposes the credential private key to
  the page, so the only way to satisfy `0.1` is to reach into a software
  authenticator for private-key material, which is what a hardware
  authenticator exists to prevent.

  Proof of control in `0.2` is the attestation, and the DID follows from the
  key it attests. The `bindingInvalid` error code is kept and now refers to
  the attestation alone.

  Breaking — two required members removed — but released as a MINOR increment
  under SPEC §5.2, which permits that for `draft` artifacts.

### Changed

- **`vtc/install/claim/start/0.1`** and **`finish/0.1`** are now `retired`,
  declaring `supersededBy` the `0.2` pair. A draft that cannot be implemented
  should not stay open for new adoption.

## [0.11.10] - 2026-08-25

### Added

- **`CommunityProfileView`** — a community profile *as read*, carrying
  `communityDid` and `createdAt` alongside every `CommunityProfile` member.

  `CommunityProfile` is the update-facing view and omits `communityDid` on
  purpose, so a patch cannot re-point a community's identity. That is right for
  a payload and wrong for a response: a client holding a profile had no defined
  way to learn which DID to verify that community's credentials against.
  `community/profile/show` and `community/profile/update` now return the view;
  the update payload keeps `CommunityProfile`, so the immutability is enforced
  by absence rather than by prose.

- **`CredentialReference`** — a pointer to an issued credential without the
  credential itself: `credentialId`, `issuedAt`, `expiresAt`.

  `Endorsement.issued` referenced `IssuedCredential`, whose own scope note says
  it is "returned by the party that minted the credential". A listing is not an
  issuance event, and requiring the signed credential in every row made a page
  grow with the size of the credentials rather than the number of them — fifty
  rows is megabytes. `Endorsement.issued` now references the pointer, and
  `endorsements/issue` gains a required `credential` member, because that is
  the one call whose caller has no other way to receive what was just minted.

- **`registry/diagnostics` response** gains the messaging and transport half:
  `syncerEnabled`, `syncerRunning`, `syncerRestarts`, `messagingStatus`,
  `vtaDid`, `mediatorUrl`, `mediatorDid`, `registryTransport` and `transports`,
  with new `TransportStatus` and `RegistryTransport` components.

  `syncerEnabled && !syncerRunning` is "spawned but dead" and no queue count
  reveals it; a rising `syncerRestarts` is the crash-loop signal that otherwise
  looks like a healthy idle queue. `TransportStatus` keeps `advertised` and
  `serviceable` separate because advertised-but-not-serviceable is the broken
  state a single boolean hides.

- **`CeremonyManifest.factsTemplate`** — the skeleton of the decision input a
  ceremony evaluates, which is the difference between rendering a form and
  simulating the decision it produces.

- **`website/files/list` item `etag`** — an opaque content hash matching the
  `ETag` a direct read returns, so a client can detect a change without
  downloading the body.

- **`website/generations/list` item `deployedAt` and `sizeBytes`** — a rollback
  target is chosen by *when* far more often than by number, and size
  distinguishes a real deployment from a truncated one.

- **`install/claim/start` payload `claimSecret`** — the out-of-band second
  factor on a claim. The install URL alone is deliberately insufficient, so a
  stolen or forwarded link cannot claim the passkey by itself. Optional in the
  schema, because a maintainer may issue tokens without one, but a maintainer
  that issues one MUST require it.

All additive on the wire, so a patch bump — but see *"A patch bump promises
wire compatibility, not source compatibility"* in CONTRIBUTING-SPECS.md.
`Endorsement.issued` changing type is the sharper edge here: consumers reading
`issued.credential` on a listing must read `endorsements/issue`'s new
`credential` member instead.

## [0.11.9] - 2026-08-25

### Added

- **`CommunityProfile.personhood`** and the matching
  `vtc/community/profile/update/0.1` payload member — what a community's
  governance asserts about the personhood of its members.

  DTG Credentials §Personhood Credentials puts PHC status in governance rather
  than in the credential — *"PHC status is determined by governance and trust
  registries, not by credential structure"* — so a verifier deciding whether a
  membership credential carries personhood weight is told to read the
  community's published position. Until now there was nowhere to publish it:
  the profile that a `show` returns and an `update` sets had no member for the
  one thing the credential spec says not to read off the credential.

  The new `PersonhoodGovernance` component carries `realHuman`,
  `singleMembership`, `acceptedIdvps` and `governanceFrameworkUrl`. Every
  member is a declaration rather than an enforcement; absent means no position
  has been published, which is not the same as asserting the negative. The
  update payload replaces the whole object rather than merging, because a
  partial update would leave a verifier unable to tell an unset member from a
  cleared one.

- **`CommunityProfileSnapshot.personhood`** — the same position, carried
  through a portable config export so a restore does not silently drop it.
  Governance state, unlike `registryStatus`, which stays absent from the
  snapshot because reachability belongs to a running maintainer rather than to
  exported state.

  Both components are `additionalProperties: false`, so one absent property put
  four tasks off their own schemas at once: `community/profile/show`,
  `community/profile/update`, `config/export` and `config/import`.

  Additive on the wire, so a patch bump — but see *"A patch bump promises wire
  compatibility, not source compatibility"* in CONTRIBUTING-SPECS.md: consumers
  building `CommunityProfile` or `CommunityProfileSnapshot` with struct-literal
  syntax will need one line each.

## [0.11.8] - 2026-08-24

### Added

- **`MemberResponse`** gains `personhood`, `personhoodAssertedAt`,
  `joinedViaInvitation`, `memberVmcId` and `memberVmcReceivedAt` — member state
  that consumers already receive and no schema described.

  `personhood` is the load-bearing one: a community that recognises members of
  another may gate on it, so a consumer that cannot read it cannot make that
  decision. `memberVmcId` distinguishes a membership the community has
  *asserted* from one the member has *acknowledged*, which is a real
  difference and was invisible.

  The component carries `additionalProperties: false`, so five absent
  properties put `members/list` and `members/update` off their own schemas at
  once.

- **`EndorsementType.createdByDid`** — who registered a type. An endorsement
  vocabulary is community-defined and shapes what every later endorsement can
  claim, so its author is audit-relevant in a way its creation time alone is
  not.

- **`vtc/members/update/0.1` payload `label`** — an operator-facing display
  name, editable unlike the DID it labels. `null` clears it.

- **`vtc/join-requests/decide/0.1` response `vmc` and `roleVec`** — the
  credentials an admitting decision issued, delivered inline. The alternative
  is a round trip whose only purpose is to collect something the community
  already held when it decided. Both absent on a refusal.

  Worth recording how this hid: the **refusing** arm always conformed, so a
  witness built from a rejection passed green over the whole divergence.

  All additive to schemas that previously forbade these members, so any
  document that validated before still validates.

## [0.11.7] - 2026-08-24

### Added

- **`vtc/relationships/publish` `pop`** (both `0.1` and `0.2`) — proof that the
  caller controls the key behind the VRC's `issuer`, when that is not the party
  issuing the document.

  An edge may be published under a relationship DID: an identifier scoped to
  one counterparty, which names no member. The community still needs to know a
  member published it, and the document's own `proof` says only that a member
  sent it, not that they control the credential's issuing key. Without this,
  any member ever handed a VRC could publish another party's edge.

  Bound to the enclosing document's `id` (SPEC §4.3) and the credential's
  digest, so an authorization cannot be replayed into another document or moved
  to another credential. Omitted when the VRC's `issuer` is the document's
  issuer, where the document's proof already establishes control.

  Consumers MUST NOT retain it after verifying. It answers one question at one
  moment, and storing it accumulates a durable link between the publishing
  member and a relationship DID that names nobody — the correlation publishing
  under a relationship DID exists to avoid.

  Additive: an optional member on a payload that previously forbade it.

## [0.11.6] - 2026-08-24

### Added

- **`JoinRequest.decision`** and **`JoinRequest.vpClaims`** — the applicant's
  half of a refusal, and the policy projection of their presentation.

  `decision` carries `code`, optional `reason` and `decidedAt`, and is distinct
  from `policyDecision`: that one records the community's internal verdict,
  while this one is written by **both** rejection paths — a policy auto-deny at
  submit and an admin's later refusal — so a client reads one shape instead of
  reconciling two.

- **`vtc/join-requests/status/0.1` response `code`, `reason`, `decidedAt`** —
  the same refusal detail on the poll an applicant actually makes, present only
  when `status` is `rejected`. A consumer MUST NOT emit them beside any other
  status: they assert that a decision was taken, and returning them with
  `pending` would tell an applicant they had been refused when they had not.

### Changed

- **`vtc/join-requests/status/0.1` payload `requestId` is no longer required.**
  An applicant whose first reply was lost never received an id, and a poll
  resolved from their own authenticated DID is the only form available to them
  — a refusal they cannot ask about is a refusal they cannot act on. A consumer
  given the id MUST prefer it over inferring the request from the caller.

  Relaxing a required member is additive for producers: every document that
  validated before still validates. A consumer that assumed `requestId` was
  always present now has a case to handle, which is the point.

## [0.11.5] - 2026-08-24

### Added

- **`CommunityProfile.relationshipIdentifierDefault`** and
  **`CommunityProfileSnapshot.relationshipIdentifierDefault`** — the identifier
  form a community declares it expects members to issue relationship
  credentials under: `attributed` (the member's membership DID, so an edge
  names them) or `pairwise` (a relationship DID unique to each counterparty).

  A declaration rather than an enforcement. The member still chooses per
  relationship, and a community that wants to require one form does so in its
  own policy. It exists so a client can read the community's expectation
  *before* minting rather than discovering it from a rejection.

  Both components carry `additionalProperties: false`, so a single absent
  property put three tasks off their own schemas —
  `vtc/community/profile/update`, `vtc/config/export`, and `vtc/config/import`,
  the last because an import replays the document an export produced.

- **`vtc/community/profile/update/0.1` response `fieldsChanged`** — the members
  the patch actually changed, in their wire spelling; empty when the submitted
  values matched what was stored. Lets a caller tell a no-op from an applied
  change without diffing the returned profile against the one it sent.

  Consumers MUST reject a `relationshipIdentifierDefault` outside the two
  defined values rather than storing it: the value is published to clients, and
  one they cannot interpret is worse than no declaration at all.

  Additive to schemas that previously forbade these members, so any document
  that validated before still validates.

## [0.11.4] - 2026-08-23

### Added

- **`vtc/members/removal-notice/0.1`** — a community tells a member it removed
  them: on whose authority, when, and why.

  Removal is the most consequential thing a community does to a member and, so
  far, the one it delivered with the least information — none. The only signal a
  removed member could observe was a side effect, the revocation bit on their
  membership credential flipping, from which they had to infer their own removal
  and could learn nothing about the reason.

  Deliberately not a receipt. `vtc/members/self-remove-receipt` answers a request
  the member made and is correlated to it; this answers nothing, because the
  member did not ask, is not waiting, and may well be offline. That asymmetry is
  what drives the rest of the design: `decidedBy` and `reason` are carried here
  and not there, because a departing member already knows why they left.

  `proof` is REQUIRED rather than RECOMMENDED. This is the one member-facing
  message whose value depends on being shown to somebody else — an appeal, a
  dispute, another community assessing a rejected applicant. Authenticated
  transport establishes the sender to the recipient and stops there, so an
  unsigned notice would evidence nothing once forwarded.

  The spec also states a delivery property rather than a transport: the act the
  notice reports is the act that ends the member's ability to ask about it, so a
  producer cannot rely on the member being reachable at that moment and cannot
  offer a poll as the fallback — the endpoint that would answer is the one that
  now refuses them.

## [0.11.3] - 2026-08-22

### Changed

- **`vta/app-state/*` — `vta/app-state:permissionDenied` is withdrawn.** It was
  declared as an *extended* code for a failure the framework already names as a
  standard one (SPEC §8.3), so a consumer switching on `permissionDenied` would
  not have matched it, and its `Extended` classification maps to `taskFailed` —
  a strictly worse signal than the standard code it duplicated. A conforming VTA
  answers an unauthorized caller with the standard code.

- **`vta/app-state:contextNotFound` is restated as OPTIONAL.** It reads as a
  diagnostic available to a maintainer whose authorization model can tell "no
  such context" from "not permitted to reach it". Where the ACL enumerates the
  contexts a caller may act in — the shape the reference VTA uses — both
  conditions are the same answer, and the code is never emitted. The prior
  wording made it a MUST that such a maintainer could not satisfy.

  No payload, response, or generated type changes; the affected specs' declared
  `errorCodes` and their Conformance prose do.

## [0.11.2] - 2026-08-22

### Added

- **`vta/app-state/{get,put,list,delete,get-many,put-many}/1.0`** — a third
  store on the VTA, beside the secrets vault and the credential vault, for
  versioned, namespaced, per-context JSON that an application owns and the VTA
  does not interpret.

  Records are addressed by `(contextId, namespace, key)`. The namespace scopes
  one application so several tools can share a context without colliding, and is
  the seam a per-namespace grant would later use — which is why it is part of
  the address rather than a prefix convention on the key.

  Three properties are what make this a store rather than a field on an existing
  one. `version` is a monotonic counter maintained **per `(contextId,
  namespace)`**, not per record, so one number is simultaneously the
  optimistic-concurrency token `expectedVersion` compares against and the
  watermark `sinceVersion` compares against; a per-record counter could serve the
  first but not the second. A failed precondition returns the maintainer's
  *current version and value* with the rejection, because a bare rejection
  obliges a re-read and the re-read races the next write. And `delete` leaves a
  versioned tombstone, without which a consumer pulling from a watermark never
  learns of a deletion and resurrects the record on its next rebuild.

  Agent memory is deliberately untouched: `MemoryItem` has no version to hang a
  precondition on and its `list` returns the whole context, but the settling
  argument is that "forget everything" must stay a safe thing to ask an agent,
  which it cannot be if account state lives there.

  Blobs are deliberately out of scope in 1.0 — adding a `blobRef` later is
  additive, and nothing in the first consumer's requirements needs one.

## [0.11.1] - 2026-08-20

### Added

- **`vta/webvh/servers/retire-orphan/0.1`** — the remedy for the one divergence
  `vta/webvh/servers/reconcile` can name but nothing could repair.

  An orphan is a slot a hosting server serves for an agent that has no record of
  it. No ordinary delete reaches one: every delete addresses a DID via its local
  record, to find the server and the signing keys, so the lookup fails before a
  request leaves the agent. Nor can the producer remove it directly — the agent
  holds the server credentials, not the producer. The slot is visible to both
  parties and removable by neither.

  The task's safety rests on the agent re-deriving orphanhood itself rather than
  accepting the producer's word: a live DID has a record, and the record makes
  the refusal automatic. The spec also forbids performing it on a sweep, because
  the signal a sweeper would act on is an *absence*, and absences are produced by
  bugs as readily as by orphaning.

## [0.9.0] - 2026-08-16

### Changed — BREAKING

- **`consume_inbound` now performs SPEC.md §7.2 item 2, and takes a
  `PayloadPolicy` to say how.** The function gains a third positional argument
  (`PayloadPolicy::Validate(&validator)` or
  `PayloadPolicy::<NoValidator>::AcceptUnvalidated`) and a seventh generic
  parameter. Every call site must be updated; `cargo check` finds them all.

  The policy is required rather than defaulted because the honest answer to
  "does this consumer validate?" was previously "no, and nothing said so". The
  crate deliberately does not bundle a JSON Schema engine — the same reason it
  bundles no cryptosuite — so the validator is yours to supply via the new
  `PayloadValidator` trait. `crate::validate::against_schema` (feature
  `validate`) is a ready-made implementation.

  **What this actually buys you in Rust is narrower than it sounds, and the
  docs now say so.** Deserializing into the generated types already enforces
  required members, member types, `additionalProperties: false`, and the
  `pattern` / `minLength` constraints typify emits as validating newtypes. The
  residue is what a Rust type cannot express — `minProperties`, `minItems` on
  an optional array, conditional subschemas. That residue is what `Validate`
  catches. (The TypeScript binding was in a far worse position: its types erase
  at runtime, so nothing was enforced at all. Both libraries now take the
  policy as a required argument so the two reach the same verdict.)

- **`ValidatedPayload::SCHEMA_JSON` is gone.** The schema moved to
  `Payload::PAYLOAD_SCHEMA: Option<&'static str>`, emitted unconditionally
  rather than behind the `validate` feature — it is a `&'static str` and costs
  no dependency, and gating it was what left the schema unreachable to anyone
  who had not already opted in. `ValidatedPayload` is now a blanket impl over
  every `Payload`, replacing ~300 generated impl blocks; `validate_value` is
  unchanged and still feature-gated.

- **`schema_index::schema_for` is no longer behind the `validate` feature**,
  for the same reason.

### Added

- **`Payload::PAYLOAD_SCHEMA`** on both the request and the **response**
  variant. Response payloads previously had no schema at all — the defect in
  #230 was reported against a response.
- **`PayloadValidator`**, **`PayloadPolicy`**, **`NoValidator`** (pins the type
  parameter on the `AcceptUnvalidated` path, which carries no validator to
  infer from).

## [0.8.2] - 2026-08-16

### Changed

- **`witness/session/submit` — the `vwc` member's pairing rule is stated in
  full.** A Verifiable Witness Credential's `taskContext` names the session
  document by `id`; its `taskDigestMultibase` must now also be that document's
  *task digest* (SPEC.md §4.9.3 — JCS over the document with its top-level
  `proof` removed, multihash, multibase). The `id` locates, the digest binds.
  Descriptive text only: `vwc` is opaque to this crate, the credential schema
  belongs to DTG Core Credentials, and no generated type, constant, or wire
  shape changes. Consumers relying on `SCHEMA_JSON` byte-equality across
  versions will see the embedded description strings differ.

## [0.8.1] - 2026-08-16

### Added

- **`trust-task-ok/0.1`** payload types, generated from the new registry entry —
  the courtesy acknowledgement reserved at SPEC §8.6 since framework 0.1.

  Additive in both libraries: a new module and its exports, no change to any
  hand-written runtime type. Adding it required no new error code and widened no
  enum, which is why this is a patch rather than the workspace event 0.8.0 was.

  Note for consumers: the acknowledgement is deliberately weak. SPEC §8.6 makes
  it normative that a producer MUST NOT rely on receiving one and that the
  absence of one carries no information, so nothing in this crate treats it as a
  signal — it is parsed and surfaced, never awaited.

## [0.8.0] - 2026-08-16

### Added

- **`StandardCode::Cancelled`** — the `cancelled` code introduced by framework
  0.4 (SPEC §8.3), for a consumer that stops a task on its own initiative.
  Deliberately distinct from a producer-requested cancellation, which is
  answered by a response to the `trust-task-control` document: without the
  distinction no party, and no auditor reading the retained documents, could
  tell a withdrawal from a refusal.

- **`trust-task-control/0.1`** payload types, generated from the new registry
  entry — the task-control request of SPEC §12 (`cancel`, `suspend`, `resume`).

### Changed

- **BREAKING for TypeScript consumers, additive for Rust.** Adding a standard
  code widens `@openvtc/trust-tasks`'s `StandardCode` string-literal union,
  which breaks exhaustive `switch` statements there. On the Rust side
  `StandardCode` has been `#[non_exhaustive]` since 0.7.0, so the same addition
  is additive and `trust-tasks-https` compiled unchanged. Both libraries move
  to 0.8.0 together to keep their versions in step, per CLAUDE.md.

- **The SDK now emits `trust-task-error/0.5`** (was `0.4`). `cancelled` is
  absent from `0.4`'s code enum and does not match its extended-code pattern,
  so a document carrying it would not validate as `0.4`. Per SPEC §5.2
  forward-minor compatibility a `0.4` consumer SHOULD accept it.

- **`trust-tasks-https` maps `cancelled` to HTTP 422** — the same bucket as
  `taskFailed`, because a deliberate stop is neither a server fault nor a
  malformed request.

## [0.7.0] - 2026-08-15

### Added

- **`StandardCode::IdConflict`** — the `idConflict` error code introduced by
  framework 0.4 (SPEC §8.3). A consumer emits it for a document whose `id`
  matches one it has already accepted but whose content differs: the case SPEC
  §7.2 item 11 requires be distinguished from a retry, which is absorbed
  silently.

### Changed

- **BREAKING. `StandardCode` is now `#[non_exhaustive]`.** Downstream `match`
  expressions over it must add a wildcard arm. This is the last time adding a
  standard error code will be a breaking change: the framework introduces one
  from time to time, and without the attribute every such addition forces a
  major bump on every consumer. Taking the break once buys minor bumps
  thereafter.

- **BREAKING. The SDK now emits `trust-task-error/0.4`** (was `0.3`). It has to:
  `idConflict` is absent from `0.3`'s code enum and does not match its
  extended-code pattern, so a document carrying it would not validate as `0.3`.
  Per SPEC §5.2 forward-minor compatibility a `0.3` consumer SHOULD accept it.

- **`trust-tasks-https` maps `idConflict` to HTTP 409**, and its status table
  now carries a wildcard arm mapping any future unmapped code to 500 — a
  server-side failure to keep up with the framework is not a client error.

## [0.6.5] - 2026-08-15

### Added

- **`vault/_shared/0.3`, `vault/{get,list,upsert}/0.3` and
  `provision/integration/0.3`** — the remaining bare-hex digests converged onto
  the framework's `DigestMultibase`, finishing the sweep #214 began.

  `VaultEntry.attachments[].sha256` becomes `digestMultibase`, and
  `provision/integration`'s `summary.digest` likewise. Both were lowercase-hex
  SHA-256, which hard-codes one algorithm into the wire contract and names no
  base encoding. Unlike the credential digests in #214 these are taken over
  opaque **bytes**, so no canonicalization defect was being fixed — the encoding
  argument alone.

  The vault change arrives through the shared component, so the three specs
  exposing `VaultEntry` re-pin to `vault/_shared/0.3` and bump with it (SPEC
  §5.4's coupling rule). `vault/proxy-login` and `vault/release` stay on `0.2`:
  they reference only `SiteTarget` and `SecretKind` and never expose an
  attachment digest, which §5.4 explicitly permits.

  Additive to the library — `0.1` and `0.2` of each remain published and
  generated.

## [0.6.4] - 2026-08-15

### Fixed

- **`Payload` aliased the wrong type in the TypeScript bindings** for 14
  published specifications. `scripts/build-ts-bindings.mjs` identified a
  schema's root type by taking the *first* `export interface|type` in the
  compiled output; where a `$ref`'d shared definition hoisted ahead of the root,
  it won — so `chat/message/0.1`, `trust-ceremony-receipt/0.1`, `audit/*`,
  `task-consent/*`, `consent/request/1.0`, `vrc/relationships/issue/0.1` and
  `witness/session/submit/0.1` each exported `type Payload = DigestMultibase`,
  a `string`, in place of their request payload interface.

  The root is now identified by the name the compiler derives from the schema's
  own `title`, used only when that name is actually present in the output. A new
  build-time invariant rejects an object-rooted schema whose root resolved to a
  bare alias, which is the shape this failure always takes.

  Not treated as a breaking change: the old alias was unusable. Assigning a real
  payload to a `string` does not compile, so no correct consumer could have
  depended on it. Nothing in the Rust bindings was affected — `trust-tasks-rs`
  0.6.4 is unchanged in content and moves only to keep the two libraries in
  step, per the versioning convention in CLAUDE.md.

## [0.6.3] - 2026-08-15

### Added

- **`vtc/relationships/request/0.2`, `publish/0.2` and `list/0.2`** — the VRC
  digest converged onto the framework's `DigestMultibase`. `0.1` carried
  `vrcSha256`, a bare lowercase-hex SHA-256, which hard-codes one algorithm into
  the wire contract, names no base encoding, and — for a JSON credential — named
  no canonicalization, so two conforming implementations could compute different
  values for the same VRC. `0.2` carries `vrcDigestMultibase` over the RFC 8785
  canonicalization. Breaking on the wire, released as a `MINOR` increment under
  SPEC §5.2's `draft` allowance; `0.1` remains published and generated, so this
  is additive to the library. All three also move to `targetFrameworkVersion`
  0.4.

  The three move together because their digests are meant to be the same value:
  `request` reports it on issuance, `publish` on lodging, `list` per entry.

## [0.6.2] - 2026-08-15

### Added

- **The witnessed relationship exchange** — bindings for four new specs:
  `vrc/relationships/propose`, `vrc/relationships/issue`, `witness/session` and
  `witness/session/submit`. Two people establish a peer-to-peer relationship
  under pairwise DIDs and issue each other a Verifiable Relationship Credential;
  where they agree to it, each opens its own session with a witness, which
  attests the exchange in a Verifiable Witness Credential. Additive: four new
  payload modules and schema-index entries, nothing existing moves.

  `witness/session/submit` is the first specification whose `#response` is
  designed as retained third-party evidence — it declares `proof` REQUIRED on
  both variants, so `IS_PROOF_REQUIRED` is `true` on the response impl as well
  as the request.

## [0.6.1] - 2026-08-14

### Added

- **`vta/webvh/servers/reconcile/0.1`** — bindings for the new spec. A producer
  asks an agent to compare the DIDs a hosting server holds for it against the
  DIDs it has records for, and to report the divergences in both directions.
  Additive: new payload module and schema-index entry, nothing existing moves.

## [0.6.0] - 2026-08-10

### Changed

- **BREAKING (behavioural).** `DigestMultibase` now accepts only the two
  multibase headers W3C Controlled Identifiers 1.0 §2.4 normatively requires —
  `z` (base58btc) and `u` (base64url-no-pad) — and enforces each alphabet rather
  than assuming it. Values a consumer previously accepted (base32, base16,
  base64pad, and strings that were not valid base58 at all) now fail to parse.
  The wire format is unchanged for conforming values; this is the
  behaviour-against-unchanged-format case the versioning note at the top of this
  file describes.

  CID permits other headers but states interoperability "is not guaranteed"
  with them, and a registry whose purpose is interoperability should not mint
  digests a conforming verifier may be unable to read.

## [0.5.1] - 2026-08-09

### Added

- `trust_ceremony_receipt::v0_1` — bindings for the newly published
  `trust-ceremony-receipt/0.1` specification, the evidence artifact for a
  completed ceremony enactment. Purely additive: no existing type changed.

## [0.5.0] - 2026-08-09

### Added

- `Ceremony` and `CeremonyPrev`, and an optional `ceremony` field on
  `TrustTask<P>` — the framework 0.4 envelope member recording that a document
  is one step of a Trust Ceremony (SPEC.md §4.11). `TrustTask::respond_with` and
  the error-response builders carry it forward, so the response and any
  rejection stay inside the enactment the request belonged to (§7.1).

### Changed

- **BREAKING.** `TrustTask<P>` gains a field, so struct-literal construction and
  exhaustive destructuring of the envelope no longer compile. `TrustTask::new`
  is unaffected. Documents are unaffected on the wire: the member is optional,
  a document without it is fully conforming, and one carrying it round-tripped
  through `extra` before this release.

## [0.4.1] - 2026-08-09

### Added

- `trust_task_next_step::v0_1` — bindings for the newly published
  `trust-task-next-step/0.1` specification, the framework response type a
  consumer returns when a task was understood but is blocked, naming what it
  expects in order to proceed. Purely additive: no existing type changed.

## [0.4.0] - 2026-08-09

### Changed

- **BREAKING.** Digest-carrying payload members are now the generated
  `DigestMultibase` newtype rather than `String`. Affects `audit/list`,
  `audit/verify`, `chat/message`, `consent/request`, `policy/evaluate` and
  `task-consent/{request,decision,granted}`. Construct via `TryFrom<&str>` /
  `FromStr`, which validate the multibase-multihash form at the boundary; a
  previously-accepted raw hex digest now fails to parse.

  The registry converged these fields on a multibase-encoded multihash over the
  RFC 8785 (JCS) canonicalization — the W3C VCDM 2.0 `digestMultibase` encoding,
  matching what `vta/credentials/issue` already used and what `did:webvh` uses
  for its SCIDs. The algorithm now travels in the multihash instead of being
  fixed by the schema.

## [0.3.0] — 2026-08-08

The framework 0.3 release. Four spec changes land together, and the leading
component moves because the document envelope gained members — adding a public
field to `TrustTask` breaks struct-literal construction.

Bumped as one workspace event, as the versioning policy warns: `trust-tasks-rs`
0.3.0, `trust-tasks-https` / `-didcomm` / `-proof` / `-tsp` 0.3.0,
`trust-tasks-capability-client` 0.2.0 (its own line), and
`@openvtc/trust-tasks` 0.3.0.

### Migration

- **`TrustTask` gained `parent_thread_id`** and **`ErrorPayload` gained
  `in_response_to`.** Struct-literal construction of either needs the new field;
  `TrustTask::new` / `for_payload` and `ErrorPayload::new` are unaffected.
- **Error responses are now emitted as `trust-task-error/0.3`**, not `0.2`. The
  runtimes populate `inResponseTo`, and `0.2`'s payload schema is
  `additionalProperties: false`, so a document carrying it would not validate as
  `0.2`. Per SPEC §5.2 forward-minor compatibility a `0.2` consumer SHOULD
  accept a `0.3` document. Anything asserting the emitted Type URI needs
  updating.
- **`trust-tasks-didcomm` now sets the DIDComm `thid` from the document's
  `threadId`**, falling back to its `id`, where it previously always used `id`.
  A response now continues its request's DIDComm thread instead of starting a
  new one.
- `DidcommError` gained a `ThreadMismatch` variant.

### Added

- **`parentThreadId` (SPEC §4.9.2).** An optional envelope member naming the
  `threadId` of the exchange containing this one. Exchanges nest — a Trust Task
  run to complete a step of a broader interaction is still its own exchange —
  and nothing recorded that, so specifications were inventing per-family payload
  conventions for it. Carried onto responses and error responses, since the
  whole exchange shares one parent. One level of containment, matching DIDComm's
  single `pthid`.
- **Per-variant proof declarations (SPEC §7.3 item 8).** `proofRequirement` now
  accepts a `request` / `response` pair as well as a single value. A response
  retained as evidence can need a proof where the request that triggered it does
  not, and the reverse is equally common; one value forced the stricter onto
  both. The floor derived from the side-effect and exposure classes splits the
  same way — `destructive` and `actsAsSubject` constrain the request,
  `discloses: secret` the response.
- **`inResponseTo` on the error payload (SPEC §8.2).** An error response
  correlated back only by `threadId`, which means something to a party that saw
  the request and nothing to anyone else, so a retained error named neither the
  task it terminated nor the instance. Optional in `trust-task-error/0.3` so a
  `0.2` consumer's output stays valid; a future major will require it. The `id`
  is withheld under `identityMismatch`, per §8.1.
- **`bindings/didcomm/0.2`** maps `thid` / `pthid` onto `threadId` /
  `parentThreadId`. Agreement is required only when both values are explicitly
  present — the layers default into their own identifier spaces, so an
  unconditional rule would reject conforming exchanges. A disagreement is
  `malformedRequest`, not `identityMismatch`: no party's identity is contested.
- **Naming an exchange from outside the framework (SPEC §4.9.1).** A citation
  treating an exchange as evidence names the innermost exchange that attests the
  event, by the `id` of the document that initiated it.
- **The document envelope schema now exists and resolves.** SPEC §7.2 item 1
  required consumers to validate against a schema at
  `https://trusttasks.org/spec/trust-task/<M.m>` that had never been published,
  and no Type URI honoured content negotiation at all — so item 2 was equally
  unimplementable and §7.3 item 7.5 unmet by every specification. Both fixed;
  the envelope schemas are validated against all 402 example documents in the
  registry.

## [0.2.60] — 2026-08-08

### Changed
- **Versioning policy restated.** The header of this file claimed the crate
  version tracked the `SPEC.md` framework version. That stopped being true when
  the framework moved to 0.3 while the libraries stayed on 0.2.x, and it was
  never quite the right claim: a framework revision can change the
  spec-authoring contract without touching a generated type, and the library can
  break its own API with no framework change at all. One number cannot answer
  both questions.

  The libraries now version by semver over their **own** API, independently of
  the framework. Read a document's framework version from the specification's
  `targetFrameworkVersion` (SPEC §7.3 item 3).

  Two consequences worth stating plainly, both now documented in CLAUDE.md:
  behavioural breaks count even when the wire format is unchanged (a spec that
  starts declaring `proof` REQUIRED makes a consumer reject documents it used
  to accept — that is breaking), and bumping the leading component is a
  workspace event, because five binding crates pin `trust-tasks-rs = 0.2`
  and must be updated and released with it.

  Documentation only; no code change.

## [0.2.59] — 2026-08-08

### Fixed
- **The root import of `@openvtc/trust-tasks` was broken, and had been since
  0.2.55.** `dist/index.js` re-exported every module with an extensionless
  relative specifier. The package is ESM, and Node requires an explicit
  extension on a relative ESM specifier — it does not probe for `.js` the way
  CommonJS resolution did — so `import … from @openvtc/trust-tasks` failed
  with `ERR_MODULE_NOT_FOUND`.

  Every check stayed green throughout: TypeScript's `Bundler` moduleResolution
  accepts the extensionless form and emits it verbatim, so `tsc --noEmit` saw
  nothing wrong. It went unnoticed because the package was types only — an
  `import type` is erased before it reaches Node, and bundlers tolerate
  extensionless paths. The 0.2.58 runtime is what made it fatal: real code
  that could not be imported.

  Subpath imports (`@openvtc/trust-tasks/acl/grant/0.1/payload`) were always
  fine; only the barrel was affected.

  `npm run smoke` now imports the built `dist/` from Node and exercises the
  pipeline through it, and runs in CI. A type-level check cannot cover this.

## [0.2.58] — 2026-08-08

No changes to `trust-tasks-rs` itself; the version moves to stay in step with
`@openvtc/trust-tasks`, which gains the TypeScript consumer pipeline.

### Added (TypeScript side)
- **`consumeInbound` — the SPEC §7.2 pipeline for TypeScript.** The npm package
  was types-only, so a TypeScript implementation had to hand-roll all eight
  §7.2 rules: expiry, recipient enforcement, the §4.8.1 identity cross-check,
  proof handling, audience binding, and the §8.1 error-routing rule that
  suppresses a response under `identityMismatch` when the transport
  authenticated no sender. One of the two reference languages offered no path
  to a conforming consumer.

  `src/_runtime/` is hand-written and mirrors `consume.rs`, `document.rs`,
  `transport.rs` and `error.rs` check for check, with a test suite that mirrors
  the Rust one — the two implementations must reach the same verdict on the
  same document.

- **Per-specification policy on every generated module.** §7.2 items 5b, 7 and
  8 are declared per specification and cannot be derived from a document, so
  each module now exports `SPEC` (and `RESPONSE_SPEC` where a response is
  defined) carrying `isBearer` / `isProofRequired` / `isRecipientRequired`.
  These mirror the Rust `Payload` associated constants and are derived from the
  same front matter, including the response-variant party swap of §7.3 item 5.
  Verified across all 556 Rust impls: zero disagreements.

  Also exported: `extendedCode` / `familyCode` (the §8.5 minting helpers, with
  the same prefix checks as their Rust counterparts), `normalizeCode` for
  reading a 0.1 peer's snake_case error codes, and `respondWith` / `rejectWith`
  document builders.

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
