# did:webs identities — conventions for the vta/webs/dids tasks (0.2)

> **Alpha.** The `did:webs` integration these tasks drive is alpha. Expect breaking
> changes as the KERI, ACDC, CESR and `did:webs` specifications settle the questions
> open on them, among them the protocol version a stream states
> ([kswg-did-method-webs-specification#216](https://github.com/trustoverip/kswg-did-method-webs-specification/issues/216))
> and the designations of §4. A `did:webs` DID created while the integration is alpha
> may have to be created again. Don't use one for a relationship that must outlive that.

This document holds what every task of the family shares:

- [`vta/webs/dids/create/0.2`](../../dids/create/0.2/spec.md) — create a `did:webs` DID of a stated class in a context, from explicit members or a DID template, and publish it through a hosting server;
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

**What 0.2 changes from [0.1](../0.1/CONVENTIONS.md).** Two things, and nothing else:

1. **Service DIDs.** A `did:webs` DID may be the durable identity of the VTA itself, a
   VTC, a VTN or a host service, as an optional, alpha method (§1). 0.1 refused every
   service DID, because VTI-KEY-001 named only `did:webvh` and `did:scid:vh:1`. Every DID
   is now created in a stated DID class (`WebsDidClass`), which the record keeps.
2. **DID templates.** A DID may be created from a DID template (§9), whose document shape
   is mapped onto what the `did:webs` derivation can express, and refused where it cannot
   be.

The tasks still at 0.1 (`get`, `list`, `update`, `rotate-keys`, `delete`) apply unchanged
to a DID `create/0.2` makes, whatever its class; the records they return do not carry
`didClass`, which a VTA keeps all the same. Where this document and
[0.1](../0.1/CONVENTIONS.md) differ for such a DID, this document governs.

## 1. Which identities

Every DID these tasks create is created in one **DID class** (VTI specification, *DID
classes*, VTI-KEY-190 – 193), stated by the request and fixed for the DID's life:

| `didClass` | What it is |
|---|---|
| `relationship` | An identifier for one relationship, one community or one trust context, and for nothing else (VTI-KEY-190) |
| `persona` | A person's own identity: the identifier under which a person, through their VTA, acts as themselves (VTI-KEY-191) |
| `service` | The durable identity of a node or a service: the VTA itself, a VTC, a VTN, a trust registry, a host service, a mediator, or an integration the VTA provisions (VTI-KEY-192) |

Every DID these tasks create is at **key assurance level 1**: one signing key, no key
roles, no `keyAssurance` member. A service DID at level 2 or 3 is a key-role identity, and
is created through the method-neutral [`vta/dids/*`](../../../dids/_shared/0.1/CONVENTIONS.md)
family and its [`webs` profile](../../../dids/_shared/0.1/method-profiles/webs.md), with
the key-role designation, not through these tasks. A VTA **MUST** answer a task of this
family for a key-role identity with `vta/webs/dids/<task>:notFound`, as it would for a DID
it does not hold.

### 1.1 Service DIDs: optional and alpha

The Verifiable Trust Infrastructure specification is being amended to name `did:webs` as
an **optional** durable-identity method beside `did:webvh` and `did:scid:vh:1`, never the
default (VTI-KEY-001 and VTI-KEY-003 as amended;
[dtgwg-vti-spec#120](https://github.com/trustoverip/dtgwg-vti-spec/issues/120)). Under
it, a node **MAY** take a `did:webs` identity, and **MUST NOT** require a peer that has
not declared `did:webs` resolution to resolve it. These tasks implement that position:

1. A VTA **MAY** create a `did:webs` service DID, and **MUST** refuse one, with
   `vta/webs/dids/create:serviceDidNotPermitted`, unless its configuration enables
   `did:webs` service DIDs. That setting is separate from the one that enables `did:webs`
   creation at all, and is off by default while the integration is alpha. Turning it off
   affects no service DID already created.
2. **A service `did:webs` is alpha.** Its key agreement key and its services are
   designations (§4) that resolve in full only for verifiers whose `did:webs` resolver
   implements them, and a peer that has not declared `did:webs` resolution may not be
   able to verify it at all. An operator giving a node a `did:webs` identity **SHOULD**
   keep a `did:webvh` or `did:scid:vh:1` identity for every peer that has not declared
   `did:webs` resolution, and **SHOULD** say so where the node's identity is configured.
3. **Level 1 only.** VTI-KEY-196 recommends level 2 or higher for a VTA's own identity, a
   VTC and a VTN. A VTA creating such an identity here **SHOULD** say, in its operator
   surface, that the identity is at level 1 and that a higher level is created through
   `vta/dids/create` with the `webs` profile.

### 1.2 One class only

A DID belongs to one class only (VTI-KEY-193). A VTA therefore:

1. **MUST** refuse, with `vta/webs/dids/create:didClassConflict`, a `create` whose
   `didClass` is `persona` or `relationship` when the new DID would be the identity of a
   node or a service: the primary DID of a context whose primary DID is a service DID, or
   of a context provisioned for a VTC, VTN or host service;
2. **MUST** refuse, with the same code, a `create` whose `didClass` is `service` and whose
   `setPrimary` would replace a context's primary DID that is a persona DID;
3. **MAY** adopt a `did:webs` DID this family created as its own or a community's durable
   identity through another task (setting the VTA's DID, provisioning a community) only
   when the DID's class is `service`, and **MUST** refuse to adopt any other — a persona or
   relationship DID, or one created by `create/0.1` — whatever that task's own error
   vocabulary is.

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
4. The DID signs as itself — as a persona, in a relationship, as a service — with the
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
   `vta/webs/dids:serviceIdConflict`. A DID template defines its own services, `#didcomm`
   and `#tsp` among them, and the VTA designates them as the template renders them (§9.2).
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
| `vta/webs/dids:serviceIdConflict` | A designated service would reuse `#didcomm` or `#tsp` beside the VTA's own, an id a KERI endpoint reply establishes, or another designated service's id. |
| `vta/webs/dids:deactivated` | The DID is deactivated, and the task would write an event to it. |

A host that cannot be reached is the framework's `unavailable`, retryable. A context the
caller may not act in is `permissionDenied` for `create`, and `<task>:notFound` for the
tasks that name a DID, so the answer does not reveal a DID the caller cannot see.

## 9. DID templates

A DID template ([`vta/_shared/0.9`](../../../_shared/0.9/did-template.schema.json)
`DidTemplate`, managed by `vta/did-templates/*`) describes the **shape of a DID
document**. A `did:webs` document is not authored: the host derives it from the key event
log and the designations of §4. So a template is **read**, not published. The VTA renders
it, maps each member of the rendered document onto what the derivation expresses — the
key state, the key agreement designation, the services designation and the aliases
designation — and refuses, before deriving or publishing anything, a template whose
document the derivation could not reproduce. A DID created from a template therefore
resolves to exactly the document the template describes, less only the members this
section says the derivation supplies itself.

### 9.1 Resolution and rendering

1. `template` is resolved in the scope `templateContext` names, else the global scope, then
   the built-ins, exactly as `vta/webvh/dids/create` resolves it. A name no scope holds is
   `vta/webs/dids/create:templateNotFound`.
2. The template is rendered by the VTA's template renderer, which checks the variables: a
   missing `requiredVars` entry, an unknown variable, or a placeholder left unresolved is
   `vta/webs/dids/create:templateVarsInvalid`. Variables are never checked by anything
   else.
3. **The keys are always the VTA's.** The template supplies no key and no derivation path.
   The VTA derives the signing key, its successor and the key agreement key as §3 says, and
   fills the reserved placeholders from them: `SIGNING_KEY_MB` / `SIGNING_KEY_JWK` from the
   signing key, `KA_KEY_MB` / `KA_KEY_JWK` from the key agreement key, and, for a
   `schemaVersion` 2 `keys` block, `{SLOT}_KEY_MB` / `{SLOT}_KEY_JWK` from whichever of the
   two the slot's `purpose` names.
4. **`{DID}` is the final DID.** The AID is the digest of the inception event, which
   carries no services or aliases, so the VTA incepts first, builds the DID from the
   domain, the path and the AID, and renders with it. The designations, written after, can
   name the DID.

### 9.2 What a template may describe

The rendered document — after the renderer has pruned the members a `null` variable removes, as `didcomm-mediator`'s optional P-256 keys are — is mapped member by member. Anything not honoured below is refused
with `vta/webs/dids/create:templateNotRepresentable`, whose `details.reason` names the
first rule that failed and `details.members` the JSON Pointers, into the template with its
rendered `document` (`/keys/verifier`, `/document/verificationMethod/2`), of what it
refused.
A VTA **MUST** check every member before it derives a key, and **MUST NOT** publish a
document that drops, moves or adds anything the template described.

| Template member | Honoured as | Refused (`reason`) |
|---|---|---|
| `schemaVersion` 1, or 2 with a `keys` block | the key shape below | `schemaVersion` 3, a key-role template: key roles are created by `vta/dids/create` (`keyRoleTemplate`) |
| `keys` slots | exactly one `signing` slot, whose `algorithms` include `ed25519`, minted as the key state's one key; and exactly one `keyAgreement` slot, whose `algorithms` include `x25519`, minted as the designated key agreement key | a second slot of either purpose, a `publish: false` slot, or a slot whose list lacks the one algorithm the derivation uses. The VTA never substitutes an algorithm (`keyAlgorithm`) |
| `id` | the DID | anything but `{DID}` rendered (`documentId`) |
| `controller` | absent, or the DID | any other value (`controller`) |
| `@context` | ignored: the host derives its own | — |
| `verificationMethod` | one entry publishing the signing key and one publishing the key agreement key, rendered by the derivation under its own ids and types (`<did>#<signingKey>`, a `JsonWebKey` for the key agreement key) | any other entry, or an entry publishing either key twice (`verificationMethod`) |
| `authentication`, `assertionMethod` | the signing key, in both, as the derivation renders it | either relationship without the signing key, or listing anything else (`verificationRelationship`) |
| `keyAgreement` | the key agreement key, as its designation | no `keyAgreement`, or one listing anything else: every DID of the family designates one key agreement key, and the records of the family require it (`keyAgreementRequired`, `verificationRelationship`) |
| `capabilityInvocation`, `capabilityDelegation` | — | always: the derivation renders neither (`verificationRelationship`) |
| `service` | each entry designated as a `DesignatedService`, its id reduced to the fragment (`<did>#didcomm` → `#didcomm`); `#didcomm` and `#tsp` are the template's to define | an id naming another DID or not a fragment, an entry without `type` or `serviceEndpoint`, or more than 32 entries (`service`); two entries with one id is `vta/webs/dids:serviceIdConflict` |
| `alsoKnownAs` | each DID designated as an alias, after the DID itself; an entry equal to the DID is listed once | an entry that is not a DID (`alias`) |
| any other member (`keyAssurance`, a profile member, an extension) | — | always: the derivation renders no member it is not given (`documentMember`) |

The template's other members are read as follows. `kind` is the **integration kind**:
the VTA records it with the DID and returns it, and it is not published. `name` and the
scope it resolved in are recorded and returned. `methods` is advisory, and a VTA **MUST
NOT** refuse a template for its `methods` alone. `defaults` are hints to setup surfaces and
are not applied, except that `defaults.portable: false` is satisfied as it stands: no task
of this family moves a DID to another domain.

A template is not a substitute for `didClass`, which the request states. A VTA **MAY**
refuse with `vta/webs/dids/create:didClassConflict` a template whose `kind` it knows to be
a node's or a service's under a `didClass` other than `service`.

### 9.3 Template or explicit members, not both

`template` is mutually exclusive with the explicit document members of
`vta/webs/dids/create` — `addMediatorService`, `addTspService`, `additionalServices` and
`alsoKnownAs` — as a template is with a supplied document in `vta/webvh/dids/create`. A
request carrying both is refused by the schema, rather than merged: two sources for one
document leave the operator unable to tell which one shaped it. `templateContext` and
`templateVars` are meaningful only with `template`, and are refused without it.
