# Changelog — `trust-tasks-go`

All notable changes to the Go bindings module.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The module versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

A Go module is published by tagging, so the released version of this module is
the `trust-tasks-go/vX.Y.Z` tag rather than anything in the tree; the `Version`
constant in `trusttasks/version.go` mirrors it. See `RELEASING.md`.

## 0.2.3 — 2026-09-23


### Added

- **git-ns**: Add the git namespaces specification family (#621)

* feat(git-ns): add the git namespaces specification family

  A VTC can say today that a DID may sign commits for an org or a repo, and
  nothing else about git: who owns a repository, who may create one, who may
  merge and who may grant commit rights all live on the forge, and drift from
  the VTC as soon as someone clicks a button. git-ns makes the VTC the source of
  truth for all of it. Rights are per resource, not per role; the Trust Registry
  is the published projection verifiers read, and the forge is the enforced one.

  Five rights, spelled as the TRQP actions the VTC publishes them under:
  git.ns.admin, git.repo.create, git.repo.own, git.repo.maintain and
  git.commit.sign. Resources are forge-qualified and lowercase
  (github.com/acme/widgets, codeberg.org/acme), so a right never crosses forges
  and a community can host where it likes.

  Member- and admin-facing tasks, addressed to the VTC:

  - git-ns/namespace/bind, git-ns/namespace/unbind
  - git-ns/repo/create, git-ns/repo/adopt, git-ns/repo/transfer,
    git-ns/repo/archive
  - git-ns/right/grant, git-ns/right/revoke
  - git-ns/view
  - git-ns/account/link, git-ns/account/link-status

  git-ns/right/grant carries the family's rights model: implied rights, the
  grant-authority table, and six fixed rules a VTC enforces in its own code
  before policy runs, which policy can narrow and never loosen: scope
  containment, no escalation, the last-owner invariant, a last-admin invariant,
  the members-only floor for namespace rights, and policy-may-only-narrow. Each
  refusal has its own error code, declared once at the family level (git-ns:*).

  VTC <-> bridge tasks, for the per-community service that holds the forge
  credentials and the forge adapters:

  - git-ns/bridge/job — seven convergent, forge-neutral job kinds
  - git-ns/bridge/result — exactly one per job, per-step outcomes
  - git-ns/bridge/event — nine forge-neutral event types plus drift

  Shared shapes (resources, namespaces, RepoSummary, RightRecord, drift) live in
  git-ns/_shared/0.1.

  Bindings regenerated for Rust, TypeScript, Go and Dart; conformance checks
  agree. trust-tasks-rs gains a git-ns feature, in all-specs.

## 0.2.2 — 2026-09-23


### Added

- **vtc/vetting/vetters/event-mode**: The exception to the constant drip (#620)

A vetter's ordinary rate is a few tokens a tick, whether or not they have
  vetted anyone. That is the right rate for ordinary weeks and the wrong one for
  a conference desk, and the answer is deliberately not a bigger drip under the
  same key: it is a separate token label for a named event, with its own rate,
  its own expiry, and a group of vetters large enough that a spend under it still
  hides one.

  This task carries the vetter's half of that, which is only ever a request. The
  approval is an act by someone else, through the community's own administrative
  surface, and the specification says why there is no Trust Task for it: a task
  the vetter could send is a task a vetter could be made to send.

  The response says where the request stands, and carries a group *count* rather
  than a group. Who else is at the event is the anonymity set, so the number is
  the most a member may be told — enough to tell "nobody has approved it" from
  "not enough people have asked", which are the two reasons a request waits.

- **vetting/attestation**: The four tasks hidden-vetter admission needs (#618)

* feat(vetting/attestation): the four tasks hidden-vetter admission needs

  A community can hide which of its vetters vetted an applicant: the vetter
  attests under a blind class credential, the applicant proves that k distinct
  holders of one attested it, and the community counts the proof with the rule
  it already counts named statements with. What was missing was the wire.

  Four specifications, and the split between them is the design:

  - vtc/vetting/vetters/pcs-root — a vetter enrols for a class label. The
    community checks its own records (a live vetter grant, no credential under
    this label yet, the identifier it was bound to) and signs a commitment it
    cannot open. Being named happens here, once per label, and nowhere else on
    the path.
  - vtc/vetting/vetters/pcs-tokens — the vetter draws its tick of attestation
    tokens, unconditionally and at a published rate. A draw that tracked demand
    would report activity, which is what the exchange exists to hide; the quota
    is the community's to enforce, never the asker's restraint.
  - vetting/attestation — vetter to applicant, carrying the facts of the session
    and no issuer. The delivering identity is deliberately NOT what makes it
    count, and a consumer is told not to record it beside the attestation: that
    would recreate, in the applicant's own store, the link the exchange removes.
  - vtc/vetting/pcs-challenge — the applicant asks the community for the
    single-use nonce its proof must bind. Without it a proof verifies as often
    as it is submitted, and the second submission counts as readily as the first.

  vetting/attestation declares identifierScope: any. Nothing in it needs a
  reusable identifier — the community never sees the document, and the applicant
  only needs the identifier the session was held under — so a pair that runs the
  whole vetting exchange pairwise loses nothing.

  Bindings regenerated for all four languages; conformance checks agree.

## 0.2.1 — 2026-09-23


### Added

- **persona**: Attribute/get — read one attribute, by identifier (#616)

The family had no narrow read, and the shape of the workaround is the
  argument for the task. A client revealing one value called
  `persona/attribute/list` with a `typePrefix` and `includeValues`, then
  filtered to the id it already held — so showing one email address
  decrypted and returned every email address the holder has, and the audit
  trail recorded a listing of the pool rather than a decision about one
  fact.

  - Values are withheld unless asked for, and a `sensitivity: high` value
    needs a second flag, on the same two-step as `attribute/list`. A
    withheld value is stated (`valueWithheld`), never left to inference: a
    consumer reading an absent value as "there is none" shows the holder an
    empty field where their passport number is.
  - `version` reads a retained earlier version, which is what makes "what
    did I show them in March" answerable — a disclosure record and a pinned
    entry both name one. A purged version is `versionPurged`, never a
    silent fall back to the current value.
  - `retainedVersions` names the versions still held, so a holder deciding
    whether to purge can see what purging would take away rather than being
    asked to make an irreversible decision blind.

  One identifier, never a list: a maintainer that accepted several would
  recreate the enumeration the task exists to avoid, one call later.

## 0.2.0 — 2026-09-23


### Added

- **persona**: An arrangement of faces is a world, not a facet (#610)

`face` and `facet` share a stem and name different things — a projection
  of the pool, and an arrangement of those projections. Every consumer's UI
  had already resolved it by saying "world" on screen while the wire said
  facet, which leaves the collision in place for anyone reading both.

  - `persona/world/{put,list,delete}/1.0` — the same tasks, with `facetId`
    as `worldId`. `persona/facet/*` is retired, `supersededBy` the new
    slug, so documents already issued stay verifiable.
  - `persona/correlation/analyze/1.1` — `facetId`, `facetIds` and
    `crossesFacets` become `worldId`, `worldIds` and `crossesWorlds`. A
    breaking rename carried as a MINOR increment, which SPEC §5.2 permits
    for a `draft`. 1.0 is retired in its favour.
  - `_shared/0.1` gains `WorldColour`, the same eight colours.
    `FacetColour` stays because a retired specification's schema is frozen
    and still references it.

  Nothing else moves: same members, same semantics, same error codes.
  `facet` survives elsewhere in the registry in its ordinary English sense
  (a facet of a trust record, of an account) and is left alone.



### Fixed

- **persona**: Local/profile/put answers the correlation index only to the holder (#609)

`correlation.matchesPoolValue` is a yes/no on "does the holder hold this
  exact value anywhere", computed from the agent-wide index — and the task
  is context-scoped, so the response handed that answer to any caller
  authorized in one context.

  A caller that can write is a caller that can guess: one value per write,
  unbounded, each answer confirming or eliminating one. No value crosses
  the boundary and none needs to — for a name, an address or a date of
  birth, confirmation is disclosure, and the guesser is inside a single
  context learning about all of them.

  So `correlation` becomes conditional: a maintainer MUST include it only
  for a caller authorized to read across the holder's contexts, and MUST
  omit the member entirely otherwise. Omit rather than soften — a coarser
  signal is still an oracle, only a slower one. The holder is still owed
  the warning that a throwaway is reusing a real value, through an audit
  entry or the holder-reach correlation task.

  The schema's own description said the index is answerable "only to the
  holder" while this response answered it to anyone in the context. The
  prose and the conformance rules now say what that means for this member.

## 0.1.21 — 2026-09-23

## 0.1.20 — 2026-09-22

## 0.1.19 — 2026-09-22

## 0.1.18 — 2026-09-22

## 0.1.17 — 2026-09-22

## 0.1.16 — 2026-09-21

## 0.1.15 — 2026-09-21


### Specifications

- Bring framework 0.6.0 into the registry (SPEC.md + specs/_framework/0.6) (#555)

Companion to trustoverip/dtgwg-trust-tasks-spec#18. Publishes the 0.6
  envelope schema (0.5's shape unchanged, so 0.6.0 becomes targetable; no
  spec is re-targeted) and brings the SPEC.md mirror to 0.6: version and
  date header, plus the Appendix B entry.

## 0.1.14 — 2026-09-21

## 0.1.13 — 2026-09-21

## 0.1.12 — 2026-09-21

## 0.1.11 — 2026-09-21

## 0.1.10 — 2026-09-21

## 0.1.9 — 2026-09-20


### Specifications

- **vtc/endorsement-types/delete**: A criterion requiring the type also blocks its deletion (#523)

The spec described one kind of reference — a live endorsement — and the
  `inUse` refusal as being about orphaned endorsements alone. A second kind
  exists wherever the consumer also holds admission criteria: a criterion
  requiring statements of the type is left asking applicants for evidence the
  community no longer recognises. Where registering such a criterion against
  an unregistered type is itself refused, deleting the type strands the
  criterion in a state it could not have been created in.

  `inUse` now covers both, and gains a `detailsSchema` so a consumer that can
  determine both reports which applies rather than leaving the caller to
  parse prose. `liveEndorsements` and `criteria` are each optional, because
  neither an endorsement store nor a criteria registry is mandatory — and the
  spec says plainly that an absent member means "not applicable here", not
  "none found", so a caller cannot read silence as an all-clear.

  Conformance asks a consumer that can determine both to gather both before
  refusing. Refusing on the first found turns one determination into as many
  round trips as there are kinds of reference, each ending in the same code.

  Security & Privacy gains what the refusal discloses: `details` names
  criteria by identifier, which is a governance fact the authorised
  administrator can already enumerate, and is reachable only after the
  community-admin capability has been verified.

  In place on 0.1 rather than a new version: the spec is `draft`, no payload
  or response shape changes, and no previously-valid document becomes
  invalid.

  Implemented in verifiable-trust-infrastructure#1584, which added the
  criterion check and reported both causes in one refusal.

## 0.1.8 — 2026-09-20

## 0.1.7 — 2026-09-19

## 0.1.6 — 2026-09-17

## 0.1.5 — 2026-09-16


### Added

- **go-capability-client**: Capability wire client for Go (trust-tasks-go/capabilityclient) (#498)

The Go port of the Rust trust-tasks-capability-client crate: pure wire logic,
  no crypto and no transport. Document builders for the governance/capability/*
  (list/enable/disable) and git-trust/* (grant/revoke) families, DIDComm envelope
  parsing, and reply classification — so a capability producer and a management UI
  cannot drift on the contract.

  A separate nested module like tsp/proof/didcomm, but with no third-party
  dependency at all (only the core, for Document and SlugFromTypeURI).

  - Builders mint a fresh id + issuedAt per call; NewAttempt() re-stamps a built
    document under a fresh id and clears proof (a SPEC §8.4 new attempt, distinct
    from a bit-for-bit retry the consumer's item-11 record absorbs).
  - ClassifyGitTrustReply / ParseCapabilityReply correlate on threadId first
    (§4.9) and key idempotent success on the SPEC §8.5 extended error code (both
    spellings), never the non-normative free-text message; the free-text path is
    opt-in via ReplyPolicy.

  Adds a capabilityclient job to go.yml, the Go capability-client cell to the
  capability matrix, and documents the module in CLAUDE.md. 12 tests, go 1.22 +
  stable, gofmt clean.

## 0.1.4 — 2026-09-16


### Added

- **go-didcomm**: DIDComm v2.1 transport binding for Go (trust-tasks-go/didcomm) (#495)

The bindings/didcomm/0.2 binding for Go, rolled on the standard library. A
  separate nested module like tsp and proof so the core stays dependency-free,
  and like proof it pulls in no third-party code: DIDComm v2.1 authcrypt
  (ECDH-1PU key agreement + A256KW key wrapping + A256CBC-HS512 content
  encryption, over X25519) on crypto/ecdh, with the RFC 3394 key-wrap (pinned to
  the RFC's test vector) and RFC 7518 content encryption written in-module.

  aries-framework-go was evaluated and rejected: its packer is a low-level JWE
  primitive needing a full KMS/Crypto/Storage/VDR provider and raw key bytes, it
  does not build/parse the v2 message envelope or verify sender_kid, and the
  framework is archived. Rolling our own keeps the four bindings aligned and
  PQC-extensible.

  The verified skid is authenticated by the ECDH-1PU static secret (a forged
  skid fails the key unwrap) and becomes the §4.8.1 transport-authenticated
  sender; anoncrypt/plaintext are rejected (binding §2/§4). Key-based like tsp
  (a ResolveSender callback, no DID resolution); the consumer keeps the §7.2
  item-11 duplicate-execution record on by default (binding §6) and routes a
  thread-header disagreement as malformedRequest (binding §3.1).

  Adds a didcomm job to go.yml, the Go implementation to the didcomm/0.2 binding
  registry, the Go binding:didcomm cell to the capability matrix, and documents
  the module in CLAUDE.md. Not release-wired yet; a shared cross-library authcrypt
  fixture is a follow-up.

## 0.1.3 — 2026-09-16


### Added

- **go-proof**: Data Integrity proofs for Go (trust-tasks-go/proof) (#493)

A ProofVerifier and signer for the core module's proof seam, rolled on the
  Go standard library — eddsa-jcs-2022 (Ed25519) and ecdsa-jcs-2019
  (P-256/P-384), did:key only, no network. A separate nested module like
  trust-tasks-go/tsp so the core stays dependency-free, but with no third-party
  code at all: the JCS canonicaliser and base58btc codec are written in-module.

  Byte-compatible with the Rust and TypeScript proof libraries — a test
  reproduces the shared eddsa-jcs-2022 fixture's proofValue exactly. Verifier
  binds the proof to the in-band issuer and returns a typed *proof.Error whose
  Kind stays in the logs (every failure is proofInvalid on the wire, SPEC
  §10.4).

  Adds a proof job to go.yml (the core ./... jobs do not reach a nested module),
  marks the Go proof cell shipped in the implementations matrix, and documents
  the module in CLAUDE.md. Not wired into a release yet — unlike tsp there is no
  blocker, so a publish-go-style tag job is a clean follow-up.

## 0.1.2 — 2026-09-16


### Added

- **go-tsp**: TSP transport binding for Go (trust-tasks-go/tsp) (#486)

A separate nested Go module so the core stays dependency-free: PackTrustTask / UnpackTrustTask for HPKE-sealed Direct TSP messages on affinidi-tsp-go, and Consumer.Receive, the guarded inbound path running SPEC §7.2 with the item-11 record on by default. The sealed {type, document} envelope matches the Rust crate and Dart package (pinned by a test).

  Not released: affinidi-tsp-go has no tag, so it is required at a pseudo-version and there is no trust-tasks-go/tsp/v* release wiring yet; go get resolves the pseudo-version from the public repo. go.yml gets a dedicated tsp job; the core ./... jobs exclude the nested module.

## 0.1.1 — 2026-09-16

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `trusttasks/`.
