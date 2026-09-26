# DID key roles — conventions for the vta/webvh/dids key-role tasks

This document holds the invariants shared by every task that reads or changes the
**key roles** of a `did:webvh` a VTA holds keys for — its own, and those of the VTCs and
VTNs provisioned on it:

- [`vta/webvh/dids/keys/list/1.0`](../../webvh/dids/keys/list/1.0/spec.md) — the roles, their keys and the rotation history;
- [`vta/webvh/dids/keys/add/1.0`](../../webvh/dids/keys/add/1.0/spec.md) — add a key to a role (a post-quantum key beside a classical one, a second operational key);
- [`vta/webvh/dids/rotate-keys/2.0`](../../webvh/dids/rotate-keys/2.0/spec.md) — a planned rotation of one role, staged and overlapped;
- [`vta/webvh/dids/keys/retire/1.0`](../../webvh/dids/keys/retire/1.0/spec.md) — a planned retirement, which is also how a rotation completes or aborts;
- [`vta/webvh/dids/keys/revoke/1.0`](../../webvh/dids/keys/revoke/1.0/spec.md) — revocation for compromise;
- [`vta/webvh/dids/update/2.0`](../../webvh/dids/update/2.0/spec.md) — document changes that are not key changes.

Each of those specifications references this file rather than restating it; conformance
is anchored here. The wire shapes are in
[`did-key-roles.schema.json`](did-key-roles.schema.json).

The requirements a VTA, a node and a verifier must meet regardless of protocol — the key
role registry, custody, rotation and revocation, judging material signed before a
rotation — are the Verifiable Trust Infrastructure specification's
*Key roles* section (VTI-KEY-070 onward, in draft). These tasks are how a client drives a
VTA that meets them. Where this document and the VTI specification disagree, the VTI
specification is the requirement and this document is the defect.

## 1. Roles and the relationships they own

A VTA **MUST** bind every key it holds for a DID to exactly one role, **MUST** publish
each key in exactly its role's relationship and nowhere else, and **MUST** keep the
document's `keyRoles` map (VTI-KEY-090) in agreement with both in every entry it
appends:

| Role | Relationship | Proof purpose | Signs / used for | Custody |
|---|---|---|---|---|
| `attestation` | `assertionMethod` | `assertionMethod` | Attestation artefacts: membership and role credentials, endorsements, status lists, every other credential the node issues — through the VTA's signing service only | Generated in the VTA. Never derived, never exported, never in a backup |
| `operational` | `authentication` | `authentication` | The node's own traffic: Trust Task requests and responses, DIDComm/TSP signatures, invitations, notices, audit checkpoints | May be held by the node's service process |
| `messaging` | `keyAgreement` | none | Key agreement; signs nothing | May be held by the node's service process |
| `update` | none — the log's `updateKeys`, and the `nextKeyHashes` committing successors | as the DID method defines | Appending to the DID's log | Generated in the VTA. Never derived, never exported, never in a backup |

Consequences a conforming VTA **MUST** honour:

1. **Attestation keys are the only `assertionMethod` keys**, and operational keys the
   only `authentication` keys. A document that lists an operational or messaging key in
   `assertionMethod` lets a key exposed to routine traffic sign a membership credential.
   A VTA **MUST NOT** publish one, and **MUST** refuse any request whose result would be
   one.
2. **`capabilityInvocation` and `capabilityDelegation` are reserved.** No role permits
   them (VTI-KEY-075). A VTA **MUST NOT** list a method in either; a document that lists one is not a key-role
   identity (§10).
3. **A key serves one role for its whole life.** Moving a key between roles is a
   retirement from one and an addition to the other, with fresh key material under a
   fresh identifier. The same public key **MUST NOT** appear in two roles, in two
   identities, or in the DID's history under a different role (VTI-KEY-071).
4. **A rotation rewrites the role everywhere it appears** — its relationship, `keyRoles`,
   and any reference elsewhere in the document. Rewriting `authentication` and leaving
   the predecessor referenced from another member is the defect this rule exists to
   prevent.
5. **Every published key has a custodian record**, created in the same atomic step as
   the log entry that publishes it. A document naming a key the VTA cannot find is a DID
   whose next rotation will silently skip it.
6. **Role keys use role-appropriate algorithms.** `attestation`, `operational` and
   `update` keys are signing keys (`ed25519`, `p256`, `mldsa44`, `mldsa65`); `messaging`
   keys are key-agreement keys (`x25519`). Any other pairing is
   `vta/webvh/dids:unsupportedKeyType`. A successor's algorithm defaults to its
   predecessor's and to nothing else — a rotation never silently downgrades an
   `mldsa44` key to `ed25519`, and never builds an `x25519` method from Ed25519 bytes.
7. **`attestation` and `update` keys are generated**, from the VTA's CSPRNG inside its
   protection boundary (`origin: internal`), never derived from a seed, so no backup
   and no seed can reproduce them (VTI-KEY-110, VTI-KEY-112, VTI-KEY-113). `operational`
   and `messaging` keys are derived as the VTA's other context keys are (VTI-VTC-040).
8. **New verification-method fragments SHOULD be derived from the key** (VTI-KEY-150),
   so that a fragment reveals nothing about the node's layout and cannot be reissued
   under different material.

How long a predecessor and its successor are published together, and what each may do in
that time, differs by role and is set out in
[§11 Overlap by role](#11-overlap-by-role).

A role may hold more than one active key — an Ed25519 and an ML-DSA-44 attestation key
signing as a proof set (VTI-KEY-103), for example. The rules above apply to each.

## 2. Authorization — who may change which role

Authority is checked against the caller's role in the VTA's access-control list for the
**context the DID belongs to**, after the proof has established who the caller is
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The minimum a VTA enforces:

| Operation | `operational`, `messaging` | `attestation` | `update` |
|---|---|---|---|
| `keys/list`, public projection | any ACL role reaching the context | same | same |
| `keys/list`, custody projection | context `admin`, `super-admin` | same | same |
| `keys/add`, `rotate-keys`, `keys/retire` | context `admin`, `super-admin` | `super-admin` | `super-admin` |
| `keys/revoke` (compromise) | context `admin`, `super-admin` | context `admin`, `super-admin` | `super-admin` |
| `update` (non-key members) | context `admin`, `super-admin` | — | — |

- A VTA **MAY** narrow any cell, and **MAY** delegate the `attestation` column to a named
  context administrator by an explicit ACL scope. It **MUST NOT** widen a cell by
  default, and **MUST NOT** grant any of these to an `initiator`, `application` or
  `reader` role, or to a service acting under its own authority. **A node's service
  process is never an administrator of its own DID's keys** (VTI-KEY-115): a VTC console
  relays a request its human administrator signed, and the VTA authorizes that
  administrator, not the VTC.
- Revocation is deliberately *easier* to reach than addition for the `attestation` role.
  Revoking removes authority and cannot be used to gain it — any replacement is generated
  inside the VTA and never leaves — so the worst an abusive revocation achieves is a
  denial of service the audit trail attributes. Making a context administrator wait for a
  super-admin before withdrawing a stolen attestation key leaves the key trusted for
  exactly as long as that wait.
- A caller that cannot reach the DID's context is answered `vta/webvh/dids:notFound`,
  never `permissionDenied`: the refusal must not confirm that the VTA holds the DID. A
  caller that can reach the context but lacks the column's role is answered with the
  framework's `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)); it
  already knows the DID exists.

## 3. Approval

Every mutating key-role task is `destructive`
([§8](#8-every-entry-under-pre-rotation-rotates-the-update-key)). Whether a particular
request needs a human's step-up, or a quorum, is the VTA's policy, and a specification
does not decide it ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)); for the
`attestation` and `update` roles a VTA **SHOULD** require at least a bound step-up, and
the reference policy does. What these tasks fix is **how** an approval is obtained and
what it binds, so that every client presents the same thing and no approval can be
spent on a change nobody saw.

### 3.1 Preview first

Every mutating task accepts `dryRun: true`. The VTA **MUST** compute the preview by
running the handler it would run to apply the change, against the DID's current log
entry, and **MUST** return a `DidDocumentPreview`: the complete resulting document
(including `keyRoles`), every change the entry would make — including the update-key
rotation that accompanies every entry under pre-rotation — and the warnings a human
should see. A dry run writes nothing except the plan itself, held under its `previewId`
until `expiresAt`. Any key a plan would add is **generated when the preview is
computed**, held `pending` (unusable, unpublished), and destroyed if the plan expires, is
superseded or is aborted. The preview therefore shows the exact public key and the exact
document, and an approval binds them: nothing generated after the human looked can be
substituted. Because a preview costs a key generation, a VTA **SHOULD** hold at most one
outstanding preview per DID, role and caller — a new one supersedes the last — and
**MAY** rate-limit dry runs.

Every mutating task's response states its `outcome` and **MUST** carry `preview` when the
outcome is `preview` or `pendingApproval`, `approvals` when it is `pendingApproval`, and
`newVersionId` when it is `applied`. (The schemas cannot state these as conditionals, because
the generated bindings cannot express one; they are requirements all the same.)

The apply request carries the `previewId` back. A VTA holding a `previewId`:

1. **MUST** apply exactly the previewed plan, or nothing;
2. **MUST** refuse with `vta/webvh/dids:previewStale` when the plan has expired, was
   computed against a log entry the log has since moved past, or was computed for a
   request whose payload (other than `dryRun` and `previewId`) differs from this one;
3. **MUST** treat the plan as consumed once applied — a second apply is a
   `previewStale`, not a second entry.

A client **MUST** render the VTA's preview, and **MUST NOT** render a document it
computed itself. A locally computed diff of a role change shows one key swapped and
hides the update-key rotation, and every signature over the resulting approval still
verifies.

### 3.2 Step-up is bound to the preview

Where the VTA's policy requires a human's step-up, the VTA **MUST** require a **bound**
step-up ([`auth/step-up/approve-request/0.3`](../../../auth/step-up/approve-request/0.3/spec.md)),
bound to the request's `previewId`, and **MUST NOT** accept a session elevation in its
place. An elevated session is a window any process holding the administrator's signing
key can spend on a change the administrator never saw; a bound approval authorizes the
one previewed plan.

When a request needs a step-up it does not yet have, the VTA refuses it with
`vta/webvh/dids:stepUpRequired`, carrying the step-up request inline in
`details.stepUpRequest` as approve-request 0.3
[Inline delivery](../../../auth/step-up/approve-request/0.3/spec.md#inline-delivery)
defines: `boundTo` set to a digest of the task type and `previewId` salted with the
step-up's `challenge`, and `sessionId` absent. A request that carries no `previewId` is
refused the same way after the VTA has computed and stored a preview for it, with that
preview in `details.preview`, so the step-up is always over a plan the human can be
shown. The `reason` the VTA puts in the step-up request **MUST** name the DID, the role
and the operation in words — *"Rotate the attestation key of vtc.example: every
membership credential and status list issued from 26 September will be signed by the new
key"* — and **MUST NOT** be a generic "confirm this action".

The client shows its human that `reason` with the preview beside it, obtains the gesture,
returns an [`approve-response/0.4`](../../../auth/step-up/approve-response/0.4/spec.md) to
the VTA, and on a `recorded` acknowledgement re-sends the same request.

### 3.3 Quorum

Where the VTA's policy requires more than one approval, the VTA **MUST** count only
distinct approvers — distinct VIDs, each holding the operation's column in
[§2](#2-authorization--who-may-change-which-role) — and **MUST** count the initiator
once at most. An approval is valid only when bound to the `previewId`, as in §3.2.

Once the initiator's approval is recorded and more are needed, the VTA answers the apply
request with `outcome: "pendingApproval"` and the `KeyRoleApprovalState`, sends approve-requests
bound to the same `previewId` to the other eligible approvers on whatever channel it has
for them, and applies the plan **itself** when the count is reached — under the
initiator's authorization, recording every approver in the audit row. It **MUST NOT**
apply a plan whose `expiresAt` has passed, and **MUST** record it as `expired`. A pending
plan holds the role: a second change to the same role is
`vta/webvh/dids:rotationInProgress` until the first is applied, expired or withdrawn —
with one exception: a **revocation for compromise is never blocked** by a pending or
overlapping change to the same role. It supersedes it, and the superseded plan is
recorded `aborted`.

Pending plans appear in `keys/list`'s custody projection, which is where every client
shows "waiting for 1 more approval".

## 4. Custody and exportability

1. `attestation` and `update` keys **MUST** be reported with `exportable: false`,
   `neverExportable: true` and `inBackups: false` from the moment they are generated.
   They are used only by the VTA — an attestation key through its signing service for
   material it has parsed as an attestation artefact of that identity (VTI-VTA-009), an
   update key to sign that identity's log entries.
2. For such a key, [`keys/export-secret`](../../../keys/export-secret/0.1/spec.md)
   **MUST** answer `keys/export-secret:neverExportable`, and
   [`keys/set-exportability`](../../../keys/set-exportability/0.1/spec.md) with
   `exportable: true` **MUST** answer `keys/set-exportability:notPermittedForThisKey` —
   whatever the caller's authority. A provisioning bundle
   ([`vta/contexts/secrets`](../../contexts/secrets/1.0/spec.md)) **MUST NOT** carry them
   (VTI-VTA-033), and no backup ([`vta/backup/*`](../../backup/initiate-export/1.1/spec.md)) may contain
   them (VTI-KEY-112).
3. [`keys/revoke`](../../../keys/revoke/0.1/spec.md) **MUST** refuse a key published in
   any role of a DID, with `keys/revoke:boundToIdentifier`. Revoking the custodian record
   while the document still publishes the key leaves verifiers trusting a key the VTA can
   no longer use and will never announce as withdrawn; `vta/webvh/dids/keys/revoke`
   withdraws it in the document and in custody together.
4. A retired `attestation` or `messaging` key **MUST** be destroyed when the retiring entry
   is published (VTI-KEY-125 for attestation; §11 for messaging); its record survives, with
   `destroyed: true`. A revoked key **MUST NOT** be used for anything; a revoked `messaging`
   key **MUST NOT** decrypt anything received after `compromisedSince`.
5. No response of any task in this family carries private key material, in any member or
   in `ext`. The only way a private half leaves a VTA is `keys/export-secret`, and point 2
   closes that for the roles that must never leave.

## 5. What a caller sees

`keys/list` answers in one of two projections, and says which in `projection`:

- **`public`** — what the published document and log already tell anyone who resolves the
  DID: each published key's verification method, role (from `keyRoles`), algorithm,
  public half, `active` or `revoked` state with `compromisedSince`, and each rotation's
  kind, state and version ids. It **MUST NOT** include `pending` keys, the `staged` and
  `retiring` distinctions, deadlines, custodian records, labels, initiators, approvers or
  reasons. `staged` and `retiring` are withheld on purpose: together they tell an
  observer which published key the VTA is *not* currently signing with, which is the key
  a thief's use of would be least likely to be noticed. Both keys are valid to a verifier
  for the overlap, so the verifier loses nothing.
- **`custody`** — the public projection plus the VTA's custody state. Only for the callers
  §2 names.

The mutating tasks return a `DidDocumentPreview` and the resulting `RoleKey` records. Those
are custody-level data and are only ever returned to a caller authorized to make the
change. A VTA **MUST NOT** echo a `reason` into anything published.

## 6. Audit

A VTA **MUST** append an audit row
([`audit/_shared/0.1`](../../../audit/_shared/0.1/audit.schema.json) `AuditEnvelope`) for
every one of these, **including refusals**:

| `action` | When |
|---|---|
| `did.keys.preview` | A preview was computed and stored |
| `did.keys.approval` | An approval bound to a preview was recorded, or declined |
| `did.keys.add`, `did.keys.rotate`, `did.keys.retire`, `did.keys.revoke`, `did.update` | A change was applied, left pending, or refused |
| `did.keys.activate` | A staged successor became active at `activatesAt` |
| `did.keys.autoRetire` | The VTA retired a predecessor itself at the end of an overlap |
| `did.keys.destroy` | A retired attestation key's private half was destroyed |
| `did.keys.expired` | A pending plan expired unapplied |

`actor` is the authenticated caller (for the VTA's own scheduled actions and a quorum's
final apply, the initiator whose authorization it acts under); `target` is the DID;
`detail` carries the role, the verification-method identifiers and public-key
fingerprints involved, the base and resulting `versionId`s, the `previewId` and
`rotationId`, the approvers and each approval's evidence kind, `compromisedSince` for a
revocation, and the refusal code for a refusal. It **MUST NOT** carry private key
material, and **SHOULD NOT** carry the full resulting document, which the log already
holds.

A refusal of an `attestation` or `update` operation is the row an incident review reads
first; a VTA that audits only successes has no record of the attempts that matter.

## 7. Judging material signed before a change

The VTI specification's *Judging material signed before a rotation* (VTI-KEY-130 to 135)
is normative; these tasks produce the evidence it relies on, and **must make a planned
retirement and a revocation distinguishable in the DID's history** (VTI-KEY-120).

- **Retired** (planned): the key leaves its relationship, `keyRoles` and the document. A
  proof it made is judged against the DID version current at issuance, resolved by
  `versionId` or `versionTime`. For an attestation key the retiring entry bounds the
  issuance time of everything it signed, because the key is destroyed (VTI-KEY-132).
- **Revoked** (compromise): the key leaves its relationship and `keyRoles` in one entry,
  with no overlap (VTI-KEY-123), and the entry **records the revocation**: until the VTI
  specification fixes an encoding (its VTI-KEY-120 open issue), a conforming VTA keeps the
  method in `verificationMethod` with the Data Integrity `revoked` property set to
  `compromisedSince`. A verifier resolving the current document therefore sees that the
  key was withdrawn for cause, not retired. A proof by the key is accepted only on
  evidence the signer cannot backdate that it predates `compromisedSince` (VTI-KEY-131);
  a `created` value inside the proof is the thief's to choose and is never such evidence.
- A producer declaring `compromisedSince` **SHOULD** err early, and where it cannot
  establish the time **MUST** give the time the key was first published (VTI-KEY-121).
  Every proof between the real compromise and the declared instant is one a verifier will
  accept.

## 8. Every entry under pre-rotation rotates the update key

A `did:webvh` with pre-rotation active requires each new entry to be authorized by a key
the previous entry committed to. Every mutating task in this family appends an entry, so
**every one of them also rotates the `update` role** — adding an attestation key moves
the update key too. That is authority-shifting whether or not the caller asked for it,
which is why each such task is `destructive`, why `DidDocumentPreview` carries
`updateKeyRotates` as its own member, and why a consent surface **MUST** show it.

The `update` role **MUST** be used with pre-rotation (VTI-KEY-124). A VTA **MUST** keep at
least one committed successor after every entry, and **MUST** refuse a request that would
set `preRotationCount: 0` on an identity VTI-KEY-001 governs (a VTA's, a VTC's or a VTN's
durable DID) with `vta/webvh/dids:preRotationRequired`.

## 9. Family error codes

Carried by the `vta/webvh/dids` family namespace
([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications) rule 2).
Every task in this family that can reach one **MUST** declare it.

```yaml
- code: vta/webvh/dids:notFound
  meaning: No such DID is held by this VTA, or the caller cannot reach its context.
  retryable: false
- code: vta/webvh/dids:keyNotFound
  meaning: The named verification method is not a key of this DID in the stated role and state.
  retryable: false
- code: vta/webvh/dids:versionConflict
  meaning: The DID's latest entry no longer matches `expectedVersionId`.
  retryable: false
- code: vta/webvh/dids:previewStale
  meaning: The `previewId` is unknown, expired, already applied, computed against an entry the log has moved past, or for a different request.
  retryable: false
- code: vta/webvh/dids:stepUpRequired
  meaning: The VTA's policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan it is bound to.
  retryable: true
- code: vta/webvh/dids:unsupportedKeyType
  meaning: The key type cannot serve the role, or is not in the VTA's accepted set.
  retryable: false
- code: vta/webvh/dids:wouldEmptyRole
  meaning: The change would leave the role with no active key.
  retryable: false
- code: vta/webvh/dids:rotationInProgress
  meaning: The role already has a change staged, overlapping or awaiting approval.
  retryable: false
- code: vta/webvh/dids:preRotationRequired
  meaning: The change would leave a durable node identity with no committed successor update key.
  retryable: false
- code: vta/webvh/dids:notKeyRoleIdentity
  meaning: The DID was not created with key roles (its document does not satisfy §1). These tasks do not operate on it; create a new identity with key roles instead.
  retryable: false
```

`stepUpRequired` is retryable because the same request succeeds once the approval it
names is recorded — that is the whole of the flow in §3.2.

## 10. Only key-role identities

These tasks operate **only** on a DID whose every published key is bound to exactly one role
as §1 requires, with `keyRoles` in agreement — a **key-role identity**. A DID created before
key roles (one key in both `authentication` and `assertionMethod`, also holding the log's
update authority), or any DID whose document breaks §1, is not converted in place. Every
task in this family, `keys/list` included, **MUST** refuse it with
`vta/webvh/dids:notKeyRoleIdentity`, and **MUST NOT** guess roles for its keys or repair it.

The remedy is a new identity: create a new DID with key roles and move the node to it.
There is no migration task and no grace period for artefacts a pre-role key signed; the
deployments this family is being introduced into are recreated from scratch. A client
**MUST** word the refusal as that instruction — "This DID was created before key roles.
Create a new identity for this community" — and not as a failure to retry.

## 11. Overlap by role

A planned rotation publishes the successor before it is used and removes the predecessor
after, so that no verifier or sender acting on a cached copy of the DID document meets a key
it has never seen, or loses one it still relies on. Revocation for compromise never overlaps.

### 11.1 The cache horizon

The **cache horizon** of an entry is its publication time plus the longer of the document's
stated validity period (its TTL, VTI-KEY-060) and the **verifier cache cap** — the maximum
age of a cached document a verifier may rely on (VTI-KEY-134; proposed as 24 hours). After the
cache horizon, no conforming verifier or sender still acts on a copy older than that entry.

The cache cap is the VTI specification's number. A VTA **MUST** use it as published there,
**MUST NOT** use a shorter one, and reports the horizon it applied as `cacheHorizonAt` on the
rotation record, so every client can show it.

### 11.2 Phases of a planned rotation

| Phase | Entry | Successor | Predecessor | Earliest |
|---|---|---|---|---|
| **1. Publish** | successor added to the role's relationship and `keyRoles` | `staged` | `active` | — |
| **2. Wait** | none | `staged` | `active` | lasts until the cache horizon of entry 1 |
| **3. Switch** | none — the VTA begins signing (or preferring) with the successor, and stops signing with the predecessor | `active` | `retiring` | `activatesAt` ≥ cache horizon of entry 1 |
| **4. Retire** | predecessor removed | `active` | `retired` | `overlapUntil` > `activatesAt` |

A conforming VTA:

1. **MUST NOT** switch before the cache horizon of the publishing entry.
2. **MUST NOT** end a planned overlap — by `autoRetire`, by a `keys/retire` completing the
   rotation, or by any other means — before the cache horizon of the publishing entry has
   passed and the successor has switched. "Complete now" completes no earlier than that; a
   request to retire sooner is refused (`vta/webvh/dids/keys/retire:notYetActive`), and a
   rotation request whose `activatesAt` or `overlapUntil` would break this is refused
   (`vta/webvh/dids/rotate-keys:overlapTooShort`).
3. **MAY** abort at any phase before retirement by retiring the *successor*; aborting never
   has to wait, because it removes a key nobody has yet been asked to rely on alone.

### 11.3 By role

| Role | During the overlap | At retirement |
|---|---|---|
| `attestation` | Both keys are in `assertionMethod` and `keyRoles`. The VTA signs new attestation artefacts only with the active key. | The predecessor leaves the document and its private half is **destroyed**. Everything it signed still verifies, judged against the DID version current at issuance (§7, VTI-KEY-130, VTI-KEY-132). |
| `operational` | Both keys are in `authentication` and `keyRoles`. The VTA signs new messages and responses only with the active key; a peer accepts either. | The predecessor leaves the document; the VTA destroys its private half. |
| `messaging` (`x25519`) | Both keys are in `keyAgreement` and `keyRoles`, and **the VTA holds both private halves for the whole overlap**, decrypting with whichever a message was encrypted to. **Senders may encrypt to either**; a sender that has seen both **SHOULD** prefer the successor, the key published later. | The predecessor leaves the document and its private half is **destroyed**. A message encrypted to it afterwards cannot be read; the VTA answers it with a decryption failure that tells the sender to re-resolve. Because the overlap outlasts the cache horizon, a sender that follows the preference above never meets this. |
| `update` | **No overlap.** The handover is pre-rotation: one log entry moves the update authority to the successor the previous entry committed, and the same entry commits a fresh next key (§8, VTI-KEY-124). | — |

### 11.4 Revocation

A revocation for compromise has **no overlap in any role** (VTI-KEY-123): the key leaves its
relationship and `keyRoles` in one entry, the VTA stops using it on receipt of an authorized
request, and any replacement is published `active`. There is no cache-horizon wait; a
verifier with a cached document meets a signature by the unknown replacement and must
re-resolve (VTI-KEY-134), which is the price of not trusting a stolen key for a day.
