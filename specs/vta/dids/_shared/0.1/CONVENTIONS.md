# DID key roles — conventions for the method-neutral vta/dids tasks

This document holds the invariants shared by every task that creates a **key-role
identity** or reads or changes its keys, whatever DID method the identity uses:

- [`vta/dids/create/1.0`](../../create/1.0/spec.md) — create a key-role identity from a key-role DID template;
- [`vta/dids/keys/list/1.0`](../../keys/list/1.0/spec.md) — the roles, their keys, the recovery commitment and the rotation history;
- [`vta/dids/keys/add/1.0`](../../keys/add/1.0/spec.md) — add a key to a role (a post-quantum key beside a classical one);
- [`vta/dids/keys/rotate/1.0`](../../keys/rotate/1.0/spec.md) — a planned rotation of one role, staged and overlapped, including the replacement of the recovery commitment;
- [`vta/dids/keys/retire/1.0`](../../keys/retire/1.0/spec.md) — a planned retirement, which is also how a rotation completes or aborts;
- [`vta/dids/keys/revoke/1.0`](../../keys/revoke/1.0/spec.md) — revocation for compromise, including recovery of the update authority;
- [`vta/dids/recover/1.0`](../../recover/1.0/spec.md) — a replacement VTA takes over an identity whose VTA was lost, with the recovery key;
- [`vta/dids/update/1.0`](../../update/1.0/spec.md) — changes that are not key changes.

Each task references this file rather than restating it; conformance is anchored here.
The wire shapes are in [`key-roles.schema.json`](key-roles.schema.json). What differs
between DID methods — and only that — is in the **method profiles** under
[`method-profiles/`](method-profiles/README.md).

The requirements a VTA, a node and a verifier must meet regardless of protocol are the
Verifiable Trust Infrastructure specification's *Key roles* section (VTI-KEY-070 onward).
These tasks are how a client drives a VTA that meets them. Where this document and the VTI
specification disagree, the VTI specification is the requirement and this document is the
defect.

This family supersedes the did:webvh-only key-role tasks (`vta/webvh/dids/keys/*/1.0`,
`vta/webvh/dids/rotate-keys/2.0`, `vta/webvh/dids/update/2.0` and `2.1`) and their
conventions (`vta/_shared/0.3`). Nothing here depends on did:webvh except its profile.

## 1. Threat model

Every rule below follows from one of these threats. A rule whose threat is not named is
a rule nobody can tell when to relax, so each section cites the threat it answers.

### 1.1 What is protected

| Asset | Why it matters | Held by |
|---|---|---|
| **A1 — the change authority** (`update`) | Whoever holds it decides every key the identity has, forever. Its theft is the theft of the identity. | The VTA, with a recovery key outside it |
| **A2 — the community's seal** (`attestation`) | It signs decisions others rely on long after delivery and without contacting the node: credentials, endorsements, **status lists**. A forged status list silently re-admits everyone suspended. | The VTA only |
| **A3 — the node's voice** (`operational`) | It signs what the node says now: Trust Task requests and responses, messages, notices. Forgery impersonates the node in conversations, not in decisions. | The node's service process |
| **A4 — confidentiality** (`messaging`) | It opens what is sent to the node. | The node's service process |
| **A5 — a person's presence** (`authenticator`) | It proves a human was there: sign-in, step-up, approval. | The person's authenticator |

### 1.2 Who attacks, and how

| Threat | Adversary and capability |
|---|---|
| **T1 — service compromise** | Code execution in the node's service process: the part that parses untrusted network input. Reads every key that process holds. The most likely compromise. |
| **T2 — VTA compromise** | Full read of one VTA's protection boundary: its seed, its store, its memory. Uses every key in it. |
| **T3 — backup theft** | A copy of a VTA backup and whatever unlocks it. Offline, unbounded time, unnoticed. |
| **T4 — VTA loss** | The VTA's boundary is destroyed: hardware failure, enclave measurement change with no way back, deleted account. Nothing is stolen; everything inside is gone. |
| **T5 — insider or stolen administrator credential** | A party able to send a correctly signed administrative request, alone. |
| **T6 — consent substitution** | Getting a human's approval for one change and spending it on another: a client that renders its own diff, an elevated session, a delegated key answering for its owner. |
| **T7 — stale verifier** | A verifier relying on a cached document after the node changed it, in either direction: refusing a new key, or trusting a revoked one. |
| **T8 — backdating** | A thief holding a key writes whatever dates it likes into what it signs. |
| **T9 — algorithm failure** | An algorithm breaks — classically or to a quantum adversary — at a moment nobody chose. |
| **T10 — correlation** | An observer learns the node's layout, operators or members from what its document publishes. |

### 1.3 The rules each threat produces

| Threat | Rule | Section |
|---|---|---|
| T1 | A key exposed to routine traffic can never sign an attestation artefact or change the identity: roles are separate relationships, separate material, separate custody | §2, §5 |
| T1 | The service process holds `operational` and `messaging` keys only; it obtains attestations by asking the VTA, which parses, constrains and records | §5.1 |
| T2 | The change authority survives a VTA compromise only through a recovery key in another protection boundary, and a witness that will not let the recovery commitment be dropped | §5.3, §5.4 |
| T3 | `attestation` and `update` keys are generated, not derived, and nothing that unlocks them is in a backup; a backup restores only what the service process could already reveal | §5.1, §5.5 |
| T4 | Loss is recovered by the recovery key and re-issuance, never by restore | §9.5 |
| T5 | Changes to the seal and the change authority need a super-admin, a bound step-up, and for the change authority a quorum; revocation stays reachable with less | §6, §7 |
| T6 | Approvals bind to a VTA-computed preview; only the approver's own evidence counts; a delegated key is never an approver | §6 |
| T7 | Planned rotations publish before use and overlap past a cache horizon; revocations take effect at once and verifiers re-resolve | §10 |
| T8 | A revoked key's compromise time is published; nothing it signed is judged by its own dates | §9, §13 |
| T9 | Successors keep their algorithm unless one is named; proof sets are all-must-verify; no conversion between curves; post-quantum keys join beside classical ones | §14 |
| T10 | `keyRoles` carries only role names and method identifiers; fragments derive from keys; custody, boundaries and schedules stay out of the public projection | §11, §15 |

### 1.4 What these tasks cannot protect against

Stated so nobody believes otherwise:

- **A VTA's own identity under T2.** A VTA is the custodian of its own keys, so its
  operational and attestation keys live in one boundary. The separation still stops the
  operational key being used to sign an attestation (a misuse, not a theft) and keeps the
  attestation key out of backups, but a full compromise of a VTA yields its own seal. Only
  the recovery key (§5.3) survives it.
- **did:webvh under T2 without witnesses.** Every did:webvh entry is authorized by a
  committed key, and the routine successor sits in the VTA. A party holding the VTA can
  append an entry that drops the recovery commitment before the operator uses it. Witnesses
  enforcing §5.4 close this; without them the recovery key protects against loss (T4) only.
- **An operator who approves what they did not read.** Binding makes an approval exact; it
  cannot make it considered.

## 2. Roles and the relationships they own

### 2.1 The registry

A VTA **MUST** bind every key it holds for a key-role identity to exactly one role, **MUST**
publish each verification method in exactly its role's relationship and nowhere else, and
**MUST** keep the document's `keyRoles` map (§15) in agreement with both in every change it
makes:

| Role | Relationship | Proof purpose | Used for | Custody | Origin |
|---|---|---|---|---|---|
| `attestation` | `assertionMethod` | `assertionMethod` | Attestation artefacts (VTI-KEY-080): credentials, endorsements, status lists — through the VTA's signing service only | The VTA only | **Generated** |
| `operational` | `authentication` | `authentication` | The node's own traffic (VTI-KEY-081) | May be held by the node's service process | Derived |
| `messaging` | `keyAgreement` | none | Key agreement; signs nothing | May be held by the node's service process | Derived (X25519 in its own right) |
| `update` | none — the method's change authority (method profile) | as the method defines | Authorizing changes to the identifier | The VTA (routine) and a recovery custodian (recovery) | **Generated** |
| `authenticator` | `authentication` | none — WebAuthn assertions only | A person's sign-in and step-up evidence | The person's authenticator | Enrolled, never generated by the VTA |

Consequences a conforming VTA **MUST** honour:

1. **`attestation` keys are the only `assertionMethod` keys.** A document listing an
   operational or messaging key in `assertionMethod` lets a key exposed to routine traffic
   (T1) sign a status list. A VTA **MUST NOT** publish one, and **MUST** refuse any request
   whose result would be one.
2. **`capabilityInvocation` and `capabilityDelegation` are reserved** (VTI-KEY-075). A VTA
   **MUST NOT** list a method in either.
3. **A key serves one role for its whole life.** Moving a key between roles is a retirement
   from one and an addition to the other, with fresh material under a fresh identifier. The
   same public key **MUST NOT** appear in two roles, in two identities, or in the identity's
   history under another role (VTI-KEY-071). Reuse would undo the separation without
   changing a single document, and would link the identities for every observer (T10).
4. **The update authority is never a role-bound verification method** (VTI-KEY-093). Where a
   method itself renders its change authority into the document as verification methods,
   the method profile says so and the identity is eligible only if the profile can keep
   those methods out of every relationship (§3.2).
5. **A rotation rewrites the role everywhere it appears** — its relationship, `keyRoles`,
   and any other reference in the document.
6. **Every key the VTA holds has a custodian record**, created atomically with the change
   that publishes it. A document naming a key the VTA cannot find is an identity whose next
   rotation silently skips it.
7. **Fragments of new verification methods SHOULD be derived from the key** (VTI-KEY-150),
   so a fragment reveals nothing about the node's layout (T10) and cannot be reissued under
   other material.

### 2.2 Algorithms by role

`attestation`, `operational` and `update` keys sign: `ed25519`, `p256`, `mldsa44` or
`mldsa65`, further limited by what the method profile accepts for `update`. `messaging`
keys agree keys: `x25519`. Any other pairing is `vta/dids:unsupportedKeyType`. §14 says
how algorithms change.

### 2.3 `authenticator` keys

§16.

## 3. Key-role identities and method profiles

### 3.1 Only key-role identities

These tasks operate **only** on a **key-role identity**: one whose every published
verification method is bound to exactly one role as §2 requires, with `keyRoles` in
agreement, whose change authority is held as §5.3 requires, and whose method has an
eligible profile. A DID created before key roles — one key in both `authentication` and
`assertionMethod`, also holding the update authority — or any DID whose document breaks §2,
is not converted in place. Every task in this family, `keys/list` included, **MUST** refuse
it with `vta/dids:notKeyRoleIdentity`, and **MUST NOT** guess roles for its keys or repair
it (VTI-KEY-146 to VTI-KEY-148).

The reason there is no migration: the pre-role key has had operational custody (T1), so it
cannot be retrofitted as an `attestation` key, and it was never destroyed, so its
retirement bounds nothing it signed (VTI-KEY-132). The remedy is a new identity created by
[`vta/dids/create`](../../create/1.0/spec.md). A client **MUST** word the refusal as that
instruction — "This DID was created before key roles. Create a new identity for this
community" — and not as a failure to retry.

A pre-role identity keeps being managed by the method-specific tasks it was created with
(for did:webvh, `vta/webvh/dids/update/1.0` and `vta/webvh/dids/rotate-keys/1.0`) until it
is replaced and deactivated. A VTA **MUST** refuse those tasks for a key-role identity with
the framework's `unsupportedVersion`, because they cannot change one role without the
others.

### 3.2 Eligibility

A DID method can carry key-role identities only if it provides all of:

1. **A verifiable history** with a change authority, so that rotation is recorded and
   attributable (VTI-KEY-004, VTI-KEY-041).
2. **Commitment to successor change authority** — pre-rotation — able to commit to more
   than one successor, so that a recovery key can be committed beside the routine one
   (§5.3).
3. **Rotation without changing the identifier**, and **deactivation**.
4. **Controller-chosen verification relationships**: the controller decides which
   relationship each verification method is listed in, and the method does not place the
   change authority in any relationship (§2.1 item 4).
5. **Historic resolution** by version, so a verifier can resolve the document current at a
   past time (§13).
6. **A place for `keyRoles`** and for a revoked method's `revoked` value in the document
   (§9.1, §15).

A method missing any of these is **ineligible**. A VTA **MUST** refuse to create a
key-role identity under an ineligible method, or under a method it holds no profile for,
with `vta/dids:methodNotEligible`, carrying in `details.missing` the numbers of the
criteria above the method does not meet. These are ineligible by construction and **MUST**
be refused:

| Method | Missing |
|---|---|
| `did:key`, `did:jwk`, `did:peer:0` | 1–6: the identifier is the key; nothing can rotate |
| `did:peer:2`, `did:peer:4` | 1, 2, 3, 5: no update at all |
| `did:web` | 1, 2, 5: a document anyone holding the host can rewrite, with no history and no commitment |

A client bootstrap identity (`did:key`, VTI-KEY-002) is not a key-role identity and is not
refused anything by this rule; it is simply outside it.

### 3.3 What a method profile states

A profile states only what differs between methods, under these headings, so that adding a
method never changes this document or the task schemas:

1. **Eligibility** — §3.2 item by item, and the method names (`DidMethodName`) it covers.
2. **The change authority** — what the `update` role is in this method; how the routine and
   recovery keys are committed; which changes move the update authority
   (`updateKeyRotates`); the threshold, if any.
3. **Versions** — what `versionId` and `expectedVersionId` carry; how a past version is
   resolved.
4. **Rendering** — how §2's relationships, `keyRoles` and `revoked` appear in the generated
   document.
5. **The cache horizon input** — the method's stated validity period (§10.1).
6. **Publication** — the artefacts a change produces (`PublishedArtifact.name`).
7. **Recovery signing** — the form of `RecoverySigningRequest.unsignedChange` and
   `proofOptions`.
8. **Method parameters** — the schema `methodParameters` is validated against in
   `create` and `update`, and which parameters are fixed at creation.
9. **Algorithms** the method accepts for the change authority.

A VTA **MUST** validate `methodParameters` against the profile's schema and refuse any
member it does not define with `vta/dids:methodParameterUnsupported`.

## 4. Creation

A key-role identity is created by [`vta/dids/create`](../../create/1.0/spec.md) from a
**key-role DID template**: a `schemaVersion` 3 template
([`vta/_shared/0.7/did-template.schema.json`](../../../_shared/0.7/did-template.schema.json))
whose `keyRoles` block is a list of `RoleLayout`s and whose `document` carries **no** key
members. A key-role template describes the identity's services, aliases and other
non-key members; the VTA renders every key member — `verificationMethod`, the
relationships and `keyRoles` — from the layout and §2, and from nothing else. A template
cannot place a key in a relationship, because it cannot name a relationship at all. That is
the point: under the v1 and v2 template shapes the template's `document` decided which
relationship a key appeared in, and every built-in put one key in both `authentication`
and `assertionMethod`.

A VTA creating a key-role identity **MUST**:

1. Refuse a template that is not `schemaVersion` 3, or whose `document` contains any of
   `verificationMethod`, `authentication`, `assertionMethod`, `keyAgreement`,
   `capabilityInvocation`, `capabilityDelegation` or `keyRoles`, with
   `vta/dids/create:templateNotKeyRole`.
2. Require a layout with at least one `operational`, one `messaging` and one `update` key.
   An `attestation` layout is required for an identity that issues attestation artefacts —
   a VTA, a VTC, a VTN — and omitted otherwise. An `authenticator` layout is refused: those
   keys are enrolled, not created.
3. Mint every key fresh, by its role's origin (§5.1). A creation **MUST NOT** accept a
   caller-supplied key, key identifier, document carrying keys, or existing history to
   adopt: none of them can show the custody §5 requires.
4. Commit, in the first version, a routine update successor and the recovery commitment
   (§5.3).
5. Do all of this under preview and approval (§6), so the approver sees the exact public
   keys and document.

## 5. Custody

### 5.1 Generated and derived roles, and why

| Role | Origin | Why |
|---|---|---|
| `attestation` | Generated from the VTA's CSPRNG inside its boundary (`origin: internal`) | T3: a derived key is reproducible from the seed, so every backup and every copy of the seed is a second seal kept wherever backups are kept. Generation removes that copy. The cost — a lost VTA loses the key — is paid by re-issuance (§9.5), which exists anyway. |
| `update` (routine) | Generated, as `attestation` | T3, and the change authority is worth more than the seal. Its loss is paid by the recovery key. |
| `update` (recovery) | Generated by the recovery custodian, never by this VTA | T2/T4: it is worth exactly as much as its independence from the VTA. |
| `operational`, `messaging` | Derived from the context's derivation base (VTI-KEY-030) | They are already exposed to the service process (T1), so a backup adds no exposure the service does not, and restoring them restores the node's voice and mailbox without republishing. Their compromise is bounded by §2: they can neither seal nor change the identity. |

A derived `messaging` key is derived as X25519 at its own path; it is **never** the
birational image of an Ed25519 key, so a compromise of the operational key yields nothing
about the messaging key, and the reverse (T1, T9).

### 5.2 Protection boundaries

A **protection boundary** is a set of hardware, software and secrets such that a full
compromise (T2) or loss (T4) of it yields neither the use nor the material of any key held
in another boundary. Two keys are in the same boundary whenever one compromise or one loss
reaches both — including through a backup, a seed, a storage key derived from a seed, or
credentials one holds to the other's store. A boundary is a property of what unlocks a key,
not of where its ciphertext sits.

What that means for the VTA deployments VTI has:

| Deployment | Boundary | What unlocks a generated key | `ProtectionClass` |
|---|---|---|---|
| **Plain VTA** | The host: its filesystem, its seed backend and the credentials it holds to that backend | Read access to the store | `software` |
| **Hardened VTA** | The host, as plain; the store is encrypted, but under a key the host can produce | The seed (or the host's credentials to it) and the store | `encryptedAtRest` |
| **TEE VTA** | The enclave image (its measurement) together with the KMS key whose policy releases material only to that measurement | Running that image | `enclave` |
| **External custodian** | A second VTA with its own seed, operator and host; a hardware token held offline; an HSM or cloud key service whose use requires credentials no VTA of this identity holds | That custodian alone | `externalCustodian` (as seen from this VTA) |

Every one of the three VTA deployments is **one boundary**. A cloud secret manager holding
a plain VTA's seed is not a second boundary for that VTA, because the VTA holds the
credentials that read it; a mnemonic printed and locked away is a second boundary for
*loss* but not for any key derived from it, because the running VTA holds the seed. A TEE
VTA is a stronger boundary — the host operator cannot read inside it — but it is still one:
the enclave holds every key it generates.

Within one boundary, a generated key **MUST** be protected at rest under a key that is not
derivable from the VTA's seed and is not present in any backup (VTI-KEY-118). Otherwise a
backup plus the seed decrypts what generation was meant to make unrecoverable (T3). A VTA
that cannot do this — a plain VTA, or one whose store key is derived from its seed —
**MUST** report the key's `protection` truthfully and warn `protectionBelowPolicy`.

### 5.3 The change authority: a routine key and a recovery key

Pre-rotation protects an identity only if the key that authorizes the next change is not
reachable by whoever compromised the current one. A VTA cannot meet that alone, because a
VTA is one boundary (§5.2). So the `update` role has two classes of key:

- **Routine** (`updateKeyClass: routine`): generated and held by the VTA. It authorizes the
  identity's ordinary changes. Its committed successor is also generated and held by the
  VTA.
- **Recovery** (`updateKeyClass: recovery`): generated and held by a **recovery custodian**
  in a boundary other than the VTA's (VTI-KEY-116). The VTA holds only its commitment.

A conforming VTA **MUST**:

1. Commit, in **every** version of a key-role identity, both a routine successor and the
   recovery key, as the method profile says how (`recoveryCommitted: true`).
2. Carry the recovery commitment forward **unchanged** in every change it makes, except a
   change that uses the recovery key (§9.4) or replaces the commitment
   ([`keys/rotate`](../../keys/rotate/1.0/spec.md) with `role: update` and
   `recoveryCommitment`).
3. Refuse to create a key-role identity, and refuse any change, that would leave it with
   no recovery commitment, with `vta/dids:recoveryCommitmentRequired`.
4. Accept a recovery commitment only with a **proof of possession** of the recovery key,
   over the request's `previewId`, so that a typing error cannot commit the identity to a
   key nobody holds. The VTA verifies the proof and then discards the public key unless
   the method requires it published (VTI-KEY-119).
5. Refuse a recovery commitment whose key is in the VTA's own boundary — derived from its
   seed, generated by it, or held in a store it can read — with
   `vta/dids:recoveryCommitmentRequired`. It cannot always tell; a recovery key the VTA
   generated, it always can.

**A VTA with only one boundary** — every VTA today — therefore never holds the recovery
key. Its operator supplies the commitment at creation from a custodian of their choosing,
and the VTA refuses to create a key-role identity without one. This is deliberate: an
identity whose every successor key sits in its VTA has pre-rotation in form and nothing in
substance, and a lost VTA takes it with it.

### 5.4 Witnesses and the recovery commitment

Item 2 above binds a conforming VTA. A compromised one (T2) is not conforming: holding the
routine successor, it can append a change that drops the recovery commitment, after which
the recovery key authorizes nothing. Where the method has witnesses (did:webvh witnesses,
KERI witnesses), a key-role identity **SHOULD** be witnessed by at least one witness in a
boundary other than the VTA's whose policy refuses a change that removes or replaces the
recovery commitment unless that change is authorized by the recovery key or carries the
approvals `keys/rotate` records for a replacement. A VTA **MUST** warn `noWitnesses` on
every preview of an unwitnessed identity.

Where the method lets the next-key threshold require more than one key (KERI), the profile
**SHOULD** make every rotation of the change authority require the recovery custodian, and
use routine keys only for changes that do not rotate it; that closes T2 without a witness.

### 5.5 Exportability and backups

1. `attestation` and `update` keys **MUST** be reported `exportable: false`,
   `neverExportable: true`, `inBackups: false` from the moment they are generated. An
   attestation key is used only by the VTA's signing service, for material it has parsed as
   an attestation artefact of that identity (VTI-VTA-009); a routine update key only to
   authorize that identity's changes.
2. For such a key, [`keys/export-secret`](../../../../keys/export-secret/0.1/spec.md)
   **MUST** answer `keys/export-secret:neverExportable` and
   [`keys/set-exportability`](../../../../keys/set-exportability/0.1/spec.md) with
   `exportable: true` **MUST** answer `keys/set-exportability:notPermittedForThisKey`,
   whatever the caller's authority. No provisioning bundle (VTI-VTA-033) and no backup
   (VTI-KEY-112) may carry them, nor anything that unlocks them.
3. [`keys/revoke`](../../../../keys/revoke/0.1/spec.md) **MUST** refuse a key published in a
   role of an identity, with `keys/revoke:boundToIdentifier`; withdrawing it is
   [`vta/dids/keys/revoke`](../../keys/revoke/1.0/spec.md), which changes the document and
   custody together.
4. A retired `attestation` or `messaging` key **MUST** be destroyed when the retiring change
   is published (VTI-KEY-125; §10.3); its record survives with `destroyed: true`. A revoked
   key **MUST NOT** be used for anything; a revoked `messaging` key **MUST NOT** decrypt
   anything received after `compromisedSince`.
5. No response of any task in this family carries private key material, in any member or in
   `ext`.

## 6. Approval

Every mutating key-role task is `destructive`. Whether a particular request needs a human's
step-up, or a quorum, is the VTA's policy, which a Trust Task specification does not decide
([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)); the VTI specification sets
the floor that policy must meet (VTI-KEY-180 to VTI-KEY-183) and §6.5 restates it. What
these tasks fix is **how** an approval is obtained and **what it binds**, so that no
approval can be spent on a change nobody saw (T6).

### 6.1 Preview first

Every mutating task accepts `dryRun: true`. The VTA **MUST** compute the preview by running
the handler it would run to apply the change, against the identity's current version, and
**MUST** return a `DidDocumentPreview`: the complete resulting document (with `keyRoles`),
every change the plan makes — including any movement of the update authority the method
requires — and the warnings a human should see. A dry run writes nothing except the plan,
held under its `previewId` until `expiresAt`. Any key the plan adds is **generated or
derived when the preview is computed**, held `pending` (unusable, unpublished), and
destroyed if the plan expires, is superseded or is aborted, so the preview shows the exact
public key and the approval binds it. A VTA **SHOULD** hold at most one outstanding preview
per identity, role and caller, and **MAY** rate-limit dry runs.

The response carries `preview` when `outcome` is `preview`, `pendingApproval` or
`awaitingRecoveryProof`; `approvals` when it is `pendingApproval`; and `newVersionId` when
it is `applied`. (The schemas cannot state these conditionals because the generated
bindings cannot express one; they are requirements all the same.)

The apply request carries the `previewId` back. A VTA holding a `previewId`:

1. **MUST** apply exactly the previewed plan, or nothing;
2. **MUST** refuse with `vta/dids:previewStale` when the plan has expired, was computed
   against a version the identity has moved past, or was computed for a request whose
   payload (other than `dryRun`, `previewId` and `recoveryProof`) differs;
3. **MUST** treat the plan as consumed once applied.

A client **MUST** render the VTA's preview and **MUST NOT** render a document it computed
itself: a locally computed diff of a role change shows one key swapped and hides the
movement of the update authority, and every signature over the resulting approval still
verifies.

### 6.2 Step-up is bound to the preview

Where the VTA's policy requires a human's step-up, the VTA **MUST** require a **bound**
step-up ([`auth/step-up/approve-request/0.5`](../../../../auth/step-up/approve-request/0.5/spec.md)),
bound to the request's `previewId`, and **MUST NOT** accept a session elevation in its
place: an elevated session is a window any process holding the administrator's signing key
can spend on a change the administrator never saw.

A request needing a step-up it does not have is refused with `vta/dids:stepUpRequired`,
carrying the step-up request inline in `details.stepUpRequest` (`boundTo` a digest of the
task type and `previewId` salted with the step-up's `challenge`, `sessionId` absent). A
request with no `previewId` is refused the same way after the VTA has computed and stored a
preview, which it carries in `details.preview`. The step-up's `reason` **MUST** name the
identity, the role and the operation in words — *"Rotate the attestation key of
vtc.example: every credential and status list issued from 26 September will be signed by
the new key"* — never a generic "confirm this action".

### 6.3 Whose evidence counts

An approval counts only when it is the **approver's own** evidence, as
[`auth/step-up/approve-response/0.7`](../../../../auth/step-up/approve-response/0.7/spec.md)
defines it: a proof by a key in the approver's own identifier (`didSigned`), a WebAuthn
assertion by a passkey registered to the approver (`webauthn`), or a statement by one of the
approver's bound step-up approvers (`approverSigned`). Therefore:

1. A proof by a key the VTA records as acting for an approver — a console signing key
   enrolled with `auth/signing-key/enroll`, or any other delegation — is **never** an
   approval, whatever evidence kind it claims; the answer is `subjectMismatch`. A delegated
   key may carry the initiator's *request* to the VTA, as it may carry any request the
   delegation covers; it may not answer the step-up that request triggers.
2. A key of the identity being changed — any role — is never an approver of that change,
   and a node's own keys are never an approver of any key-role change (VTI-KEY-115). A VTC
   console relays a change its human administrator approves; the VTA authorizes the
   administrator, not the VTC.
3. An `authenticator` key (§16) is approval evidence only for the person whose identity
   lists it.

### 6.4 Quorum

Where policy requires more than one approval, the VTA **MUST** count only distinct
approvers — distinct VIDs, each holding the operation's column in §7, each with its own
evidence under §6.3 — and **MUST** count the initiator once at most. Every approval is bound
to the `previewId`.

Once the initiator's approval is recorded and more are needed, the VTA answers
`outcome: "pendingApproval"` with the `KeyRoleApprovalState`, sends approve-requests bound
to the same `previewId` to the other eligible approvers, and applies the plan **itself**
when the count is reached — under the initiator's authorization, recording every approver
in the audit row. It **MUST NOT** apply a plan whose `expiresAt` has passed, and records it
`expired`. A pending plan holds the role: a second change to it is
`vta/dids:rotationInProgress` until the first is applied, expired or withdrawn — except a
**revocation for compromise, which is never blocked** by a pending, staged or overlapping
change to the same role. It supersedes it, and the superseded plan is recorded `aborted`.

### 6.5 What each change requires

The floor the VTI specification sets for a VTA's policy (VTI-KEY-180 to VTI-KEY-182):

| Change | Authority (§7) | Bound step-up | Quorum |
|---|---|---|---|
| `create` | context `admin` | **MUST** | MAY |
| Add, rotate, retire — `operational`, `messaging` | context `admin` | SHOULD | MAY |
| Add, rotate, retire — `attestation` | `super-admin` | **MUST** | SHOULD (2) |
| Rotate `update` (routine) | `super-admin` | **MUST** | SHOULD (2) |
| Replace the recovery commitment | `super-admin`, plus the new key's proof of possession | **MUST** | **MUST** (2) where two eligible approvers exist |
| Revoke — any role but `update` | context `admin` | **MUST** | **MUST NOT** be required |
| Revoke `update`; recover | `super-admin`, plus the recovery custodian's proof where it is used | **MUST** | **MUST NOT** be required |
| `update` (non-key members) | context `admin` | SHOULD | MAY |

Revocation is deliberately easier to reach than addition. Revoking removes authority and
cannot be used to gain it — any replacement is generated inside the VTA and never leaves —
so the worst an abusive revocation achieves is a denial of service the audit trail
attributes. Making a context administrator wait for a quorum before withdrawing a stolen
seal leaves it trusted for exactly as long as the wait. Replacing the recovery commitment
is the opposite: it is the one change that, made by an adversary (T5), silently disarms the
identity's recovery, and it is never urgent.

## 7. Authorization

Authority is checked against the caller's entry in the VTA's access-control list for the
**context the identity belongs to**, after the proof has established who the caller is
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)):

| Operation | `operational`, `messaging` | `attestation` | `update` | `authenticator` |
|---|---|---|---|---|
| `keys/list`, public projection | any ACL role reaching the context | same | same | same |
| `keys/list`, custody projection | context `admin`, `super-admin` | same | same | same |
| `keys/add`, `keys/rotate`, `keys/retire` | context `admin`, `super-admin` | `super-admin` | `super-admin` | — (passkey ceremony) |
| `keys/revoke` | context `admin`, `super-admin` | context `admin`, `super-admin` | `super-admin` | context `admin`, `super-admin`, or the person the identity represents |
| `recover` | — | — | `super-admin` of the recovering VTA | — |
| `create`, `update` | context `admin`, `super-admin` | | | |

- A VTA **MAY** narrow any cell and **MAY** delegate the `attestation` column to a named
  context administrator by an explicit ACL scope. It **MUST NOT** widen a cell by default,
  and **MUST NOT** grant any of these to an `initiator`, `application` or `reader` role, or
  to a service acting under its own authority.
- A caller that cannot reach the identity's context is answered `vta/dids:notFound`, never
  `permissionDenied`, so the refusal does not confirm the VTA holds the identity. A caller
  that can reach the context but lacks the cell is answered `permissionDenied`.
- Act scope is read through the ACL entry's act scope, never by testing whether its context
  list is empty.

## 8. Key states

A key moves `pending` → `staged` → `active` → `retiring` → `retired` on a planned rotation,
and from any published state to `revoked` on compromise. An `update` key is `committed`
until the identifier reveals it, then `active`, then `retired` when the authority moves on
(or `revoked`). `retired` and `revoked` are terminal; a key never returns from either.

## 9. Compromise

### 9.1 How a revocation is recorded

A planned retirement and a revocation for compromise **MUST** be distinguishable in the
identifier's history and in its current document (VTI-KEY-120), and the compromise time
**MUST** be published (VTI-KEY-121). The encoding, wherever the method lets the controller
write the document (method profile, §3.3 item 4):

- **Retired** — the verification method leaves its relationship, `keyRoles`, and
  `verificationMethod`.
- **Revoked** — the verification method leaves its relationship and `keyRoles`, and
  **stays in `verificationMethod`** with the `revoked` property of W3C Controlled
  Identifiers (the `revoked` term of the Multikey and Data Integrity contexts) set to
  `compromisedSince`. It stays there for the identity's life (VTI-KEY-126, VTI-KEY-127).

Why this encoding and not a VTI-specific member: `revoked` is already defined, already in
the contexts a Multikey document uses, and already acted on by generic Data Integrity
verifiers, which will not verify a proof by a method whose `revoked` time has passed. A
bespoke member would protect only VTI verifiers. Why keep the method listed rather than
record revocations in `keyRoles`: `keyRoles` holds only roles and method identifiers
(VTI-KEY-150), and a method absent from `keyRoles` is already relied on for nothing
(VTI-KEY-092), so a revoked method kept in `verificationMethod` is inert to a VTI verifier
and refused by a generic one. `revoked` is used **only** for compromise; a VTA **MUST NOT**
set it on a planned retirement, because a verifier reading it applies VTI-KEY-133 and
refuses everything the key signed that is not otherwise proven earlier.

### 9.2 The compromise time

`compromisedSince` is the earliest instant from which the key is to be treated as
compromised. A producer **SHOULD** err early; where it cannot establish the time it **MUST**
give the time the key was first published. Every proof between the real compromise and the
declared instant is one a verifier will accept (T8). A `compromisedSince` in the future is
refused (`vta/dids/keys/revoke:compromiseTimeInFuture`).

### 9.3 Emergency replacement

A revocation has **no overlap** (VTI-KEY-123) and is never blocked (§6.4). Where revoking
would leave the role with no active key — or the caller asks — the VTA generates (or, for
`operational` and `messaging`, derives at a fresh path) a replacement **in the same
change**, published `active` at once, with no cache-horizon wait. A verifier with a cached
document meets a signature by an unknown key and must re-resolve (VTI-KEY-134): the price
of not trusting a stolen key for a day. The replacement of a revoked `operational` or
`messaging` key **MUST NOT** be derived from the same path or any path derivable from
material the compromise exposed; where the compromise may have exposed the seed (T2),
replacements of every derived role **MUST** be generated, and the operator told to plan a
new identity.

### 9.4 Compromise of the change authority

`keys/revoke` with `role: update` takes a `scope`:

- **`key`** — one routine update key is believed exposed, the VTA's boundary is not (a key
  leaked through a signing side channel, say). The VTA moves the authority to its committed
  routine successor and commits fresh successors, carrying the recovery commitment forward.
- **`boundary`** — the VTA's boundary may be compromised, so its routine successor is too.
  Only the recovery key can authorize the change. The VTA computes the change — revealing
  the recovery key, revoking the routine keys, publishing replacements for every role key
  the compromise reached, and committing a fresh routine successor **and a fresh recovery
  commitment** supplied with its proof of possession — and answers `awaitingRecoveryProof`
  with a `RecoverySigningRequest`. The recovery custodian checks the change itself, signs
  it, and the proof is returned as `recoveryProof` on the apply request. A recovery key is
  used once: every change that reveals it commits a new one (VTI-KEY-128).

Where no uncompromised committed key remains — the recovery key is lost or exposed too —
the identity **MUST** be deactivated and a successor identity created (VTI-KEY-124). A VTA
**MUST NOT** go on operating an identity whose history it no longer solely controls.

### 9.5 Loss of the VTA

When a VTA is lost (T4), its generated keys are gone and nothing restores them: they are
not in any backup (§5.5). A replacement VTA takes the identity over with
[`vta/dids/recover`](../../recover/1.0/spec.md): it obtains the identity's history, generates
new `attestation`, routine `update` and — where the lost VTA's seed is not available —
`operational` and `messaging` keys, and computes one change that reveals the recovery key,
publishes the new keys, retires the lost `attestation` keys, and commits a fresh routine
successor and recovery commitment (VTI-KEY-117). The custodian signs it as in §9.4. Where
the loss may have exposed the keys rather than destroyed them, the lost keys are revoked,
not retired. Afterwards the node re-signs every status list at once and re-issues every
other attestation artefact in force no later than its renewal.

### 9.6 Re-issuance after an attestation-key compromise

A revoked `attestation` key makes everything it signed suspect from `compromisedSince`
(VTI-KEY-133). The node **MUST** re-sign every status list under a sound attestation key
at once — a status list nobody can trust re-admits everyone suspended — and **MUST**
re-issue, under a sound key, every attestation artefact still in force that its VTA's
signing record (VTI-VTA-006) shows it issued, with identical claims and expiry. For a VTC
that is [`vtc/credentials/reissue`](../../../../vtc/credentials/reissue/0.1/spec.md). The
revocation's response states the obligation in `followUp`, and its preview warns
`attestationReissuanceRequired`.

## 10. Overlap by role

### 10.1 The cache horizon

The **cache horizon** of a change is its publication time plus the longer of the
document's stated validity period (the method profile's TTL, VTI-KEY-060) and the
**verifier cache cap** — the oldest copy of a document a verifier may rely on (VTI-KEY-134,
24 hours). After it, no conforming verifier or sender acts on a copy older than that
change. A VTA **MUST** use the cap the VTI specification publishes, **MUST NOT** use a
shorter one, and reports the horizon as `cacheHorizonAt`.

### 10.2 Phases of a planned rotation

| Phase | Change | Successor | Predecessor | Earliest |
|---|---|---|---|---|
| **1. Publish** | successor added to the role's relationship and `keyRoles` | `staged` | `active` | — |
| **2. Wait** | none | `staged` | `active` | until the cache horizon of change 1 |
| **3. Switch** | none — the VTA signs (or prefers) the successor, never again the predecessor | `active` | `retiring` | `activatesAt` ≥ cache horizon |
| **4. Retire** | predecessor removed | `active` | `retired` | `overlapUntil` > `activatesAt` |

A VTA **MUST NOT** switch before the cache horizon, **MUST NOT** end a planned overlap by any
means before the horizon has passed and the successor has switched
(`vta/dids/keys/retire:notYetActive`, `vta/dids/keys/rotate:overlapTooShort`), and **MAY**
abort at any phase before retirement by retiring the successor.

### 10.3 By role

| Role | During the overlap | At retirement |
|---|---|---|
| `attestation` | Both in `assertionMethod` and `keyRoles`; new artefacts signed only by the active key | Predecessor removed and **destroyed**; what it signed verifies against the version current at issuance (VTI-KEY-132) |
| `operational` | Both in `authentication`; new traffic signed only by the active key; a peer accepts either | Predecessor removed and destroyed |
| `messaging` | Both in `keyAgreement`; the VTA holds both private halves and decrypts with either; senders may encrypt to either and **SHOULD** prefer the later | Predecessor removed and **destroyed**; a message to it afterwards is refused with a decryption failure that tells the sender to re-resolve |
| `update` | **No overlap.** The authority moves to a committed successor in one change, as the method profile says | — |
| `authenticator` | Not rotated; a person enrols a new passkey and removes the old | — |

A revocation has no overlap in any role (§9.3).

## 11. What a caller sees

`keys/list` answers in one of two projections and says which in `projection`:

- **`public`** — what the published document and history already tell anyone who resolves
  the identity: each published key's verification method, role, algorithm, public half,
  `active` or `revoked` state with `compromisedSince`, the update keys the method reveals,
  `recoveryCommitted`, and each rotation's kind, state and version identifiers. It **MUST
  NOT** include `pending` keys, the `staged` and `retiring` distinctions, deadlines,
  custody, boundaries, labels, initiators, approvers or reasons. `staged` and `retiring` are
  withheld because together they tell an observer which published key the VTA is not
  signing with — the one whose misuse would be noticed last.
- **`custody`** — the public projection plus custody. Only for the callers §7 names.

A VTA **MUST NOT** echo a `reason` or a boundary label into anything published.

## 12. Audit

A VTA **MUST** append an audit row ([`audit/_shared/0.1`](../../../../audit/_shared/0.1/audit.schema.json)
`AuditEnvelope`) for every one of these, **including refusals**:

| `action` | When |
|---|---|
| `did.create` | An identity was created, left pending, or refused |
| `did.keys.preview` | A preview was computed and stored |
| `did.keys.approval` | An approval bound to a preview was recorded or declined, with its evidence kind |
| `did.keys.add`, `did.keys.rotate`, `did.keys.retire`, `did.keys.revoke`, `did.update` | A change was applied, left pending, or refused |
| `did.keys.recoveryRequested`, `did.keys.recoveryProof` | A recovery signing request was issued; a recovery proof was received and verified or refused |
| `did.keys.recover` | A replacement VTA took an identity over |
| `did.keys.activate`, `did.keys.autoRetire` | The VTA switched or retired on schedule |
| `did.keys.destroy` | A retired key's private half was destroyed |
| `did.keys.expired` | A pending plan expired unapplied |

`actor` is the authenticated caller (for scheduled actions and a quorum's final apply, the
initiator whose authorization it acts under); `target` is the identity; `detail` carries the
role, verification-method identifiers and public-key fingerprints, base and resulting
version identifiers, `previewId` and `rotationId`, approvers with evidence kinds,
`compromisedSince`, `scope`, and the refusal code. It **MUST NOT** carry private key
material or boundary labels. A refusal of an `attestation` or `update` operation is the row
an incident review reads first; a VTA that audits only successes has no record of the
attempts that matter.

## 13. Judging material signed before a change

The VTI specification's *Judging material signed before a rotation* (VTI-KEY-130 to 135)
is normative. These tasks produce the evidence it relies on:

- A **retired** key's proofs are judged against the identifier version current at
  issuance, resolved by version (method profile, §3.3 item 3). For an attestation key the
  retiring change bounds the issuance time of everything it signed, because the key was
  destroyed (VTI-KEY-132).
- A **revoked** key's proofs are accepted only on evidence the signer cannot backdate that
  they predate `compromisedSince` (VTI-KEY-131). A `created` value in the proof, a
  credential's `validFrom`, or any other date the signer wrote is never such evidence (T8).

## 14. Algorithm agility and post-quantum keys

1. **No silent downgrade.** A successor keeps its predecessor's algorithm unless the request
   names another (`keyType`), and a named one must suit the role (§2.2) and be in the VTA's
   accepted set (VTI-KEY-013). A key-role template's layout mints every algorithm it lists,
   and a VTA that cannot mint one refuses the creation (`unsupportedKeyType`) rather than
   substituting — the template v2 preference fallback does not apply to key roles, because
   a fallback is a downgrade nobody approved.
2. **Proof sets.** A role may hold keys of several algorithms — an Ed25519 and an ML-DSA-44
   `attestation` key — that sign as a proof set (VTI-KEY-103), and a verifier evaluates every
   proof in its accepted set and refuses on any failure (VTI-KEY-102). A post-quantum key
   joins a role with [`keys/add`](../../keys/add/1.0/spec.md); a classical key leaves it,
   once verifiers are ready, with `keys/retire`, whose preview warns
   `roleWillHaveNoClassicalKey`.
3. **No conversion between curves.** An `x25519` key is generated or derived as X25519. A
   VTA **MUST NOT** publish an X25519 key computed from an Ed25519 key, in any role, even
   where a method's guidance suggests it (VTI-KEY-184): it would make the messaging key a
   function of the operational key.
4. **The change authority** uses what the method profile accepts. Commitment by hash keeps a
   committed key's public half unpublished until it is used, so a recovery key committed by
   hash is not exposed to an adversary who can recover private keys from public ones
   (T9) for as long as it is unused.
5. **Retiring an algorithm** (VTI-KEY-013) is a rotation of every key using it, role by
   role, through these tasks.

## 15. The `keyRoles` property

`keyRoles` lives **in the identifier document**, as a top-level member — not in the method's
own parameters, which differ by method and which many resolvers do not return, and not in a
linked resource, which a verifier would have to fetch and trust separately. It is a map from
each registered role the identity uses (except `update`) to a non-empty array of
verification-method identifiers (VTI-KEY-090, VTI-KEY-095).

Its JSON-LD term definition is fixed by the VTI specification (VTI-KEY-096):

```json
{
  "@context": {
    "keyRoles": {
      "@id": "https://w3id.org/vti#keyRoles",
      "@container": "@index",
      "@type": "@id"
    }
  }
}
```

The role names are index keys: they are VTI registry identifiers, compared as strings, not
IRIs. A document using it adds the VTI key-roles context (`https://w3id.org/vti/v1`) to its
`@context`. A VTA **MUST** render it from the roles it holds, never from a template.

## 16. `authenticator` keys (passkeys)

A passkey verification method is a key on a **person's** authenticator, enrolled by the
WebAuthn ceremony ([`vta/passkey-vms/*`](../../../passkey-vms/)), whose private half no VTA
holds. It has a role of its own, `authenticator`, because no other role fits it: it is not
the node's voice (`operational`), because it is held by a person and not the node; and it
signs no Data Integrity proof, only WebAuthn assertions.

1. An `authenticator` key is listed in `authentication` and `keyRoles.authenticator` of the
   identity of the **person who holds it** — their persona — and nowhere else.
2. It is **never** permitted on the durable identity of a VTA, a VTC or a VTN, or of any
   integration. A person's device in a node's document is a key the node does not hold
   that can speak as the node (T1 without even a compromise), and it lists a person in a
   node's public document (T10, VTI-KEY-151). The passkey ceremony **MUST** refuse such an
   identity, and `keys/add` refuses the role for every identity
   (`vta/dids:notApplicableToRole`).
3. A verifier **MUST** accept an `authenticator` key only for a WebAuthn assertion — sign-in,
   or `webauthn` step-up evidence (§6.3) — and **MUST NOT** accept a Data Integrity proof
   under it. A verifier of operational material accepts only `operational` keys. The two
   share `authentication`, and `keyRoles` is what tells them apart (VTI-KEY-073).
4. It is removed by the passkey ceremony's revoke, or for compromise (a lost device) by
   [`keys/revoke`](../../keys/revoke/1.0/spec.md), which the person the identity represents
   may also request. It is not rotated, added or retired by these tasks.

## 17. Family error codes

Carried by the `vta/dids` family namespace
([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications) rule 2). Every
task that can reach one **MUST** declare it.

```yaml
- code: vta/dids:notFound
  meaning: No such identity is held by this VTA, or the caller cannot reach its context.
  retryable: false
- code: vta/dids:keyNotFound
  meaning: The named key is not a key of this identity in the stated role and state.
  retryable: false
- code: vta/dids:versionConflict
  meaning: The identity's latest version no longer matches `expectedVersionId`.
  retryable: false
- code: vta/dids:previewStale
  meaning: The `previewId` is unknown, expired, applied, computed against a superseded version, or for a different request.
  retryable: false
- code: vta/dids:stepUpRequired
  meaning: Policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan.
  retryable: true
- code: vta/dids:unsupportedKeyType
  meaning: The key type cannot serve the role, is not in the VTA's accepted set, or cannot be minted by this VTA.
  retryable: false
- code: vta/dids:wouldEmptyRole
  meaning: The change would leave the role with no active key.
  retryable: false
- code: vta/dids:rotationInProgress
  meaning: The role already has a change staged, overlapping or awaiting approval or a recovery proof.
  retryable: false
- code: vta/dids:recoveryCommitmentRequired
  meaning: The identity would have no recovery commitment, or the offered one is not in another protection boundary or lacks a valid proof of possession.
  retryable: false
- code: vta/dids:recoveryProofInvalid
  meaning: The recovery proof does not verify over the previewed change under the committed recovery key.
  retryable: false
- code: vta/dids:notKeyRoleIdentity
  meaning: The identity was not created with key roles. These tasks do not operate on it; create a new identity with key roles instead.
  retryable: false
- code: vta/dids:methodNotEligible
  meaning: The identity's DID method has no profile, or its profile is ineligible for key roles; `details.missing` lists the criteria it fails.
  retryable: false
- code: vta/dids:methodParameterUnsupported
  meaning: A `methodParameters` member is not defined by the method profile, or may not change after creation.
  retryable: false
- code: vta/dids:notApplicableToRole
  meaning: The operation or a member given does not apply to this role.
  retryable: false
```

## 18. Adding a method

A method is added by publishing its profile — a `method-profiles/<name>.md` under §3.3's
headings and, where it takes parameters, a `method-profiles/<name>.schema.json` — in a new
version of this shared folder. Nothing in this document, the shared schema or the task
schemas changes: `DidMethodName` is a pattern, `methodParameters` is validated against the
profile, and `PublishedArtifact` and `RecoverySigningRequest` carry the method's own forms
as data. A profile that cannot meet §3.2 says so and declares the method ineligible, which
is also a profile: the refusal it produces is then a statement, not an accident.
