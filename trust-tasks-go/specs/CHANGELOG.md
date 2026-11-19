# Changelog

All notable changes to `trust-tasks-go/specs` (the generated specification
packages) are documented here. Entries are generated from conventional commits
by the release automation (see RELEASING.md).

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

