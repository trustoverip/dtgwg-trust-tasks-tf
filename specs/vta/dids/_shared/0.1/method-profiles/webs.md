# Method profile — `did:webs` (KERI)

Covers `DidMethodName` **`webs`**.

**Status: ineligible as the method is published.** A VTA **MUST** refuse to create a
key-role identity under `did:webs`, with `vta/dids:methodNotEligible` and
`details.missing: [4, 6]`, until the method provides what §1 below says it lacks. The rest
of this profile is the mapping that applies from then, so that the extension needed can be
proposed against a fixed target and the VTA work can start. It is flagged for
working-group review.

## 1. Eligibility

Against [CONVENTIONS.md §3.2](../CONVENTIONS.md#32-eligibility):

| Criterion | did:webs |
|---|---|
| 1. Verifiable history | **Met.** The key event log (KEL), with witness receipts |
| 2. Commitment to successors, more than one | **Met, and better than required.** Next-key digests `n` with a next threshold `nt`, which may be fractionally weighted |
| 3. Rotation without change; deactivation | **Met.** The AID never changes; rotation to no next keys ends it |
| 4. Controller-chosen relationships | **Not met.** The DID document is *derived* from the KEL: every key in the current key state `k` becomes a verification method in **both** `authentication` and `assertionMethod` (a `ConditionalProof2022` method when `kt` exceeds 1), and the method gives the controller no normative way to publish other keys or choose their relationships. The current signing keys — the change authority — are therefore role-less verification methods in two relationships, which CONVENTIONS.md §2.1 items 1 and 4 forbid (VTI-KEY-071, VTI-KEY-091, VTI-KEY-093) |
| 5. Historic resolution | **Met.** The `versionId` DID parameter selects a KEL sequence number |
| 6. Room for `keyRoles` and `revoked` | **Not met.** The document is generated, and must equal the one derived from `keri.cesr`; there is no member for either |

The method's informative *Key Agreement* guidance — use the X25519 key corresponding to an
Ed25519 key in `k` — is refused in any case (CONVENTIONS.md §14 item 3).

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

Per extension A: role keys from the anchored record, in their roles' relationships;
`keyRoles`, `keyAssurance` and `revoked` from the record; key-state keys in no relationship. Witnesses
(`b`, `bt`) receipt every event, and **SHOULD** be at least one in a boundary other than the
VTA's (CONVENTIONS.md §5.4).

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

To be defined with extension A (witness set `b` and threshold `bt` at creation and
update). No schema is published while the method is ineligible.

## 9. Algorithms

What KERI accepts for key-state keys: Ed25519, secp256k1, P-256. Role keys as
CONVENTIONS.md §2.2.
