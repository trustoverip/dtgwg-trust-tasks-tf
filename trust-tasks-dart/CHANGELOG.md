# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

Entries before 0.9.0 are in [`CHANGELOG-archive.md`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-dart/CHANGELOG-archive.md),
kept out of the package: pub.dev refuses a `CHANGELOG.md` over 256 KiB.

## 0.9.37 — 2026-11-11


### Added

- **vta/webs/dids**: Level-1 did:webs identities for personas, relationships and integrations (0.1) (#1082)

create, get, list, update, rotate-keys and delete at 0.1, with the family's
  conventions and shared record under vta/webs/_shared/0.1. KERI v2 with JSON
  bodies; one Ed25519 signing key (kt 1) and one pre-rotated successor (nt 1),
  both derived by the VTA; aliases, key agreement and services as designations
  anchored by interaction events; hosted only, through a did-hosting server with
  method webs; remote first, rotations staged. Never a node's service DID
  (VTI-KEY-001). Draft; the did:webs integration is alpha.

  Bindings regenerated for Rust, TS, Go and Dart.



### Specifications

- **vtc/rooms/community/update**: Item 10 carries every profile member the community does not manage, recordMembers included (#1080)

Item 10 listed the members to carry over on a re-write as `tags`, `emoji`,
  `ext`, a list written before the room profile gained `recordMembers`. A
  community following it to the letter drops `recordMembers` on the next
  label or description edit, and the room's writers lose replies and
  reactions (`kind`, `inReplyTo`, `writtenAt`).

  Item 10 (0.2 and 0.3, identically) now names the rule rather than a list:
  carry over every member but `name` and `description`, naming the four
  there are today and any a later profile version adds. And a community that
  cannot read the current profile SHOULD NOT write one without them; the
  write is then pending under item 11, so it is retried rather than written
  blind.

  Prose only: no schema, bundle or generated type changes.

## 0.9.36 — 2026-11-10


### Added

- **messaging/mux**: ConnectionFull detach reason; didcomm-mux subprotocol binds a DIDComm socket (#1078)

* feat(messaging/mux): connectionFull detach reason; didcomm-mux subprotocol binds a DIDComm socket

## 0.9.35 — 2026-11-10


### Added

- **acl**: Purpose of use on grants and on memory reads (AclEntry 0.3; vta/memory put/delete/list 0.3, get 0.2) (#1074)

* feat(acl): purpose of use on grants and on memory reads (AclEntry 0.3; vta/memory put/delete/list 0.3, get 0.2)

  An access-control entry says who may do what, where and until when, but not
  what for. An agent granted a read of its principal's memory can use the content
  for anything, and nothing records the use it intended. The VTI specification
  now requires the purpose (VTI-ACL-100 – 110, trustoverip/dtgwg-vti-spec#115);
  these are the wire shapes.

  acl/_shared/0.3 — AclEntry 0.3 adds one REQUIRED axis, `purposes`:
  `{"scope": "any"}` or `{"scope": "listed", "purposes": [...]}` with at least
  one `Purpose` — a lowerCamelCase name registered in the VTI purpose registry
  (personalisation, taskExecution, analytics, modelTraining) or an absolute IRI
  such as a DPV purpose. `any` never grants `modelTraining`. Delegated purposes
  are a subset of the granter's. CONVENTIONS.md §8a maps a 0.2 entry onto 0.3
  (`any`, the authority it already held, now stated) and §12 states the rules.

  acl tasks on AclEntry 0.3: grant/0.3, update/0.3, show/0.3, revoke/0.3,
  change-role/0.3, list/0.4. New errors on grant and update: `purposeOutsideDelegator`
  (purposes wider than the granter's) and `purposeRefused` (policy will not grant
  a purpose, refused by name, never dropped). update/0.3 replaces `purposes`
  wholesale; narrowing it is an audited privilege reduction. `purposes` is never
  redacted by show or list. No `extends`: a REQUIRED member on the entry cannot
  be added to a version a 0.2 consumer still reads.

  vta/memory put/0.3, delete/0.3, list/0.3, get/0.2 (each `extends` its
  predecessor) add an optional `purpose` (`MemoryPurpose`, new in
  vta/_shared/0.1/memory-record, a reference to acl's `Purpose`) and a
  `purposeNotGranted` error that discloses nothing and is distinct from
  `contextForbidden`. A recall, and a listing with values, is purpose-bound:
  refused for a purpose the entry does not grant, or for none under an entry
  listing several, and audited with its purpose either way. A write is not
  purpose-bound, but a conflict that returns `currentValue` is: the value is
  withheld unless the request's purpose is granted.

  Data rooms (a room authority credential's `purposes`, VTI-ACL-110) are a
  follow-up.

  Samples in other specs that named the superseded acl versions now name the new
  ones, with `purposes` on every embedded entry; prose links were moved by
  `npm run fix-stale-references`.



### Fixed

- **mux/grant**: Unknown-key rule fails closed on direct authentication; no exemption for a grant without recorded keys (#1075)

A grant that recorded no key-agreement keys exempted every grant on its DID from the DIDComm direct-authentication rule. A grant is now refused (didUnresolvable) when its keys cannot be read, an empty set means the DID listed none, and the rule is applied before a direct authentication succeeds, refusing it when it cannot be applied.

- **messaging/mux**: Compare a direct authentication with keys of its own relationship; send grantsExpiring for every unrenewed grant (#1072)


### Specifications

- **rooms**: Purpose of use on room reads and authority credentials (VTI-ACL-102, -110) (#1076)

* spec(rooms): purpose of use on room reads and authority credentials (VTI-ACL-102, -110)

  Room content a key holder releases to its caller is purpose-bound, as agent
  memory is (vta/memory/get 0.2), and a room's authority credential may state
  the purposes it grants.

  - rooms/keys/open 0.3, rooms/keys/read 0.3: optional `purpose`; refused with
    `purposeNotGranted` where the caller's entry, or the principal's authority
    over the room, does not grant it. read passes it to the host.
  - rooms/keys/file-key 0.3: the same as `purposeOfUse` (`purpose` already
    says seal or open), bound only on `open`.
  - rooms/records/get 0.4: the host holds a read to the purposes the presented
    chain states; a chain stating none grants `any`.
  - rooms/owner/issue-authority 0.3: optional `purposes` (AclEntry 0.3's
    PurposeScope), stated in the credential; absent grants `any`.
  - rooms/_shared 0.2: `RecordPurpose`, and AuthorityPresentation's rule for
    purposes along a chain: a link stating none takes its parent's, a link
    stating wider is refused whole (VTI-ACL-107).

  Each new version `extends` its predecessor.

## 0.9.34 — 2026-11-09


### Documentation

- **vta/dids**: Did:webs is eligible with the key-role designation extension (#1070)

* docs(vta/dids): did:webs is eligible with the key-role designation extension

  The did:webs method profile declared the method ineligible as published
  (criteria 4 and 6: no controller-chosen relationships, no room for
  keyRoles or revoked) and recommended an extension A: role keys anchored in
  the KEL, rendered into their roles' relationships with keyRoles and
  revoked, key-state keys in no relationship. affinidi-did-webs 0.9.1
  implements it as a key-role designation (an anchored ACDC), proposed to
  the did:webs specification.

  The profile now states the status as eligible with that extension and
  ineligible without it, and renders §4 as built: the record's schema and
  attributes, one standing record, accumulated revocations, the document's
  members, the fail-closed checks. It states the interop caveat: a resolver
  without the extension refuses the published did.json.

## 0.9.33 — 2026-11-09


### Added

- **messaging/mux**: DID multiplexing at a mediator (messaging/mux/*/0.1) (#1068)

* feat(messaging/mux): DID multiplexing at a mediator (messaging/mux/*/0.1)

  One authenticated connection (the principal) carries live delivery and
  pickup for many member DIDs: session attaches bound to one connection's
  sessionId/muxNonce, and stored grants a member DID signs once. Members
  sign closed statements (muxAttach, muxGrant, muxRevokeAll) with an
  embedded Data Integrity proof (proofPurpose authentication), verified in a
  normative order before any state changes.

  Adds attach, detach, grant, list, update (narrow only), revoke,
  revoke-all (epoch advance), event (live: ready/attached/detached/refused,
  incl. the tsp-mux subprotocol) and notice (durable: grantsSuspended,
  grantsRevoked, grantsExpiring, grantsPresent), with shared definitions in
  messaging/mux/_shared/0.1/mux.schema.json, stable per-statement refusal
  codes, the unknown-key suspension rule and replay floors.

  Allowlists eddsa-rdfc-2022 as W3C Data Integrity vocabulary for the
  casing lint.

- **vta/memory**: Versioned writes, tombstones, a change feed and named reads (put/delete/list 0.2, get 0.1) (#1067)

vta/memory 0.1 is an unconditional upsert key/value store: put overwrites
  with no version, so two writers clobber each other undetected; delete is a
  hard delete, so another device's cached copy never learns the item is gone;
  and list returns the whole context with no prefix, paging or change feed.

  The new versions apply vta/app-state's semantics to memory, restated per
  spec so each can be implemented on its own:

  - A per-context write counter (`MemoryRecord.version`, in the new shared
    definition vta/_shared/0.1/memory-record) serves both as the
    optimistic-concurrency token and the sync watermark.
  - put/0.2: optional `expectedVersion` (0 = create only). A failed
    precondition is `versionConflict`, with the current version and value
    in its details so the loser need not re-read. The response returns
    `version`, `wasCreated`, `updatedAt` and `valueBytes`. `valueTooLarge`
    carries `limitBytes` / `actualBytes`. `currentValue` is withheld from a
    caller that may write the context but not read it.
  - delete/0.2: leaves a tombstone, takes an optional positive
    `expectedVersion`, and treats deleting nothing as success
    (`existed: false`).
  - list/0.2: a snapshot (prefix, cursor/limit, values by default as in
    0.1) or a change feed from `sinceVersion` that includes tombstones,
    with `highWatermark`, `watermarkTooOld` and `cursorInvalid`.
  - get/0.1 (new): up to 256 named keys, each answered once in `items`,
    `missing` or `deferred`. It is the read whose audit can name exactly
    which items an agent saw.

  The 0.1 versions stay published and are not superseded; a VTA serving
  both writes one store, and the 0.2 prose says how 0.1 writes and deletes
  take versions and leave tombstones there.

  Requested in OpenVTC/verifiable-trust-infrastructure#2115. The
  item-level read audit in #2116 builds on get/0.1.

## 0.9.32 — 2026-11-09


### Documentation

- **vtc/badges**: Declare the known implementation of every badges task (#1061)


### Fixed

- **vtc/members/rotate**: A rotation onto did:scid or did:webs can be reported (#1064)

`vtc/members/rotate/0.1`'s response `method` allowed only `did:key` and
  `did:webvh`, but a community verifies a rotation onto a `did:scid:vh:1`
  identifier (VTI-KEY-161) and, in VTI, onto a `did:webs` (VTI-KEY-003,
  VTI-LNK-052): a successful rotation onto either had no conforming response.
  Appends `did:scid` and `did:webs` to the enum, in place (an appended enum
  value is an additive change), and says what `method` is.



### Other

- The community issues the badge credential (#1063)

## 0.9.31 — 2026-11-08


### Added

- **vtc/badges**: Claim codes and the badge rule counts (VTI-BDG-015, VTI-BDG-025 – 027) (#1059)

* feat(vtc/badges): claim codes and the badge rule counts (VTI-BDG-015, VTI-BDG-025 – 027)

## 0.9.30 — 2026-11-08


### Added

- **vtc/badges**: Show answers the caller's own awards (myAwards), so a holder can share one (#1057)

* feat(vtc/badges): show answers the caller's own awards (myAwards), so a holder can share one

  vtc/badges/awards/share and unshare (#1054) name an award, but no read answered a holder their own accepted award identifiers for a community badge: mine/list carries badges, show only myCount. vtc/badges/show/0.1#response gains the optional myAwards - the caller's own accepted awards of the badge, newest first, each with its shared - and conformance item 4 says so. Additive, on a draft.

- **auth/step-up**: Requests/show — the subject's wallet fetches a step-up refused to a delegated producer (auth/step-up/requests/show/0.1) (#1056)

A relying party that refuses an operation until its subject steps up carries
  the request inline in the refusal (approve-request/0.5, Inline delivery). That
  only works when the producer of the operation can gather the evidence itself.
  A process acting with a key the subject delegated to it — an assistant, an
  automation — can redeem a recorded step-up but never make one. It cannot
  forward the inline request to the subject's wallet either, because the inline
  request carries no proof.

  This adds a hand-off by reference. Beside the inline request, the relying
  party MAY carry `details.stepUpHandoff` (`$defs.Handoff`): a single-use handle
  of at least 128 bits, its expiry and this task's Type URI. The person's
  approver agent sends the handle and receives two things:

  - the relying party's own signed approve-request/0.5, addressed to the
    subject, with the same challenge and binding as the inline request;
  - the operation the step-up is bound to, exactly as received.

  The agent verifies the proof and shows the operation itself. It recomputes the
  binding where it knows the construction. It then answers with approve-response
  0.7, signed by the subject's own DID, directly to the relying party. The
  producer resends the identical operation, and the recorded step-up is redeemed.

  A handle is answered at most once, atomically. Unknown, spent, expired and
  settled handles all return `notFound`. Reading a handle is never part of an
  approval, so a party that sees the handle can spend the read but cannot
  approve.

## 0.9.29 — 2026-11-07


### Added

- **vtc/badges**: Awards/share and awards/unshare — a holder shows an award outside the community (vtc/badges/awards/{share,unshare}/0.1) (#1054)

* feat(vtc/badges): awards/share and awards/unshare — a holder shows an award outside the community (vtc/badges/awards/{share,unshare}/0.1)

  The two tasks the VTC badges design note lists for showing a badge outside
  the community (VTI-BDG-070 – 072), completing the vtc/badges/* 0.1 family.

  - `vtc/badges/awards/share/0.1`: the holder of an accepted award of a badge
    the community marked `shareableOutside` asks, and the community answers the
    public locations it serves the award at — a verification location that
    answers the credential exactly as issued with its status, and a baked image
    where it serves one. Locations are the community's, unguessable (≥ 128
    bits, never derived from the award), idempotent on a repeat, and disclose
    no other holder nor a count (VTI-BDG-071). They stop serving on unshare,
    take-back, retire and departure. Codes: badgesOff, notFound (unknown,
    another member's, not accepted — alike), notShareable (insideOnly | kudos
    | retired), publicLocationUnavailable.
  - `vtc/badges/awards/unshare/0.1`: stops it; the locations answer as ones
    that never existed; idempotent; not a revocation. Codes: badgesOff,
    notFound.
  - `_shared/0.1/badge.schema.json`: `AwardShare` and the optional
    `Award.shared` (verificationUrl, bakedImageUrl?, since) — additive.

  Both are acts of the member: a portal key the member authorized for
  `vtc/badges` under auth/oob/grant/0.3 may sign them (VTI-BDG-095).

## 0.9.28 — 2026-11-06


### Added

- **vtc/badges**: Member badges — community badges backed by groups, awards accepted by the member, kudos, Open Badges credentials (vtc/badges/*/0.1) (#1052)

* feat(vtc/badges): member badges — community badges backed by groups, awards accepted by the member, kudos, Open Badges credentials (vtc/badges/*/0.1)

  A new family for the VTI Community Badges chapter (VTI-BDG-001 – 095): pictures
  with a name a member earns, holds and shows on their directory card.

  - A community badge is backed by a community group: `create` creates the group
    as vtc/groups/create does, its managers are the badge's owners, its members
    the holders; requests reuse vtc/groups/join and requests/decide, owners reuse
    managers/add and remove. The shared Group record gains an optional `badgeId`
    (additive).
  - An award is an offer until its recipient accepts (awards/offer → mine/accept
    or mine/decline); mine/accept issues the badge credential as the group's DID
    first and records the membership after. mine/accept states the credential
    profile: an Open Badges 3.0 OpenBadgeCredential with the community as the
    Achievement's creator and endorser, one evidence entry per award; short-lived
    with no status inside the community, long-lived with a slot in the community's
    own status set where holders may show it outside.
  - Hidden is portal-only (mine/hide, mine/show): walls, holders/list and counts
    leave a hidden holder out; awards/list shows owners the hidden mark.
  - Kudos badges (kudos/give, kudos/list): given by any member to any other within
    per-giver limits, each giving an award of its own; promote turns one into a
    community badge.
  - settings/show and settings/update hold the community's switches and limits.
  - images/upload and images/get mirror the directory's picture pipeline.

## 0.9.27 — 2026-11-05


### Added

- **vtn/registry**: Check a party against the network's trust registry through the network, and the registry status for administrators (#1050)

vtn/registry/check/0.1 lets anyone ask a network to ask its trust registry whether a party is published as its member, returning the registry's own signed answer verbatim, so a verifier with no DIDComm or TSP session (a browser, or a registry that serves no HTTPS) can still check the registry rather than take the network's word for it. Unreachable is indeterminate, never not a member.

  vtn/registry/status/0.1 lets an administrator read the registry, the writer and whether it is connected, the last reconcile, and every record compared with what the entries project (ok, missing, stale, wrong).

- **vta/collection**: Settings/show and settings/set — collection by default per trust context, stored and changed at run time (#1049)

Collection by default was one operator setting, off unless enabled
  (VTI-NTF-093). The VTI specification withdraws that in favour of a setting of
  each trust context, enabled unless disabled there (VTI-NTF-094 – VTI-NTF-098):

  - vta/collection/settings/show/0.1: a party with standing in a context reads
    its setting, who last changed it and when, and whether the VTA's operator
    has suspended collection by default everywhere (VTI-NTF-096).
  - vta/collection/settings/set/0.1: a party with administrative authority over
    the context (VTI-ACL-022) enables or disables it; stored, audited, applied
    before the answer — disabling closes the VTA's connections first
    (VTI-NTF-095). Answers the setting and whether it changed.
  - _shared/0.1 gains a CollectionSetting definition (new $defs entry), and the
    `disabled` reason is re-described: the context's own setting, or the
    operator's suspension (VTI-NTF-097, VTI-NTF-098). No enum value changes.
  - claim/list/release 0.1 prose and an error-code meaning now cite the
    per-context setting instead of VTI-NTF-093 (description edits only).

  The setting is not folded into list's response: it belongs to the context,
  not an identifier, and an added member in that released response would be
  refused by every existing client's additionalProperties: false.

  Bindings regenerated (Rust, TS, Go, Dart).

## 0.9.26 — 2026-11-05


### Added

- **vtc/networks/preview**: Read a network before joining it — its profile, required roles, and whether it lists the community (#1047)

## 0.9.25 — 2026-11-05


### Added

- **vta/pcs/voter**: Check a receipt after close — receipts/list 0.1, read 0.7 signs a tombstone read (#1045)

vta/pcs/voter/receipts/list/0.1 returns the receipts a VTA keeps for one
  election, whole and exactly as received, with the ballot box recorded at
  registration — so after voting closes a client can fetch each cast's
  voting/board/inclusion proof from the ballot box over its own connection and
  hand it, with the head the community kept at close, to vta/pcs/voter/receipt
  (VTI-VOT-053). The client need keep nothing between the cast and the check.

  vta/pcs/voter/read/0.7 adds read: tombstone, signing a
  vtc/elections/tombstones/show/0.1 request as the member: once a community
  deletes the board an election was opened on, the heads it kept are in the
  tombstone, which it shows only to an active member (VTI-BRD-009). Only a show
  still carries a presentation. Two text references to read/0.6 move to 0.7.

  Rust, TypeScript, Go and Dart bindings regenerated; check-bindings agrees.

- **vtc/boards**: Appoint, remove and list a board's facilitators (#1044)

## 0.9.24 — 2026-11-05


### Added

- **rooms/keys**: Rooms/keys/watch and rooms/keys/changed — a key holder watches rooms for its member's wallet; act 0.12 lists rooms/watch (#1040)

A wallet whose member keys are held by its key holder cannot present as
  the member and has no address a host can reach, so it could not use
  rooms/watch (#1036).

- **did-management/stats/set-resolution**: Choose a slot's statistics resolution (#1041)

An owner can keep less statistical detail over time for their own DID (coarse, or totals only), or return it to the service's own choice; an administrator can set any level for any slot, including always keeping the finest series. Bindings regenerated.



### Specifications

- **vtc/elections**: A deleted group's DID is never a member party (VTI-VOT-029) (#1039)

vtc/elections/show/0.6, vtc/elections/list/0.5 and
  vtc/elections/tickets/issue/0.3 defined a member party as a
  `roll.memberOf` that is "neither the community's own DID nor one of its
  groups". Once a group is deleted its DID is no longer one of the
  community's groups, so read literally the definition let the election
  fall onto the member-party presentation path, where a holder of an
  unexpired credential the deleted group issued could read it.
  VTI-VOT-029 forbids exactly that: such an election is shown, and
  confirmed to exist, only to a holder of `vtc.elections.manage`, and
  the deleted group's identifier is never treated as a party outside
  the community.

  The member-party test is now decided by what `roll.memberOf` was when
  the election was created; show item 1 and list item 1 say a deleted
  group's election is seen only by a manager; show item 8.3 says a
  presentation of the group's credential shows it to nobody. The
  payload descriptions of show and list say the same.

  Description and prose only: every schema is otherwise unchanged, as
  CONTRIBUTING-SPECS "Version rules" allows in place for a released
  draft version.

## 0.9.23 — 2026-11-04


### Added

- **trust-task-discovery**: Publish 0.4, a responder advertises the horizon up to which it honours a stated expiresAt (#1030)

SPEC §7.2 (Bounding the record) applies a consumer's maximum age only where a
  document states no expiresAt, and since trust-tasks-rs 0.37 (#1021)
  validate_freshness does so, honouring a stated expiry up to the policy's
  expiry_horizon. discovery 0.3 cannot describe that: its window "governs
  issuedAt only" and its responder refuses any document past
  issuedAt + maxAgeSeconds + clockSkewSeconds, whatever expiresAt it states. A
  responder that honours an expiry could only advertise a narrower window than
  it applies, and a producer could not learn that stating an expiry keeps a long
  delivery acceptable.

  - specs/trust-task-discovery/0.4: acceptanceWindow gains an optional
    expiryHorizonSeconds (whole seconds, at least 1), and the spec defines a
    document's acceptance end: issuedAt + maxAgeSeconds without an expiresAt;
    the earlier of expiresAt and issuedAt + expiryHorizonSeconds with one; and,
    where the window carries no horizon, the earlier of expiresAt and
    issuedAt + maxAgeSeconds, which is what a 0.3 window says.
    - Responder: MUST NOT advertise a horizon wider than it applies, nor one at
      all if it applies its maximum age whatever expiresAt states.
    - Discoverer: send bounds follow the acceptance end. MAY state an expiresAt
      within the horizon, no later than it will pursue delivery, instead of
      issuing a chain of new attempts; a new attempt carries it unchanged.
      Absent a horizon, it MUST assume the maximum age applies to every
      document.
    - AcceptanceWindow is closed in 0.3, so an added optional member is a MINOR
      increment (§5.2): a new version, not an edit.
    - Response-side invalid examples: a zero horizon, a duration string, and a
      fractional entry-level horizon.
  - trust-tasks-rs: DiscoveryRegistry::respond_to_v0_4 is the current answer.
    acceptance_window_from reads the policy's expiry_horizon, and
    expiry_horizon() sets one by hand. respond_to_v0_3 moves to
    discovery/superseded.rs and states the window without the horizon (narrower
    than applied, which 0.3 permits).
  - trust-tasks-https: enable_discovery answers 0.4, and 0.3 beside 0.2 and 0.1.
  - trust-tasks-dart-https: enableDiscovery answers 0.4. Dart's FreshnessPolicy
    has no horizon, so its window states none, which says the maximum age
    applies to every document. 0.3 moves to the superseded block.
  - Examples and tests that named the 0.3 types use 0.4.
  - Bindings regenerated (Rust, TS, Go, Dart). 0.3 is superseded.

  Follows from #1021 and trustoverip/dtgwg-vti-spec#96 (VTI-OPS-024,
  VTI-OPS-028, VTI-TRN-048), whose implementation in
  OpenVTC/verifiable-trust-infrastructure states delivery deadlines as expiresAt.



### Specifications

- **process-attestation**: Label WritersProof as the Holder, and check that an implementation's role names a party (#1029)

Follow-up to #994 and #997.

  - process-attestation/0.1 lists WritersProof in the { url, role, name } form
    #997 added, so the spec page shows which party it plays.
  - The outcome-evidence rule says "a relying party MUST NOT", not "a consumer":
    the rule binds whoever relies on the citing artifact, and "consumer" means a
    document's recipient (SPEC §7.2).
  - build-registry fails a knownImplementations role that names none of the
    specification's parties and is not "both". The meta-schema can only bound it
    as a string, and a typo would put an undefined party on the page.

- **process-attestation**: Declare outcome evidence, bind receipts by digest, add the CPoE profile (#994)

* spec(process-attestation): declare outcome evidence, bind receipts by digest, add the CPoE profile



### Website

- Show a task's outcome evidence and size bound, and which party an implementation plays (#997)

* site: show a task's outcome evidence and size bound, and which party an implementation plays

## 0.9.22 — 2026-11-04


### Added

- **vtc/boards**: Community boards — propose ideas, leave notes, vote on each accepted idea through its own election (vtc/boards/*/0.1) (#1025)

Thirteen tasks implementing the VTI Community Boards chapter (VTI-BRD-001 – VTI-BRD-080): create, update, list and show a board; propose, update, withdraw, decide (accept opens an election from the board's template) and select ideas; add, list, withdraw and hide notes. A board never reports tickets or ballots (VTI-BRD-033); the shared ElectionTemplate is a manifest less the members the community fills in per idea, and is never binding.

## 0.9.21 — 2026-11-04


### Added

- **vta/collection**: Claim, release and list who collects a persona DID's messages at the mediator (#1023)

Implements dtgwg-vti-spec's collection for the principal's identifiers
  (VTI-NTF-080 – VTI-NTF-092).

  A VTA collects, at the mediator each DID document advertises, the messages
  for every identifier whose messaging keys it holds in one of the principal's
  trust contexts — its personas — unless a client has claimed it. The mediator
  permits one connection per DID, so at most one party collects at a time.

  New tasks (family vta/collection, category messaging):

  - vta/collection/claim/0.1 — a client becomes an identifier's one collector
    until a stated end (a lease, at most thirty days), or renews its claim. The
    VTA stops collecting and closes its connection before it grants; the client
    connects only after the grant and closes before the end unless renewed.
    notCollectable, claimHeld, claimNotRenewable. Retry class 2.
  - vta/collection/release/0.1 — the claimant ends its claim at once, having
    stopped collecting; any other party with standing makes it unrenewable,
    and it still ends at its end. notCollectable. Retry class 2.
  - vta/collection/list/0.1 — who collects for each identifier of a context:
    the VTA, a claimant (with the end and whether renewable), or nobody, with
    the reason (limit, conflict).



### Specifications

- **vta/collection**: Category notifications, a disabled reason, and the 24-hour lease (#1024)

Follows dtgwg-vti-spec#97's VTI-NTF-093 and lease recommendation.

  - Category messaging -> notifications for claim, release and list.
  - NotCollectedReason gains `disabled` (appended): the operator has not
    enabled collection by default (VTI-NTF-093). Claims are still granted,
    renewed, released and listed while it is off; notCollectable and list
    scope now refer to the identifiers VTI-NTF-080 names whether or not it is
    enabled, and a claimant's release resumes collection only where enabled.
  - Claim: a producer SHOULD request 86400 seconds and renew at half the
    period granted; the VTA's stated maximum SHOULD NOT exceed thirty days.

  Rust, TS, Go and Dart bindings regenerated.

## 0.9.20 — 2026-11-03


### Added

- **vtn**: The Verifiable Trust Network's own Trust Tasks (vtn/*/0.1) (#1020)

A Verifiable Trust Network curates communities and networks into a
  nestable published set (VTI-VTN-001 - 063). Its API was interim REST;
  these are its Trust Tasks:

  - a listed party toward the network: entries/show, membership/{offer,
    acknowledge,repudiate}, groups/register, relay/deliver, roles/list,
    rooms/{list,pass}
  - reading the published set: directory/{search,federated,summary},
    graph/{show,member}
  - administration: listings/{create,update,suspend,reinstate,remove,list,
    show}, audit/list, groups/list, broadcasts/{send,list,show},
    outbox/{list,retry}
  - the network as a member of a parent: memberships/{list,join,repudiate,
    leave,groups/add}
  - vtn/_shared/0.1: entry, directory, graph, groups, relay, membership and
    rooms shapes

  Trust Tasks are the API; DTG credentials remain the evidence: lists/1,
  the VMC pair, repudiates/1, and a revocable groupRegistered acceptance
  binding the signed vtn/groups/register request by digest. Browser
  sign-in reuses auth/oob/* and auth/signing-key/*. Short lifetimes for
  anything not revocable: room passes at most 24 hours, broadcasts at
  most 7 days.

- **vetting**: Recover or reset a hidden-vetting key (vtc/vetting/vetters/pcs-binding/{show,reset}/0.1, vta/pcs/vetter/recover/0.1) (#1018)

* feat(vtc,vta): recover or reset a hidden-vetting key — vtc/vetting/vetters/pcs-binding/{show,reset}/0.1 and vta/pcs/vetter/recover/0.1

  A community binds each vetter to one PCS identifier for good, so a vetter
  whose VTA engine lost track of its key was locked out of hidden vetting
  for as long as they stayed a member; revoking and re-granting the vetter
  role does not touch the binding.

  - vtc/vetting/vetters/pcs-binding/show/0.1: a member reads their own
    binding, and the state of any reset.
  - vta/pcs/vetter/recover/0.1: the VTA searches the keys it can derive for
    the context and restores the bound one, while the engine's current key
    holds nothing a community issued.
  - vtc/vetting/vetters/pcs-binding/reset/0.1: an administrator resets the
    binding when the key is gone. A new identifier is bound only once no
    label the old one enrolled under is live, and a retired label never
    comes back, so the member is never counted twice.

  pcs-root/0.1 and the vta/pcs conventions point at the new tasks.

- **notifications**: Deposit to the VTA, the community's received notifications, and four fixes (#1017)

Follow-up to #1012, implementing dtgwg-vti-spec's Notifications follow-up
  (VTI-NTF-006, 025, 034, 047, 054 – 056, 070 – 074).

  New tasks:

  - vta/notifications/deposit/0.1 — a client that receives a notify or relay
    for a DID whose keys its VTA holds (a persona DID whose mediator socket the
    client holds) hands the document to the VTA exactly as received. The VTA
    verifies it exactly as a direct delivery, counts for acceptance the party
    that delivered the document and never the depositing client, keeps one
    record per originator and notificationId, and answers with the same
    receipt whatever it did. Gated by standing in the context; retry class 2;
    notAddressedHere, documentInvalid.
  - vtc/notifications/list/0.1, show/0.1 — a community's administrators read
    the notifications addressed to the community (vtc.notifications.read).
  - vtc/notifications/relay/0.1 — an administrator's decision to relay one to
    all members, one group's or one role's (vtc.notifications.relay, qualified
    by group), refused until the disclosure is acknowledged; never to the
    community itself, the originator or any earlier relayer; later revisions
    follow without a further decision. destructive; retry class 2.

  Fixes to notifications/notify/0.1 and notifications/relay/0.1, prose only
  (no schema, error code or proof requirement changes):

  - relay producer item 2 now agrees with consumer item 2: a relayer may relay
    only where its relay makes at most four relays in all.
  - notification and relay proofs carry proofPurpose authentication with the
    operational key (VTI-KEY-081/082); examples updated from assertionMethod.
    VTI-KEY-107 applies to the relay received, not to forwarded layers.
  - a withdrawal keeps the period of the revision it replaces.
  - the originator re-checks VTI-NTF-005 for every revision it issues, except
    a withdrawal; election withdrawal covers an early close.
  - a relayer never relays to the originator or an earlier relayer.

## 0.9.19 — 2026-11-03


### Added

- **git-ns**: Bridge event 0.6 and job 0.7 — orphaned namespaces, and binding an owner again (#1015)

## 0.9.18 — 2026-11-03


### Added

- **vtc/networks**: A community's side of Verifiable Trust Network membership (#1014)

A Verifiable Trust Network (VTN) curates communities and networks
  (dtgwg-vti-spec 11a-vtn.md, VTI-VTN-001..063). Listing is one-sided;
  membership is the listed party's acknowledgement. These tasks are the
  community's side of that relationship.

  - vtc/networks/list, vtc/networks/show (0.1): the community's network
    memberships, with the credentials that evidence each state.
  - vtc/networks/join (0.1): accept a listing. The community checks the
    network at the URL is the DID the administrator named, verifies the
    listing, registers groups for the network's roles (required ones first),
    signs the acknowledgement as itself and verifies the grant.
  - vtc/networks/repudiate (0.1): reject a listing; the network publishes
    the repudiation beside it (VTI-VTN-012).
  - vtc/networks/leave (0.1): depart by repudiating the grant (VTI-VTN-053).
  - vtc/networks/groups/register (0.1): nominate a community group to serve
    a network role. The community is the registrant; the network learns
    the group's DID, never its members.
  - vtc/networks/notice (0.1): the community passes a network broadcast on
    to members of the registered group, carrying the network's signed
    envelope unchanged. Distinct from notifications/notify, which carries
    a reference and none of the content (VTI-NTF-003).
  - vtc/networks/evidence (0.1): a member gets the registration and
    acceptance linking their group to a network, to present for a network
    room pass.

  The shared NetworkMembership, RegisteredGroup and RoleAssignment records
  are in vtc/_shared/0.6/network-membership.schema.json. Rust, TypeScript,
  Go and Dart bindings are regenerated.

- **notifications**: A generic notification family — notify, relay, and the VTA's list, show and dismiss (#1012)

notifications/notify/0.1 tells the party a DID names (an individual,
  group, community or network) that a thing exists it, or those it stands
  for, may take part in: where it is held, its identifier there, the task
  that opens it, and the audience it concerns, with a kind from an
  extensible registry (`election` registered) and nothing of the thing's
  content. Signed by the originator; deduplicated by issuer +
  notificationId, with revision for change and withdrawal; the receipt is
  the same whatever the recipient did.

  notifications/relay/0.1 lets the addressee pass it on, carrying the
  received document whole and opaque so every signed member survives; the
  recipient verifies every layer, to at most four relays.

  vta/notifications/list, show and dismiss (0.1) are the principal's view
  of what its VTA holds, per context; show returns the delivering document
  as received.

  Implements VTI-NTF-001 – VTI-NTF-063 (trustoverip/dtgwg-vti-spec#91).

- **voting**: A member-party election's ticket claim says nothing to an unproven claimant, and a presentation outage is indeterminate whatever the election (vtc/elections/tickets/issue/0.3, vtc/elections/show/0.6, vta/pcs/voter/register/0.5, vta/pcs/voter/read/0.6) (#1011)

Follows trustoverip/dtgwg-vti-spec#90 (VTI-VOT-015 - VTI-VOT-018,
  VTI-VOT-025).

- **voting**: An election put to a member party's members is shown only to a reader who proves that membership (vtc/elections/show/0.5, vtc/elections/list/0.5, vta/pcs/voter/read/0.5) (#1010)

Follows trustoverip/dtgwg-vti-spec VTI-VOT-015/016. Under show/0.4 and
  list/0.4, an election whose credential roll names a member party was shown
  to any caller, so its title, questions and options were public.

  - vtc/elections/show/0.5 (extends 0.4): optional `presentation` in the form
    tickets/issue/0.2 carries (nonce = document id, domain/aud = community,
    in a document the holder signs). Considered only for a member-party
    election the caller cannot otherwise see, in a signed document within the
    acceptance window and used once; checked as tickets/issue/0.2 item 5.2 to
    5.4. Every failure, indeterminate included, is electionUnknown. Never a
    refusal for carrying one. maxDocumentBytes.request 98304.
  - vtc/elections/list/0.5 (extends 0.4): prose only. A member-party
    election is listed only to administrators and the party itself; to
    everyone else it is omitted, with no stub. A list never carries a
    presentation.
  - vta/pcs/voter/read/0.5: signs list/0.5 and show/0.5; optional
    `membershipCredential` (only with read = show, a dependentSchemas rule),
    which the engine presents from the vault in the show it signs. New code
    membershipCredentialUnusable.
  - CONVENTIONS.md: a Member-party election term, and the §4 row for
    list/show.

- **voting**: A credential roll, proven when the ticket is claimed (voting/_shared/0.4, vtc/elections/tickets/issue/0.2) (#1008)

* feat(voting): a credential roll, proven when the ticket is claimed (voting/_shared/0.4, vtc/elections/tickets/issue/0.2)

  Follows trustoverip/dtgwg-vti-spec#88 (VTI-VOT-021, VTI-VOT-022,
  VTI-VOT-025 – VTI-VOT-028, VTI-VOT-090).

  voting/_shared/0.4 appends `credential` to the roll rule, with `roll.memberOf`
  naming exactly one DID: the community, one of its groups, or a community or
  network it recognises. memberOf is required with `credential` and refused with
  every other rule, and `listed` is refused with `credential` (dependentSchemas,
  which the generators read as an optional member). Disclosure gains an optional
  RollCredentialDisclosure saying whose credential counts (VTI-VOT-090).

  Every task carrying the manifest takes a new version re-pinned to 0.4, and the
  signed reads name the newer reads. vtc/elections/tickets/issue/0.2 (extends 0.1)
  carries the membership presentation, bound to the document's id and the
  community, checked at issuance, with five appended error codes.
  vtc/elections/create/0.5 refuses an untrusted memberOf (rollUntrusted,
  rollIndeterminate). vta/pcs/voter/register/0.4 presents a vault credential in
  the ticket document it signs. vtc/elections/list/0.4 (extends 0.3) says who sees
  an election under a credential roll.

## 0.9.17 — 2026-11-02


### Added

- **vtc/rooms/community**: Archive, unarchive, remove and removal/cancel 0.1; get/list 0.4, members/add+list 0.2, memberships/list 0.2 (#1006)

A community could create a room it owns and change its members, but never
  close or end one. This adds an owner's lifecycle for a community-owned room,
  distinct from the host's computed clock (live/lapsed/dormant/reclaimable):

  - active
  - archived, with an access: readOnly (members keep reading; the host refuses
    every write, upload and curation) or noAccess (the community's VTA resets
    the room — one `room` withdrawal — and re-issues the community's own
    credentials; each admitted member becomes `suspended`). Open offers are
    withdrawn either way. Reversible: vtc/rooms/community/unarchive restores
    suspended members in the background by re-issuing their membership and
    authority, collected as an offers/list renewal.
  - removalScheduled: only from archived (removal is the second of two
    deliberate acts). The room is noAccess and its content kept until purgeAt,
    after a grace period (graceDays, or the community's configured default;
    30 days in the reference implementation). vtc/rooms/community/removal/cancel
    returns it to archived/noAccess.
  - purging: graceDays 0, or the grace period ended. Deletes every record,
    epoch link, commit, revocation list, file in every storage config, transfer
    and the host record; drops kept credentials and owed removals; unbinds
    groups; stops serving a serverless DID log; SHOULD have the VTA delete the
    room's DID (a refusal there does not hold the purge). Retried with backoff,
    progress in status.purge, never reversible.
  - removed: a tombstone with the host record as it last stood.

  A change part-way through at the VTA (a reset the host accepted before the
  community's own credentials were re-issued) is reported as
  status.pendingChange and finished by the community.

  New shared definitions in vtc/_shared/0.5 (RoomState, ArchiveAccess,
  RoomStatus, PendingChange, PurgeProgress, CommunityRoomV3, MemberStateV2,
  RoomMemberV2, RoomMembershipV2). get/list 0.4 carry status (list gains a
  `states` filter; removed rooms only when asked for); members/add 0.2 refuses
  a room that is not active (wrongState); members/list 0.2 and
  memberships/list 0.2 add `suspended`, and memberships/list the room's own
  state so member surfaces can say read-only, closed or going away.
  rooms/keys/act 0.10 adds vtc/rooms/memberships/list/0.2 to its table so a
  member's key holder can sign it.

  Rust, TS, Go and Dart bindings regenerated.

## 0.9.16 — 2026-11-01


### Added

- **voting**: TextReleaseThreshold may be as low as 1, disclosed below 10 (voting/_shared/0.3) (#1002)

* spec(voting): textReleaseThreshold may be as low as 1, disclosed below 10 (voting/_shared/0.3)

  VTI-VOT-065 now lets an election release text answers from a threshold of
  1: it MUST be at least 1 and SHOULD be at least 10, and below 10 the
  disclosure a member is shown before requesting a ticket MUST say so and
  that a written answer may identify its author (VTI-VOT-090). Small
  communities and surveys need written answers; the risk is disclosed rather
  than forbidden.

  voting/_shared/0.3 is 0.2 with Manifest.textReleaseThreshold minimum 1.
  Every task whose schema carries the Manifest gets a new version pinned to
  it: voting/election/register/0.3, voting/election/show/0.3,
  vta/pcs/trustee/{deal/0.4, enrol/0.3, justify/0.4, keys/0.4},
  vta/pcs/vote/0.3, vta/pcs/vote-encrypted/0.4, vta/pcs/voter/register/0.3,
  vtc/elections/create/0.4 and vtc/elections/show/0.3. The two signed reads
  that name a show version on the wire move with it: vta/pcs/trustee/read/0.3
  and vta/pcs/voter/read/0.3. Prose references in current specs follow.

## 0.9.15 — 2026-11-01


### Added

- **vtc/elections/ballot-boxes/list**: 0.1 reads the community's trusted ballot boxes (#998)

A manifest must name one of the community's trusted voting services in
  ballotBox (VTI-VOT-082), or vtc/elections/create refuses it with
  ballotBoxNotTrusted, but nothing let an administrator read that list: they
  had to ask whoever operates the community's host. VTI's console manifest
  builder could offer only a free-text DID field.

  vtc/elections/ballot-boxes/list/0.1 answers a holder of
  vtc.elections.manage with every trusted ballot box: its DID exactly as
  ballotBox must name it, and the operator and sameOperator members are shown
  for an election naming it (VTI-VOT-090), so the administrator choosing one
  sees what their members will. Read-only; the list stays host configuration,
  changed by no task.

  Rust, TS, Go and Dart bindings regenerated.

## 0.9.14 — 2026-10-30


### Added

- **vta/webvh/dids/update**: 1.1 lists or removes an existing log's did:scid alias (#992)

* feat(vta/webvh/dids/update): 1.1 lists or removes an existing log's did:scid alias

  Every did:webvh log is also a did:scid:vh:1 identity, but the two are distinct
  subjects until the did:webvh document lists the did:scid DID URL in
  alsoKnownAs (VTI-KEY-164). create/1.2 lists it in the first entry by default;
  a log created before it, or without it, had no way to get it but sending a
  whole replacement document with a hand-built DID URL — and the src encoding
  (`%3A` for a port, `/` for each `:`) is exactly what a client gets wrong.

  1.1 adds `scidAlsoKnownAs` (boolean): true lists the DID URL for the log's
  current location (replacing one left from before a move), false removes
  every did:scid entry naming this SCID. The agent derives the DID URL; the
  caller never writes it. The response reports `scidAlsoKnownAs` and
  `scidDidUrl`. New codes: `scidAliasRequired` (removing the alias of an
  identity the agent presents as did:scid, VTI-KEY-166) and `noChange` (an
  entry identical to the latest is not published, so the update key is not
  rotated for nothing).

  This is the pre-key-role path: vta/dids/update refuses every identity created
  before key roles (notKeyRoleIdentity), which is every did:webvh DID that
  exists today. Only adds, so 1.1 extends 1.0. All four bindings regenerated.

- **peer-check**: Peer-check/ask 0.1 — a signed yes or no from a peer's own device (#973)

* feat(peer-check): peer-check/ask 0.1 — a signed yes or no from a peer's own device

  One party asks another a short question, such as "did you just ask me
  for this?", over a channel they already share. The responder's device
  prompts its user and returns a signed answer that echoes the challenge
  and carries the task digest of the request, so the asker can verify it
  against the responder's DID and it cannot be replayed against another
  ask.

  Draft. Specification, schema, negative examples and the regenerated Rust,
  TypeScript, Go and Dart bindings.

- **rooms/keys/act**: 0.9 completes rooms/records/put/0.4 (#990)

rooms/records/put/0.4 (#987) lets a record on an open room carry a file,
  its description and random file key in cleartext.file. 0.8's table is
  closed (Recipient item 1) and names records/put at 0.1–0.3 only, so a
  conforming key holder refuses 0.4 with typeNotActable: a member whose keys
  their key holder holds could upload an open room's blob (upload/begin/0.2
  is in the table) but never write the record that names it, and the blob
  would sit unreferenced until collected.

  0.9 adds rooms/records/put/0.4 to the put row, write, presentation
  inserted, completed exactly as 0.3. The key holder neither mints nor checks
  the file key: the producer mints it (put 0.4 Producer item 7), the host
  checks the record (Host item 12), and the key holder signs what it was
  handed unaltered (Recipient item 6).

  Schemas are 0.8's and no error code is added; the table only widens, so
  0.9 declares extends: "0.8" and every spec naming 0.8 stays current. All
  four bindings regenerated.

## 0.9.13 — 2026-10-30


### Added

- **rooms/records/put**: 0.4 lets an open room's record carry a file (#987)

An open room could hold notes and nothing else. A file's description
  (FileManifest) could only travel inside a sealed record body, and its key could
  only be derived from the room's per-epoch storage key, and an open room has
  neither: no sealed body, and no group to derive a key from. rooms/blobs/* never
  cared about the tier; the gap was entirely on the record side.

  - rooms/records/put/0.4 (extends 0.3): cleartext gains an optional `file`, a
    FileManifest, and `blobs` may name that file's blob. On an open room the
    host, which can read the record, refuses with malformedRequest a record whose
    `blobs` is not exactly `cleartext.file.blobRef`, names a blob without a
    `file`, or whose `file` carries no `fileKey`.
  - rooms/_shared/0.1 FileManifest gains an optional `fileKey` (additive): on an
    open room the uploader mints the file key at random for that one file and
    carries it there; it MUST be absent from a sealed body, where the key is
    derived. Padding, STREAM, associated data and reading are unchanged, so
    there is one blob format and one reader for every tier. The key in the clear
    withholds nothing from the host, which the open tier never did, but keeps
    the stored blob opaque to a party holding only the storage.
  - The descriptions of `name` and `digest` now say what holds on each tier.

  All four bindings regenerated.

- **vtc/admin/invites**: Create and list 0.2 — passkey invitations, with did optional (#989)

* feat(vtc/admin/invites): create and list 0.2 — passkey invitations, with did optional

  vtc/admin/invites/create 0.1 required `did`, so an administrator could invite
  only someone who already held a DID in a wallet; a passkey invitation — whose
  administrator DID is derived from the passkey registered at
  vtc/install/claim/passkey/finish 0.1 — could be issued only by the operator on
  the host.

  create 0.2:
  - `did` is optional. Present: a wallet invitation (claimed with
    vtc/install/claim/{start,finish} 0.3). Absent: a passkey invitation.
  - `did`, when present, must begin `did:`.
  - Both kinds confer the same authority: an administrator entry with the
    community's full administrative authority, delegated by the inviter,
    labelled with `label`. A wallet invitation writes it now for a DID with no
    entry; a passkey invitation records it with the invitation and
    passkey/finish item 9 writes it for the derived DID.
  - A passkey invitation costs what an entry-writing wallet invitation costs
    (the inviter's step-up, another administrator's consent): at its claim no
    administrator is present to authorise it, so issuing it is the grant.
  - The response gains a required `kind` (`wallet` | `passkey`);
    `aclEntryCreated` is always false for a passkey invitation.

  list 0.2 (with vtc/_shared/0.2/invite): each InviteSummary gains a required
  `kind`; a passkey invitation's `targetDid` is absent until it is consumed, then
  the derived DID.

  revoke is unchanged (it takes only `jti`); its prose now links the 0.2
  versions.

- **vta/dids**: Method-neutral key-role family, creation and recovery, with per-method profiles (#986)

* feat(vta/dids): method-neutral key-role family, creation and recovery, with per-method profiles



### Specifications

- **vtc/install/claim/finish**: An invitation binds the approver at finish, not at bootstrap (#988)

vtc/install/claim/finish/0.3 item 6 made the founder's approver binding wait
  for vtc/admin/bootstrap. On an invitation of an additional administrator —
  a token naming a DID whose administrator entry the invitation already wrote —
  bootstrap answers alreadyBootstrapped, so the binding never took effect and
  the invited administrator could answer no step-up.

  New item 7: when the community already has an administrator, the binding
  takes effect at finish, with the claim, as vtc/install/claim/passkey/finish
  0.1 item 9 already writes an invited administrator's entry at finish. The
  fresh-community rule (bind at bootstrap, discard if the token lapses) is
  unchanged. Prose only; no schema change.

## 0.9.12 — 2026-10-29


### Added

- **auth/step-up/approve-response**: 0.7 — the approver's own passkey assertion is evidence on its own (#983)

0.5 and 0.6 required the approver's framework proof on every answer, and 0.6
  counted a console key's proof as the administrator's it acts for, while
  auth/signing-key/enroll/0.2 item 7 forbids a delegated key's proof on any
  approve-response. A console administrator therefore had no conforming answer:
  the console holds only a delegated key.

  0.7 resolves the contradiction in enroll's favour:
  - a delegated key's proof never counts as the approver's, for any gate
    (subjectMismatch);
  - approval evidence is the approver's own: a proof by its own DID
    (didSigned, approverSigned, every denial), or a user-verified WebAuthn
    assertion by a passkey the relying party registered to the approver, over
    the step-up challenge it bound to this approval — with no proof required;
  - an answer with no proof decides only what the assertion and the relying
    party's own state determine (decision approved, subject, session, bound
    operation, acr from the assertion; grantedAcr and ext ignored), and the
    pending step-up is taken atomically, since an unproved document's id is the
    sender's to choose;
  - proofRequirement is RECOMMENDED, conditional per gate;
  - assertionInvalid reasons are lowerCamelCase, adding userNotVerified and
    credentialNotAccepted.

  auth/signing-key/enroll/0.2's reference design (step 2) now answers with 0.7,
  unsigned, naming identityDid as issuer. Prose references to 0.6 elsewhere move
  to 0.7 (npm run fix-stale-references); approve-request/0.5 no longer says the
  answer is always signed.

- **vtc/members/role/set**: A community role moves with its own task (#982)

vtc/members/role/set/0.1 moves a subject's community role — member,
  moderator, issuer, admin, or a custom role — with a compare-and-swap on
  fromRole. Until now the community role moved with acl/change-role/0.1, which
  0.2 superseded with a purely administrative-role move; the community role,
  its role credential and the community's role-change policy are the VTC's
  notions, not the access-control list's, so they get a VTC task and
  acl/change-role stays the generic operation.

  What the move carries:
  - the administrative authority the community role implies, when the entry's
    authority is exactly the one its current role implies; authority stated on
    its own (acl/grant or acl/update 0.2) is kept and only the role moves;
  - a role credential (role VAC) re-issued at the new role for a subject with a
    member record, returned as roleVac and delivered best-effort.

- **vtc/install/claim/passkey**: Passkey install ceremony under its own slug (#976)

* feat(vtc/install/claim/passkey): passkey install ceremony under its own slug

  vtc/install/claim/{start,finish} 0.3 is a different ceremony from 0.2: the
  founder claims under a DID they already control, with a step-up approver.
  Because 0.3 is a later version of the same slug, it superseded 0.2, and the
  passkey install was scheduled for withdrawal with it, although 0.3 itself
  says "a passkey install keeps using 0.2".

  The two cannot share a 0.4: finish 0.3 declares `proof` REQUIRED (the
  founder's signature as their DID is the proof of control, enforced from the
  front matter by every consumer), and the passkey ceremony cannot carry one
  (the installer holds no key until the passkey exists). One version serving
  both would leave the wallet ceremony's proof requirement to prose.

  So the passkey ceremony moves, wire-unchanged, to
  vtc/install/claim/passkey/{start,finish}/0.1, and each ceremony versions on
  its own:

  - Payloads are 0.2's; framework Ext re-pinned to 0.4; Security & Privacy,
    Authorization, Request/Response examples written to current conventions.
  - passkey/finish declares the response `proof` REQUIRED and
    `discloses: secret`: it carries the bearer setup-session token (as
    finish 0.3's does). 0.2 declared neither.
  - vtc/install/claim/{start,finish} 0.2 are retired with `supersededBy`
    naming the passkey slug, so their deprecation note points at the right
    successor rather than at the wallet ceremony.
  - start 0.3's `tokenNamesNoDid` and the 0.3 change notes point at the new
    slug instead of 0.2.

  Also fixes persona/contact/put 1.2's Status paragraph, which still carried
  1.1's "Version 1.0 remains in force; 1.1 is a backwards-compatible MINOR
  increment".

- **rooms/owner/group/remove**: 0.4 — remove a private room's member from the group (#975)

0.2 and 0.3 refused a `private` room with `privateRoom` and sent the owner
  to 0.1, which the spec called the way to remove a member of a private room.
  0.1 is superseded all the same, so once it is withdrawn no task could take
  a private-room member out of the MLS group.

  0.4 takes that tier in. On an `attributed` room it is 0.3: withdraw the
  member's credentials at the host first, then advance. On a `private` room
  it removes the leaf and advances the room, builds no revocation list (the
  host cannot be told whom to refuse), and answers `withdrawn: false`.

  - `privateRoom` is no longer declared.
  - `signingKeyId` is optional; still required (as `noSigningKey`) on any
    room that is not private.
  - The response gains a required `withdrawn` boolean; `revocation` is
    present only when it is true.
  - Item 8 (settling a pending removal) is 0.3's, with a pending removal that
    carries no list settled by its step 3 alone.
  - A producer MUST NOT report `withdrawn: false` as a withdrawal.

  Prose links to remove/0.3 in neighbouring rooms specs move to 0.4
  (fix-stale-references). vtc/rooms/community/members/remove 0.2 and
  rooms/owner/revoke 0.1 are clarified in place to allow 0.4 for a private
  room alongside 0.1.



### Fixed

- Stamp issuedAt in whole seconds in every library (#970) (#979)

Rust wrote `issuedAt` with `Utc::now()`, which chrono serializes at the
  clock's precision: nine fractional digits on Linux, on every error response
  `dispatch_or_reject` / `reject_with` built and every response `respond_with`
  built. TypeScript wrote milliseconds with `toISOString()`. SPEC §13.1 item 3
  calls a full-precision timestamp a fingerprint and lets a producer round
  `issuedAt` down; readers with a narrower RFC 3339 profile refused these
  documents.

  Every instant the framework stamps is now UTC in whole seconds, rounded
  down, the form Go (`SystemClock`) and Dart (`systemClock`) already wrote:

  - trust-tasks-rs: new `issued_at_now()`, used by `reject_with_recipient`,
    `respond_with` and the dispatchers' error responses.
  - trust-tasks-https: the client's default `issuedAt` and the server's two
    fallback error documents.
  - trust-tasks-capability-client: built documents and `new_attempt`.
  - @openvtc/trust-tasks: new `issuedAtNow()`, the default clock of
    `rejectWithRecipient` / `rejectWith` / `respondWith`.
  - @openvtc/trust-tasks-capability-client: built documents and `newAttempt`.

  A whole second is inside every acceptance window a consumer applies, so no
  specification has to opt in; the minute or hour §13.1 permits is left to a
  specification that states its skew. Rounding down never future-dates a
  document (§7.2 item 13). Readers are unchanged and keep accepting any RFC
  3339 precision and offset; received documents are never re-stamped, so
  proofs and item-11 digests are unaffected.

  The same vectors are pinned in all four libraries (tests/issued_at_precision.rs,
  test/issued-at.test.ts, trusttasks/issued_at_test.go,
  test/issued_at_test.dart): what is written, and that 0, 1, 3, 6 and 9
  fractional digits and an offset are all read.



### Specifications

- **vta/contexts/delete**: 1.1 bounds its acceptance of 1.0 by 1.0's withdrawal (#981)

"A 1.1 consumer MUST still accept 1.0" read as a perpetual obligation. It
  only restated SPEC §5.2, which requires an implementation at M.N to accept
  every earlier MINOR, and §5.3.3 ends that obligation when the earlier version
  is withdrawn. 1.1 now says so: until 1.0 is withdrawn a 1.1 consumer accepts it
  and answers it as 1.0 specifies; afterwards it MAY refuse it with
  unsupportedVersion, and a producer MUST NOT send it.

  An in-place prose edit: 1.1 is a draft, and the change touches neither its
  schema nor its front matter (SPEC §5.2, editorial changes to a draft).

- **vtc/install/claim/passkey**: Prove possession of the passkey before any authority (#978)

A WebAuthn registration under the `none` attestation format, which most
  platform passkeys use, signs nothing: it names the key the administrator's
  DID is derived from but does not prove the installer holds it. The passkey
  install now proves possession before a setup-session token is minted or any
  administrator entry is written, whatever the attestation format.

  - start's response adds `assertionOptions` (required): request options with
    a second fresh challenge (>=128 bits, distinct from the creation
    challenge, single-use, bound to the registration), no allowCredentials.
  - finish's request adds `webauthnAssertion` (required): one
    navigator.credentials.get() assertion by the just-registered credential
    over that challenge. finish verifies it as WebAuthn §7.2 does, against
    the registered key (EdDSA/Ed25519 or ES256/P-256), before consuming the
    token, minting the setup-session token or writing an entry. Refusal:
    new declared code `possessionUnproven`, nothing consumed.
  - For an invitation of an additional administrator (the community already
    has one, so vtc/admin/bootstrap answers alreadyBootstrapped), finish
    writes the entry for the derived DID, after the assertion. On a fresh
    community bootstrap writes it from the setup-session token, as before.
  - Security & Privacy: possession is proven before any authority is granted.

  Tightened in place: both versions are `draft` and unreleased (merged in
  #976, held in Release PR #977), so SPEC §5.2's draft allowance applies and
  check-published-schemas has no released copy to compare against.
  vtc/admin/bootstrap/0.1 is unchanged and keeps working for its other caller,
  vtc/install/claim/finish 0.3: the assertion rides in finish, not bootstrap,
  because bootstrap never runs for an invited administrator.

  VTI note: `vtc admin invite` will have two kinds — `--did X` uses the 0.3
  wallet ceremony (token names X); no `--did` is a passkey invite, whose ACL
  entry is written from the derived DID at finish (or at bootstrap on a fresh
  community).

## 0.9.11 — 2026-10-28


### Added

- **vta/contexts/delete**: 1.1 — settle host copies first; abandonHostCopies (#967)

1.0 removed every local record and reported a host copy it could not
  remove in daemonCleanupErrors after the fact, so the caller learned that
  DIDs were still resolving only once the records — and the keys that
  could remove them — were gone. A DID whose hosting server was no longer
  registered refused the whole deletion, with no way to complete it through
  this task: a context whose host had gone for good could not be deleted.

  1.1 settles every host copy before anything local is removed. A copy
  that cannot be confirmed removed (host down, refusing, unanswered, or no
  longer known to the VTA) refuses with the new
  vta/contexts/delete:hostCleanupFailed — retryable, details pinned by a
  detailsSchema listing each DID, its server and why — and nothing local
  is deleted. A producer shows those DIDs to a human and asks whether to
  wait for the host or retry with the new abandonHostCopies, which
  completes the deletion and reports the abandoned copies in
  daemonCleanupErrors. A host answering that it no longer holds the log has
  removed it, so a retry after a partial settle completes. The consumer
  SHOULD bound the time spent on hosts, in total, to answer within the
  requester's timeout.

  abandonHostCopies is an explicitness flag like force, not a privilege,
  and a producer MUST NOT set it on a first attempt without being told to.
  A 1.1 consumer still accepts 1.0 and answers it as 1.0 specifies.

  Regenerated the Rust, TypeScript, Go and Dart bindings; 1.0 is now
  marked superseded. The stale-reference fixer moved create/1.0 and
  preview-delete/1.0's links to 1.1.

## 0.9.10 — 2026-10-27


### Added

- **ts, go**: Trust-task-error/0.6 and the unsupportedExtension and indeterminate standard codes (#961)

* feat(ts)!: trust-task-error/0.6 and the unsupportedExtension and indeterminate standard codes

  Framework 0.7.0 added unsupportedExtension to the standard codes, and the
  indeterminate ruling (SPEC §8.3) adds indeterminate: the recipient could
  not decide because something it needed and does not control could not
  be obtained or checked, which is neither a refusal nor a failed
  verification. Both live in the closed code list of the trust-task-error
  payload schema, so they need a new version.

  - specs/trust-task-error/0.6: both codes appended to the enum, and
    extends: "0.5", which the build checks. References to 0.5 across the
    registry therefore stay current, and no other spec changes.
  - All four libraries recognize both codes and emit 0.6:
    trust_task_error_type_uri(), TRUST_TASK_ERROR_TYPE_URI,
    TrustTaskErrorTypeURI and trustTaskErrorTypeUri move together.
    Rust's StandardCode is hand-written (trust-task-error is in
    SKIP_SLUGS) and gains UnsupportedExtension and Indeterminate, with
    indeterminate retryable by default as §8.3 gives it.
  - The HTTPS binding's status table maps unsupportedExtension to 422 and
    indeterminate to 503, in the binding prose, trust-tasks-https and
    trust_tasks_https. The Dart package spells the new codes out so it
    still builds against a core that predates them.

  Breaking for @openvtc/trust-tasks only: StandardCode is a closed union
  there, so an exhaustive switch over it stops compiling. Rust's
  StandardCode is #[non_exhaustive] and Go's and Dart's are open string
  types, so the change is additive for them.

  Part of trustoverip/dtgwg-trust-tasks-spec#42.

- **registry**: Declare carried protocols in front matter (#959)

SPEC §9.4 Carried Protocols and §7.3 item 23 add a carries front-matter
  object naming the external protocol, version, defining reference and
  operation a specification carries.

  - spec.meta.schema.json accepts carries (all four members required).
  - The build records it in registry.json and checks §9.4 rule 1 across
    the tree: a slug carries one operation in every live version, and no
    two slugs carry the same operation of the same protocol version.
  - The spec page shows a Carries row linking the protocol.
  - registry/authorization/0.2 and registry/recognition/0.2 declare that
    they carry TRQP 2.0's authorization and recognition operations; their
    TRQP field-name sections now say so and name the party mapping.
  - CONTRIBUTING-SPECS gains "Carrying another protocol".
  - SPEC.md is regenerated from canonical main (bdf2cdd), which this
    needs for the §9.4 anchor.

  Part of trustoverip/dtgwg-trust-tasks-spec#43.



### Specifications

- **policy**: A consent decision carries its quorum and self-approval rule (#960)

policy/_shared/0.3 gave requireConsent.minApprovals a default of 1 and
  requireConsent.excludeRequester a default of false. JSON Schema defaults
  are annotations, but generators apply them, so a decision that omitted
  either sent a quorum of one, or let the requester approve itself, with
  nothing on the wire to show that nobody had chosen it. SPEC §6.6 rule 5
  now forbids a governance default in a shared schema component.

  - policy/_shared/0.4: both members required, defaults removed.
    task-consent/request/0.1 already requires both, so the policy decision
    and the consent request now agree.
  - policy/evaluate/0.4: policy/evaluate/0.3 re-pinned to the new
    component, which is its only change; the Security & Privacy section
    gains the four sub-headings a new version must carry.
  - policy/get/0.1 and policy/active/0.1 stay on 0.3: they use only
    PolicyModule, which is unchanged.
  - Stale references in task-consent prose moved to policy/evaluate/0.4.
  - Rust, TypeScript, Go and Dart regenerated. 0.1 to 0.3 of
    policy/evaluate are now marked superseded by 0.4.

  Part of trustoverip/dtgwg-trust-tasks-spec#39.

## 0.9.9 — 2026-10-27


### Added

- The prebuilt specification modules are deprecated in all four libraries; generate the tasks you use (#954)

Milestone 5 of docs/design/consumer-side-generation.md. Every prebuilt module
  outside the framework's own specifications (trust-task-*, trust-ceremony-*,
  _framework/) is deprecated, so that an application is told to pin the tasks it
  uses with the trust-tasks CLI and generate them instead. Nothing is removed:
  the withdrawal batch that removes them is chosen later, at least 90 days after
  this ships.

  Each library marks the prebuilt entry point where its toolchain reports a
  deprecation, never the module body, which is also what the consumer-side
  generators emit:

  - Rust: #[deprecated] on each gated family module in trust-tasks-rs's
    specs/mod.rs (trust-tasks-codegen). It reaches every path and item under the
    module; framework modules and trust-tasks-build's output are unmarked.
  - TypeScript: the root index re-exports each module as
    `import * as _X …; /** @deprecated … */ export import X = _X;`, because
    TypeScript ignores @deprecated on `export * as`. API compatibility against
    0.27.8: nothing removed or changed. A deep import of a module file is not
    marked; TypeScript has no module-level deprecation.
  - Go: a `Deprecated:` paragraph in each prebuilt package's doc, which gopls and
    staticcheck (SA1019) report on import.
  - Dart: @Deprecated on each prebuilt library's `library` directive, which the
    analyzer reports on import from another package. This package's own tests
    use them, so analysis_options ignores the same-package diagnostic SDKs
    before 3.4 report.

  The TS, Go and Dart equivalence tests mask exactly the mark; every consumer
  generator is still held to the prebuilt tree as before. Hand-written Rust that
  uses prebuilt modules on purpose — the acl-specimen test modules, the
  integration tests and the examples — carries #[allow(deprecated)]; none of the
  runtime does.

  GETTING-STARTED, the four library READMEs, the design doc and CLAUDE.md say
  so, and point at each language's generator.

- **dart**: Parse every SPEC §6.1 Type URI form, so third-party specs get extended codes and extends routing (#952)

The Dart runtime recognised only the public registry's Type URIs. A new
  `TypeUri` (exported from the barrel, with `TypeUriVariant`, `TypeUriError`
  and `TypeUriFormatException`) parses all three §6.1 forms — the registry's,
  the path form under any authority (https URL, DID URL, …) and the URN form —
  and refuses what trust-tasks-rs's `TypeUri` refuses: `http` in every form, a
  `urn:` without `:spec:` or without an NID and prefix, a malformed or reserved
  slug, and a version that is not MAJOR.MINOR.

  - `slugFromTypeUri`, and with it `extendedCode` / `familyCode`, accept every
    form; codes stay authority-free `<slug>:<local>`, as Rust's
    `TrustTaskCode::Extended` renders them. Invalid input still throws
    `ArgumentError`.
  - `extensionChain`, `extendsTo` and `readForVersion` walk a declared-extension
    chain with `TypeUri.withVersion`: same authority, form and slug, only the
    version changed, so a chain never crosses authorities and a third-party
    document never routes to a registry spec of the same slug.
  - Fixes `_splitVersion`, which split the version on the last `/` and so found
    none in a URN-form Type URI: a URN document was never routed forward.



### Documentation

- An extended error code is read against the Type URI it answers, in SPEC.md and all four libraries (#953)

SPEC.md is regenerated from canonical main (trustoverip/dtgwg-trust-tasks-spec
  #35): an extended code carries a slug but no authority, so a party resolves
  it against the Type URI of the document the error reports on
  (inResponseTo.typeUri, else the request correlated by threadId) and never by
  its string alone; the namespace is always the slug's /-form, and the local
  code contains no colon. The regeneration also brings in #33 (declared
  `extends`, and `withdrawn` after a version is superseded), which the site
  already serves.

  Each library's extended-code docs say the same where a reader meets it:
  Rust's TrustTaskCode::Extended, TypeScript's extendedCode, Go's ExtendedCode
  and Dart's extendedCode.

## 0.9.8 — 2026-10-26


### Added

- **dart**: Dart run trust_tasks:generate generates only the tasks an application pins (#942)

* feat(dart): generate only the tasks an application pins, with dart run trust_tasks:generate

  `trust-tasks-dart/bin/generate.dart` reads a trust-tasks.lock and the bundles
  vendored beside it, checks each bundle's SHA-256 against the lock, and writes
  one library per pinned version to <out>/<slug>/v<M>_<N>/payload.dart plus an
  extends_index.dart — the same API as the prebuilt lib/specs/ libraries,
  importing the runtime as package:trust_tasks. An independent port of
  scripts/build-dart-bindings.mjs with bundles as input; no new dependencies
  (SHA-256 and the lockfile reader are in-package).

  scripts/check-dart-consumer-generation.mjs runs it over every bundle in the
  registry and compares with the prebuilt tree: 458 libraries identical, 662
  identical declaration for declaration in a different top-level order (the
  bundle does not carry $defs order), every embedded schema JSON-equal.
  examples/consumer-app-dart pins the same three tasks as examples/consumer-app.

## 0.9.7 — 2026-10-26

## 0.9.6 — 2026-10-26


### Added

- **specs**: A version may declare it extends an earlier one, and every library reads the newer version as the older (#922)

* feat(specs): an extends declaration the build checks, and that keeps references along its chain current

- **codegen**: Superseded and retired spec versions are deprecated in every library, and withdrawn ones leave them in a quarterly batch (#921)

* feat(go): mark superseded and retired spec versions deprecated, and omit withdrawn ones

  Every generated request package now exports `const Deprecation` (empty for a
  current version, otherwise "superseded by <slug>/<M.N>", "retired; use
  <slug>/<M.N>" or "retired"), and a deprecated version's Payload and Response
  types carry a `// Deprecated:` paragraph. A version with `status: withdrawn`
  gets no package at all. The rule is derived independently from front matter,
  per SPEC "Superseded and Withdrawn Versions".

- **vault**: Vault/credentials/get 0.2 returns an SD-JWT or an mdoc, not only a JSON object (#919)

get/0.1 answered with `credential`, a JSON object, so a maintainer holding an
  SD-JWT (a compact string) or an mdoc (CBOR) had no conforming answer: it could
  neither return the body nor turn it into an object without destroying the
  proof the body carries.

  0.2 returns the body the way receive/0.1 accepts it: `credential` for a JSON
  object, `credentialBase64` for anything else, exactly one of the two, plus a
  required `format` (an OpenID4VC format identifier: ldp_vc, dc+sd-jwt,
  mso_mdoc, jwt_vc_json) so a consumer never sniffs the body. The request is
  unchanged. 0.1 is retired, superseded by 0.2; references to it now name 0.2.

- **build**: Fail the build on a reference to a superseded spec version, and bring every current spec up to date (#918)

A new check, scripts/lib/stale-references.mjs, run by the registry build:
  no latest, non-retired spec may name another spec at a version that spec
  has since superseded — in a schema, a description, a prose link or
  mention, or an example. `npm run fix-stale-references` rewrites the text
  references (links, mentions, descriptions; a lone version in a moved
  link's label moves with it; a link whose #anchor the new version lacks is
  left for a person) and lists what it will not touch: wire pins and
  normative values (they take a new version) and examples (their payloads
  must be checked against the new version). History is exempt — Changes
  from / "What 0.4 changed" sections, version runs ending at the latest,
  "since"/"added in", "or later" — and anything else deliberate goes in
  stale-references-allowlist.json with its reason.

  Run once over the registry, in place in draft specs:

  - 728 text references across about 280 files, mostly prose links.
  - Examples moved after checking their payloads against the new version:
    trust-task-discovery's supportedTypes, trust-task-error's inResponseTo,
    auth/oob/grant's directory listing, keys/derive-and-sign-document's
    authenticate document.
  - git-ns/bridge/event 0.5 says what it meant: a bridge that lists
    git-ns/bridge/job 0.5 or later (the closePullRequest kind).
  - Two allowlisted: rooms/records/get and list say a record written by
    rooms/records/put/0.1 has no blobs.

  New versions, because the reference is a value an implementation emits —
  the `expects` entry of a deferred operation's next-step named
  vtc/admin/actions/show/0.2, which #907 superseded:

  - vtc/elections/create 0.3, vtc/rooms/storage/assign 0.2,
    vtc/rooms/storage/migrate 0.3, vtc/storage/configs/create 0.3,
    retire 0.3 and update 0.3, vtc/storage/credentials/set 0.3.

- **vtc/storage**: Vtc/storage/configs/retire 0.2 — refuse while a migration targets the config (#912)

* feat(vtc/storage): vtc/storage/configs/retire 0.2 — refuse while a migration targets the config

  retire 0.1 refused only while rooms were assigned to the config or it was
  the default. vtc/rooms/storage/migrate 0.2 fixes a migration's toConfig at
  approval and keeps it across a reassignment, and a migration can wait
  queued or paused. So: start moving room R onto config X, reassign R to Y,
  retire X (no rooms assigned) — X leaves `active` and the move into it can
  never finish; it pauses with cause `destination` and retries forever.

  0.2 adds vtc/storage/configs/retire:migrationsInbound (retryable: false,
  details.migrations ≥ 1): the community MUST refuse while any migration
  whose toConfig is the config is queued, running or paused, MUST make the
  check again when a parked retirement executes, and MUST make the check and
  the state change as one step with respect to a migration starting.
  Migrations off the config are unaffected. The converse order is already
  closed by migrate 0.2's configNotActive and assign's refusal of a
  non-active config. A community serving 0.1 beside 0.2 SHOULD apply the
  same refusal to a 0.1 request, reporting it with the same slug-namespaced
  code (SPEC §8.5); a 0.1 producer falls back to taskFailed.

  Request and response schemas are unchanged from 0.1.

- **specs**: Re-pin auth/step-up/start to approve-request 0.5, and point stale version references at current versions (#913)

A sweep of the latest non-retired version of every spec for references
  to another spec at an older version than that spec's latest.

  The one wire pin, in a new version:

  - auth/step-up/start 0.2: approveRequest.type is now the const
    auth/step-up/approve-request/0.5 (was 0.3, two versions behind: 0.4
    made `accepts` required, 0.5 renamed `ttl` to `ttlSeconds`).

  Editorial, in place, in draft specs — schema descriptions that named a
  superseded version of another spec:

  - device/heartbeat 0.3 (device/wipe), git-ns/bridge/job/list 0.4
    (bridge/job, bridge/result), persona/contact/get and list 1.1
    (contact/put), rooms/records/get 0.2 (records/put), sync/event 0.2
    (vault/sync), task-consent/decision 0.2 (vtc/admin/actions/list,
    approve-response), vault/get 0.3 (vault/release), vault/sign-trust-task
    0.2 (acl/grant, proxy-login), vault/sync 0.3 (sync/event — its "same
    shape as sync/event/0.1" was wrong: 0.1 uses sync/_shared/0.1, vault/sync
    0.3 uses 0.2), vtc/join-requests/submit 0.3 and wallet-session/start
    0.1 (manifest).

  Examples that carried an old Type URI as sample content, in invalid
  examples and the matching spec.md examples: git-ns/projection/show,
  rooms/keys/act, trust-ceremony-receipt, trust-task-discovery, and the
  four vtc/admin/actions specs — whose embedded acl/grant payloads also
  gain the `act`, `keys` and `capabilities` acl/grant 0.2 requires.

## 0.9.5 — 2026-10-24


### Added

- **rooms**: Rooms/keys/act 0.8 — records prune and retention set (#910)

rooms/keys/act's table is closed, so a key holder conforming to 0.7
  refuses rooms/records/prune (0.1, 0.2) and rooms/records/retention/set/0.1
  with typeNotActable. A member's own client therefore cannot make either
  request as the member: it falls back to rooms/keys/present and signs with
  its own key, and the host records the client key, not the member, as the
  administrator who changed the room's retention or pruned it.

  0.8 adds both to the table with action admin and a presentation inserted,
  as their own specifications require. Both name a room, so the member is
  resolved by Recipient item 2; the private-room same-subject binding and
  the producer's REQUIRED reason are already covered by Recipient items 4
  and 6. No error code is added and the schemas are 0.7's.

  rooms/epoch/prune stays absent. The table section now states why a
  records prune may ride act and an epoch prune may not: a records prune
  removes only what the room's record retention (stated in advance,
  readable by every member, set only by admin via retention/set) already
  made eligible, leaves the room's keys and its ability to read its past
  intact, carries a reason and is recorded; an epoch prune destroys key
  history to an epoch the request names, against no rule stated in
  advance.

  A MINOR increment per SPEC §5.2; 0.7 remains published. Bindings
  regenerated for Rust, TypeScript, Go and Dart.

- **build**: Lint that every instant is named <event>At, and rename serverTime to respondedAt (#911)

The wire-conventions lint gains an `instant-name` rule: a `date-time`
  string is named `<event>At`, unless it is a bound or filter (`since`,
  `until`, `from`, `to`, `asOf`, or a name ending in Until/Since/From/After/
  Before/To) or the Data Integrity `created`. `timestamp-name` also refuses
  a bare `at`.

  Two specs had their own non-At instant, renamed in new versions:

  - device/heartbeat 0.3: serverTime -> respondedAt
  - messaging/ping 0.2: serverTime -> respondedAt (and its Security &
    Privacy section now carries the four sub-headings)

  External vocabularies are allowlisted with their source: TRQP's `time`,
  `time_evaluated` and `time_requested` (registry/*), and did:webvh's
  `versionTime` (did/log) and `latestVersionTime` (did/info).

  Depends on #760, which renames the git-ns bare `at` members this rule
  also catches.

- **specs**: Naming consistency — event instants, one spelling per concept, durations name their unit, in 69 new spec versions (#907)

* feat(build): the wire-conventions lint requires an integer duration to name its unit



### Specifications

- **git-ns**: Nested repository paths for forges with subgroups (#760)

* spec(git-ns): copy each affected version to its next version, unchanged

  Mechanical copy so the next commit shows only what the new versions change.

## 0.9.4 — 2026-10-23


### Added

- **specs**: One spelling for instants, pagination, since and created, in 74 new spec versions (#901)

* spec(auth/oob, vtc/auth): instants are RFC 3339 strings named *At

## 0.9.3 — 2026-10-23

## 0.9.2 — 2026-10-22


### Added

- **vault/credentials/present**: Present a held credential to a verifier the client names, bound to its nonce, disclosing only the named claims (#871)

* feat(vault/credentials/present): present a held credential to a verifier the client names, bound to its nonce, disclosing only the named claims

  A client that holds no holder key (a browser wallet, a phone) could not
  carry a presentation to a verifier that takes one inside its own request
  document, such as vtc/directory/card/present: credential-exchange/present
  answers only a verifier that asks the agent. The maintainer now signs as the
  credential's subject for a named audience and nonce, refuses over-disclosure
  of a credential without selective disclosure (wouldOverDisclose), records the
  disclosure before answering, and conflates custody failures as notFound.

- **vta/webvh/dids/create**: 1.2 chooses the did:scid identity and the alsoKnownAs alias (#862)

* feat(vta/webvh/dids/create): 1.2 chooses the did:scid identity and the alsoKnownAs alias

  A did:webvh log can be presented as its did:scid:vh:1 DID, an identity
  the same log verifies but which does not name its host (VTI-KEY-160 to
  VTI-KEY-166 of the VTI specification). 1.2's request may name `didMethod`
  (`webvh` or `scid`) and `scidAlsoKnownAs`. Each falls back to the VTA's
  configuration, then to `webvh` and `true`.

  The response REQUIRES both as applied, plus `webvhDid` (the log) and
  `scidDidUrl` (the did:scid with the `src` a recipient needs to resolve it).
  `did` is the identity in the applied form, the bare did:scid DID when
  `didMethod` is `scid`, and the family's other tasks accept either `did` or
  `webvhDid`.

  Asking for the did:scid identity when the alias cannot be listed (refused by
  the request, forbidden by the template, or absent from a supplied log) is
  refused with the new `scidAliasRequired` code.

  1.2 is a new version because 1.1's response refuses unknown members. The
  request members are additive, but a 1.1 client would refuse the response.

  Bindings regenerated (Rust, TS, Go, Dart).

- **rooms/records/prune**: A room states how long it keeps its records, and an administrator prunes what that rule makes eligible (#868)

Records only had room-level lifecycle: a room could not keep a meeting note
  for a month and a decision for years, and nothing ever removed a retracted
  record's tombstone.

  - rooms/_shared/0.1 (in place, additive): RecordRetention ({tombstoneDays,
    recordDays}, by host-visible updatedAt; pinned records exempt from
    recordDays; absent or {} means nothing is eligible) and RecordPrune (the
    mark a prune leaves: version, time, count — never which records or what
    they said). HeadVersion answers the note it carried about erasure: a prune
    takes the next version, and on a pruned room headVersion is the higher of
    the highest record version and the latest prune's, so two roots straddling
    a prune never share a version.
  - rooms/records/prune 0.1 (new): admin only, never curate; kinds retracted |
    aged; required reason; dryRun. Removes eligible records outright, releases
    their blobs, records lastPrune. noRetention when the room states no rule
    for the kinds asked. Hosts never prune on their own. Audit keeps the mark,
    reason, requester where the tier discloses one, and the retention applied,
    never content; a reader holding an earlier listing can check every
    disappearance against the reported retention.
  - rooms/records/retention/set 0.1 (new): admin replaces the retention whole,
    with a reason and an optional expectedRetention precondition.
  - rooms/create 0.2: recordRetention at creation. rooms/info 0.2: serves
    recordRetention and lastPrune to every reader. rooms/records/list 0.3 and
    rooms/keys/browse 0.2: lastPrune on every page; a reader synced below it
    rebuilds and drops what is gone.
  - rooms/records/curate 0.1: editorial pointer to the prune.
  - Rust, TypeScript, Go and Dart bindings regenerated.

  Refs OpenVTC/verifiable-trust-infrastructure#2118.

- **vtc/rooms/storage**: Migrate 0.2 and rooms/get 0.2 — reassignment, retry ownership, resume without re-consent, orphans, pause reporting, queueing (#867)

migrate/0.2 fixes a migration's toConfig at approval and keeps it if the room
  is reassigned (new uploads go to the new assignment; a second migration may
  follow); makes the community the one retry owner (capped exponential backoff
  with jitter before `paused`); answers a request for a room with a migration in
  progress at once, resuming a paused one, without parking it for consent again;
  does not migrate or count orphaned blobs; reports `pausedCause`, `lastError`
  and `nextAttemptAt`; and permits one migration at a time with a `queued`
  state.

  rooms/get/0.2 reports the same migration through the new shared component
  vtc/_shared/0.1/room-migration, and adds the required `toConfig`.

- **vtc/storage**: Ambient and sealed storage auth — roleArn, room-scoped credential rules, sealed cleartext schema, host gating (#866)

vtc/_shared/0.2/room-storage adds `roleArn` to `ambient` auth (S3 only), the
  `notAllowed` credential status, and `SealedStorageCredential` — the cleartext a
  `sealed` credential is carried in (s3 access-key pair with optional session
  token, gcs service-account key file, walrus suiprivkey), naming its configId
  inside the seal.

  vtc/storage/configs/{create,update,get,list}/0.2 re-pin to it. create/0.2 makes
  room-scoped credentials normative for ambient (and sealed where the backend can
  mint them): one per (config, room), limited to the room prefix
  `rooms/<32 hex of SHA-256(roomId)>/` and the object actions needed, built by a
  JSON encoder / CEL builder and never by interpolation, at most 15 minutes,
  never used for another room. A host MAY refuse ambient or sealed by
  configuration (`authNotAllowed`, now with `details.reason`), and SHOULD label
  sealed as offering no protection against the machine's operator.

  vtc/storage/credentials/set/0.2 requires the sealed cleartext to be a
  SealedStorageCredential for the request's config, and refuses sealed when the
  host does not permit it.

- **vtc/admin/events**: 0.4 adds a communityRooms topic — the owner's view of the rooms the community owns (#864)

The console's Community rooms tab polls every 60 s, because no topic covers
  it: `rooms` is the host's view (storage, limits, lifecycle) for holders of
  `vtc.rooms.admin`, carries no room's details, and is not readable by a
  `vtc.rooms.manage` holder who lacks `vtc.rooms.admin`.

  - vtc/admin/events/_shared/0.4: Topic gains `communityRooms` (10 topics).
  - vtc/admin/events/subscribe/0.4: `topics` maxItems 10; "What 0.4 adds";
    Authorization row — holders of `vtc.rooms.manage`, prompting
    vtc/rooms/community/list 0.2, get 0.2 and members/list 0.1;
    `vtc.rooms.admin` alone does not qualify.
  - vtc/admin/events/event/0.4: producer rules 8–9 — hint when anything those
    reads show changes (a room created or its label, description or tags
    changed; an admission offered, withdrawn, accepted or renewed; a removal
    owed, retried, settled or failed, including on departure; the profile
    write's state; the authority-renewal state). No room, member, DID, offer,
    credential or record identifier in any member, ext included. Not a count
    topic. Coalescing, standing re-checks and stream-ending on a shrink are
    unchanged.
  - vtc/rooms/community/update 0.1 and 0.2 (draft, editorial in place):
    "Live updates" now points at `communityRooms` in events 0.4 instead of
    "a later version".

  0.1, 0.2 and 0.3 are unchanged; a community serving several versions answers
  each subscription at its own.



### Fixed

- A replay record is kept until the skew-extended window closes, so a late replay cannot execute twice (#872)

Freshness accepts a document until issuedAt + maxAge + skew, but the replay
  record was retained only until issuedAt + maxAge. A replay arriving in the
  final skew interval (60s under the consequential defaults) passed freshness,
  found its record already dropped, and executed again — the double execution
  SPEC §7.2 item 11 forbids, and a break of the "same bound" rule the record
  expiry documents.

  recordExpiry now returns issuedAt + maxAge + skew in Rust, TypeScript, Go and
  Dart. The expiresAt branch is unchanged: expiry is checked without skew, so it
  was already exact. Rust and TS gain a regression test at the boundary.



### Specifications

- **rooms/epoch/prune**: 0.2 — the host keeps the room's epoch floor, and members' key holders forget what they hold below it (#869)

* spec(rooms/epoch/prune): 0.2 — the host keeps the room's epoch floor, and members' key holders forget what they hold below it

  Each member's key holder keeps its own copy of a room's epoch chain, so
  erasing the host's rungs left every key holder able to derive what the owner
  had decided nobody should.

  - rooms/_shared/0.1 (in place, additive): EpochFloor — the earliest epoch
    whose key the room still lets anybody derive; monotonic. Hosts keep it,
    never store or serve a rung at or below it, and serve it. A member's key
    holder that learns a higher floor discards rungs at or below it and every
    storage, file or cached key below it (including keys derived in passing
    while catching up), never stores them again from any source, refuses to
    derive below it (beyondChain), and keeps the floor. States plainly that
    nothing already read, decrypted or exported is recalled, and that the
    floor is honoured, not enforced.
  - rooms/epoch/prune 0.2: the host raises the floor to beforeEpoch − 1 (also
    where nothing is dropped: a gap, a fromJoin room) and returns it as the
    required epochFloor. beforeEpoch keeps 0.1's meaning; the off-by-one is
    stated with a table, and two 0.1 misstatements are corrected.
  - rooms/epoch/chain 0.2: epochFloor on every response once pruned; never a
    rung at or below it.
  - rooms/keys/chain 0.2: the member relays the floor (alone if need be); the
    key holder applies it. rooms/keys/backfill 0.2: fetches with epoch/chain
    0.2, applies and reports the floor.
  - Editorial pointers in rooms/keys/open 0.2, rooms/keys/file-key 0.2,
    rooms/keys/read 0.1 and rooms/epoch/prune 0.1.
  - Rust, TypeScript, Go and Dart bindings regenerated.

  Refs OpenVTC/verifiable-trust-infrastructure#2118.

- **rooms/records**: A record says what kind of claim it is, what it cites, what it may be used for, and when it lapses (#865)

A sealed record proves who wrote it and whether a member or an agent did,
  but not what kind of claim it is: an agent's inference read the same as an
  observed fact or a credential-backed statement. And records could not
  expire individually — a meeting note and a decision lived equally long.

  - rooms/_shared/0.1 (in place, additive, as `roomProfile` was):
    SealedRecordBody gains optional `basis` (declared | observed | verified |
    inferred), `confidence` (0–1), `evidence` (record or credential
    citations, discriminated by `kind`), `validFor` (purposes as IRIs plus a
    plain-text note) and `validUntil`, all inside the author's signature.
    Provenance is display and weighting only, never authority; writers
    SHOULD set `basis`, agent writers that do not know SHOULD write
    `inferred`. Readers hide a record past its `validUntil` from default
    listings, search and recall and MAY show it on explicit request; no host
    enforces it and nothing about it is in the clear on sealed tiers.
    Records written before verify unchanged; readers that predate the
    members refuse bodies carrying them by rule (1), showing nothing rather
    than something wrong.
  - rooms/records/put 0.3 (new): an open room's `cleartext` carries the same
    five members, stored and served as received, never acted on by the
    host. `sealed` writes are unchanged.
  - Rust, TypeScript, Go and Dart bindings regenerated.

  Refs OpenVTC/verifiable-trust-infrastructure#2117, #2118.

## 0.9.1 — 2026-10-21


### Fixed

- **voting**: Trustees and voters act only on signed evidence; the community authorizes decryption; the key needs t dealers; ballots bind their tag (#860)

* fix(voting): trustees and voters act only on signed evidence; the community authorizes what is decrypted; the key needs t dealers; ballots bind their tag; the board is in order

  An adversarial review of community voting found that a dishonest ballot box could take the
  trustees' place, and a single corrupt trustee could hold the election key.

  - **The box could feed the trustees' VTAs.** They acted on values voting/trustees/status reported,
    which carry no trustee's proof. The box could:
    - substitute transport keys, and open the shares sealed to them;
    - invent complaints, and collect the shares revealed in answer;
    - replace dealings, and overwrite a key share;
    - list one voter's ciphertexts as a "total", or a cast before the close, and collect honest
      trustees' partials of it.
  - **One trustee could hold the key.** The key formed from a single qualified dealer, so a corrupt
    trustee complaining of every other dealer at the last moment was left holding it alone.
  - **The voter's VTA took any key.** It encrypted under whatever key it was handed, so a community
    could give one member a key it held.
  - **Ballots could be copied.** Nothing bound a ballot to its cast: a voter could re-cast another's
    encrypted ballot verbatim under their own tag (Cortier–Smyth).
  - **Certification ignored the lifecycle.** It accepted casts after close, a result before it, and
    an early close a manifest forbade.

  The changes:

  - **vta/pcs/trustee/{deal,keys,justify,decrypt}/0.2 and vta/pcs/vote-encrypted/0.2** (the 0.1
    versions retire) take the ceremony as its signed steps, which status now carries as `steps`. The
    VTA verifies each step itself (CONVENTIONS §9.6). In particular:
    - deal pins the transport keys it dealt to;
    - justify needs the complainant's own confirm;
    - keys never replaces a held share;
    - the voter's VTA forms the key itself;
    - vtc/elections/show carries `ceremony` for it.
  - **voting/trustees/authorize/0.1 (new).** After close the community computes the decryption
    targets from the board it checked, and signs their digest under the closing head. The box
    accepts it only if its own targets match, and appends a `decryptionAuthorized` entry. A
    trustee's VTA decrypts only an authorized set, once (decrypt/0.2). The box refuses partials
    before an authorization.
  - **CONVENTIONS §9.6 item 5:** the key forms only from at least t qualified dealers, with at least
    t confirmations.
  - **§9.3/§9.4:** an encrypted form's proofs, and a sealed form's AAD, bind the cast's tag. The box
    refuses a form whose ciphertexts are already on the board under another tag.
  - **§3.9 (new):** the board's order — opened, casts, close, authorization, partials, one result
    whose head is the board before it — enforced at certification (`orderInvalid`,
    `ceremonyInvalid` appended to boardInconsistent's faults).

## 0.9.0 — 2026-10-21


### Added

- **persona**: Star a directory member whose DID you have not been shown — persona/contact/put/1.1 linking form (#858)

A community directory shows members' DIDs to other members only where each member chooses to, so most entries a holder reads carry no DID to file a contact under, and persona/contact/put/1.1 required one (subjectDid) to keep a star, a petname or a list on anyone.

  put/1.1 gains a third form, LINKING: directoryRef + knownByPersona, without contactId, subjectDid, document or credentialRefs. The agent finds the contact it keeps for that directory entry and persona, or makes one (rev 1, an empty document, no subjectDid), and appends no revision. A later filing naming the member's DID is a contact of its own and is not merged with the linked one (item 4, extended). Two invalid examples pin the form's edges.

  get/1.1 and list/1.1 responses: subjectDid is no longer required — absent for a linked contact.
