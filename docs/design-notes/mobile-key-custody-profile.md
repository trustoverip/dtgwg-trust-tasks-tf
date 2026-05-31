# Design Note: Mobile Key-Custody Profile

| | |
|---|---|
| **Status** | Draft |
| **Date** | 2026-05-30 |
| **Applies to** | Mobile holders/agents (iOS, Android) participating in `auth/*` (step-up, passkey) and DIDComm-carried Trust Tasks |
| **Related** | `specs/device/_shared/0.1/device-binding` (`keyCustody`), `specs/auth/step-up/*`, `bindings/didcomm/0.1`, `bindings/push/0.1` |

*This note is non-normative rationale. The normative surface is the `keyCustody`
declaration on the device record and the SHOULDs called out in §6.*

## 1. Problem

A mobile authenticator holds key material for two roles:

- **Signing / authentication** — proving control of the holder DID (AAL step-up
  `approve-response`, DID-ownership auth).
- **Key agreement** — DIDComm authcrypt/anoncrypt (ECDH) to decrypt inbound
  messages and encrypt responses.

The security goal is to keep that key material in the device's **secure
keystore** (iOS Secure Enclave, Android StrongBox/Keystore) so a compromise of
app memory does not expose private keys. The obstacle is that the SSI/DIDComm
default key types — **Ed25519** (signing) and **X25519** (key agreement) —
**cannot be held in mobile secure hardware**.

## 2. The hardware constraint (what actually drives the design)

| Algorithm | iOS Secure Enclave | Android StrongBox | Android TEE Keystore |
|---|---|---|---|
| **P-256** (ES256 sign, ECDH) | ✅ (the *only* asymmetric curve it supports) | ✅ (commonly) | ✅ |
| Ed25519 / X25519 | ❌ | ❌ / rare | inconsistent (some A13+) |
| P-384 / P-521 | ❌ | usually ❌ | device-dependent |
| secp256k1 | ❌ | ❌ | ❌ |

The iOS Secure Enclave is **P-256-only** (Apple CryptoKit exposes exactly
`SecureEnclave.P256`). Therefore:

- **P-256 is the only curve that is hardware-custody-able on both platforms.**
  P-384/P-521 do not help (the iOS enclave cannot do them); secp256k1 is backed
  by neither.
- Hardware custody requires P-256 for **both** roles: **ES256** signing and
  **ECDH-P256** key agreement.
- Ed25519/X25519 keys are necessarily **software-held** on mobile, regardless of
  storage hardening.

This is the same reason **FIDO2 / Apple & Google passkeys use P-256** — it is
the only path to hardware-bound credentials on phones.

## 3. Tiered custody model

A device uses the **highest tier it can** for each key:

- **Tier 1 — Hardware (P-256).** Key generated non-exportable inside the Secure
  Enclave / StrongBox. Every `sign` (ECDSA) and `keyAgreement` (ECDH) operation
  is performed *inside* the chip; private key bytes never exist in app memory.
  Requires P-256 keys end-to-end (see §5/§6).
- **Tier 2 — Software, hardware-wrapped at rest.** For Ed25519/X25519 (today's
  default) or any device lacking a usable enclave: the key is generated in
  software and stored **wrapped by a hardware-backed symmetric key** (Android
  Keystore AES-GCM; iOS Keychain item with a Secure-Enclave-gated, biometry-bound
  access control). It is unwrapped into memory only for the duration of an
  operation, then **zeroized**. Access is biometric-gated.

Tier 2 is the floor, not a failure mode — it is exactly what Signal, WhatsApp,
Matrix/Element, and most crypto wallets do (their Curve25519/secp256k1 keys
cannot be hardware-bound either). With Tier 2 we are **on par** with those; with
Tier 1 (P-256) we reach **passkey-grade** custody. We are never worse off.

## 4. The custody seam is tier-agnostic (no engine churn)

The `vta-mobile-core` engine never holds, and is never told the location of, a
private key. It depends only on **callback interfaces** the native app
implements:

- `Signer` (already shipped): `sign(bytes) -> signature`, `did()`.
- `KeyAgreement` (to add): `ecdh(peer_public_key) -> shared_secret`,
  plus key metadata.

The native side wires those to **Tier 1 (enclave op)** or **Tier 2 (unwrap →
op → zeroize)**, transparently to the engine. The tier is a native concern
behind a stable interface. (This is why the merged `Signer` callback and the
DID-signed proof builder need no rework — they are already tier-agnostic; the
key *type* determines the tier, the *interface* does not change.)

## 5. Additive upstream-library requirements

Reaching Tier 1 (P-256, key never leaves hardware) needs additions to the
crypto libraries. **All MUST be additive** — no change to existing signatures,
types, or behaviour; existing raw-key paths remain byte-identical. (Note: in
Rust, *adding a variant* to a public enum can break downstream exhaustive
`match`es; additivity therefore means **new traits / functions / `#[non_exhaustive]`
types**, not mutating existing public enums.)

1. **DIDComm — external key-agreement callback.** `affinidi-messaging-didcomm`
   today performs ECDH with a raw in-memory `PrivateKeyAgreement`. Add a *new*
   `KeyAgreement` trait + *new* `*_with(&dyn KeyAgreement)` pack/unpack entry
   points so the enclave performs the ECDH. (P-256 key agreement is *already*
   supported as a curve; only the external-signer seam is missing.)
2. **DIDComm — ES256 JWS signing.** Message signing is currently EdDSA-only
   (`sign_ed25519`). Add a *new* `sign_es256` / external-signer path for P-256
   message signatures.
3. **Data Integrity — ECDSA P-256 suite.** `affinidi-data-integrity` exposes
   `eddsa-jcs-2022` / `eddsa-rdfc-2022` / `bbs-2023` only. Add an
   `ecdsa-jcs-2019` (P-256) cryptosuite so the DID-signed step-up proof can be
   produced/verified with a P-256 holder key. (Its `prepare_sign_input`
   remote-signer seam already supports the enclave flow.)

Until (1)–(3) land, mobile holders operate at **Tier 2** with Ed25519/X25519
(functionally complete; see the engine's current step-up + resolver slices).

## 6. Normative guidance (the spec hook)

- A mobile holder's keyAgreement (and signing) key **SHOULD** be **P-256** so it
  can be hardware-custodied, **once** the upstream support in §5 is available.
- A device **SHOULD** declare its `keyCustody` (tier + algorithms) on its
  `DeviceBinding` (see `specs/device/_shared/0.1/device-binding`). This is
  **policy input**, mirroring `attestation`: a maintainer **MAY** apply stricter
  policy (shorter sessions, more frequent step-up) to `software` (Tier 2)
  devices.
- The onboarding/registration flow **SHOULD warn** when a device is on Tier 2
  *but the platform could support Tier 1*, and offer to provision a P-256 key —
  so users get hardware custody where the hardware allows it.

## 7. Sequencing

1. This profile + the `keyCustody` device declaration (normative hook).
2. Ship the mobile engine at **Tier 2** (Ed25519/X25519) — already in progress.
3. Land the additive upstream library support (§5).
4. Provision P-256 keyAgreement/signing VMs on mobile holder DIDs (a multi-VM
   method — `did:peer` / `did:webvh`), flip mobile holders to **Tier 1**, and
   enable the §6 warning.

## 8. References

- Apple, *CryptoKit / Secure Enclave* — `SecureEnclave.P256` (P-256 only).
- Android Keystore / StrongBox `KeyAgreement`/`Signature` — EC P-256.
- W3C *Data Integrity* `ecdsa-jcs-2019`; DIDComm v2 ECDH-1PU/ES (P-256, X25519).
- FIDO2 / WebAuthn platform authenticators — ES256 (P-256) hardware-bound.
