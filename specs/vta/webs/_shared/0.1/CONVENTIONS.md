# did:webs identities — conventions for the vta/webs/dids tasks

> **Alpha.** The `did:webs` integration these tasks drive is alpha. Expect breaking
> changes as the KERI, ACDC, CESR and `did:webs` specifications settle the questions
> open on them, among them the protocol version a stream states
> ([kswg-did-method-webs-specification#216](https://github.com/trustoverip/kswg-did-method-webs-specification/issues/216))
> and the designations of §4. A `did:webs` DID created while the integration is alpha
> may have to be created again. Don't use one for a relationship that must outlive that.

This document holds what every task of the family shares:

- [`vta/webs/dids/create/0.1`](../../dids/create/0.1/spec.md) — create a `did:webs` DID in a context and publish it through a hosting server;
- [`vta/webs/dids/get/0.1`](../../dids/get/0.1/spec.md) — one DID, optionally with its `keri.cesr`;
- [`vta/webs/dids/list/0.1`](../../dids/list/0.1/spec.md) — the DIDs the caller may see;
- [`vta/webs/dids/update/0.1`](../../dids/update/0.1/spec.md) — replace the designated services, the key agreement key or the designated aliases, with no key change;
- [`vta/webs/dids/rotate-keys/0.1`](../../dids/rotate-keys/0.1/spec.md) — rotate to the committed successor and commit a fresh one;
- [`vta/webs/dids/delete/0.1`](../../dids/delete/0.1/spec.md) — deactivate, then remove the slot.

Each task references this file rather than restating it, and conformance is anchored
here. The wire shapes are in [`webs-records.schema.json`](webs-records.schema.json).
The requirements a VTA must meet whatever protocol it is driven by are the Verifiable
Trust Infrastructure specification's; where this document and that specification
disagree, that specification is the requirement and this document is the defect.

## 1. Which identities

These tasks create and manage **persona, relationship and integration DIDs** at key
assurance level 1: one signing key, no key roles (VTI specification, *DID classes*,
VTI-KEY-190 – 192).

A `did:webs` DID **MUST NOT** be a node's **service DID**: the durable identity of a VTA,
a VTC, a VTN or a host service. VTI-KEY-001 names `did:webvh` and `did:scid:vh:1` for
those, and VTI-KEY-003 forbids requiring a peer to resolve any other method. A VTA
therefore:

1. **MUST** refuse a `vta/webs/dids/create` request that would make the new DID its own
   identity or a service DID it provisions — the primary DID of a context whose primary
   DID is a node's service identity, or of a context provisioned for a VTC, VTN or host
   service — with `vta/webs/dids/create:serviceDidNotPermitted`;
2. **MUST** refuse to adopt a `did:webs` DID this family created as its own or a
   community's durable identity through any other task (setting the VTA's DID,
   provisioning a community), whatever that task's own error vocabulary is.

Naming `did:webs` for a service DID waits on the adoption of the key-role designation by
the `did:webs` specification. A key-role identity under `did:webs` (levels 2 and 3) is
created through the method-neutral [`vta/dids/*`](../../../dids/_shared/0.1/CONVENTIONS.md)
family and its [`webs` profile](../../../dids/_shared/0.1/method-profiles/webs.md), not
through these tasks, and a VTA **MUST** answer a task of this family for a key-role
identity with `vta/webs/dids/<task>:notFound`, as it would for a DID it does not hold.

## 2. The key event log

A VTA writes **KERI v2** events with **JSON bodies**, and a stream that opens with the
CESR genus-version counter. It never writes a v1 event. It reads a stream for one of its
own DIDs from its own record (§5), never from the host.

| Event | Written by | Signed by | Changes |
|---|---|---|---|
| Inception (`icp`) | `create` | the signing key | establishes the AID, `k`, `n`, and no witnesses (`b` empty, `bt` 0) |
| Interaction (`ixn`) | `create`, `update` | the current signing key | anchors a designation (§4); `k` and `n` unchanged |
| Rotation (`rot`) | `rotate-keys`, `delete` | the committed successor | installs the successor as `k`; commits a fresh successor, or none (`delete`) |

## 3. Keys

1. The key state holds **one Ed25519 signing key** (`kt` 1) and commits **one
   pre-rotated successor** (`nt` 1). The VTA derives both from its seed in the DID's
   context, as it derives a `did:webvh` DID's signing and pre-rotation keys. No caller
   supplies, names or receives a key or a derivation path.
2. The successor's key record is made when it is committed and is **inactive** until the
   rotation that installs it. Each rotation derives the next successor the same way.
3. **Key agreement is a designated X25519 key** (§4), derived as a `did:webvh` DID's
   level-1 key agreement key is. A VTA **MUST NOT** publish, and **MUST NOT** decrypt with,
   the X25519 key converted from a signing key: that would make the change authority an
   encryption key, and a designation outlives a rotation where a converted key does not
   (VTI-KEY-184). A party encrypting to *another* `did:webs` DID that has designated no
   key agreement key MAY use the converted key, as KERI practice does (VTI-KEY-185); that
   gives a VTA nothing to publish.
4. The DID signs as itself — as a persona, in a relationship, as an integration — with the
   current signing key, under the verification method `<did>#<signingKey>` the key state
   names. A VTA answers for the key from its own record, without resolving.

## 4. Designations

What a key event log alone cannot say about a DID document — its aliases, its key
agreement keys and its non-KERI services — the AID says by **designation**: an ACDC it
issues to itself, in a registry of its own, anchored by an interaction event. Under
KERI v2 the registry is a `rip`/`bup` registry and the credential an ACDC 2.x, issued by
an anchored update that discloses it.

| Designation | Origin | Becomes |
|---|---|---|
| **Designated aliases** | the `did:webs` specification | `alsoKnownAs` |
| **Key agreement** | proposed by `affinidi-did-webs`, schema `EIW1ONwn…` (v2) | `JsonWebKey` methods in `keyAgreement` |
| **Services** | proposed by `affinidi-did-webs`, schema `EJVZ789s…` (v2) | entries in `service` |

1. **One of each kind stands.** Replacing a designation issues a new one and revokes the
   one in force, in the same anchored update. Nothing else changes, and no key rotates.
2. **The DID designates itself.** A `did:webs` resolution fails unless a designated-aliases
   credential names the DID being resolved, so the VTA's aliases designation always lists
   the DID first, then the caller's `alsoKnownAs`. A request's `alsoKnownAs` **MUST NOT**
   repeat the DID; a VTA lists it once.
3. **Designated services are typed.** `#didcomm` is typed `DIDCommMessaging` and `#tsp`
   `TSPTransport`, each with the VTA's mediator's DID as its endpoint, as a `did:webvh` DID
   the VTA creates advertises them, so a peer matches them on `type`. A service id
   **MUST NOT** be one a KERI endpoint reply of the AID establishes, and a caller's
   additional service **MUST NOT** use `#didcomm` or `#tsp`; either is refused with
   `vta/webs/dids:serviceIdConflict`.
4. **Interoperability.** The key agreement and service designations are proposals to the
   `did:webs` specification. A resolver that derives the document from KERI data and
   designated aliases alone does not reproduce `keyAgreement` or the designated services,
   and refuses a `did.json` that carries them. Until another implementation reads them, a
   DID created here resolves in full only for verifiers whose resolver implements them.
   A consent surface or operator guide **SHOULD** say so.

## 5. Hosting

1. A `did:webs` DID is **hosted only**. The VTA publishes it through a did-hosting server
   it has registered (`vta/webvh/servers/*`), by `did-management/did/register/0.1` with
   `method` `webs`, the slot and the `keri.cesr` stream as `didData`, and republishes the
   whole stream on every change. There is no serverless or self-hosted `did:webs`: a VTA
   has no inbound surface to serve one from.
2. The host must run with `did:webs` hosting enabled (`method-webs`). A VTA that learns a
   server does not host `did:webs` refuses `create` with
   `vta/webs/dids/create:hostDoesNotHostWebs` and names, in the message, what the
   operator must enable.
3. **The slot is the caller's path, then the AID.** The DID is built from where it is
   published: `did:webs:<domain>:<path segments>:<AID>`, a port in the domain
   percent-encoded. The host checks that the stream establishes exactly the slot's AID,
   and that an update neither rewinds nor forks the log it holds. There is no
   `.well-known` form.
4. **The host derives `did.json`** from `keri.cesr` on every read. The VTA does not submit
   one; one it submitted would be cross-checked and discarded.
5. The VTA keeps one record per DID (`WebsDidRecord`): the DID, its context, its server,
   domain and slot, the published `keri.cesr`, the controller state (sequence number,
   latest SAID, keys, next-key digest), the key ids, the designations in force, and any
   staged rotation (§6). The record is part of the VTA's backup: a restored VTA continues
   the log from it.

## 6. Remote first

Every change is **published to the host before the VTA's record commits it**. A process
that dies between the two leaves the host ahead of the record, never behind it, and the
VTA recovers by reading what it staged, never by guessing.

1. **Create** derives the keys and writes the inception and designation events, stages
   the new record, publishes, then commits. A VTA that finds a staged creation on restart
   **MUST** either complete it — the stream is published, or publishing it now succeeds —
   or remove the slot and discard the record and its keys.
2. **A rotation is staged.** The VTA derives the next successor, writes and signs the
   rotation event, and stores both in the record as staged before publishing. On restart
   with a staged rotation it **MUST** publish that same event again — the host accepts a
   republish of the log it already holds — and then commit it. It **MUST NOT** write a
   different rotation over a staged one: the successor the staged event installs has been
   disclosed to the host, and a second event at the same sequence number is a fork.
3. **While a change is staged, nothing else is written.** A request that would write an
   event to a DID with a staged change **MUST** first complete it; if the host cannot be
   reached, the VTA answers with the framework's `unavailable`, and writes nothing.
4. **`expectedVersionId`** (`<sequence number>:<SAID>`) is an optional precondition on
   every task that writes an event. A VTA **MUST** refuse a request whose
   `expectedVersionId` does not name the DID's latest committed event with
   `<task>:versionConflict`, before staging anything.

## 7. Deactivation

`delete` deactivates in two steps, in this order:

1. **Rotate to no next keys** (`n` empty, `nt` 0). Published and committed remote first,
   like any rotation. From then on the AID cannot rotate again, and every copy of the
   stream — at the host, at a watcher, in a verifier's cache — says it is deactivated.
2. **Remove the slot** from the host.

The order is what makes a failure safe. If the second step fails, the DID resolves, and
resolves as deactivated; it never resolves live once a delete has been answered. The VTA
keeps its record, marked deactivated, with the final stream — the evidence against which
anything the DID signed is still checked — and signs nothing more as the DID.
Deactivation is irreversible: no task of this family revives a deactivated DID, and a
create always incepts a new AID.

## 8. Errors shared by the family

Declared by every task that publishes:

| Code | Meaning |
|---|---|
| `vta/webs/dids:publishRefused` | The hosting server refused the publication. Nothing was committed; `details.hostError` carries what the server said. |
| `vta/webs/dids:serviceIdConflict` | A designated service would reuse `#didcomm`, `#tsp` or an id a KERI endpoint reply establishes. |
| `vta/webs/dids:deactivated` | The DID is deactivated, and the task would write an event to it. |

A host that cannot be reached is the framework's `unavailable`, retryable. A context the
caller may not act in is `permissionDenied` for `create`, and `<task>:notFound` for the
tasks that name a DID, so the answer does not reveal a DID the caller cannot see.
