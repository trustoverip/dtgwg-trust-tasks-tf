---
slug: device/register
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — Register
summary: A newly-onboarded Companion or Service claims its device record on the maintainer, supplying form factor, display name, HPKE public key, and optional device attestation; wraps the maintainer's existing provision-integration → acl/swap-key bootstrap.
status: draft
targetFrameworkVersion: "0.2"
category: identity
keywords:
  - device
  - registration
  - enrolment
  - companion
  - service
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: device
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Registration binds a device identity to operator-significant capabilities. The producer's identity MUST be verifiable so the maintainer can attribute the registration to a specific consumer key (the one the operator authorised in provision-integration).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Registration binds a device to the account. A replayed registration re-admits a device the owner has since removed, which is the second execution SPEC §7.2 item 11 exists to absorb and can only absorb inside a bounded window.
sideEffects:
  level: mutating
  rationale: "Claims a device record on the maintainer; revocable via disable/wipe."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    The response is not an acknowledgement: it returns the whole
    `DeviceBinding` — the maintainer-assigned `deviceId`, the `consumerDid`
    the device authenticates with, its display name and platform string, the
    attestation, the key-custody tier, and the capability set the maintainer
    granted it. Descriptive data about one device rather than released credential
    material, which is what separates `metadata` from `secret` here.
    Inbound, the request carries `displayName` — free text whose conventional
    value names its owner, as the shared schema's own examples ("Glenn's MacBook
    — Chrome") show — alongside a `platform` descriptor and a platform-issued
    `attestation` blob. That is data about an identifiable person's hardware, not
    merely about a resource, which is why `ingests` is `personal` and not
    `metadata`.
retention:
  class: durable
  rationale: >-
    The DeviceBinding this task creates is the maintainer's permanent record of
    which device holds which capabilities. It survives decommissioning by design
    — `device/disable` and `device/wipe` stamp `disabledAt` and `wipedAt` rather
    than removing the row, and `deviceId` is never re-used — so that an action
    attributed to a device years ago is still attributable. A consumer that
    deleted the binding would keep the audit line and lose the only thing that
    resolves it to a device.
errorCodes:
  - code: device/register:noPendingEnrolment
    meaning: The producer's DID is not the result of a recent provision-integration + acl/swap-key flow. Registration cannot proceed without first being granted via the maintainer's normal enrolment path.
    retryable: false
  - code: device/register:attestationFailed
    meaning: The supplied device attestation could not be verified against the platform's attestation infrastructure.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: "string", enum: ["signature_invalid", "untrusted_root", "stale", "rooted_device", "unsupported_attestation_kind"] }
  - code: device/register:alreadyRegistered
    meaning: A DeviceBinding for this consumer DID already exists. Re-registration is rejected; the consumer SHOULD swap to a fresh key via acl/swap-key and retry.
    retryable: false
  - code: device/register:hpkeKeyInvalid
    meaning: The `hpkePublicKey` is not a valid X25519 did:key.
    retryable: false
---

## Abstract

The **Device — Register** Trust Task is the public-facing wrapper around the maintainer's existing two-phase enrolment:

1. **Provision-integration** (out of scope of this Trust Task): An existing admin uses the maintainer's `provision-integration` flow to mint a VTA-derived long-term key for the new device and grant an ephemeral did:key into the ACL with the target role and scopes. The maintainer seals the long-term-key bundle to the ephemeral's HPKE key and delivers it to the new device.
2. **ACL swap** (an `acl/swap-key/0.1` Trust Task): On first connection, the new device swaps out the ephemeral and binds the long-term VTA-derived key into the ACL.
3. **Device register** (this task): The device claims its DeviceBinding record on the maintainer by supplying form factor, display name, HPKE public key, and optional device attestation. The maintainer creates the DeviceBinding and returns it.

The three steps together are the canonical onboarding for every Companion and Service. Third parties writing new client implementations follow the same three-step sequence.

## Conformance

A conforming **producer** **MUST**:

1. Have completed provision-integration and acl/swap-key before issuing this task. (The maintainer will reject otherwise.)
2. Populate `consumerKind`, `displayName`, `hpkePublicKey`.
3. **SHOULD** populate `attestation` when the platform supports it. Maintainers MAY require attestation by policy.
4. **SHOULD** populate `keyCustody` — especially mobile Companions — declaring how private keys are held (`tier` + signing/keyAgreement algorithms). See the [Mobile Key-Custody Profile](../../../../docs/design-notes/mobile-key-custody-profile.md).
5. Carry a `proof`.

> **Note.** Push wake-up is **not** configured here. A device that needs to be woken in the background (any mobile Companion) first registers its platform push token with a **push gateway** (which returns an opaque `WakeHandle`), then conveys that handle to its VTA via [`device/set-wake`](../../set-wake/0.1/spec.md) and to its mediator via the [push wake-up binding](../../../../bindings/push/0.1/spec.md)'s `set-device-info` exchange. The raw push token is held by the **gateway** only — never by the mediator or the maintainer/VTA, which hold just the opaque handle and the VTA-owned trigger allowlist. The maintainer's view here is the non-secret `pushCapable` flag on the returned `DeviceBinding`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Verify proof. The producer's DID MUST already be in the ACL (placed there by step 2 above). If not → `device/register:noPendingEnrolment`.
2. Verify any supplied `attestation` against the platform's attestation infrastructure. Failure → `device/register:attestationFailed` with `details.reason`. The maintainer's policy decides whether a failed attestation is fatal or merely downgrades the device's policy class.
3. Treat any supplied `keyCustody` as **policy input** (not a gate): a `software`-tier device MAY be assigned a stricter policy class (shorter sessions, more frequent step-up), the same way a missing/failed attestation is handled.
4. If a DeviceBinding for this DID already exists → `device/register:alreadyRegistered`. (Re-registration is intentionally not idempotent — the consumer must rotate keys and try again.)
5. Validate `hpkePublicKey` as a well-formed X25519 did:key. Failure → `device/register:hpkeKeyInvalid`.
6. Create the DeviceBinding with `registeredAt = now`, `lastSeenAt = now`, capabilities mirrored from the ACL entry. Return the DeviceBinding.

## Payload

`consumerKind` (REQUIRED), `displayName` (REQUIRED), `hpkePublicKey` (REQUIRED), `platform` (optional), `attestation` (optional, RECOMMENDED), `keyCustody` (optional, RECOMMENDED for mobile Companions). Push wake-up is registered with the mediator, not here (see the note above).

## Response

`binding` — the created DeviceBinding.

## Security & Privacy

### Data carried

The request is a device fingerprint assembled by the device about itself.
`displayName` is the personal member: it is free text bounded only at 128
characters, and the convention the shared schema documents — "Glenn's MacBook —
Chrome", "iPhone 17" — attaches a person's given name to a hardware model, which
is what turns a device record into a record about a human. Nothing in this
specification requires that; a producer that names a device by its role rather
than its owner registers a conforming device and discloses one fact less.
`platform` is a free-form build string ("macOS 16 / Chrome 142") and
`attestation` carries a platform-issued identifier whose shape depends on `kind`
— a WebAuthn `aaguid` naming the authenticator model, an Apple App Attest
`keyId`, a Play Integrity `token`, a TPM or Nitro `quote`. `keyCustody` states
the tier and algorithms of the device's private keys but never the keys;
`hpkePublicKey` is a public X25519 key and carries no secret.

The schema makes only `consumerKind` and `displayName` mandatory, so
`platform`, `attestation`, and `keyCustody` are each a producer's choice to
narrow itself further, and each is a policy input rather than an entry
requirement. The response returns the whole `DeviceBinding`, which is a
superset of the request: the maintainer-assigned `deviceId`, the `consumerDid`,
the granted `capabilities`, and the enrolment timestamps.

Attestation statements deserve a rule of their own. `attestationStatement`,
`token`, and `quote` exist to be *verified*, once, at registration. A maintainer
**SHOULD** retain the verdict and discard the statement: keeping the raw blob
keeps a platform-issued device identifier alive long after the only question it
could answer has been answered.

### Correlation

`deviceId` is the durable join key of this entire family. The shared schema is
explicit that it is stable for the device's lifetime and **never re-used** after
disable or wipe, so every later `device/heartbeat`, `device/list` entry, ACL row,
and audit line about this device resolves to the same value. `consumerDid` joins
just as widely, because it is the key the device authenticates with on every
subsequent exchange with this maintainer — a registration is therefore the point
at which a hardware device acquires a permanent identity in the maintainer's
records.

The registration is also joinable *backwards*. This task is step three of a
three-step bootstrap, and steps one and two name the existing administrator who
minted the device's key through `provision-integration` and swapped it in through
`acl/swap-key`. The chain is deliberately traceable — the maintainer **SHOULD**
log every provision-integration and every subsequent register precisely so it
is — and the cost of that traceability is that the record says not only which
device this is but who introduced it.

Two members correlate beyond the maintainer. `displayName` is echoed to every
other Companion that calls `device/list`, so a name chosen for one audience
reaches a wider one; it is also producer-supplied and unauthenticated, so a
consumer **SHOULD** treat it as informational and rely on `deviceId` for any
security decision. `attestation.aaguid` identifies an authenticator *model*
rather than a unit, which does not single out a device on its own but does
narrow the population, and combined with `platform` it is a serviceable browser-
and-hardware fingerprint.

The device party declares `identifierScope: pairwise`. Its `consumerDid` is a
VTA-derived long-term key minted for this one maintainer relationship, and
nothing in the task asks any third party to recognise it; a device that reused
one identifier across several maintainers would let them join their records of
it, which this design does not need and does not want.

### Retention

Durable, and the durability is the design rather than a side effect. The
`DeviceBinding` outlives the device it describes: `device/disable` and
`device/wipe` stamp `disabledAt` and `wipedAt` on the record instead of removing
it, and `deviceId` is never recycled, so an action attributed to a device
remains attributable after the device is gone. A maintainer that deleted
bindings on decommissioning would keep its audit trail and lose the only thing
that resolves the entries in it.

What does *not* need to be durable is the material that only mattered once. The
attestation statement is verified at registration and never again; the same is
true of any platform token inside it. Retaining the verdict preserves the policy
decision, and retaining the blob preserves a device identifier that no longer
serves a purpose.

One item is retained on a short clock rather than a long one: the document `id`
is the maintainer's idempotency key, so a retry of the same id inside the
idempotency window returns the same `DeviceBinding` rather than creating a
second one. That window is the only part of this exchange a maintainer is
expected to forget.

### Consent/purpose

The data moves so that a maintainer can recognise this device on its next
connection and decide what policy class it belongs to. That is the whole
purpose, and it bounds the reuse: `attestation` and `keyCustody` are policy
*input*, not gates. A failed or absent attestation does not deny registration —
the maintainer's policy decides, with strict deployments requiring attestation
and permissive ones accepting `kind: none` and applying a stricter class
instead (shorter sessions, more frequent step-up). `keyCustody.tier` reads the
same way: on mobile, hardware custody is only reachable with **P-256** keys
because the iOS Secure Enclave is P-256-only, so an Ed25519/X25519 holder is
necessarily `software`, and a maintainer **MAY** treat `software` as it treats a
weak attestation. Per the [Mobile Key-Custody Profile](../../../../docs/design-notes/mobile-key-custody-profile.md)
a mobile holder's keys **SHOULD** be P-256 once the supporting libraries land,
and onboarding **SHOULD** warn when a device sits on `software` tier that its
platform could have hardened.

The basis on which the device is admitted at all is the prior authorisation
recorded in the bootstrap chain: an existing administrator granted this key
before the device ever sent this document, and `device/register:noPendingEnrolment`
is the refusal when that basis is missing. `displayName` serves a different
purpose entirely — helping a human pick their own laptop out of a list — and
**MUST NOT** be repurposed as a security input by any consumer that renders it.
Whether a human is prompted before a device joins is a maintainer policy
question on which this specification takes no position.
