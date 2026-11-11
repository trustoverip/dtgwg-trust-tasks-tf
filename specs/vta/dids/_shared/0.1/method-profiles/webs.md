# Method profile — `did:webs` (KERI)

Covers `DidMethodName` **`webs`**.

**Status: eligible with the key-role designation extension; ineligible without it.** A
VTA **MAY** create a key-role identity under `did:webs` only with extension A below — the
*key-role designation* — and **MUST** refuse otherwise, with `vta/dids:methodNotEligible`
and `details.missing: [4, 6]`. Extension A is proposed to the did:webs specification
(Trust Over IP KERI Suite WG) and has a reference implementation in `affinidi-did-webs`
0.9.1 (`key_roles` module; proposal text in its `docs/key-role-designation.md`). Until the
did:webs specification adopts it, a did:webs resolver that does not implement it derives a
different document and so refuses the published `did.json`: a key-role identity under
`did:webs` resolves only for verifiers whose resolver implements the extension. It is
flagged for working-group review.

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
- **By level** (CONVENTIONS.md §3.4). The shape above is level 3, and the next-key threshold
  stands in for the witness of CONVENTIONS.md §5.4 (VTI-KEY-129). At level 2, `n` commits
  R′ alone with `nt: "1"`. `keyAssurance.recoveryCommitment` names K's digest as it appears
  in `n`, and a verifier checks that K alone satisfies `nt` — which is what a level-3
  structure means under this method, and a stronger check than did:webvh allows.

## 3. Versions

`versionId` carries the event's sequence number and SAID as `<sn>:<said>`;
`expectedVersionId` is compared on both. A past document is resolved with the `versionId`
DID parameter.

## 4. Rendering

Extension A, as the key-role designation implements it:

- **The record** is an ACDC of schema `EPulJFl5SEb8qb3ca5teXHRGiWmTsTkylGTrS2vB4ZQx`,
  issued by the AID in a registry it anchors, and anchored by an interaction event. Its
  attributes carry `roles`, an object from a role identifier to a non-empty array of CESR
  public keys (`attestation` and `operational`: Ed25519, P-256 or secp256k1 signing keys;
  `messaging`: X25519, P-256 or secp256k1 key-agreement keys), and `revoked`, an array of
  `{ "key": <CESR>, "compromisedAt": <RFC 3339> }`.
- **One record stands.** Issuing a record revokes the one before it. A planned rotation is
  a new record without the retired key, which then leaves the document (VTI-KEY-126). A
  revocation for compromise is a new record listing the key under `revoked`.
- **Revocations accumulate.** The resolver collects every `revoked` entry from every record
  the AID has issued, replaced ones included, and keeps the earliest compromise time per
  key, so a later record cannot un-revoke a key (VTI-KEY-127). The issuer carries the
  entries forward as well.
- **The document** lists as verification methods only the role keys (`JsonWebKey`, id
  `#<CESR key>`), each in the relationship its role requires and in no other
  (VTI-KEY-091), and the revoked keys, with `revoked` and in no relationship. `keyRoles`
  is a top-level member (VTI-KEY-090, VTI-KEY-095), and so is `keyAssurance`
  (VTI-KEY-197), rendered from the record's `keyAssurance` attribute. The record schema
  above predates key assurance levels and has no such attribute; a revision adding it is a
  new schema, with a new SAID, and until it is published a `did:webs` key-role identity
  declares no level and is judged level 1 (VTI-KEY-199). The key-state keys `k` and the
  `ConditionalProof2022` method are in no relationship and not in `keyRoles`
  (VTI-KEY-093); the reference implementation omits them from `verificationMethod`
  altogether, which the proposal leaves open.
- **Key agreement designations give way** to the `messaging` role; designated services and
  aliases apply as before.
- **The record is checked on resolution, failing closed.** A role key that is a key of the
  current key state or is committed in `n`, or a key under two roles or both bound and
  revoked, invalidates the record. An AID that has issued a record stays a key-role
  identity: if the standing record is invalid, or two stand at once, the document lists no
  role keys (its revoked keys remain) and never falls back to the key-state keys. An
  unrecognised role identifier is ignored (VTI-KEY-074) and refused at issuance.
- **Witnesses** (`b`, `bt`) receipt every event, and **SHOULD** be at least one in a
  boundary other than the VTA's (CONVENTIONS.md §5.4).

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
