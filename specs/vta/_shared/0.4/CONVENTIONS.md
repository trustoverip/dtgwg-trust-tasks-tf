# PCS engine — conventions for the vta/pcs tasks

A community that runs **hidden vetting** admits an applicant on a zero-knowledge proof that
enough distinct eligible vetters vetted them, without the community learning which vetters
did ([`vtc/vetting/hidden/publish`](../../../vtc/vetting/hidden/publish/0.1/spec.md)). The
proof rests on a predicate credential system (PCS). The community is the scheme's *helper*;
every vetter and every applicant holds a PCS secret key, `usk`, from which every tag and proof
they make is derived.

These tasks let a VTA hold those keys and run every member-side PCS operation for its
principal. The VTA is the **PCS engine**. The client that drives it — a wallet, a desktop
agent — carries the messages between the engine, the community and the other party, and never
holds a `usk`. That is what lets a client with no PCS implementation, such as a mobile wallet,
take part in hidden vetting at all.

The tasks are:

| Side | Task | What the engine does |
|---|---|---|
| Applicant | [`vta/pcs/applications/create`](../../pcs/applications/create/0.1/spec.md) | Derives a fresh `usk` for one application and returns its PCS identifier |
| Applicant | [`vta/pcs/applications/show`](../../pcs/applications/show/0.1/spec.md) | Reads an application back |
| Applicant | [`vta/pcs/applications/delete`](../../pcs/applications/delete/0.1/spec.md) | Forgets an application, its key and its attestations |
| Applicant | [`vta/pcs/verify-attestation`](../../pcs/verify-attestation/0.1/spec.md) | Checks an attestation a vetter sent, and keeps it |
| Applicant | [`vta/pcs/prove`](../../pcs/prove/0.1/spec.md) | Builds the proof a join submission carries |
| Vetter | [`vta/pcs/root-request`](../../pcs/root-request/0.1/spec.md) | Builds the blinded request for a class credential |
| Vetter | [`vta/pcs/root-accept`](../../pcs/root-accept/0.1/spec.md) | Unblinds the community's answer into a class credential |
| Vetter | [`vta/pcs/tokens/request`](../../pcs/tokens/request/0.1/spec.md) | Builds one draw of the token drip |
| Vetter | [`vta/pcs/tokens/accept`](../../pcs/tokens/accept/0.1/spec.md) | Takes the community's answer to a draw into the wallet |
| Vetter | [`vta/pcs/tokens/abandon`](../../pcs/tokens/abandon/0.1/spec.md) | Discards a draw the community refused |
| Vetter | [`vta/pcs/attest`](../../pcs/attest/0.1/spec.md) | Spends a token and attests for an applicant |
| Vetter | [`vta/pcs/vetter/show`](../../pcs/vetter/show/0.1/spec.md) | Reads the vetter engine's state back |

A vetter's withdrawal of one attestation (the tag, and a proof of knowledge of the key behind
it) is an engine operation too, and will join this family once the community-side task that
accepts a withdrawal is specified: an engine task whose output no community can receive would
be a promise, not a protocol.

Each of those specifications references this file rather than restating it, and conformance
is anchored here. The wire shapes are in [`pcs-engine.schema.json`](pcs-engine.schema.json).

## 1. Terms

- **Engine.** The VTA, acting for its principal. Never call it a *helper*: in the scheme's own
  terms the helper is the community.
- **Vetter engine.** The engine's state for one vetter in one community: the vetter's `usk`, its
  PCS identifier, its class credentials by period, its attestation tokens, its draws in flight,
  and a log of what it attested. One per context and community (§3).
- **Applicant engine.** The engine's state for one application: the applicant's `usk`, its PCS
  identifier, the join DID it is bound to, and the attestations it has received. One per
  application.
- **Community parameters.** The community's published hidden-vetting parameters
  (`CommunityParameters`), exactly as its join manifest carries them.

## 2. Custody

1. An engine **MUST NOT** return a `usk`, a class credential, an unspent token, or the blinding
   state of a request in flight, in any response, under any task. No task in this family has a
   member that could carry one. This is the property the family exists for: the client carries
   messages and nothing it receives lets it act as the vetter or the applicant without the
   engine.
2. A `usk` belongs to a trust context, and an engine **MUST** derive it from that context's
   derivation base (VTI-KEY-030). A vetter's `usk` is derived once per community; an applicant's
   at a fresh position for each application.
3. An engine **MUST** keep each engine's state under the same protection as the key records of
   the context it belongs to, and **SHOULD** carry it in the VTA's backup. An applicant's
   attestations are not replaceable without the vetters sitting again; a vetter's credentials
   and tokens can be drawn again, but only in a later period or tick.
4. An engine **MUST NOT** sign or prove over bytes the caller supplies. Every input is parsed
   into the structure the operation defines — an identifier, statement metadata, a challenge,
   the community's answer — and the engine builds what it proves over itself (VTI-VTA-004,
   VTI-VTA-005).

## 3. One vetter key per community

A vetter engine **MUST** hold one `usk` for a given (context, community) for the whole of its
life, and **MUST NOT** create a second. Every tag a vetter makes for an applicant is a function
of its `usk` and the applicant's identifier, which is what lets the community tell two vetters
apart without knowing either. A vetter with two keys counts as two vetters in one proof.

The first [`vta/pcs/root-request`](../../pcs/root-request/0.1/spec.md) for a (context,
community) creates the vetter engine and records the `memberDid` it names. Every later task for
that pair runs against the same engine; a later `memberDid` that differs is refused with
`<slug>:memberMismatch`, because the community binds every token draw to the member that
signs it.

An applicant engine holds a fresh `usk` for each application and **MUST NOT** reuse one: an
applicant identifier seen in two applications links them.

## 4. Authorization

The authority is **standing over the context**: the VTA has recorded the caller as entitled
to act in the context the engine belongs to. Where that entitlement is expressed as a
capability, the registered values are:

- **`pcsVetter`** — for `vta/pcs/root-request`, `root-accept`, `tokens/request`,
  `tokens/accept`, `tokens/abandon`, `attest` and `vetter/show`;
- **`pcsApplicant`** — for `vta/pcs/applications/create`, `applications/show`,
  `applications/delete`, `verify-attestation` and `prove`.

See `Capability` in [`device/_shared`](../../../device/_shared/0.2/device-binding.schema.json).
Neither is `sign`, and neither confers it: these tasks produce PCS artefacts and nothing else, so
a caller that may drive a vetter's drip is not thereby a caller that may sign as the vetter. A
VTA **MUST NOT** satisfy a request for either under a grant of the other (VTI-VTA-007), and
**MUST NOT** require `sign` for any task in this family.

Holding `pcsVetter` lets a caller act as the vetter towards the community. `vta/pcs/attest` is
the most consequential act in the family — it is the vetter's word that a person was checked —
and a VTA's own policy may place a further gate on it. That gate is the VTA's to choose
(SPEC §7.3 item 13); this specification states only that the act is consequential.

Verifying the document's `issuer`, transport identity or `proof` establishes who asked
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## 5. Community parameters

Every task that runs the scheme takes the community parameters in its payload. The engine
**MUST** refuse a `suite` it does not implement with `<slug>:unsupportedSuite`, and **MUST**
refuse parameters it cannot decode with `<slug>:parametersInvalid`.

The engine does not authenticate the parameters: the caller read them from the community's
manifest, and the caller is the engine's own principal's delegate. Wrong parameters cost the
caller a credential, a token or a proof the community refuses. They cannot make the engine
reveal a key, because nothing in the parameters is an input to anything the engine returns
except an artefact the community verifies.

## 6. Audit

An engine **MUST** record every operation in this family that changes state or produces an
artefact — the operation, the caller, the context, the community and the time (VTI-VTA-006) —
and for a token draw, its label and tick.

An engine **MUST NOT** record, in clear, an applicant's PCS identifier, a tag, or statement
metadata beside the vetter that produced it. That pairing is exactly what hidden vetting keeps
from the community, and an audit trail is read by more people than the engine. Where an
operator needs to answer "did this vetter attest for this applicant", the engine **MAY** record
a digest of the identifier keyed with a secret local to the VTA, which answers the question for
someone holding both the identifier and the key, and for nobody else.

## 7. Error codes

Each task declares its own codes under its own slug. Where tasks share a condition, they use
the same local part:

| Local part | Meaning |
|---|---|
| `unsupportedSuite` | The parameters or the attestation name a suite the engine does not implement. |
| `parametersInvalid` | The community parameters cannot be decoded. |
| `notFound` | No engine exists for the named application, or for the (context, community). |
| `memberMismatch` | The `memberDid` differs from the one the vetter engine was created with. |
| `notEnrolled` | The vetter holds no class credential under a label the parameters name as live. |
| `noToken` | The vetter holds no free token under a live label. |
| `attestationInvalid` | An attestation, or its token, does not verify under the parameters. |
| `noUsableAttestation` | No held attestation is usable under the parameters as they stand. |

Every other refusal uses the framework's standard codes
([SPEC §8.3](/SPEC.md#83-standard-error-codes)): `permissionDenied` for a caller without
standing, `malformedRequest` for a payload the schema rejects.
