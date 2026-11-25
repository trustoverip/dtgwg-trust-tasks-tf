# Changelog

All notable changes to `trust-tasks-go/specs` (the generated specification
packages) are documented here. Entries are generated from conventional commits
by the release automation (see RELEASING.md).

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

