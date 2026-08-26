---
slug: vta/passkey-vms/enroll-submit
version: "0.1"
title: VTA Passkey-VM — Enroll Submit
summary: An administrator submits the WebAuthn registration result for an open ceremony; the VTA re-derives the public key from the attestation, rejects browser tampering, and publishes the passkey as a verificationMethod on the DID.
status: draft
targetFrameworkVersion: "0.5"
category: authentication
keywords:
  - vta
  - passkey
  - webauthn
  - verification-method
  - did
  - enrolment
  - attestation
  - webvh
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Submission mutates a DID document (a passkey verificationMethod is appended via a WebVH log entry) and is admin-gated. The VTA MUST attribute the change to a producer holding the admin role on the target DID's context; transport-independent producer identity is required so the published change is non-repudiable and auditable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Enrolment submission binds an authenticator into the agent's verification method set. A replayed submission re-binds one the owner has since removed.
sideEffects:
  level: mutating
  rationale: "Publishes a passkey as a verificationMethod on the DID via a log entry."
consequences:
  - "Adds a verificationMethod to the DID document; the change resolves publicly."
subjectPath: /did
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "The request carries raw WebAuthn registration material from a human's own authenticator — `attestationObject`, `authenticatorData`, `clientDataJson`, `credentialId` — together with an operator-chosen device `label`. Depending on the attestation format the authenticator selected, `attestationObject` can carry an AAGUID identifying the authenticator model and, under full-basic attestation, a certificate narrower than a model; `credentialId` and the derived key are stable identifiers for one physical device a person carries. Nothing here is confidential material the VTA holds on the producer's behalf — the published key is public and carries no secret — so this is `personal` rather than `secret`. Nothing is returned that the caller did not supply or already know, which is why `discloses` remains `none`."
retention:
  class: durable
  rationale: "Success appends the verificationMethod to the DID document through a WebVH log entry, and WebVH is append-only: the entry recording which key was added, under which label and transports, and at what time is the non-repudiable account of who could authenticate for this DID from that moment. A verifier checking an assertion made against an earlier version of the document needs it, so deleting the entry — if the log allowed it — would break verification of historic assertions and erase the only record of the change. `vta/passkey-vms/revoke` removes the method going forward; it does not unwrite the entry that added it."
errorCodes:
  - code: vta/passkey-vms/enroll-submit:unknownCeremony
    meaning: The `ceremonyId` is unknown, has expired, or has already been consumed. Re-running this submission will not succeed; the producer must obtain a fresh challenge.
    retryable: false
  - code: vta/passkey-vms/enroll-submit:ceremonyDidMismatch
    meaning: The submitted `did` does not match the DID bound to the ceremony at challenge time — a cross-DID replay.
    retryable: false
  - code: vta/passkey-vms/enroll-submit:invalidAttestation
    meaning: The WebAuthn attestation could not be parsed or verified, or its credential key could not be converted to a Multikey.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum:
            - unparseable
            - webauthnVerificationFailed
            - unsupportedAlgorithm
  - code: vta/passkey-vms/enroll-submit:publicKeyMismatch
    meaning: The browser-supplied `publicKeyMultibase` does not match the key the VTA re-derived from `attestationObject.authData`. The browser tampered with (or miscomputed) the public key; the submission is rejected.
    retryable: false
  - code: vta/passkey-vms/enroll-submit:alreadyEnrolled
    meaning: A passkey with this `credentialId` is already enrolled on the DID (the derived verificationMethod fragment already exists).
    retryable: false
related:
  - vta/passkey-vms/enroll-challenge
  - vta/passkey-vms/list
  - vta/passkey-vms/revoke
---

## Abstract

The **VTA Passkey-VM — Enroll Submit** Trust Task is step 2 of the two-step ceremony that publishes a WebAuthn passkey as a `Multikey` verificationMethod (purpose `authentication`) on a VTA-managed DID. After running `navigator.credentials.create` against the challenge from [`vta/passkey-vms/enroll-challenge`](../../enroll-challenge/0.1/spec.md), the *DID administrator* submits the WebAuthn registration result. The *VTA* verifies the ceremony, **re-derives the public key from the attestation**, rejects any mismatch with the browser-claimed value, and — on success — appends the verificationMethod to the DID document via a WebVH log entry. It returns the published verificationMethod and the WebVH version that recorded it.

The browser's `publicKeyMultibase` is **advisory only**. The authoritative public key is the one the VTA re-derives from `attestationObject.authData`; a divergence is treated as tampering and rejected (`vta/passkey-vms/enroll-submit:publicKeyMismatch`).

This task is **not idempotent**: it consumes a single-use ceremony. A retry of the same document fails with `vta/passkey-vms/enroll-submit:unknownCeremony` once the ceremony is consumed; to enrol again the producer obtains a fresh challenge.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Echo the `ceremonyId` from [`enroll-challenge`](../../enroll-challenge/0.1/spec.md) unchanged, and set `payload.did` to the same DID used at challenge time.
3. Populate the WebAuthn registration result: `credentialId`, `publicKeyMultibase`, `coseAlgorithm`, `attestationObject`, `clientDataJson`, `authenticatorData`. All byte-valued members are base64url-encoded with no padding.
4. Include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the admin role on the target DID's context, respond with the framework's `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)).
3. Where the `ceremonyId` is unknown, expired, or already consumed, respond with `vta/passkey-vms/enroll-submit:unknownCeremony`.
4. Where `payload.did` does not match the DID bound to the ceremony, respond with `vta/passkey-vms/enroll-submit:ceremonyDidMismatch`.
5. Verify the WebAuthn registration against the ceremony `challenge`. On a parse failure, verification failure, or unsupported credential algorithm, respond with `vta/passkey-vms/enroll-submit:invalidAttestation` and set `details.reason`.
6. **Re-derive** the credential public key from `attestationObject.authData` and compare it to the submitted `publicKeyMultibase`. On mismatch, respond with `vta/passkey-vms/enroll-submit:publicKeyMismatch`. The re-derived key — never the submitted one — is published.
7. Where a passkey with this `credentialId` is already enrolled on the DID, respond with `vta/passkey-vms/enroll-submit:alreadyEnrolled`.
8. On success, append the verificationMethod to the DID document via a WebVH log entry and return the published `verificationMethod` and the `webvhVersion`.

## Definitions

* **DID administrator.** The party invoking the task; identified by `issuer`. Holds the admin role on the target DID's context.
* **VTA.** The Verifiable Trust Agent that verifies the ceremony and mutates the DID document; identified by `recipient`.
* **Ceremony.** The single-use server-side registration state opened by [`enroll-challenge`](../../enroll-challenge/0.1/spec.md), identified by `ceremonyId`.
* **PasskeyVerificationMethod.** The published `Multikey` verificationMethod; see [`vta/_shared/0.1/passkey-vm`](../../../_shared/0.1/passkey-vm.schema.json).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Submit a completed WebAuthn registration

```json
{
  "id": "2b8d3e5c-9a43-4c12-be41-5d1f3a9c77e3",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-16T10:00:30Z",
  "threadId": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "payload": {
    "did": "did:webvh:QmcExampleScid:example.com",
    "ceremonyId": "cer_8Q1m2n3o4p5q6r7s",
    "credentialId": "AQIDBAUGBwgJCgsMDQ4PEA",
    "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "coseAlgorithm": -7,
    "attestationObject": "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YVjF...",
    "clientDataJson": "eyJ0eXBlIjoid2ViYXV0aG4uY3JlYXRlIiwiY2hhbGxlbmdlIjoiazlYeTJaIn0",
    "authenticatorData": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MFAAAAAA",
    "transports": ["internal", "hybrid"],
    "label": "MacBook Touch ID"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-05-16T10:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ verificationMethod, webvhVersion }`. The `verificationMethod` is the entry exactly as it now appears in the DID document — its `publicKeyMultibase` is the server-re-derived (authoritative) key. The producer **SHOULD** treat this as the source of truth, since the VTA may have rejected or corrected the browser-supplied value.

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### Successful enrolment

Response to the request example:

```json
{
  "id": "3c9e4f6d-0b54-4d23-cf52-6e2g4b0d88f4",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1#response",
  "threadId": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-16T10:00:31Z",
  "payload": {
    "verificationMethod": {
      "id": "did:webvh:QmcExampleScid:example.com#passkey-3q2r1s0tUvWxYz",
      "type": "Multikey",
      "controller": "did:webvh:QmcExampleScid:example.com",
      "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "webauthnCredentialId": "AQIDBAUGBwgJCgsMDQ4PEA",
      "webauthnTransports": ["internal", "hybrid"],
      "label": "MacBook Touch ID"
    },
    "webvhVersion": "3-QmExampleLogEntryHash"
  }
}
```

## Security & Privacy

### Data carried

This request carries raw output from a human's own authenticator, and the members
divide sharply into three groups.

**What gets published, permanently.** `credentialId` becomes the published
`webauthnCredentialId` and also determines the verificationMethod's `id` fragment,
which is `passkey-<base64url(sha256(credentialId))>` — content-derived so a verifier
can locate the method by recomputing the hash. `transports` becomes the advisory
`webauthnTransports`; verifiers **MUST NOT** make trust decisions based on transport
hints, but they are readable by anyone and they say what kind of device this is.
`label` is published verbatim. And the re-derived public key is published as
`publicKeyMultibase`. All of this is public by design and none of it is secret —
the point of the family is that a resolver can verify a WebAuthn assertion with no
callback to the VTA and no shared secret. Producers **SHOULD NOT** place sensitive
information in `label`, and should read "sensitive" broadly, because unlike most
mistakes this one cannot be withdrawn (see *Retention*).

**What is consumed and should not be.** `attestationObject` is the member the
previous version of this section passed over, and it is the privacy-sharpest thing
in the payload. The VTA needs exactly one thing from it — `authData`, from which it
re-derives the authoritative public key — but a raw CBOR attestation object can
carry considerably more, depending on the attestation format the authenticator
chose: an AAGUID identifying the authenticator model, and, under a full-basic
attestation, a certificate that can be narrower than a model. WebAuthn treats
attestation as a privacy-relevant disclosure for precisely this reason. Nothing in
this task uses the attestation statement for anything, so a VTA **SHOULD** parse
`authData`, derive the key, and discard the rest; it **MUST NOT** copy an AAGUID,
a certificate, or any other identifier taken from the attestation into the DID
document, where it would become permanent and public alongside the key.
`authenticatorData` similarly carries the RP ID hash, the user-verification flags,
and a signature counter — the counter being a well-known tracking channel — and
`clientDataJson` carries the origin the ceremony actually ran at.

**What is advisory and deliberately distrusted.** The browser-supplied
`publicKeyMultibase` is never authoritative. The VTA re-derives the public key from
`attestationObject.authData` and publishes *that*; a divergence
(`vta/passkey-vms/enroll-submit:publicKeyMismatch`) means the browser-side value was
tampered with or miscomputed, and publishing it would let an attacker bind a key
they do not control to the DID. This check is mandatory. `ext` reaches the VTA on a
document that is about to become a public DID-document write; a producer **MUST NOT**
place anything there expecting it to stay between the two parties, because the
natural implementation of an unrecognised extension on an enrolment is to carry it
through to the published method, exactly as `label` and `transports` are carried
through.

### Correlation

The published verificationMethod is a permanent, globally-readable record, and taken
together the methods on one DID are an inventory of a person's devices: how many
authenticators can act for this DID, roughly what kind each is (`webauthnTransports`
distinguishes `internal` from `hybrid` from `usb`), what their owner calls them
(`label`), and — because each was added by its own WebVH log entry — when each was
enrolled and in what order. Nobody has to be authorized to read any of it; they have
to resolve a DID.

Two joins deserve naming. First, `label` correlates *across* DIDs: an operator who
labels devices consistently, which is the whole point of labelling them, publishes
the same string in every DID document they administer, linking those DIDs to one
person's device set for anyone who looks. Second, the `id` fragment is
`sha256(credentialId)`, so it is a standing test oracle: a party that learns a
credential id by any route can confirm or refute that this DID holds that credential
without asking anyone. WebAuthn's rpId scoping limits the blast radius — a credential
registered at one relying party is a different credential, with a different id, from
one registered at another — but within this relying party the hash is a stable public
handle for one device.

Against all of that, the administrator stays out of the record entirely, and
`identifierScope: pairwise` on the DID administrator is the declaration of that
choice. The published method names the DID as its `controller`; it does not name the
party that enrolled it. A stranger resolving the DID learns which keys may
authenticate for it, not who added them, how many administrators exist, or which of
them is which. The account of *who* enrolled *what* lives at the VTA next to the
admin-role check and the **REQUIRED** `proof` that makes the change non-repudiable —
which is the right place for it, because that record has an audience of one and the
DID document has an audience of everyone. Nothing in this task would work better if
the administrator's identifier were recognisable elsewhere.

### Retention

Durable, and durable in the specific way that append-only logs are: the write cannot
be taken back. Success appends the verificationMethod to the DID document *via a
WebVH log entry*, and [`vta/passkey-vms/revoke`](../../revoke/0.1/spec.md) removes
the method by appending *another* entry. The current document stops listing the key;
the history still contains the entry that added it, with its `credentialId`, its
`label`, its transports, and its timestamp, retrievable by anyone who holds or
fetches the log. Revocation is a forward-looking act and not an erasure. The
practical consequence is worth stating plainly to whoever implements the enrolment
UI: a label chosen carelessly at enrolment cannot be un-chosen by revoking the key.

That permanence is not an accident and it is what makes the record worth keeping. The
log entry is the non-repudiable account of who could authenticate for this DID from
which moment, and a verifier evaluating an assertion made against an earlier version
of the document needs the historic entry to check it. Deleting it — were the log
mutable — would break verification of every assertion made while that key was live,
and would remove the only evidence that the change happened at all. A consumer that
wants a key gone must revoke, and must understand that revoking is the whole of what
is available.

Short-lived state goes the other way. `ceremonyId` is consumed by exactly one
submission and bound to one DID; a re-used or expired ceremony
(`vta/passkey-vms/enroll-submit:unknownCeremony`) and a cross-DID submission
(`vta/passkey-vms/enroll-submit:ceremonyDidMismatch`) are both rejected, and the
WebAuthn `challenge` from the ceremony is bound into `clientDataJSON` so a replayed
registration cannot be retargeted. Once the key is derived and published, the
ceremony record, the attestation object, and the client data have no further role,
and a VTA **SHOULD NOT** retain them.

### Consent/purpose

The material is submitted for one purpose: so that the VTA can re-derive a public key
it can trust and publish it on the DID, making assertions from that authenticator
verifiable by anyone who resolves the DID. The basis on which the VTA accepts it is
the administrator's admin role on the DID's context — the same gate as the challenge
step, re-applied here rather than inherited from the ceremony.

The limit follows from *why* the attestation is present at all. It is carried because
it is the only trustworthy source of the public key, not because the VTA is entitled
to whatever else the authenticator chose to put in it; deriving a device fingerprint,
an authenticator-model inventory, or a fleet report from attestations collected this
way is a different purpose from the one they were sent for. There is a second
asymmetry implementers should not paper over: an administrator submitting this
document understands they are publishing a *key*, and may not have registered that
they are also publishing a durable, public, unretractable entry naming a device and
the moment it was added. Surfacing that at the point the `label` is chosen is the
honest design. What the human does at their authenticator — the biometric or PIN
gesture the platform prompts for — is governed by the platform and is outside this
specification, which takes no position on what further gate a VTA applies before
accepting a submission.
