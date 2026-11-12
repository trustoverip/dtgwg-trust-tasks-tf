# Method profile — `did:webs` (KERI)

Covers `DidMethodName` **`webs`**.

**Status: eligible with the key-role designation extension; ineligible without it.** A
VTA **MAY** create a key-role identity under `did:webs` only with extension A below — the
*key-role designation* — and **MUST** refuse otherwise, with `vta/dids:methodNotEligible`
and `details.missing: [4, 6]`. Extension A is proposed to the did:webs specification
(Trust Over IP KERI Suite WG) and has a reference implementation in `affinidi-did-webs`
0.13.1, on KERI 2.0 (`affinidi-keri` 0.8): its `key_roles` module, with the proposal text
in its `docs/key-role-designation.md`. Until the did:webs specification adopts it, a
did:webs resolver that does not implement it derives a different document and so refuses
the published `did.json`: a key-role identity under `did:webs` resolves only for verifiers
whose resolver implements the extension (VTI-KEY-173). It is flagged for working-group
review.

**An optional, alpha method.** The VTI specification admits `did:webs` to a node's durable
identity as an *optional* method at alpha maturity (VTI-KEY-001, VTI-KEY-003): no peer is
required to resolve it. A node acts as a `did:webs` key-role identity only toward a peer
whose current document declares `"webs"` in `resolvedMethods` (VTI-KEY-008), and otherwise
fails the exchange with `methodNotDeclared`, sending nothing and never falling back to
another identifier (VTI-KEY-009). A peer that declares `did:webs` but whose resolver lacks
extension A still refuses the document, so until the extension is adopted an operator
**SHOULD** keep a `did:webvh` or `did:scid:vh:1` identity for every peer that cannot verify
this one, as [`vta/webs/_shared/0.2` §1.1](../../../../webs/_shared/0.2/CONVENTIONS.md)
says for the level-1 identities of that family. A key-role identity created now may have to
be created again, under a new identifier, as the did:webs and KERI specifications settle
(VTI-KEY-009, *Note (alpha)*).

## 1. Eligibility

Against [CONVENTIONS.md §3.2](../CONVENTIONS.md#32-eligibility):

| Criterion | did:webs |
|---|---|
| 1. Verifiable history | **Met.** The key event log (KEL), with witness receipts |
| 2. Commitment to successors, more than one | **Met, and better than required.** Next-key digests `n` with a next threshold `nt`, which may be fractionally weighted |
| 3. Rotation without change; deactivation | **Met.** The AID never changes; rotation to no next keys ends it |
| 4. Controller-chosen relationships | **Met with extension A; not met without it.** Without it, the DID document is *derived* from the KEL: every key in the current key state `k` becomes a verification method in **both** `authentication` and `assertionMethod` (a `ConditionalProof2022` method when `kt` exceeds 1), and the method gives the controller no normative way to publish other keys or choose their relationships. The current signing keys — the change authority — are therefore role-less verification methods in two relationships, which CONVENTIONS.md §2.1 items 1 and 4 forbid (VTI-KEY-071, VTI-KEY-091, VTI-KEY-093) |
| 5. Historic resolution | **Met.** The `versionId` DID parameter selects a KEL sequence number |
| 6. Room for `keyRoles` and `revoked` | **Met with extension A; not met without it.** Without it the document is generated, must equal the one derived from `keri.cesr`, and has no member for either |

The method's informative *Key Agreement* guidance — use the X25519 key corresponding to an
Ed25519 key in `k` — is refused for a key-role identity in any case: a VTA never publishes
such a key (CONVENTIONS.md §14 item 3, VTI-KEY-184). A party *encrypting to* a `did:webs`
identifier that has never designated key agreement keys or key roles — any identifier made
by KERI software — MAY use it, as KERI practice does (VTI-KEY-185); this is a rule for
reaching other parties' identifiers and gives a VTA nothing to publish.

### What would make it eligible

Either of these, decided by the working group with the KERI community:

- **A (recommended).** A did:webs extension in which the controller anchors a **role-key
  record** in the KEL — a seal in an interaction event over a record listing each
  verification method with its key, its role and, for a revoked one, `revoked` — and the
  derivation renders those methods into the relationships their roles require, renders
  `keyRoles` from the record, and renders the key-state keys `k` into **no** relationship.
  This meets 4 and 6 and keeps VTI-KEY-093 as it is.
- **B.** The extension of A without the last clause, with VTI-KEY-093 amended to tolerate a
  change-authority key the *method* renders into `authentication` and `assertionMethod`,
  provided it is absent from `keyRoles` (so VTI verifiers rely on it for nothing,
  VTI-KEY-092) and the VTA never signs anything with it but KEL events. Weaker: a generic
  verifier would accept a credential signed by the change authority, which is the most
  protected key, but also the one key whose misuse must be impossible.

## 2. The change authority

The `update` role is the key state: the current signing keys `k` with threshold `kt`
(routine, `active`), and the next-key digests `n` with threshold `nt` (`committed`).

- **Routine changes do not rotate the authority.** A role-key change is anchored by an
  interaction event (`ixn`) signed by `k` (`updateKeyRotates: false`). This is the
  difference from did:webvh that matters most: the keys that authorize the next rotation
  are never used for ordinary changes.
- **Rotation of `k` needs the recovery custodian.** `n` commits the routine successor R′
  and the recovery key K with weights such that K alone satisfies `nt` and R′ alone does
  not — `nt: ["1/2", "1"]` over `[R′, K]`. A rotation event (`rot`) is then signed by K, or
  by R′ and K. A VTA holding R′ cannot rotate the authority alone, so a VTA compromise (T2)
  yields interaction events — role-key changes the operator revokes after recovery — but
  never the identifier. This meets CONVENTIONS.md §5.4's second paragraph without a
  witness.
- **Every rotation exposes K, and commits a fresh K′.** A KERI rotation verifies each
  signature against the key it lists in its own `k`, and a signature counts toward the
  prior `nt` only by matching a digest in the prior `n`. K therefore appears in `k` of every
  rotation it signs, and stops being hidden behind its digest (T9) from then on. Each
  rotation **MUST** commit a fresh recovery key K′ in its `n`, and the VTA **MUST** refuse a
  rotation that carries K's digest forward. This is the same exposure a did:webvh recovery
  entry has, and it is why the recovery key is used only for rotations. Partial
  (custodial) rotation does not change it: it would let R′ rotate without K, which this
  profile exists to prevent.
- **The shape of each establishment event.** At level 3, inception and every rotation carry
  `n: [hash(R″), hash(K′)]` with `nt: ["1/2", "1"]`, and a rotation signed by K, or by R′
  and K, carries `k: [R′, K]` with `kt: "1"`. After it R′ alone signs routine interaction
  events. K can too, since it is in `k`; it is the custodian's key and has no reason to,
  and a VTA **MUST NOT** ask for it. A weighted `kt` such as `["1/2", "1"]` would stop R′
  signing routine changes alone, and is refused. Inception's own `k` is the routine key R
  alone, with `kt: "1"`: K is never in `k` before its first rotation.
- **Replacing the recovery commitment** is a rotation, and so needs K: it commits a fresh K′.
- **Raising the level** (`vta/dids/upgrade`, VTI-KEY-147). A KERI rotation is signed by keys
  the prior `n` committed, and each signer appears in the rotation's own `k`, so the first
  rotation of a raise makes a committed successor S the routine authority. Where S's custody
  meets the new level — a successor the VTA alone held, at level 2 — the raise is that
  rotation, committing as the new level's row above says, and an interaction event
  anchoring the first key-role designation (§4) with fresh role keys and the new
  `keyAssurance`. Where it does not — level 3 requires a generated routine key, and S was
  derived — a second rotation follows at once, signed by S, to a routine key minted as
  level 3 requires, and only that second rotation commits K. The events are published
  together, in one `keri.cesr`; the versions between them declare no level and are judged
  level 1 (VTI-KEY-199), and the designation's version is the first at the new level. The
  level-1 key itself is in no `n`, so it signs none of this; under `previousKeys: keep` it
  becomes the `operational` key, which the designation binds now that it is no key of the
  key state. From level 2 to level 3 the same applies, with the designation replacing the
  `attestation` keys (VTI-KEY-147 item 1).
- **By level** (CONVENTIONS.md §3.4). The shape above is level 3, and the next-key threshold
  stands in for the witness of CONVENTIONS.md §5.4 (VTI-KEY-129). At level 2, `n` commits
  R′ alone with `nt: "1"`. `keyAssurance.recoveryCommitment` names K's digest as it appears
  in `n`, and a verifier checks that K alone satisfies `nt` and that no set of the other
  committed keys does — which is what a level-3 structure means under this method, makes the
  version count as witnessed for its level (CONVENTIONS.md §5.4), and is a stronger check
  than did:webvh allows.

## 3. Versions

`versionId` carries the event's sequence number and SAID as `<sn>:<said>`;
`expectedVersionId` is compared on both. A past document is resolved with the `versionId`
DID parameter.

## 4. Rendering

Extension A, as the key-role designation implements it:

- **The record** is an ACDC the AID issues to itself in a registry it anchors, anchored by
  an interaction event signed by `k`. Under KERI 2.0 it is an ACDC 2.x credential, in a
  registry of its own, of schema `EBMCaUhHZqulCtix3fzRRsdW0_gfT3bMLsrp3c3ixpxx`; an
  identifier whose log is KERI 1.x uses the ACDC 1.x schema
  `EPulJFl5SEb8qb3ca5teXHRGiWmTsTkylGTrS2vB4ZQx`, and a resolver reads both. Its
  attributes carry `roles`, an object from a role identifier to a non-empty array of CESR
  public keys (`attestation` and `operational`: Ed25519, P-256 or secp256k1 signing keys;
  `messaging`: X25519, P-256 or secp256k1 key-agreement keys), and `revoked`, an array of
  `{ "key": <CESR>, "compromisedAt": <RFC 3339> }`. The 2.x schema adds `critical`, always
  `true`, so that a resolver that does not implement the designation fails resolution
  rather than render the key-state keys, and `keyAssurance` (below).
- **What each role is used for** is §2.1 of the conventions, unchanged by the method: an
  `attestation` key signs attestation artefacts and is the only key in `assertionMethod`; an
  `operational` key is the node's `authentication` key, the one its Trust Task requests and
  responses and its DIDComm sender prove with; a `messaging` key is its `keyAgreement` key
  and signs nothing. The registry has no role for a sealed-transfer producer assertion: a
  node signs one with its `attestation` key (VTI-KEY-072).
- **One record stands.** Issuing a record revokes the one before it. A planned rotation is
  a new record without the retired key, which then leaves the document (VTI-KEY-126). A
  revocation for compromise is a new record listing the key under `revoked`. The reference
  implementation's `Change::KeyRoles` replaces the designation and `Change::RevokeRoleKey`
  revokes one key for compromise; neither touches the key state.
- **Rotating a role key** (`vta/dids/keys/rotate` for `attestation`, `operational` or
  `messaging`) is a sequence of records, each anchored by an interaction event, so it needs
  `k` and never the recovery custodian: publishing the successor `staged` is a record
  listing predecessor and successor under the role; retiring the predecessor at
  `overlapUntil` is a record without it. A role key is never a key of the key state or of
  `n` (VTI-KEY-071), so rotating one changes no `k`, and rotating `k` changes no role key.
- **Role-key ids are derived from the key.** A role key's verification method id is
  `#<CESR key>`, so every rotation of a role key changes the id under which the document
  publishes that role (VTI-KEY-150). A node's own identity is affected the same way: see
  [`keys/rotate`](../../../keys/rotate/1.0/spec.md).
- **Revocations accumulate.** The resolver collects every `revoked` entry from every record
  the AID has issued, replaced ones included, and keeps the earliest compromise time per
  key, so a later record cannot un-revoke a key (VTI-KEY-127). The issuer carries the
  entries forward as well.
- **The document** lists as verification methods only the role keys (`JsonWebKey`, id
  `#<CESR key>`), each in the relationship its role requires and in no other
  (VTI-KEY-091), and the revoked keys, with `revoked` and in no relationship. `keyRoles`
  is a top-level member (VTI-KEY-090, VTI-KEY-095), and so is `keyAssurance`
  (VTI-KEY-197), rendered from the record's `keyAssurance` attribute, which only the 2.x
  schema has. `affinidi-did-webs` 0.13.1 neither issues nor renders that attribute yet, so
  until an implementation does, a `did:webs` key-role identity declares no level and is
  judged level 1 (VTI-KEY-199) — and an identity on the 1.x schema is level 1 for good, as
  far as the record is concerned. The key-state keys `k` and the
  `ConditionalProof2022` method are in no relationship and not in `keyRoles`
  (VTI-KEY-093); the reference implementation omits them from `verificationMethod`
  altogether, which the proposal leaves open.
- **Key agreement designations give way** to the `messaging` role; designated services and
  aliases apply as before.
- **A key-role template's `document`** (CONVENTIONS.md §4) is rendered as the
  designations render it: its `service` entries as designated services and its
  `alsoKnownAs` as designated aliases, under the rules of
  [`vta/webs/_shared/0.2` §9.2](../../../../webs/_shared/0.2/CONVENTIONS.md). Its
  `@context` and `resolvedMethods` are **ignored, not refused**: the host derives its own
  context, and a did:webs document renders no declaration of further methods — a node
  whose identity is `did:webs` declares `did:webs` by using it (VTI-KEY-008, and the open
  issue under VTI-KEY-009). A VTI node's template carries `resolvedMethods` for the methods
  that render it, and the member must not make the template unusable here. Any other
  member the derivation renders nothing for is refused, before a key is minted, with
  `vta/dids/create:templateNotKeyRole`: a VTA **MUST NOT** publish a document that drops
  what the template described.
- **The record is checked on resolution, failing closed.** A role key that is a key of the
  current key state or is committed in `n`, or a key under two roles or both bound and
  revoked, invalidates the record. An AID that has issued a record stays a key-role
  identity: if the standing record is invalid, or two stand at once, the document lists no
  role keys (its revoked keys remain) and never falls back to the key-state keys. An
  unrecognised role identifier is ignored (VTI-KEY-074) and refused at issuance.
- **Witnesses** (`b`, `bt`) receipt every event, and **SHOULD** be at least one in a
  boundary other than the VTA's (CONVENTIONS.md §5.4). The next-key threshold, not the
  witnesses, guards the recovery commitment; witnesses are what detect duplicity — a second
  key event log for the AID — which the one host the identifier names cannot show.

## 5. The cache horizon input

did:webs states no validity period, so the horizon is publication + the verifier cache
cap (24 hours).

## 6. Publication

`did.json` (`application/did+json`) and `keri.cesr` (`application/cesr`), both regenerated
on every change.

## 7. Recovery signing

`unsignedChange` is the rotation event's body, as JSON; `proofOptions` is
`{"format": "cesrIndexedSignature", "index": <K's index in the prior n>}`; `recoveryProof`
is the indexed signature, which the VTA attaches before receipting.

## 8. Method parameters

The witness set `b` and threshold `bt` at creation and update. No parameters schema is
published yet: it follows the working group's review of extension A.

## 9. Algorithms

What KERI accepts for key-state keys: Ed25519, secp256k1, P-256. Role keys as
CONVENTIONS.md §2.2.
