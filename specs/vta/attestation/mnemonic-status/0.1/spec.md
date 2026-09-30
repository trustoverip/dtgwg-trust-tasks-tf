---
slug: vta/attestation/mnemonic-status
version: "0.1"
title: "VTA Attestation — Mnemonic Status"
summary: "A super administrator checks whether a TEE agent's one-time mnemonic export window is open, before asking for the release itself."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - mnemonic
  - seed
  - backup
  - tee
  - super-admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: super administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: VTA
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Whether the export window is open, and whether the mnemonic has already been taken, is itself sensitive: it tells a caller when the one chance at an offline backup remains. The VTA authorises on the document's proven issuer, holding super-admin standing, exactly as it does for the export the status precedes.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: >-
    A read with no effect to replay; the window's remaining seconds date themselves, so a stale cached answer is self-evidently stale.
sideEffects:
  level: none
  rationale: "Reads the guard's own state; nothing is written, exported or wiped by being read."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Reveals only the guard's own state — window open or not, exported or not, entropy available or not, seconds remaining. None of it is the mnemonic itself.
retention:
  class: transient
  rationale: "The request carries nothing but its envelope; the response is a point-in-time reading of a guard whose state changes on its own schedule."
errorCodes:
  - code: vta/attestation/mnemonic-status:notAvailable
    meaning: "This agent has no mnemonic export guard — it is not running in TEE mode, or no KMS bootstrap created one. There is no window to report on, ever, on this deployment."
    retryable: false
related:
  - vta/attestation/mnemonic-export
  - vta/attestation/status
  - vta/backup/initiate-export
---

## Abstract

A TEE agent generates its BIP-39 seed inside the enclave on first boot and holds it in a guard that releases it, once, within a short window after boot ([`vta/attestation/mnemonic-export`](../../mnemonic-export/1.0/spec.md)). Before a super administrator asks for that release — or decides it has already been missed — it needs to know the guard's state: is the window still open, has the mnemonic already been taken, does entropy remain to export at all, and how long is left.

This task answers that. It changes nothing: the window closes and the entropy wipes on its own schedule regardless of whether anyone asks.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A conforming VTA:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without super-admin standing with `permissionDenied`.
3. **MUST** refuse with `vta/attestation/mnemonic-status:notAvailable` when the agent holds no mnemonic export guard at all — it is not running in TEE mode, or no KMS bootstrap created one.
4. Otherwise answers with the guard's current `windowActive`, `alreadyExported`, `entropyAvailable` and `windowRemainingSecs`, read at the instant of the request.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is the same **super-admin standing** that [`vta/attestation/mnemonic-export`](../../mnemonic-export/1.0/spec.md) requires for the release itself: the operator authorised to take the one-time backup is the same operator authorised to check whether the chance remains. An administrator without that standing learns nothing about the window's state. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Export window** — the bounded period after boot during which [`vta/attestation/mnemonic-export`](../../mnemonic-export/1.0/spec.md) will release the mnemonic. Opened only when the VTA was started with an export window configured, and only on the boot that generated the entropy.

**Entropy available** — whether the guard still holds the 32 bytes of seed entropy. Cryptographically wiped once exported or once the window expires, and never present on a boot after the first, because a VTA generates its seed only once.

## Request

A super administrator (`issuer`) sends an empty payload to the VTA (`recipient`). See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Check before asking for the release

```json
{
  "id": "urn:uuid:2b3c4d5e-6f70-4a2b-9c3d-4e5f6a7b8c01",
  "type": "https://trusttasks.org/spec/vta/attestation/mnemonic-status/0.1",
  "issuer": "did:web:superadmin.example",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-30T09:05:00Z",
  "payload": {}
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures — `permissionDenied`, `vta/attestation/mnemonic-status:notAvailable` — use `trust-task-error`.

### The window is open, unexported, three minutes left

```json
{
  "id": "urn:uuid:2b3c4d5e-6f70-4a2b-9c3d-4e5f6a7b8c02",
  "type": "https://trusttasks.org/spec/vta/attestation/mnemonic-status/0.1#response",
  "threadId": "urn:uuid:2b3c4d5e-6f70-4a2b-9c3d-4e5f6a7b8c01",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:web:superadmin.example",
  "issuedAt": "2026-09-30T09:05:01Z",
  "payload": {
    "windowActive": true,
    "alreadyExported": false,
    "entropyAvailable": true,
    "windowRemainingSecs": 180
  }
}
```

### No guard on this deployment

```json
{
  "id": "urn:uuid:2b3c4d5e-6f70-4a2b-9c3d-4e5f6a7b8c04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:2b3c4d5e-6f70-4a2b-9c3d-4e5f6a7b8c03",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:web:superadmin.example",
  "issuedAt": "2026-09-30T09:06:00Z",
  "payload": {
    "code": "vta/attestation/mnemonic-status:notAvailable",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing but its envelope. The response carries four booleans and an integer describing the guard's own state — never the mnemonic, never any key material. A producer **MUST NOT** put anything into `ext` that would let a reader infer more about the entropy than the guard's own four fields already state.

### Correlation

Both parties declare `identifierScope: public`: the super administrator's DID is what the VTA's access-control entry is keyed on, and the VTA's DID is the one every administrator addresses. There is nothing here for a pairwise scope to protect that the entitlement check does not already gate.

### Retention

Nothing about the request needs keeping. The response is a point-in-time reading that goes stale as the window counts down; a caller **SHOULD** treat `windowRemainingSecs` as advisory the moment it re-checks.

### Consent/purpose

The purpose is to let the one operator entitled to the mnemonic export decide, before asking for it, whether the window remains and whether the release has already happened. It **MUST NOT** be polled by anything other than that operator's own workflow, and answering it reveals nothing that changes the guard's behaviour — the window closes and the entropy wipes on its own schedule whether or not anyone asks.
