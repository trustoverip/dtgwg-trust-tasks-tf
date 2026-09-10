# Changelog — `@openvtc/trust-tasks`

All notable changes to the TypeScript bindings package.

This file starts at 0.15.0. Earlier releases are in the git history of
`trust-tasks-ts/`; `trust-tasks-rs/CHANGELOG.md` records the changes the two
libraries shipped together, which is most of them.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

## 0.18.4 — 2026-09-10


### Added

- **process-attestation**: Add Process Attestation Trust Task 0.1 (#430)

* feat(process-attestation): add Process Attestation Trust Task 0.1

- **rooms/epoch/commits**: A host relays commits, so a renewal is not O(n) deliveries (#432)

Every epoch change is a commit and every member must apply it or fall out of the
  group. Without somewhere to fetch one from, the only way it reaches a member is
  the owner sending it — O(n) deliveries per renewal, to parties the owner must
  have addresses for, all of which have to be online or have inboxes. A room whose
  members are people with browsers does not have that.

  `rooms/epoch/mint` gains an optional `commit`, published where it is produced:
  minting is the moment the committer holds it, and a separate publish task would
  be a second chance to forget. A room that advances without leaving the commit
  somewhere fetchable **forks** — every member who missed the delivery is left at
  an epoch the room has moved past.

  **A host may relay a commit although it may not relay a Welcome**, and the note
  says so rather than leaving it to be re-derived. A Welcome names the party
  joining, which on `private` is the thing the tier withholds. A commit is
  ciphertext plus a leaf index and names nobody: a host relaying them learns that
  the room moved, which it knew from `epoch`, and that some member is behind, which
  fetching anything already tells it. So it authorises exactly as a record read
  does — a `read` chain, and no host session on `private`.

  Two rules that exist because MLS is strict and the failures are silent:

  - **Ordered, with no gaps.** Commits apply in sequence; one applied out of order
    or over a gap is rejected by the group. A host missing one MUST return the run
    it holds UP TO the gap and stop. A short answer is recoverable; a set with a
    hole in it is a member stuck at an epoch with no explanation.
  - **`roomEpoch` is not `sinceEpoch` plus the count**, and a consumer MUST NOT
    compute it that way. They differ exactly when a commit is missing or a page
    ended early, and that difference is the useful part: a member who applies
    everything served and is still behind knows a delivery is missing rather than
    concluding their own state is broken.

  What it does not do, stated: it does not make a commit REACH an offline member,
  only make one fetchable — a member who never comes back never catches up, and a
  room that removes them is doing the right thing. And it is not a substitute for
  the epoch key chain: a commit moves a member forward, a rung lets them read
  backwards, and a member who applies every commit and fetches no rungs can write
  to the room and read nothing written before they arrived.

  422 specifications against 422 TypeScript and 417 Rust modules, all agreeing.

- **rooms/epoch/prune**: A room decides to stop being able to read its own past (#431)

The verb behind `rooms/create`'s `retentionPolicy`. A room that chose `chained`
  keeps its whole history readable; this is how such a room decides, later and
  deliberately, that some of it should stop being.

  **What is destroyed is the rungs, not the records**, and describing it as
  deleting records would describe something else. A pruned room still holds every
  record and a host still serves them. What is gone is the ability to *derive* the
  keys they were sealed under.

  That cuts both ways and both are easy to get wrong on screen. A member who
  already walked the chain keeps what they derived — a key someone has read is a
  key they have, and nothing here reaches into an agent. A member who joins
  afterwards can never read that span however much authority they are granted: the
  host serves them ciphertext and no key exists to open it. Closer to losing a key
  than to shredding a document.

  `admin`, not `curate` and not `write`. Pruning makes no statement about any
  record, so `curate` is the wrong shape as well as the wrong strength — and every
  member who can write can curate, which would put "end the room's readable
  history" within reach of every writer. It belongs beside `rooms/epoch/mint`,
  being the same class of act.

  `beforeEpoch` at or above the room's current epoch is REFUSED rather than
  performed. `beforeEpoch: currentEpoch` reads like "keep from here" and would drop
  every rung, leaving a member who restarts unable to derive anything below the
  epoch they are handed next — a plausible typo with a consequence nobody would
  choose.

  `earliestRung` reports **reach, not the request**, and a host MUST NOT echo
  `beforeEpoch` into it. A chain can already have a gap, and a prune below one
  changes nothing about how far back a member can actually walk; echoing the
  request would tell an operator they had achieved something they had not.
  `pruned: 0` is a success for the same reason.

  No soft delete, stated as a MUST: a host that kept the rungs would defeat the
  operation while reporting success, and a rung retained "just in case" is a rung
  that can be produced under compulsion — which is the state the owner was trying
  to leave.

  422 specifications against 422 TypeScript and 417 Rust modules, all agreeing.

- **rooms/owner/anchor**: A room writes its own state where a host cannot (#429)

Everything else in this family produces values a host asserts. An anchor is the
  one statement a host does not make, cannot forge, and cannot show two members two
  versions of — because witnesses co-sign the `did:webvh` log entry it rides, so it
  is singular. An anchor that did not ride one would be a value the host could
  equally have made up.

  Three attacks die together and none dies without it: a rolled-back room, whose
  every value is its own; a forked group, which the MLS epoch authenticator catches
  because every member derives it independently and no host can compute it; and
  equivocation about contents, where an anchored root is the copy the host did not
  choose — for **every member at once**, needing neither a gossip channel rooms
  deliberately lack nor durable state in a member's agent.

  **The owner holds one of the three values it publishes**, and getting that wrong
  is what an earlier design note did. `rooms/epoch/mint` answers `{roomId, epoch}` —
  no watermark, and no reason there should be one: minting acts on the epoch, while
  the watermark and the commitment are facts about the room's records, which live at
  the host. So an anchor is assembled from a read, and all three head values MUST
  come from ONE response: a root and a version from two reads can straddle a write,
  and the pair is then individually correct and jointly false.

  That the owner anchors a value the host gave it looks circular and is not. The
  owner does not vouch for the root — it did not compute the tree. The anchor makes
  the root singular and witnessed, which is Certificate Transparency's arrangement
  exactly: the operator's own tree head is what gets published, and gossip is what
  makes equivocation fatal.

  A recipient SHOULD reconcile first and MUST report the outcome, and **MUST NOT
  refuse to anchor on a failed reconciliation.** An owner withholding an anchor
  from a suspect room leaves it with no witnessed statement at all, which is the
  position a misbehaving host benefits from.

  `notWitnessed` is a refusal rather than a warning: an entry nobody co-signed
  looks like an anchor and carries none of the property, which is worse than its
  absence because a member checking it would believe they had checked something.

  The cost is stated where an operator will read it. `vta/webvh/dids/update/1.0`
  needs nothing added, but supplying a document ROTATES the DID's update key and
  refreshes its pre-rotation commitments — so cadence is an operational decision
  rather than only a freshness one.

  Two members ride along, because an anchor nobody reads is a log entry:

  - **`ReadVerification.anchor`** on `rooms/keys/{read,browse}`. The only one of
    those checks a FIRST-TIME reader can make — no history, no peer, no gossip —
    and the only place a **rollback** is visible: a host serving a state older than
    the room's own published statement is `behind`, and nothing else in this family
    catches that. `notChecked` is deliberately not a synonym for `none`: one says
    the room published nothing, the other says nobody looked.
  - **`anchorCadence`** on `rooms/create`, which makes silence legible. A room that
    says `renewal` and has not anchored in ten epochs is telling a member
    something, and a member who did not know what to expect could not have noticed.
    Deliberately not a duration: a room promising "daily" would make a claim its
    owner's availability cannot keep, and a member comparing against a clock would
    read an owner's holiday as a host's misbehaviour.

  Scaffolded with `npm run new-spec`. 422 specifications against 422 TypeScript and
  417 Rust modules, all agreeing; `cargo test --workspace` green.

## 0.18.3 — 2026-09-09


### Added

- **rooms/keys**: Read and browse — a member's agent fetches, checks, and opens (#426)

The last unbuilt piece of the rooms surface, and the missing half of verified
  reads. Both were the same piece, as `data-rooms-read-through.md` works out: the
  console cannot address a host (its carrier drops the recipient, deliberately),
  and a `dataCommitment` is inert without a party that keeps the last root. The
  member's own agent is the answer to both.

  `rooms/keys/read` is four acts that today belong to three parties — mint the
  presentation, ask the host, check what came back, open it. The fourth decides
  where the other three go: the epoch key never leaves the key holder, so a member
  doing the middle two themselves holds a half-verified record in between and
  still makes two more round trips.

  What the recipient checks, and each is stated because each has a way of being
  skipped:

  1. the reply is signed by the host that was ADDRESSED — the proof verifies and
     the proven signer binds to the `host` named, or it is a reply from somebody
     else;
  2. the trace reaches the commitment served BESIDE IT, with the leaf preimage
     reassembled from the response itself — never a root kept from an earlier read;
  3. the root against what the agent has seen at this `headVersion`, which is the
     comparison no other party on the member's side can make.

  **A verdict is never an error**, and `ReadVerification` says so in the shared
  schema rather than in two places. A member's own agent refusing a record because
  the HOST misbehaved punishes the member for somebody else's act, and locks them
  out of the room holding the records that would show what happened. The
  consequence belongs on the write path. `priorRoots` is REQUIRED so an agent that
  keeps no history has to answer `notChecked` rather than omit the question —
  which is not a synonym for `noneHeld`: one says nothing was found, the other
  says nothing was looked for.

  `rooms/keys/browse` carries the check that only a listing can do. A reader
  cannot recompute a root from a listing — a leaf commits to a whole record and a
  listing returns a projection — but it can COUNT, and a host that omits a record
  while committing to a tree holding it contradicts itself inside one exchange,
  with no second party and no anchor. The condition is narrow and stated: no
  prefix, no `sinceVersion`, and `complete: true`. `complete` exists because a
  page bound and a withheld record produce the same shorter array, and a consumer
  that could not tell them apart would either cry wolf on every paged listing or
  learn to ignore the one that mattered.

  Both specs say the part that decides whether any of this works: *serve reads,
  refuse writes* is a rule nobody would guess, so an adverse verdict MUST be
  surfaced in words naming what was observed rather than as a status colour. A
  member who dismisses an icon here is a member who later reads a refused write as
  their own agent malfunctioning, and a detection attributed to the wrong party is
  worse than no detection.

  Scaffolded with `npm run new-spec`. Both generators re-run: 421 specifications
  against 421 TypeScript and 416 Rust modules, all agreeing; `cargo test
  --workspace` green.

## 0.18.2 — 2026-09-09


### Specifications

- **rooms/records/list**: A root names no state, so give it a tree head (#422)

`dataCommitment` shipped as a bare root, and the section explaining what it is
  worth ended:

      A host that shows two members two different roots for the same room has
      been caught, and cannot claim a transient.

  That is false. A room moves — every put, curate and retraction changes the tree —
  so two roots taken at two moments differ for the most ordinary reason there is. A
  host shown to have served two different ones answers *there was a write between
  your reads*, and with a bare root nothing contradicts it. The comparison this
  member exists for could not be performed by anybody.

  The error came from taking half of the analogy the same paragraph draws.
  Certificate Transparency does not have this problem because **an STH is a root
  and a tree size**; this family shipped the root and dropped the rest.

  So two members travel with it, both from the snapshot the root was taken over:

  - `headVersion` — the room's highest assigned record version. A version advances
    on exactly the mutations that change the tree, so it names the *state* the root
    describes. Two roots at the same `headVersion` that differ is a host caught,
    with no write to attribute the difference to; two at different ones are two
    moments and a reader must draw nothing from them.
  - `recordCount` — how many records the room held, tombstones included. A reader
    cannot recompute the root from a listing (a leaf commits to a whole record and
    a listing returns a projection), but it can **count**: a host that omits a
    record from a complete unfiltered listing while committing to a tree holding it
    contradicts itself inside one response, with no second party and no anchor.

  Both OPTIONAL in exactly the sense the root is — a host maintaining no tree
  asserts none of it. A host serving `dataCommitment` SHOULD serve both, and a
  reader receiving a root without them MUST NOT compare it against another root.
  Such a root is still usable against a witnessed anchor, where the epoch pins the
  state, which is why it is not simply refused.

  A host can understate both together. That is the point rather than a hole: the
  omission stops being silence and becomes a specific claim about how many records
  the room holds and how far it has been written — which any other member's view,
  and any writer's signed put acknowledgement (vti #1334/#1335), contradicts
  directly. Making an omission attributable is the whole of what this machinery
  buys; it never claimed to make one impossible.

  Not a timestamp: a time is host-asserted and unverifiable, and two roots a second
  apart are not evidence of anything while two roots at one version are.

  `checkCommittedRecordMirror` — added last change — caught both new members on the
  first build and named the two ways to resolve it. That is the guard doing the job
  it was written for, on a question nobody would have got from reading two files.



  Amended after checking how a host would actually compute it: `headVersion` is
  the highest version among the records the root **covers**, derived from the same
  set, and NOT read from the room's own `next_version` counter.

  The two agree for any host that has never erased a record. They are not
  interchangeable, because a root and a counter are *two reads*, and two reads are
  not a snapshot. A write landing between them labels a root with a version from a
  different moment — so two honest members end up holding roots over different
  trees under one version, which reads as equivocation and is not. **A false
  accusation discredits the mechanism rather than the host**, which is the worst
  outcome available here, and it is the same reasoning that put the root and its
  trace in one scan.

  Ordering the two reads does not help, in either direction: whichever is taken
  first, a write in the gap admits a pair that is individually correct and jointly
  false. Only deriving both from one read closes it.

  Stated corollary: a host that **erases** a record — as distinct from retracting
  it, which leaves a tombstone in the tree — moves the root without necessarily
  moving this value. `vti_rooms::storage::purge_record` exists and today has no
  caller outside its own tests, so nothing exposes it; a family that exposes one
  owes this definition another look.

## 0.18.1 — 2026-09-09


### Specifications

- **rooms/records/get**: Traces, and the leaf preimage they need (#419)

A commitment lets a reader catch a host that equivocates. A trace is what proves
  a *particular* record sits under a particular root. `trace` is that member, on
  the single-record read.

  Adding it exposed a defect in the commitment already shipped. `DataCommitment`
  described its leaf as "`RecordMetadata` plus its stored content" — exact-sounding
  and not reproducible. `RecordMetadata` is the projection a listing returns: it
  renders `updatedAt` as a timestamp, lifts `epoch` to the top level, carries
  `title` and `description` pulled out of an `open` room's body, and has no
  `pinned` at all. A host hashing what it stores and a reader hashing that
  projection reach different roots, and nothing said which one counted. That was
  survivable while the only use was comparing two roots from the same
  implementation; a trace has to be computable by someone else.

  So the preimage is pinned exactly, as `CommittedRecord`, and it is defined to
  *be* this response payload with `dataCommitment`, `trace` and `ext` removed.
  Reassembly is a deletion rather than a reconstruction, and no copy of the
  ciphertext is carried twice — to be paid for on every read, and to disagree with
  itself on the read where it mattered.

  `status`, `updatedAt`, `pinned` and `author` are added to the response because
  they are committed, and a response omitting them was one no reader could hash.
  They are OPTIONAL so this stays non-breaking for a published 0.1;
  `dependentRequired` makes `status`, `updatedAt` and `dataCommitment` mandatory
  wherever `trace` is present, which is the only place their absence can do harm.
  `author` is a new disclosure on this task — `rooms/records/list` already returns
  it on `open` and `attributed`, so the family withheld nothing, but a
  single-record read now names the writer where before it did not.

  Two things stated rather than left to be discovered:

  - A trace binds a record to a root and says nothing about whether that root is
    the room's. A reader that verifies one against a root the same host handed it
    a moment earlier has checked the host's arithmetic and nothing more.
  - A listing cannot be reconciled against the root at all, not even a complete
    one: a leaf commits to a whole record and a listing returns a projection
    without the body. Reconciliation means reading the records.

  Host-visible break, though not a library one. A host that implemented the old
  wording literally computes a different root than this requires; the generated
  libraries gain only additive surface — `RecordTrace`, and optional members on a
  `#[non_exhaustive]` struct — and reject nothing they used to accept.

  `checkCommittedRecordMirror()` in the registry build guards the one hand-kept
  correspondence this introduces: CommittedRecord's members against the get
  response's, by name. Verified non-vacuous — removing `status` from the response
  fails the build, naming the member and both ways to fix it.

## 0.18.0 — 2026-09-09


### Fixed

- **rooms/owner/issue-authority**: `validUntil` is REQUIRED; 0.2 requires it (#418)

`0.1` typed `validUntil` as optional, so a caller could ask this task for a
  grant that never lapses. DTG Core Credentials requires the property on every
  authority credential, and states the reason: "Unlike the base structure,
  `validUntil` is REQUIRED for a VAC … nothing about the subject's current
  standing is consulted when a VAC is verified, so authority that does not expire
  is authority nobody can withdraw by waiting."

  So `0.1` described a request no conforming implementation could honour. An
  issuer refuses to build the credential; a verifier refuses a chain link that
  carries no expiry. A grant minted without one would have failed at its first use
  at a host, for a reason the holder could not act on.

  This bites hardest here because the task mints a chain ROOT. A root is the grant
  nothing else can withdraw — there is no status list consulted at verification
  and no membership check behind it — so for a root, expiry is not one of several
  ways a grant ends. It is the only one that works without the room reissuing or
  revoking.

  ## No default, deliberately

  A consumer MUST NOT substitute one for an absent value. How long a room's
  authority should last is the owner's judgement about their own room, and a
  recipient that picked a lifetime would be making that judgement silently, in the
  one place the owner cannot see it. Refusing puts the choice back where it
  belongs. A caller with no view should say so with a short value rather than by
  omission — and a shorter value is always safe, since attenuation may only
  narrow, so the root's lifetime is the ceiling on everything derived from it.

  ## Also here

  `0.1` is retired, superseded by `0.2`.

  Found while bumping `verifiable-trust-infrastructure` to dtg-credentials 0.9,
  where `new_vac` stopped accepting an `Option` — the library making the rule
  structural is what surfaced a registry spec that had been out of step since
  Working Draft 02. The generated `0.2` binding types `valid_until` as a plain
  `DateTime`, so the consumer compiles against 0.9 unchanged once it re-pins.

  Sibling specs were checked and are correct as they stand: `issue-membership`
  mints a VMC and `invite` mints a VIC, where the base structure applies and an
  optional `validUntil` is right. This is the only rooms spec that mints a VAC.

  Bindings regenerated; library versions untouched per RELEASING.md.

- **rooms/keys/present**: `audience` and `nonce` could never work; 0.2 removes both (#415)

## 0.17.11 — 2026-09-09


### Specifications

- **rooms/records**: A data commitment on the read responses (#411)

Second of the verified-reads pieces (#1343's order: the store — merged as
  vti#1346 — then the commitment on read responses, then traces).

  A listing has no completeness property of its own. Records are signed and
  room-bound, so a host cannot forge, alter or relocate one; omitting one from
  a response costs nothing and looks like a room that never held it.
  `dataCommitment` turns that silence into something checkable.

  **The construction is normative**, in the shared schema, because two hosts
  computing different roots over the same room make every comparison
  meaningless. Sorted by key in unsigned byte order; leaf
  `SHA-256(0x00 || JCS(record))`; node `SHA-256(0x01 || left || right)`; an
  odd node promoted and never duplicated — duplicating makes a tree of n
  leaves collide with one of n+1 whose last is repeated; an empty room commits
  to `SHA-256("")` rather than zeroes, since zeroes are what an uninitialised
  buffer looks like.

  The leaf covers the whole record, not its body: a host that could flip
  `status` to retracted, move `pinned`, or rewrite `author` on an attributed
  room rewrites what the room means without touching a byte of ciphertext. The
  plaintext is never involved — a sealed-tier host commits to the ciphertext
  it actually stores.

  Three things the prose says out loud because each has a plausible wrong
  reading. **A commitment read once proves nothing** — it is the host's own
  assertion, and becomes evidence only compared against a root the host did
  not choose for the reader: one given another member, one given earlier, or a
  witnessed anchor. **It is OPTIONAL**, because a host maintaining no tree
  cannot honestly assert a root and its absence is itself informative; a
  consumer MUST NOT read absence as failure or presence as proof. **It commits
  to the whole room, never the page** — a page-scoped root is satisfied by
  construction and could never fail, which would let a member feel checked
  while checking nothing.

  It is evidence at all only because responses are signed (SPEC §7.3 item 7,
  implemented in vti#1334/#1335). An unsigned root is a number from nobody in
  particular, and a host could deny having said it.

## 0.17.10 — 2026-09-09


### Added

- **persona/correlation**: Say whether a link crosses a part of the holder's life (#408)

Additive members on `persona/correlation/analyze/1.0`: `crossesFacets` and
  `facetIds` on a finding, `facetId` on each `sharedWith` location.

  **Severity is how linkable. Facets are whether the holder minds.** Two axes,
  kept apart, because collapsing them loses whichever one is inconvenient.

  `severity` is a fact about the value and the proof beneath it — a credential
  presented whole links every verifier that sees it, and that is true whatever
  the holder intended. `crossesFacets` is a fact about the holder's own
  arrangement: a work email in every work profile is linkage they built on
  purpose, and a consumer that alarms on it teaches them to dismiss alarms,
  which costs them the one that matters.

  So the spec is explicit that a maintainer **MUST NOT** reduce severity, or
  omit a finding, because a linkage stays inside one facet. The linkage is
  real either way — two verifiers who see both profiles link the holder
  regardless of which part of their own life they filed each under — and a
  severity that softened on intent would report a false all-clear about
  something a counterparty can still do.

  **Absent is unknown, not false.** A maintainer without facets omits all
  three rather than emitting `crossesFacets: false`, because `false` asserts
  the holder keeps these identities in one part of their life and an
  implementation with no facets has made no such finding.

  **An unarranged profile is not a second facet.** `crossesFacets` is true
  only where two or more DISTINCT facets appear; otherwise every holder who
  has arranged one part of their life and not the rest would see a crossing on
  everything they own.

  **No facet NAME crosses.** The caller holds the facet records and can
  resolve an identifier; the name is the member of a facet worth protecting,
  and this response already carries the linkage map.

  Bindings regenerated per CONTRIBUTING-SPECS.md; no version bump or CHANGELOG
  entry (release-plz owns both). Rust workspace builds --all-features;
  trust-tasks-ts 79/79.

## 0.17.9 — 2026-09-09


### Added

- **persona/facet**: A named part of a life, and what belongs to it (#405)

* feat(persona): facet — a named part of a life, and what belongs to it

  A holder who uses this model for a while does not end up with three
  profiles. They end up with twenty — one per site, one per counterparty, one
  made once and no longer explicable — and a flat list of twenty is a list
  nobody reads. A facet is the arrangement over them: Work, Home, Play, and
  whatever else a particular life has in it.

  Three tasks: `put`, `list`, `delete`, all agent-scoped like the pool and the
  profiles they group.

  **Why a Trust Task and not a client preference.** A grouping kept in a
  browser's local storage is a grouping the holder's phone does not have. It
  also tells the maintainer something it cannot otherwise know — that two
  profiles are, to the holder, parts of one life — which turns a linkage
  report from a list of every shared value into a list of the ones crossing a
  boundary the holder actually drew.

  **Membership lives on the facet, not on the attribute.** The obvious
  alternative is a `facetId` member on `persona/attribute/put`, and it is the
  wrong shape for a mechanical reason: that task REPLACES the attribute, and a
  well-behaved consumer does not hold the values it would have to resend —
  `attribute/list` withholds the plaintext of anything resolving to
  `sensitivity: high` unless asked for by name. So such a consumer either
  requests every sensitive value it holds in order to perform an arrangement
  that has nothing to do with values, or sends a put without one and silently
  destroys them. One record on the facet has neither problem.

  **A profile belongs to at most one facet; an attribute may belong to
  several.** The first because a facet is where a consumer reads a profile's
  colour from and two answers is no answer — refused with
  `faceAlreadyPlaced`, whose details name the facet already holding it so a
  producer can offer to move rather than guess. The second because a mobile
  number is genuinely part of both a working life and a home one.

  **`colour` carries a NAME, never a literal.** A hex value cannot be legible
  in a terminal, a light theme and a dark one at once, so a stored one is
  wrong somewhere and the holder cannot know where; and a consumer that
  reserves colours to mean something must be able to keep a decorative choice
  out of that channel, which it can do with a closed set and cannot do with an
  arbitrary value. The eight are distinguishable and none is named for
  success, warning or danger.

  **A facet is an arrangement, not a container.** Delete removes the word and
  the statement about what belonged to it; every profile and attribute
  survives. There is deliberately no `cascade` member — an arrangement that
  could take its members with it is a folder, and a holder who reads it as one
  is right to fear it. `releasedFaces` reports what now belongs nowhere.

  **It never crosses into a trust context.** A facet is the holder's own
  statement that two identities belong to the same part of one life, which is
  exactly the join multiple personas exist to deny a verifier. MUST NOT appear
  in a materialised projection, a disclosure, or any document a verifier
  receives.

  `FacetColour` is added to the persona `_shared` definitions so `put` and
  `list` resolve the same enum; cross-spec `$ref`s outside `_shared` do not
  resolve in this build.

## 0.17.8 — 2026-09-08


### Added

- **rooms**: Let a member's own agent reach a room's host for them (#402)

Two tasks, one reason: **the surfaces people actually use cannot reach a
  room's host.** A browser extension or a phone holds a channel to its own
  agent and to no third party, so every host-served room verb is unreachable
  from them — not refused, not degraded, simply unaddressable.

  `rooms/keys/backfill` collapses present → `rooms/epoch/chain` → store into
  one act performed by the party that can perform all three. The member's key
  holder already has the credentials, already mints the presentation, and
  already holds the group state the rungs extend; the only hop it was missing
  was the one the member could not make either. It presents `read` and no
  more — reading a room and reading the parts written earlier are the same
  act — bound to the named host as audience, which is what makes a
  caller-supplied host safe: the presentation is useless anywhere else, and
  the caller is already a member.

  The response carries three numbers because there are three ways it ends,
  and they are not restatements of each other. Nothing served means the
  history begins there or was severed before this member joined. Rungs served
  with the reach unmoved means they sit below a gap — early rather than
  wrong, and a consumer MUST NOT discard them. A consumer reporting only what
  arrived would say "12 rungs stored" over a room that still cannot open a
  word of its history.

  `rooms/owner/register` is `rooms/create` performed by the agent, and it
  closes a half-finished state that is worse than a failure: minting the
  room's identity is the agent's own work and succeeds, so a surface that
  cannot then register it leaves an owner holding a DID and a signing key for
  a room that exists nowhere — real, unrecoverable, and now theirs to keep
  safe by hand. The order is unchanged and still forced: identity first, then
  a host is told about a room that already exists, because a host that named
  the room would be a host the room could not leave.

  Both echo what the recipient actually reached rather than what the caller
  asked for, and both store nothing: a key holder holds custody, not a
  hosting register, and a remembered host is a value that goes stale the
  moment a portable room moves.

## 0.17.7 — 2026-09-08


### Added

- **rooms/owner**: Mint the credentials that make a room joinable (#401)

The piece the design note calls "Library only": nothing served "issue this
  member a VMC and a VAC", so an owner minted them by hand with dtg-credentials
  and no surface could create a room and add people to it.

  Three tasks, each signing AS the room with a key the owner's key holder holds:

    invite            a VIC — joining is consent, and this is the artefact
    issue-membership  the VMC a member presents on every operation
    issue-authority   a VAC chain root: read / write / curate / admin

  ## Two design calls worth recording

  **The key is named, not looked up.** Nothing maps a DID to the key it was minted
  with, and inventing that mapping would add a lifecycle to get wrong — one that
  can go stale, be rebuilt incorrectly, or disagree with the DID document after a
  rotation. A wrong key produces a credential that fails to verify against the
  room's DID document, which is loud and at first use. It also matches the grain:
  did:webvh creation already takes an explicit signing_key_id.

  **Three tasks, not four.** A DTG membership credential is half of a pair, and the
  member issues the other half. Room authorization verifies only the grant half —
  `vti_rooms_dtg` compares the presented membership's subject against the chain
  root and never looks for an acknowledgement — so a room is joinable without one.
  Stated in issue-membership rather than left for someone to discover.

  ## Authorization is the key gate and nothing else

  These tasks do not check that the caller is the room's owner. "Owner" is a fact
  about the room's DID controller, and a key holder is not a DID resolver.
  Controlling the signing key and controlling the DID are the same thing while the
  key is the one the document names — and when they have come apart, the credential
  minted here simply fails to verify. The VTA already gates key naming by context,
  with a resource-bound policy limit that binds super-admins too.

  ## The recipient keeps nothing

  No roster, no issuance log. A room's membership and authority live in the
  credentials themselves, so the owner is the only party who knows what they
  issued — which is invariant I1 working, not a gap.

  Seventeen negative-space fixtures. The sharpest is issue-membership refusing a
  `role` in place of a subject: a room that admitted a role would be a room whose
  membership its host could compute.

- **rooms/keys**: Seal and list, so a client can write a sealed room and find one (#399)

Two gaps that between them make a room UI impossible, both in the family that
  exists so keys never leave the key holder.

  ## seal — the mirror of open

  A client can read a sealed room and cannot write to one. Sealing needs the
  epoch's storage key, the key never leaves the key holder by design, and nothing
  asked the key holder to seal on a caller's behalf — so every writing surface for
  an `attributed` or `private` room had to *be* the key holder. `pnm rooms put`
  says so in its own help: "open rooms only".

  Plaintext in, ciphertext out, key stays put. It does not write: the caller takes
  the result to a host and presents its own authority there. Sealing and being
  allowed to store are different questions asked of different parties — the key
  holder knows the key and nothing about the room's ACL; the host knows the
  credentials and cannot read a byte.

  `version` is an input because the associated data commits to it. The host
  assigns versions, so a writer does not know it at sealing time; this takes the
  version the writer INTENDS, and a caller that lets the host assign a different
  one finds the record does not open. That is the correct failure — accepting
  whatever came back would mean the binding commits to nothing.

  ## list — where a roomId comes from

  Every other room task takes a `roomId` the caller already knows, and nothing
  said where. A surface could act in a room it was told about out of band and
  could not show a principal their own rooms.

  It reports two epochs per room, and neither alone is enough: `epoch` behind the
  room's own means a commit was not delivered; `earliestReadableEpoch` equal to it
  means the chain has not arrived. Different repairs, and without both a member
  reads "less than I expected" as loss rather than as delivery.

  **Key custody, not membership.** A principal may hold a good VMC for a room whose
  Welcome never came — absent here, correctly. And a key holder not yet told of a
  removal still opens what it already had, which is what removal has always meant.
  A consumer MUST NOT present this as authority to act.

  Nine negative-space fixtures. The sharpest is `list` refusing a `roomId` filter:
  that would turn it into "is my principal in THAT room", a different question
  with no authorization story of its own.



### Specifications

- **persona/_shared**: The agent serves this table now (#398)

§6 still asked "Should the agent serve this table? … worth doing when the first
  extension type ships, not before" — and `persona/claim-types/list/1.0` shipped
  in #390, with the agent serving it since VTI #1315. A reader arriving here today
  would conclude the task does not exist and compile in a copy, which is the thing
  it was built to stop.

  The answer moves to the top, where a reader looking for the table will be,
  rather than living only in a resolved-questions list: a client SHOULD read the
  maintainer's own copy. This document stays the normative account of what the
  table means; the task is how a client learns what one agent's table says,
  including extension types its build predates.

## 0.17.6 — 2026-09-08


### Added

- **auth/step-up/approve-response**: Add a recorded status for a bound approval (#391)
- **persona/claim-types/list**: Serve the claim-type registry (#390)

CLAIM-TYPES.md §6 deferred this until "the first extension type ships". The
  reasoning has not survived contact with the copies: both implementations
  vendored the table, between them they found four holes in it, and each fix
  became a pull request against two repositories that do not own the data. The
  cost being deferred is paid per change, not once.

  The response carries `unregistered` and `strictness` as REQUIRED, not just the
  entries. §4 rule 3 takes the longest registered prefix and the floor and keeps
  whichever is more protective — a client cannot compute that from entries alone,
  and one that hard-codes the floor or the ordering cannot be tightened by a
  maintainer that raises it.

  Exact tokens and family prefixes appear undistinguished, because marking them
  would invite a client to walk only one kind — the hole that let a gated family
  be escaped by inventing a member.

  Authorization is any authenticated caller: the response is a constant, and both
  of the usual answers refuse a caller that legitimately needs it.

## 0.17.5 — 2026-09-07


### Specifications

- **rooms**: An epoch key chain, so joining a room means being able to read it (#387)

A record is sealed under the storage key of the epoch current when it was
  written, and a group key schedule offers no way to derive an earlier epoch's key
  from a later one. That property is what makes removing a member mean something.
  It also means that, left alone, the first membership change makes every record
  already in the room unopenable by everyone — the writer included.

  A room is a library rather than a message stream. What was written is supposed
  to stay readable to whoever is in the room, and a new member joining an
  apparently empty room is the same defect seen from the other side.

  ## The chain

  `rooms/epoch/mint` gains an optional `link`: the outgoing epoch's storage key
  sealed under the incoming one. Minting is the only moment at which one party
  holds both, so it is the only moment the bridge can be made.

  `rooms/epoch/chain` is new — a member fetches the accumulated rungs from the
  host and walks them backwards. Backwards only: a member holding an earlier key
  still derives nothing later, so removal stays exactly as forward-only as
  `rooms/epoch/mint` already describes.

  Rungs are ciphertext under keys no host holds. A host stores them and learns the
  number of epochs a room has had, which the room's epoch number told it already
  — which is why this is a host-served read rather than something the room's owner
  must be online to answer.

  ## Why epoch/ and not keys/

  Every `rooms/keys/*` task terminates at a KeyHolder or an Oracle — a member's
  own agent, in the family that exists so keys never travel. Every host-served
  task is under `create`, `epoch`, `owner` or `records`. This one is
  Member → Host and is epoch bookkeeping, so it belongs beside `epoch/mint`.

  ## What it costs, and where that is chosen

  A chain means a member's current key reaches every retained epoch, so
  compromising one member's current key exposes the room's retained history rather
  than only what came after. That is the cost of a library and it is real. It is
  not chosen by this task: a room that made the other choice produces no rungs, and
  `chain` returns an empty array for it. `link` is optional for that reason as
  well as the mechanical one — it was added to an already-published version, where
  requiring it would break every conforming producer.

  ## Additive, and the struct literals

  `link` on `MintEpochPayload` and the new `EpochLink` shared type are additive
  on the wire, but generated Rust types are plain structs with public fields, so
  existing `MintEpochPayload { .. }` literals need the new member.

  Negative-space fixtures accompany both specs — the rooms family had none, and
  nothing else tests a schema's rejections.

## 0.17.4 — 2026-09-07


### Specifications

- **provision**: Separate where an admin lives from how far it reaches (#385)

`provision/integration` writes the minted admin's ACL entry naming the
  target context, and there is no way to ask for any other shape. That is
  right for every integration-class consumer — a mediator acts where it was
  provisioned — and it makes one consumer unrepresentable: an operator
  console, whose whole job is administering the maintainer, needs an entry
  with no context list at all.

  The obvious spelling, making `context` optional to mean "everywhere", is
  the wrong one. A console still has to keep its own configuration
  somewhere, and that somewhere is one ordinary context; dropping `context`
  would leave it with nowhere to put it, and would collapse "provision me
  everywhere" and "provision me wherever you like" into the same document.

  So `adminScope` is a second axis beside `context`, not a replacement for
  it: `context` says where the admin DID is minted and where its owner
  keeps its configuration, `adminScope` says whether the ACL entry names
  that context or nothing at all. `context` is resolved on every request
  either way.

  Two MUSTs come with it. `unrestricted` is refused with `forbidden` unless
  the relayer is itself a super-admin, because routing a grant through a
  provisioning maintainer does not launder authority the caller does not
  hold. And the outcome is echoed as `summary.adminScope` rather than
  inferred from the ask — a maintainer that predates the member ignores it
  and writes a context-scoped entry, which is otherwise indistinguishable
  from success. `summary.context` is echoed for the same reason: a producer
  that omitted `context` and let inference run currently has no way to learn
  where it landed except by guessing at the maintainer's layout, which is
  precisely the thing the inference rules exist because it cannot do.

- **persona**: The step-up approval is the maintainer's state, not the request's (#384)

#377 said `present` must refuse a `stepUp` claim when "no fresh approval bound
  to that previewId **accompanies the request**". The request has no member to put
  one in — `present` carries `contextId`, `previewId`, `challenge`, `mint` and
  `ext` — so the rule as written was unimplementable, and the implementation found
  it immediately.

  The fix is not a new member. **An approval a producer carried would be a bearer
  token**, and a bearer token authorising a disclosure is replayable by whoever
  holds it — the property the single-use `previewId` exists to deny. So the rule
  now says what it should have said: the maintainer holds the approval, bound to
  the preview, sharing that preview's lifetime. Consumed with it, expired with it,
  and unable to outlive the decision it belongs to.

  Also stated explicitly, because it is what makes the error retryable and was
  previously only implied by the error's own description: the refusal MUST NOT
  consume the preview.

- **persona**: Say what kind of prefix rule 3 means (#382)

The VTA implementation asked, and the answer was not written down: is the
  "longest registered prefix" matched over bytes or over dot segments?

  It matters because **the same task answers the same question differently a few
  lines away.** `attribute/list`'s `typePrefix` is explicitly a byte comparison
  over UTF-8 and says so — "a prefix that ends mid-segment is a byte comparison
  like any other" — so an implementer meeting that first would reasonably carry it
  into §4, and `paymentology.card` would join the `payment` family.

  Rule 3 can only tighten, so that reading yields spurious strictness rather than
  a leak: an unrelated token gated behind a step-up nobody asked for. Wrong, and
  worth stating, without pretending it was a hole.

  Also stated: a proper prefix, so a token is not its own prefix — rule 2 has
  already answered for an exact match, and a token compared against the floor
  would contradict "an exact entry is used as written".

  And a note for whoever extends the table: **"longest" is not currently
  observable.** No two nested families disagree on any axis, and the floor is
  already maximal on `sensitivity` and `mask`, so rule 3 can only move `release`
  today — a shortest-prefix implementation passes every outcome these entries can
  express. The rule is written for the table that adds `payment.crypto` with a
  treatment of its own, and an implementation should be tested on the mechanism
  rather than on a result the current entries make unobservable.

  Third round of gaps found by implementing this table rather than reading it.

## 0.17.3 — 2026-09-07


### Specifications

- **persona**: Say what a mask does at its two edges (#380)

Two clients implemented this table independently and both met the same two
  questions, neither of which `maskStyles` answered.

  **A value shorter than the tail a style keeps.** `last4` of a four-character
  value is the value — a four-digit card number printed whole is not a masked card
  number. Both clients masked in full. Now stated.

  **Whether a mask tracks the value's length.** Both drew a fixed run, and the
  reasoning is worth keeping rather than rediscovering: a run of one character per
  character hidden publishes the length of a passport number, a date of birth or a
  card, which is what the mask was drawn to withhold and is often enough to
  identify the format on its own. The exact width is the client's; that it does
  not vary with the value is not.

  They guessed alike, which is luck rather than a specification — and the third
  implementation is the one that would have differed.

  Also stated for `emailLocal`: a value with no local part or no `@` is masked in
  full rather than guessed at.

## 0.17.2 — 2026-09-07


### Specifications

- **persona**: A claim-type registry, and the two axes that key on it (#377)

* feat(persona): a claim-type registry, and the two axes that key on it

  A holder's attributes carry a type, a value and a provenance, and nothing that
  says how carefully to handle either. Every consumer therefore decides for
  itself whether a card number is shown in the clear — which is to say, nobody
  decides.

  Four mechanisms wanted somewhere stable to attach to: a mask format, a
  sensitivity default, a release requirement, and any mapping to an external
  vocabulary. All four key on the claim type, and the claim type had a *pattern*
  but no *registry* — nothing distinguished `phone.mobile` from `mobile.phone`,
  and both validated.

  ## The registry

  `claim-types.json` names 26 core tokens with, for each, the expected
  `valueType`, a sensitivity, a release requirement, a mask style, whether it is
  in the minimum set, and the OIDC standard claims it answers.
  `CLAIM-TYPES.md` carries the reasoning.

  It does not change the decision `ClaimType` already records — the token stays
  ours, and external vocabularies are mappings applied at presentation. It
  *records* those mappings rather than adopting one, and keeps the two properties
  no flat vocabulary has: a hierarchy (so `payment.*` classifies a family without
  enumerating it) and an open `x:` namespace.

  ## Three axes, kept apart

  - **sensitivity** — how carefully a value is shown *to its own holder*
  - **release** — what it takes to let it *leave*
  - **linkability** — what it *costs* once gone (already computed, by
    `correlation/analyze`)

  A payment card is highly sensitive, release-gated and barely linkable — every
  card number is unique, so knowing one tells a second verifier nothing about the
  first. A passport number is all three. Reading any one as a proxy for another
  builds a consumer that hides the wrong things and warns about the wrong things.

  ## Store the override; derive the default

  `Attribute.sensitivity` and `Attribute.release` are optional, and their absence
  means *the holder did not decide* — not `normal`. A consumer resolves absence
  from the registry, and `attribute/put` MUST NOT persist a resolved default in
  its place: an attribute that recorded one would keep it after the registry
  tightened, so a reclassification would protect new attributes and leave the
  existing ones exposed.

  An unregistered or `x:` token resolves to the conservative answer. That is where
  "absence is most restrictive" belongs — applied to a vocabulary nobody has
  reasoned about, not to an unset field on a known type, where it would mask every
  legal name and teach holders to reveal reflexively.

  ## The half that is not cosmetic

  `attribute/list` gains `includeSensitive`. A listing that asked for values still
  omits the plaintext of a `high` attribute unless it also asks for those.
  Separate from `includeValues` because the two escalations answer different
  callers: a picker wants every name and no card number.

  Masking a value already received defends a screen. It is no defence at all
  against a log, a crash dump, or the memory of the process holding it — the
  control that matters is the one deciding whether the plaintext is sent.

  ## Release, enforced where it can be

  `disclosure/present` MUST refuse a `stepUp` claim without a fresh approval bound
  to **that `previewId`** — `stepUpRequired`, retryable, and the preview is not
  consumed, since refusing for want of an approval must not cost the holder the
  decision they already made. Binding to the session instead would turn "each
  time" into "once per login", which is the whole of what the requirement asks
  for.

  ## Also: whoami reports capabilities

  Effective, not stored — the question is "what may I do", and an entry that
  narrows nothing means everything its role implies.

  Without it a client seeing only roles and scopes cannot tell whether a caller
  holds a capability its role does not imply, so it must either refuse a caller
  the service would honour or offer every action and let the service refuse.
  Both are live problems: the console does the former today.

  The persona conventions state the rule this sits under — **a response may name
  identity; it may not carry identity.** whoami answers what a producer may do,
  never who they are made of.

## 0.17.1 — 2026-09-06


### Documentation

- **persona**: Say why a context-local value carries no provenance (#374)

`persona/local/profile/put/1.0`'s inline entry is `{type, valueType, value,
  label?}` — narrower than a pool profile's inline entry, which also carries
  `provenance` and requires it. The spec explains the missing `ref`, pinned
  and override forms at length and says nothing about this one, so the reader
  is left to decide whether it is a rule or an oversight.

  It is a rule, and a load-bearing one. A `credentialBacked` provenance names
  a `credentialId` and a `claimPath`, and a value authored inside a context
  has nowhere to put either — so a context-local value is self-asserted by
  construction, and that is what a maintainer must present it as. It is the
  same boundary the missing reference forms enforce, one member along: those
  stop a context-authored object acquiring pool *reach*, this stops it
  acquiring an issuer's *authority*, asserting that a value is attested when
  no credential was ever checked, over a value the issuer never saw.

  Written down because the absence is doing work that only its author can
  currently see. An implementer meeting a required `provenance` in the
  disclosure response and no way to supply one from a local profile has to
  invent an answer; and the next reader, finding a member here that its
  sibling has, could reasonably conclude it was forgotten and add it — which
  would be a privilege escalation dressed as a convenience.

  Prompted by implementing it: OpenVTC/verifiable-trust-infrastructure#1268
  had to supply `SelfAsserted` at the mapping boundary with the reasoning in
  a code comment, which is the wrong place for a rule every implementation
  needs.

  No schema change — a description, one normative sentence, and the
  regenerated doc comments. Bindings conformance: 403 specs against 403 TS and
  398 Rust modules, all agree.

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
