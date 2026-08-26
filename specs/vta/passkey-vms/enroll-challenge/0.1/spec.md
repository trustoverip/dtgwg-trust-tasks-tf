---
slug: vta/passkey-vms/enroll-challenge
version: "0.1"
title: VTA Passkey-VM — Enroll Challenge
summary: An administrator of a VTA-managed DID requests a fresh WebAuthn registration challenge so a browser can create a passkey to be published as a verificationMethod on that DID.
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
  - challenge
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
  rationale: The challenge initiates a mutation of a DID document (a passkey verificationMethod will be published on success) and is admin-gated. The VTA MUST attribute the request to a producer holding the admin role on the target DID's context; transport-independent producer identity is required so the request cannot be replayed or attributed to the wrong party.
sideEffects:
  level: none
  rationale: "Requests a WebAuthn registration challenge; begins a ceremony, persists no key."
subjectPath: /did
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries the target `did` — an identifier for a VTA-managed resource whose document is publicly resolvable — and an optional operator-chosen `label` describing a device (\"MacBook Touch ID\"). Neither is an attribute of a natural person, but `label` is unconstrained free text that the ceremony will carry through to a published verificationMethod if it completes, so it is the member where personal data would arrive if a producer put it there."
retention:
  class: exchange
  rationale: "The VTA keeps the ceremony — `ceremonyId` bound to the challenge, the target DID and the label — only until `vta/passkey-vms/enroll-submit` consumes it or it expires. `sideEffects` is `none`: an abandoned ceremony leaves nothing behind, which is the right outcome for an operation a human can walk away from at the authenticator prompt. The durable record is created by `enroll-submit`, not here."
errorCodes:
  - code: vta/passkey-vms/enroll-challenge:didNotFound
    meaning: The target DID is not managed by this VTA, so no challenge can be issued.
    retryable: false
related:
  - vta/passkey-vms/enroll-submit
  - vta/passkey-vms/list
  - vta/passkey-vms/revoke
---

## Abstract

The **VTA Passkey-VM — Enroll Challenge** Trust Task is step 1 of the two-step ceremony that publishes a WebAuthn passkey as a `Multikey` verificationMethod (purpose `authentication`) on a VTA-managed DID. A *DID administrator* asks the *VTA* for a fresh WebAuthn registration challenge bound to a specific DID; the VTA returns the challenge and the relying-party / user parameters the browser needs to call `navigator.credentials.create`. The administrator's browser runs the WebAuthn ceremony and then completes enrolment with [`vta/passkey-vms/enroll-submit`](../../enroll-submit/0.1/spec.md), echoing the `ceremonyId` returned here.

Once enrolment completes, any verifier that resolves the DID can validate a WebAuthn assertion against the embedded public key — there is no callback to the VTA and no shared secret.

This task is **not idempotent**: each invocation mints a fresh, single-use ceremony. The returned `ceremonyId` is consumed by exactly one [`enroll-submit`](../../enroll-submit/0.1/spec.md) and cannot be reused.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.did` with the DID the new verificationMethod is to be added to.
3. Include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof).
4. Treat the returned `ceremonyId` as a single-use, short-lived secret and present it unchanged to [`enroll-submit`](../../enroll-submit/0.1/spec.md).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the admin role on the target DID's context, respond with the framework's `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)).
3. Where the target DID is not managed by this VTA, respond with `vta/passkey-vms/enroll-challenge:didNotFound`.
4. Where the passkey feature is not configured on this VTA, respond with the framework's `unavailable` (retryable).
5. On success, mint a fresh single-use ceremony bound to the target DID and the issued `challenge`, and return the WebAuthn registration parameters.

## Definitions

* **DID administrator.** The party invoking the task; identified by `issuer`. Holds the admin role on the target DID's context.
* **VTA.** The Verifiable Trust Agent that manages the DID and runs the WebAuthn ceremony; identified by `recipient`.
* **Ceremony.** A single-use server-side registration state that binds a `challenge`, the target DID, and the eventual [`enroll-submit`](../../enroll-submit/0.1/spec.md) together. Identified by `ceremonyId`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Request a challenge for a DID

```json
{
  "id": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-16T10:00:00Z",
  "payload": {
    "did": "did:webvh:QmcExampleScid:example.com",
    "label": "MacBook Touch ID"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-05-16T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response carries the WebAuthn registration parameters. All byte-valued members (`challenge`, `userHandle`) are base64url-encoded with no padding; the browser decodes them and passes them to `navigator.credentials.create`. The producer **MUST** echo `ceremonyId` unchanged into the subsequent [`enroll-submit`](../../enroll-submit/0.1/spec.md).

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### Issued challenge

Response to the request example:

```json
{
  "id": "1a7c2d4b-8f32-4b01-ad30-4c0e2f8b66d2",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1#response",
  "threadId": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-16T10:00:00Z",
  "payload": {
    "ceremonyId": "cer_8Q1m2n3o4p5q6r7s",
    "challenge": "k9Xy2Zr1Tq8sV3wB7nM0pL4eR6uA1cD2fG3hJ5kN8o",
    "rpId": "example.com",
    "rpName": "Example",
    "userHandle": "dXNlci1oYW5kbGUtZXhhbXBsZQ",
    "userName": "did:webvh:QmcExampleScid:example.com",
    "userDisplayName": "MacBook Touch ID",
    "timeoutMs": 120000
  }
}
```

## Security & Privacy

### Data carried

The request is a target `did` and an optional `label`. The label is the only
member a human authors, and it is easy to under-read: it does not stay between the
administrator and the VTA. The schema says it is carried through to the WebAuthn
user name and, if the ceremony completes, to the *published* verificationMethod —
so choosing a label here is making a public commitment at the one moment the
operator has least reason to think they are. A producer **SHOULD** describe a
device ("MacBook Touch ID", "office YubiKey") and **SHOULD NOT** use a person's
name, a location, a role, or a ticket reference. `label` is also not authenticated
as belonging to any particular device: it is informational, and consumers
**MUST NOT** make trust decisions based on it.

The response carries a full set of WebAuthn relying-party and *user* parameters —
`rpId`, `rpName`, `userHandle`, `userName`, `userDisplayName` — which is why
`exposure.discloses: none` deserves an explanation rather than a shrug. It holds
because every one of those values is derived from what the caller already supplied
or already knows: `userName` and `userDisplayName` come from the DID or the
operator-supplied label, and `rpId`/`rpName` are the VTA's own configuration. The
response tells the administrator nothing about any person that the administrator
did not put into the request. Implementers **SHOULD** keep it that way. Populating
`userDisplayName` from an account record — a real name, an email address, a
directory lookup — is the obvious convenience and it would silently turn this into
a disclosure the declaration does not cover, in a member that then travels to the
browser and into the authenticator's own credential list.

`userHandle` deserves its own note because WebAuthn is explicit that it is meant to
be an opaque, non-correlating value and warns against deriving it from a username
or an email. This task honours that: the schema specifies a **per-DID** handle, so
it carries nothing about the administrator and does not join one DID's enrolments
to another's. Two consequences follow. A VTA **MUST NOT** derive `userHandle` from
the administrator's identity or from any account identifier. And because a per-DID
derivation is a function of a publicly-resolvable DID, the handle may be
*predictable* to anyone who knows that DID — it is not a secret, confers nothing,
and **MUST NOT** be treated as an authenticator of anything.

`ext` on the response reaches a browser that is about to hand adjacent values to
`navigator.credentials.create`. A VTA **MUST NOT** use it to smuggle WebAuthn
parameters this schema does not name — a client that honours them is running a
registration ceremony this specification did not describe, and the anti-tamper
gate at [`enroll-submit`](../../enroll-submit/0.1/spec.md) checks the key, not the
ceremony's shape.

Three members of the response are free text and are bounded at 64 characters
each: `rpName`, `userName` and `userDisplayName`. Sixty-four is the figure
[WebAuthn’s user entity definition](https://www.w3.org/TR/webauthn-3/#dictdef-publickeycredentialuserentity)
tells authenticators to truncate these entity names to, so a longer value would
be silently cut on the device rather than carried. All three are
**REQUIRED** — WebAuthn's `PublicKeyCredentialCreationOptions` has no shape
without them — which is the departure from item 19's SHOULD noted here rather
than hidden. Their reader is the *authenticator*, which may store and redisplay
them at future ceremonies on hardware the VTA does not control; that is the
retention that matters, and it is outside this exchange. `userName` in
particular is documented as "the DID or the operator-supplied label", so a VTA
that puts a DID there has published it to the authenticator's own credential
list.

### Correlation

`rpId` is a single DNS name matching the origin the administration UI is served
from, and every passkey this VTA enrols shares it. Authenticators group credentials
by relying party, so a human's own device will display all passkeys for all DIDs
managed by that VTA as one account cluster — a real correlation, local to the
authenticator, that no member of this payload can vary. `userHandle` cuts the other
way: being per-DID, it keeps enrolments for different DIDs apart in the
authenticator's view, while deliberately joining every enrolment for the *same* DID
so that a re-registration replaces rather than duplicates.

The interesting separation in this task is between the administrator and the
subject. The target `did` at `subjectPath` is or will be publicly resolvable, and
[`enroll-submit`](../../enroll-submit/0.1/spec.md) writes into its document
permanently. The administrator's own identifier goes nowhere near it. That is what
`identifierScope: pairwise` on the DID administrator records: nothing in this task
requires the administrator to be recognisable outside its relationship with this
VTA, and a stranger who later resolves the DID learns which keys may authenticate
for it but not who added them or how many people hold admin over it. The account of
*who enrolled what* lives at the VTA, next to the admin-role check that authorized
it, and does not become public as a side effect of the key becoming public.

### Retention

Exchange-scoped, and unusually cleanly so. The VTA holds the ceremony — the
`ceremonyId` bound to its challenge, the target DID, and the label — only until
[`enroll-submit`](../../enroll-submit/0.1/spec.md) consumes it or it expires. The
`challenge` **MUST** carry at least 32 random bytes so that replay is infeasible
within that window, and the ceremony is single-use: the VTA **MUST** reject a
re-used or expired one (`vta/passkey-vms/enroll-submit:unknownCeremony`). Note that
`timeoutMs` is advisory — the authenticator and the browser apply their own limits
and the VTA applies its own expiry, which is the authoritative one; a consumer
**MUST NOT** read it as a retention commitment.

Because `sideEffects` is `none`, a ceremony that is never completed leaves nothing
behind at all. That is the correct default for an operation a human can abandon at
a biometric prompt, and it is worth preserving: a VTA **SHOULD** retain, after a
ceremony is consumed or expires, only enough to *refuse* a replay — the id, marked
spent — and not the challenge, the label, or the browser parameters, none of which
have any use once the ceremony has closed. The one member that outlives this
exchange is `label`, and it does so only by being copied into the durable record
that `enroll-submit` publishes.

### Consent/purpose

The parameters exist for one purpose: to let one browser attempt one WebAuthn
registration against one named DID. The basis on which the VTA issues them is the
administrator's admin role on that DID's context, which the VTA re-checks at
submission rather than treating this response as a token — holding a `challenge`
authorizes nothing, because it is a freshness value and not a capability.

The limit on reuse is correspondingly narrow. `userHandle` is stable, which makes
it tempting as a general account key; it is not one, and a VTA **SHOULD NOT** use
it outside the WebAuthn ceremonies it was minted for. The same applies to the
ceremony record: it is scoped to establishing that one submission belongs to one
challenge, not to building a history of enrolment attempts against a DID. What
happens at the authenticator itself — the biometric or PIN gesture the human is
asked for — is governed by the platform and is outside this specification, and this
specification takes no position on what further gate a VTA applies before issuing a
challenge.
