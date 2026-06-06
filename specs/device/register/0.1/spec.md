---
slug: device/register
version: "0.1"
title: Device — Register
summary: A newly-onboarded Companion or Service claims its device record on the maintainer, supplying form factor, display name, HPKE public key, and optional device attestation; wraps the maintainer's existing provision-integration → acl/swap-key bootstrap.
status: draft
targetFrameworkVersion: "0.1"
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
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Registration binds a device identity to operator-significant capabilities. The producer's identity MUST be verifiable so the maintainer can attribute the registration to a specific consumer key (the one the operator authorised in provision-integration).
errorCodes:
  - code: device/register:no_pending_enrolment
    meaning: The producer's DID is not the result of a recent provision-integration + acl/swap-key flow. Registration cannot proceed without first being granted via the maintainer's normal enrolment path.
    retryable: false
  - code: device/register:attestation_failed
    meaning: The supplied device attestation could not be verified against the platform's attestation infrastructure.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: "string", enum: ["signature_invalid", "untrusted_root", "stale", "rooted_device", "unsupported_attestation_kind"] }
  - code: device/register:already_registered
    meaning: A DeviceBinding for this consumer DID already exists. Re-registration is rejected; the consumer SHOULD swap to a fresh key via acl/swap-key and retry.
    retryable: false
  - code: device/register:hpke_key_invalid
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

1. Verify proof. The producer's DID MUST already be in the ACL (placed there by step 2 above). If not → `device/register:no_pending_enrolment`.
2. Verify any supplied `attestation` against the platform's attestation infrastructure. Failure → `device/register:attestation_failed` with `details.reason`. The maintainer's policy decides whether a failed attestation is fatal or merely downgrades the device's policy class.
3. Treat any supplied `keyCustody` as **policy input** (not a gate): a `software`-tier device MAY be assigned a stricter policy class (shorter sessions, more frequent step-up), the same way a missing/failed attestation is handled.
4. If a DeviceBinding for this DID already exists → `device/register:already_registered`. (Re-registration is intentionally not idempotent — the consumer must rotate keys and try again.)
5. Validate `hpkePublicKey` as a well-formed X25519 did:key. Failure → `device/register:hpke_key_invalid`.
6. Create the DeviceBinding with `registeredAt = now`, `lastSeenAt = now`, capabilities mirrored from the ACL entry. Return the DeviceBinding.

## Payload

`consumerKind` (REQUIRED), `displayName` (REQUIRED), `hpkePublicKey` (REQUIRED), `platform` (optional), `attestation` (optional, RECOMMENDED), `keyCustody` (optional, RECOMMENDED for mobile Companions). Push wake-up is registered with the mediator, not here (see the note above).

## Response

`binding` — the created DeviceBinding.

## Security & Privacy

**Bootstrap chain integrity.** The security of the device-registration path rests on provision-integration: only an existing admin can introduce a new device into the ACL. If provision-integration is compromised, device-register is also compromised. The maintainer SHOULD log every provision-integration and every subsequent register so the chain is traceable.

**Attestation as policy input, not gate.** A failed or absent attestation does not automatically deny registration — the maintainer's policy decides. Strict deployments REQUIRE attestation; permissive deployments accept `none` and downgrade the device's policy class (shorter sessions, more frequent step-up).

**Key custody as policy input.** `keyCustody.tier` tells the maintainer whether the device's private keys are hardware-bound (`hardware`) or software-held (`software`). On mobile, hardware custody is only achievable with **P-256** keys (the iOS Secure Enclave is P-256-only); Ed25519/X25519 holders are necessarily `software`. A maintainer MAY treat `software` like a weak/absent attestation — register but apply a stricter policy class. Per the [Mobile Key-Custody Profile](../../../../docs/design-notes/mobile-key-custody-profile.md), a mobile holder's keys SHOULD be P-256 once the supporting libraries land, and onboarding SHOULD warn when a device is on `software` tier but the platform could support `hardware`.

**Display-name spoofing.** The `displayName` is producer-supplied and not authoritative. Other Companions seeing this device in a device/list response SHOULD treat the name as informational only and rely on `deviceId` for any security decision.

**Replay.** The `id` is the maintainer's idempotency key; a retry of the same id within the idempotency window returns the same DeviceBinding without re-creating it.
