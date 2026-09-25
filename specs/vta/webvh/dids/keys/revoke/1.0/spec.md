---
slug: vta/webvh/dids/keys/revoke
version: "1.0"
title: "VTA WebVH DIDs — Keys Revoke"
summary: "An administrator revokes a did:webvh key for compromise: it leaves its role in one entry without overlap, the compromise time is recorded so verifiers stop trusting what it signed from then on, and a generated replacement takes its place where the role would otherwise be empty."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - vta
  - webvh
  - key roles
  - revoke
  - compromise
  - incident response
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A revocation tells every verifier to stop trusting a key's signatures from a stated instant, and for an attestation key it obliges the node to re-issue its credentials. An unattributable revocation is a denial of service with no record of who caused it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Irreversible, and `compromisedSince` is bounded by it: a compromise time later than the request's own issuedAt is refused, which is only checkable if issuedAt is present."
sideEffects:
  level: destructive
  rationale: "Irreversible and authority-shifting: the key is withdrawn from the identity with no overlap, verifiers are told to distrust what it signed from the compromise time, a replacement may take its place, and under pre-rotation the entry moves the update key."
consequences:
  - "The key stops being trusted immediately — there is no overlap."
  - "Verifiers are told to distrust anything the key signed from the compromise time onward, unless there is independent evidence it was signed earlier."
  - "Revoking an attestation key obliges the community to re-sign every status list at once and re-issue every credential still in force that the key signed."
  - "A replacement key may be generated and published in the same entry."
  - "Appending the entry also rotates this DID's update key to its committed successor."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview, the revoked key's public record, any replacement's public record, and — for an attestation revocation — counts of what the node must re-issue. No key material."
retention:
  class: durable
  rationale: "The revocation record is what every later verification of the key's signatures is judged against, for as long as anything it signed exists."
errorCodes:
  - code: "vta/webvh/dids:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webvh/dids:keyNotFound"
    meaning: "The verification method is not a published key of this DID in the stated role."
    retryable: false
  - code: "vta/webvh/dids:versionConflict"
    meaning: "The DID's latest entry no longer matches `expectedVersionId` — for an update-key compromise, possibly an entry the thief appended."
    retryable: false
  - code: "vta/webvh/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, already applied, based on an entry the log has moved past, or for a different request."
    retryable: false
  - code: "vta/webvh/dids:stepUpRequired"
    meaning: "The VTA's policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/webvh/dids:unsupportedKeyType"
    meaning: "The replacement's key type cannot serve the role, or is not in the VTA's accepted set."
    retryable: false
  - code: "vta/webvh/dids/keys/revoke:compromiseTimeInFuture"
    meaning: "`compromisedSince` is later than the request's `issuedAt`."
    retryable: false
  - code: "vta/webvh/dids/keys/revoke:noSoundSuccessor"
    meaning: "An update-key revocation cannot proceed: no committed successor update key remains that is not itself compromised. The identifier must be deactivated and a successor identity established (VTI-KEY-124)."
    retryable: false
  - code: "vta/webvh/dids/keys/revoke:notApplicableToRole"
    meaning: "`verificationMethod` is missing for a role that needs it, or `verificationMethod` or `replacement` was given for `role: update`."
    retryable: false
related:
  - vta/webvh/dids/keys/retire
  - vta/webvh/dids/rotate-keys
  - vta/webvh/dids/keys/list
  - vta/webvh/dids/delete
  - keys/revoke
  - auth/step-up/approve-request
---

## Abstract

**VTA WebVH DIDs — Keys Revoke** is incident response for one key of a DID. The key is
believed to be in someone else's hands from some time — the **compromise time**, often
earlier than the moment it was discovered — and from that time its signatures establish
nothing.

It differs from a planned retirement ([`keys/retire`](../../retire/1.0/spec.md)) in every
way a verifier cares about:

| | Retire (planned) | Revoke (compromise) |
|---|---|---|
| Overlap | Staged, overlapped | **None** — withdrawn in one entry (VTI-KEY-123) |
| What it says about the past | Everything the key signed is sound | Nothing the key signed from `compromisedSince` is, without independent evidence |
| Recorded as | Removal from the document | Removal from relationships and `keyRoles`, **plus** the method kept with `revoked` = `compromisedSince` ([conventions §7](../../../../../_shared/0.3/CONVENTIONS.md#7-judging-material-signed-before-a-change)) |
| Attestation role | Key destroyed | Key disabled at once; the node must re-sign status lists and re-issue credentials in force (VTI-KEY-133) |
| Blocked by an in-progress change? | Yes | **Never** — it supersedes it |

For the `update` role, revocation is the recovery pre-rotation exists for: the log's update
key moves to a committed successor the thief never held (VTI-KEY-124). Where none remains,
the identity cannot be recovered and must be deactivated.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../../../_shared/0.3/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. **Stop using the key at once** — on receipt of a request it is authorized to apply, and
   before approval, publication or any other step completes. Until the entry is published
   the VTA refuses every signing request naming the key; a revocation waiting on a second
   approver must not leave a stolen key's legitimate twin still signing.
2. Refuse `compromisedSince` later than the request's `issuedAt`
   (`vta/webvh/dids/keys/revoke:compromiseTimeInFuture`).
3. In one entry: remove the key from its relationship and from `keyRoles`; keep it in
   `verificationMethod` with `revoked` set to `compromisedSince`; and, when `replacement` is
   given or the role would otherwise have no active key, generate a replacement in the same
   role and publish it **active** — an emergency replacement is not staged, because the
   alternative is a role with no usable key, and verifiers with cached documents will
   re-resolve on the unknown method (VTI-KEY-134).
4. Supersede any change to the same role that is pending, staged or overlapping, recording
   it `aborted` (conventions §3.3). Where the revoked key is a rotation's successor, return
   the predecessor to `active` only if it is not itself named compromised.
5. For `role: update`, move the update key to a committed successor and commit fresh
   successors, in one entry. Where no committed successor remains, or the log already holds
   entries the VTA did not append (the thief used the key), refuse with
   `vta/webvh/dids/keys/revoke:noSoundSuccessor` or `vta/webvh/dids:versionConflict`
   respectively, and **MUST NOT** continue to operate the identifier as if it were sound.
6. For an `attestation` key, report in `followUp` how many status lists and attestation
   artefacts still in force its signing record shows the key signed, so the node can
   re-sign and re-issue them (VTI-KEY-133), and emit the warning
   `attestationReissuanceRequired` in the preview.
7. Never destroy the revoked key's record, and never reactivate it.

A consumer **MAY** allow a lower approval threshold for a revocation than for the addition
or rotation of the same role (conventions §2) — an abusive revocation can deny service but
cannot grant authority, and delaying a genuine one extends the thief's window.

## Authorization

Context `admin` or `super-admin` for `operational`, `messaging` and `attestation`;
`super-admin` for `update`
([conventions §2](../../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.role` (REQUIRED) — the role of the compromised key.
`payload.verificationMethod` — the compromised key; REQUIRED except for `update`, forbidden for it. A VTA **MUST** refuse a request that breaks this, or gives `replacement` for `update`, with `vta/webvh/dids/keys/revoke:notApplicableToRole`.
`payload.compromisedSince` (REQUIRED) — the compromise time. Err early; if unknown, the time the key was first published.
`payload.replacement` (OPTIONAL) — generate a replacement of this key type in the same entry.
`payload.expectedVersionId`, `payload.dryRun`, `payload.previewId`, `payload.reason` — as in [conventions §3](../../../../../_shared/0.3/CONVENTIONS.md#3-approval).
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

### A VTC's attestation key was on a host that was breached

```json
{
  "id": "urn:uuid:b1c2d3e4-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/revoke/1.0",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-25T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "attestation",
    "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttestation",
    "compromisedSince": "2026-09-24T00:00:00Z",
    "replacement": { "keyType": "ed25519" },
    "reason": "INC-2291: signing host breach"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-25T11:00:00Z",
    "verificationMethod": "did:key:z6MkContextAdmin#z6MkContextAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:b1c2d3e4-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/revoke/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkContextAdmin",
  "issuedAt": "2026-09-25T11:00:03Z",
  "threadId": "urn:uuid:b1c2d3e4-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "applied",
    "newVersionId": "9-QmEntryNine",
    "keys": [
      { "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttestation", "role": "attestation", "relationships": [], "keyType": "ed25519", "publicKeyMultibase": "z6MkNewAttestation", "state": "revoked", "compromisedSince": "2026-09-24T00:00:00Z", "removedInVersionId": "9-QmEntryNine" },
      { "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkReplacement", "role": "attestation", "relationships": ["assertionMethod"], "keyType": "ed25519", "publicKeyMultibase": "z6MkReplacement", "state": "active", "publishedInVersionId": "9-QmEntryNine" }
    ],
    "rotation": { "rotationId": "rot-0004", "role": "attestation", "kind": "compromise", "state": "completed", "startedInVersionId": "9-QmEntryNine" },
    "followUp": { "statusListsToResign": 3, "artefactsToReissue": 412 },
    "serverless": false
  }
}
```

## Security & Privacy

### Data carried

The request names the compromised key, the compromise time and optionally a replacement type; the
response carries public records and, for an attestation key, counts of what must be re-issued.
No key material, and `reason` is never published — it goes to the audit trail and to the other
administrators of the context.

**Declare the compromise time early.** Every signature between the real compromise and the
declared `compromisedSince` is one a verifier will accept. A client **SHOULD** default the field to
the key's first publication and make the operator choose a later time deliberately.

**The record protects relying parties, not the removal.** Removing a key from the document is
indistinguishable, to a verifier, from a planned retirement, which vouches for the key's past.
Keeping the method with `revoked` set is what tells a verifier to demand independent evidence of
issuance time (VTI-KEY-131).

**Custody-level revocation is not enough.** [`keys/revoke`](../../../../../../keys/revoke/0.1/spec.md)
on a role key is refused (`keys/revoke:boundToIdentifier`): disabling the custodian record while the
document still publishes the key leaves verifiers trusting a key nobody will announce as withdrawn.

### Correlation

The world learns that the key was revoked, and from when; that is the point. It does not learn why,
by whom, or how many credentials are affected — `followUp` is custody data.

### Retention

The revocation record is kept for as long as anything the key signed may be verified, and the
revoked method stays in the document for the same period.

### Consent/purpose

The purpose is to stop a stolen key from being trusted. The consent surface states the compromise
time in words ("anything signed by this key since 24 September will be rejected"), the re-issuance
obligation for an attestation key, and the replacement; a VTA **MAY** set a lower approval threshold
for revocation than for addition ([conventions §2](../../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).
