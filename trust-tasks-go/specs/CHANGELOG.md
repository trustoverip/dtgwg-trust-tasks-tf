# Changelog

All notable changes to `trust-tasks-go/specs` (the generated specification
packages) are documented here. Entries are generated from conventional commits
by the release automation (see RELEASING.md).

## 0.3.0 — 2026-11-27


### Specifications

- **hcp**: A re-delivered grant request gets a new token; questions 58 and 59 for the HCP editors (#1401)

* spec(hcp): a re-delivered or resubmitted grant request gets a new token, and the TSP references name binding 0.3

  hcp/grants/request 0.1 told the provider both to answer a re-delivered request with
  what the first delivery was answered with and to keep the access token in no record of
  the exchange; after a response that carried a token it could not do both. SPEC §7.2 item
  11 asks only that the document is not executed twice. A re-delivered copy of an answered
  request is now answered as a resubmission within the hour is: the same grant with a newly
  issued token, and a resubmission no longer MAY repeat the token it sent.

  hcp/grants/request and hcp/grants/revoked cite TSP binding 0.3 §6 (the text is unchanged
  from 0.2). QUESTIONS.md question 39 and its status line are reworded to match.

## 0.2.0 — 2026-11-27


### Added

- **build**: Declare the rules a payload schema does not check (uncheckedRules) and check the declarations (#1381)

* feat(hcp): carry HCP 2.0-draft-3 query, grant revoke and revocation notice as Trust Tasks

  Add the hcp/* protocol family (SPEC §9.4) with three draft 0.1 specs
  targeting framework 0.6.0, each carrying one Human Context Protocol
  2.0-draft-3 operation:

  - hcp/entries/query: Query (HCP 5.2.1). The HCP access token travels
    opaquely in `accessToken`; the result's `timestamp` is renamed
    `respondedAt`; `categories` gets a framework bound of 256.
  - hcp/grants/revoke: Revoke grant (HCP 6.2, 9.2), destructive and acting
    for the Authorizing Principal, with an empty response for HCP's 204.
  - hcp/grants/revoked: the revocation notice (HCP 6.2.3, 9.9). The SET
    claims map to the envelope, and an empty response carries RFC 8935's
    202. The notice is signed with a provider VID key that the provider's
    JWKS must also publish.

  HCP's invalid-token condition maps to the family code hcp:invalidToken.
  Shared shapes (common, context entry, disclosure, directive) live in
  specs/hcp/_shared/0.1. They were written from the CC BY 4.0 HCP text,
  with attribution, not copied from HCP's schema files. HCP's kebab-case
  enum values are allowlisted as external.

- **hcp**: Add the hcp/* specifications, carrying the Human Context Protocol (HCP 2.0-draft-3) as Trust Tasks (#1380)

* feat(hcp): carry HCP 2.0-draft-3 query, grant revoke and revocation notice as Trust Tasks

  Add the hcp/* protocol family (SPEC §9.4) with three draft 0.1 specs
  targeting framework 0.6.0, each carrying one Human Context Protocol
  2.0-draft-3 operation:

  - hcp/entries/query: Query (HCP 5.2.1). The HCP access token travels
    opaquely in `accessToken`; the result's `timestamp` is renamed
    `respondedAt`; `categories` gets a framework bound of 256.
  - hcp/grants/revoke: Revoke grant (HCP 6.2, 9.2), destructive and acting
    for the Authorizing Principal, with an empty response for HCP's 204.
  - hcp/grants/revoked: the revocation notice (HCP 6.2.3, 9.9). The SET
    claims map to the envelope, and an empty response carries RFC 8935's
    202. The notice is signed with a provider VID key that the provider's
    JWKS must also publish.

  HCP's invalid-token condition maps to the family code hcp:invalidToken.
  Shared shapes (common, context entry, disclosure, directive) live in
  specs/hcp/_shared/0.1. They were written from the CC BY 4.0 HCP text,
  with attribution, not copied from HCP's schema files. HCP's kebab-case
  enum values are allowlisted as external.



### Build & CI

- **go,go-specs,go-proof,go-didcomm,go-capabilityclient**: Require Go 1.24 (#1374)

Raise the `go` directive from 1.22 to 1.24 in the core, specs, proof,
  didcomm and capabilityclient modules and in the two Go examples
  (consumer-app-go, getting-started-go). tsp stays at 1.27, the floor it
  inherits from affinidi-tsp-go. The go.yml matrices that prove the floor
  move from ["1.22", "stable"] to ["1.24", "stable"], and the comments, the
  dependabot note, go.md and the go.mod the generator and smoke tests write
  for their throwaway modules say 1.24.

  1.24 is the first release whose encoding/json honours `omitzero`, which an
  optional member that admits null needs in order to be a comparable struct
  that is still left out when absent.

  A consumer whose toolchain is older than 1.24 and that runs with
  GOTOOLCHAIN=local (a distribution's Go, a pinned CI image) can no longer
  build these modules; with the default GOTOOLCHAIN=auto the go command
  fetches 1.24 itself. Go supports only its two newest releases, and 1.22
  and 1.23 are long out of support.



### Fixed

- **go,go-specs**: Write a required nullable member's null instead of dropping its key (#1378)

* test(go): show a required nullable member losing its null on marshal

  A required member whose schema admits null is generated with omitempty, so
  a nil pointer drops the key and the document fails its own schema's
  required. vta/contexts/update-did 1.2's did is the case: a null did means
  "no DID of its own", and Go writes the member out entirely.

  specs/smoke drives the prebuilt package through a marshal and a round trip,
  and trust-tasks-gen's test checks the tags the per-module generator emits.



### Specifications

- **auth/oob**: DeviceEnrol: a client asks for scoped, long-lived access to a person's VTA, approved from their wallet (#1386)

* spec(auth/oob): copy the versions deviceEnrol changes, unchanged

  _shared 0.1 to 0.2, request 0.2 to 0.3, claim 0.2 to 0.3, identify 0.1
  to 0.2, prove 0.3 to 0.4, grant 0.3 to 0.4, respond 0.3 to 0.4 and
  redeem 0.2 to 0.3, copied byte for byte so the next commit shows only
  what the new versions change. The copies still carry their old version
  numbers and $ids; this commit does not build on its own.

- **vtc/directory/query**: 0.2 answers the member's directory entry (#1383)

* spec(vtc/directory/query): 0.2 answers the member's directory entry

  A caller that meets a member by their DID (the author of a record in a data
  room, the signer of a document) can read their projection with
  vtc/directory/query, but every other directory read is addressed by entry:
  the member's picture (images/get), their entry with shared groups
  (entries/show), and the card a member surface opens. Without the entry
  identifier such a caller can name the member but not show their picture or
  open their card.

  0.2 = 0.1 + a required `entryId` (directory-card `EntryId`) on every answer,
  never on a notFound or in a refusal's details. The notFound rule is now
  stated normatively: an answer is given only where the projection shows the
  caller the member's DID, to the member themselves, or to an administrator,
  so the identifier links the named DID only to an entry that caller could
  already open; Security & Privacy says so. Targets framework 0.6.0.

  All four libraries regenerated.

## 0.1.9 — 2026-11-25


### Specifications

- **vtc/rooms/community**: An administrator sets a community room's record members (#1373)

A community-owned room's profile is written by the community with its label
  and description only, so a room meant as a conversation started with no record
  members, and turning replies and reactions on took a member holding `curate`,
  which most of a community room's members do not hold. The community holds
  `admin` in every room it owns but had no task to set them.

  - create/0.6: optional `recordMembers`, written into the room's profile with
    its label and description; absent is none, as in 0.5. The response carries
    what was recorded.
  - update/0.5: optional `recordMembers` (the whole set) or
    `removeRecordMembers: true`, mutually exclusive; either re-writes the
    profile. Absent, the profile's record members are carried over as before.
  - get/0.6: optional `recordMembers`, as the community last wrote or read back
    the profile.
  - vtc/_shared/0.5: `RoomRecordMembers`, the constraints of
    `RoomProfile.recordMembers`.

  An administrator's surface may offer them as one choice (replies and
  reactions: kind, inReplyTo, writtenAt) and may start it on for a new room;
  the request it sends is still the administrator's explicit choice. Other
  draft specs now point at the new versions. All four libraries regenerated.

## 0.1.8 — 2026-11-25


### Specifications

- **vta/dids**: A step-up hold is a next step, never an error (vta/dids next versions) (#1360)

SPEC §8.6 requires a held task to be answered with trust-task-next-step,
  never an error response. Ruling 1 on trustoverip/dtgwg-trust-tasks-spec#36
  (reading (a)) confirmed that inline step-up delivery in an error response
  was not an intended exception. auth/step-up/approve-request/0.6, which this
  builds on, landed in #1362.

  - vta/dids create 1.2, keys/add 1.1, keys/retire 1.1, keys/revoke 1.1,
    keys/rotate 1.1, recover 1.2, update 1.1, upgrade 1.1: drop
    vta/dids:stepUpRequired. A request with a previewId that is held for a
    step-up is answered with a next step naming approve-response, its hint
    an approve-request/0.6 payload bound to the previewId as before. A
    request with no previewId is answered with a next step naming the task
    itself (continuation: proceed), its hint the dry-run #response carrying
    the stored preview. keys/rotate, recover and upgrade declare the maxItems
    their baselined request arrays lacked.

  Earlier versions stay published and are superseded by the new ones.
  Bindings regenerated for all four languages.

## 0.1.7 — 2026-11-25


### Specifications

- **auth/step-up**: Approve-request 0.6 — a step-up hold is a next step, never an error (#1362)

Implements ruling 1, reading (a), on trustoverip/dtgwg-trust-tasks-spec#36
  for the step-up request itself. Split out of #1360 so it can land while the
  trust-tasks-rs crate is near crates.io's upload limit.

  - `auth/step-up/approve-request/0.6` (`extends: "0.5"`): inline delivery is a
    `trust-task-next-step` whose `expects` entry names
    `auth/step-up/approve-response/0.7` and carries this payload as its `hint`;
    after approval is recorded the producer re-sends the operation
    (`continuation: resubmit`). Delivery in an error response is forbidden. The
    schema is unchanged.
  - The schema-bounds `accepts` baseline moves from 0.5 to 0.6: a version that
    extends 0.5 cannot narrow a bound, and `uniqueItems` over a three-value enum
    already caps it at 3.
  - `auth/oob/redeem/0.1` is `status: retired`, `supersededBy:
    auth/oob/redeem/0.2`.

  The new `vta/dids` versions from #1360 follow once the crate is smaller.
  Libraries regenerated.

## 0.1.6 — 2026-11-25


### Specifications

- **credential-exchange/present**: 0.2 — OID4VP keyed vp_token, either Data Integrity nonce placement (#1359)

`credential-exchange/present/0.2` settles the two readings #1358 found in 0.1.

  - `vp_token` is the OID4VP 1.0 DCQL response (§8.1): an object keyed by
    credential query `id`, each value an array of one or more presentations
    (SD-JWT-VC string or Data Integrity VP object). A bare SD-JWT-VC string is
    still accepted for compatibility; producers send the keyed form.
  - Data Integrity nonce and audience: a consumer MUST accept them as the holder
    proof's `challenge`/`domain` (OID4VP §14.1.2) or as the signed presentation's
    own `nonce`/`domain`; where both are stated they MUST agree, and a
    presentation stating neither is refused. Producers SHOULD use the proof's
    members. "The proof's `nonce`" in 0.1 was a wording slip for `challenge`.
  - Worked SD-JWT-VC and Data Integrity examples (the latter using
    `proof.challenge`/`proof.domain`).

  0.1 stays published and is superseded by 0.2 (the libraries deprecate it).

  `vtc/directory/card/present/0.1` takes the same placement rule in place: it is
  a draft whose wire shape does not change (only the presentation member's
  description), which SPEC §5.2 allows within a draft version, as #831 did for
  the same sentence.

  Prose links to present/0.1 in other specs move to 0.2 (fix-stale-references).
  Libraries regenerated.

## 0.1.5 — 2026-11-24


### Specifications

- **vtc,vetting**: A community's correlation scope is directed or public, and its profile declares it (#1356)

DTG Credentials Working Draft 0.7.0 makes a community's own correlation
  scope `directed` at minimum and `public` where the community is open to
  outsiders; it is no longer always `public`.

  - vtc/_shared/0.1 community: CommunityProfile and CommunityProfileView
    gain an optional `correlationScope` (`directed` | `public`), the
    `issuerScope` the community's credentials carry. Absent, implementations
    issue `public`, as before.
  - vtc/community/profile/update 0.1 accepts it (and refuses `pairwise`);
    profile/show 0.1 returns it through the shared view.
  - vetting/request 0.1, vtc/join-requests/decide 0.1, vtc/endorsements/issue
    0.1, vtc/vetting/vetters/grant 0.1, vtc/members/{credentials,rotate,renew}
    0.1, vtc/members/personhood/{assert 0.2,revoke 0.1} and the shared
    endorsement schema: the community-issued credential's `issuerScope` is
    `directed` or `public`, not `public` only.

  All changes are in place: an optional member and descriptions/prose on
  draft versions, which CONTRIBUTING-SPECS allows for a released version
  (check-published-schemas: additive only).

## 0.1.4 — 2026-11-24


### Fixed

- **build,rs,ts,ts-gen,go,dart**: Keep the free-standing $defs a specification declares in the request schema constant (#1353)

#1243 pruned every root $defs entry the request root does not reach. That also
  dropped definitions a specification declares for clients and references from
  nowhere, such as vtc/storage/credentials/set's SealedCredentialCleartext, which
  travels only inside a seal. VTI reads it, and the OpenAPI components its console
  uses, out of PAYLOAD_SCHEMA, and so stayed on trust-tasks-build 0.1.

  The request constant now also keeps each free-standing definition (one other
  than Response that no #/$defs/<name> reference anywhere in the schema names)
  and what it references. A shared definition spliced in from another document is
  always referenced, so it is still pruned when only the response uses it. The
  response document keeps only what it reaches.

  Applied in all seven generator copies and re-derived in check-bundles from
  defOrigins; regenerated, five specifications change in all four languages.

## 0.1.3 — 2026-11-19


### Specifications

- **vtc/rooms/community**: A community-owned room states and changes its signal delay (create 0.5, update 0.4, get 0.5) (#1336)

A room a member owns states its signal delay at rooms/create 0.3 and its
  administrators change it at the host with rooms/signal-delay/set 0.1. A
  room the community owns could do neither: the community is both the
  holder of `admin` and the host, and vtc/rooms/community/create and
  update carried no such member, so every community room waited a
  randomised few seconds before each rooms/changed signal however its
  members used it.

  - create 0.5 takes an optional `signalDelay` (absent = randomised),
    records it in the room's host record and always echoes it. A `none`
    create is audited with the create.
  - update 0.4 takes an optional `signalDelay`, applied to the host record
    with the other details, together or not at all; a change of the delay
    alone is a change (audited, details timestamps move) that needs no
    profile write. It touches no watch.
  - get 0.5 always serves the room's `signalDelay`.
  - Both writes refuse `signalDelayNotPermitted` when the community's
    governance as host keeps the delay everywhere, applying nothing.
  - A surface offering `none` SHOULD say what it gives up.

  Links to create 0.4, update 0.3 and get 0.4 move to the new versions.

## 0.1.2 — 2026-11-17


### Added

- **vta/dids**: List, get and methods for every identity a VTA holds; method-neutral context deletion (#1332)

A VTA holds identities of several methods, and the only way to ask which
  was once per method: vta/webvh/dids/list, vta/webs/dids/list. Every client
  asked only the methods it knew about, so a VTA whose identities were all
  did:webs reported none, and a new method would add a question to every
  client.

  - vta/dids/_shared/0.2: CONVENTIONS §20 (every identity: which are
    summarised, who sees what, the summary, methods), DidSummary and
    MethodInfo, a did:key profile, and a Summary details heading in each
    profile. §3.1's level-1 refusal is scoped to the key-role tasks.
  - vta/dids/list/0.1, get/0.1, methods/list/0.1.
  - vta/contexts/update-did/1.2: the DID must be one the VTA holds in that
    context (notHeld), and its class may be the context's (didClassConflict).
  - vta/contexts/delete/1.2: every DID in the subtree is deleted as its
    method's delete task deletes it, host work settled first. 1.1 named
    did:webvh only, so a did:webs record was removed while its key event log
    stayed published as a live identity.
  - vta/contexts/preview-delete/2.0: `dids` (every method) replaces
    `webvhDids`, which left did:webs out of the confirmation.
  - vta/webvh/dids/list/1.0 and vta/webs/dids/list/0.1 retired in favour of
    vta/dids/list/0.1.

  Bindings regenerated for all four languages.

- **vtc/storage/credentials/recipient**: Add vtc/storage/credentials/recipient/0.1 (#1329)

A room-hosting administrator's client asks the community for the key a
  vtc/storage/credentials/set sealed block is sealed to: the Ed25519 key it
  opens with, as its did:key, and the verification method in its DID
  document that publishes it.

  A client that cannot resolve and verify the community's DID could not
  seal at all. A browser console reading a did:webs would need KERI
  verification, and a hosted did:webvh would need a log from another
  origin. The VTC console read only the same-origin did.jsonl, which a
  did:webs community never serves. The answer comes over the same
  authenticated channel the credential then goes over, and is the same
  for every DID method. `notPublished` covers a key the community's
  document no longer lists, so nothing is sealed past a rotation.

  credentials/set 0.3's sealed-block definition and `related` now point
  at it (prose only).

## 0.1.1 — 2026-11-17


### Added

- **vtc/community/logo**: Add vtc/community/logo/update/0.1 and vtc/community/logo/show/0.1 (#1324)

* feat(vtc/community/logo): add vtc/community/logo/update/0.1

  An administrator hands the community a new logo, which the community
  re-encodes, strips of metadata and stores in place of the one it holds;
  `image: null` removes it. The answer is the stored re-encoding's digest,
  media type, size and dimensions.

