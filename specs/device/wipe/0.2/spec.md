---
slug: device/wipe
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — Wipe
summary: The maintainer issues a wipe to a Companion or Service. Target destroys its cache (and optionally device-local keys); the maintainer additionally revokes ACL access and rotates the device's cache-key derivation root so defence in depth neutralises non-compliant targets.
status: draft
targetFrameworkVersion: "0.2"
category: identity
keywords:
  - device
  - wipe
  - revoke
  - recovery
  - lost-device
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault maintainer
    requirement: REQUIRED
    member: issuer
  - role: device
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Wipe is destructive and irreversible from the target's perspective. The maintainer's authority MUST be verifiable so the target can confirm the wipe is genuine before executing it (defence against an attacker who has captured the transport channel attempting to silently wipe legitimate Companions).
sideEffects:
  level: destructive
  rationale: "Destroys the target's cache and optionally its device-local keys, revokes access, and rotates the cache-key root."
consequences:
  - "Erases the device's local cache and may destroy device-local keys; the wipe cannot be undone on the target."
subjectPath: /deviceId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: device/wipe:notFound
    meaning: No DeviceBinding with this id.
    retryable: false
  - code: device/wipe:permissionDenied
    meaning: The issuer lacks DeviceAdmin capability on the maintainer.
    retryable: false
  - code: device/wipe:wipePartial
    meaning: The target executed the wipe but could not complete every step (e.g. OS keychain APIs returned errors). The target completed as much as possible and reports `diagnostics.partialReasons`.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        partialReasons: { type: "array", items: { "type": "string" } }
---

## Abstract

The **Device — Wipe** Trust Task instructs a Companion or Service to destroy its local cache and (per `scope`) its device-local key material. Wipe is best-effort from the target's perspective — a compromised device may silently drop the message — so the maintainer pairs it with **defence-in-depth** server-side actions:

1. The target's ACL entry is **revoked at the moment the wipe is issued**, not when the target acknowledges. Existing refresh tokens expire (existing 24h TTL); proxy-login and release calls return `permissionDenied`.
2. The maintainer rotates the **cache-key derivation root** for that device. Future sync deltas would be encrypted under a key the device can never derive again.
3. The maintainer instructs the mediator to **refuse forwarding** of inbound traffic to the wiped device's holder DID.
4. A `sync/event/0.1` of kind `aclChanged` with `change: "deviceWiped"` is fanned out to every other Companion, so their UIs surface the wiped device in the device-manager.

The net effect: even if the lost device is fully under attacker control and silently drops the wipe message, the attacker gets nothing of value beyond what was already in the device's RAM at the moment of loss (bounded by short release TTLs).

## Conformance

A conforming **producer** (the maintainer) **MUST**:

1. Populate `deviceId`, `scope`, `reason`. `issuedAt` matches the document's `issuedAt`.
2. Carry a `proof` whose verificationMethod resolves via the maintainer's admin key.
3. Execute the server-side defence-in-depth steps (ACL revoke, cache-key rotation, mediator suppression, sync-event fan-out) **at issuance time**, not on acknowledgement.

A conforming **consumer** (the target Companion or Service) **MUST**:

1. Verify proof on receipt. If verification fails, the target MUST NOT execute the wipe (defends against spoofed wipes on a hostile transport).
2. Execute the wipe before any other operation. If queued operations exist (offline writes, etc.), they MUST be discarded; do not flush queued state to the maintainer before wiping.
3. Apply the requested `scope`:
   - `cache` — wipe the encrypted vault cache and any other maintainer-derived caches; preserve the device's long-term key material so re-sync can resume with the same identity.
   - `cacheAndKeys` — wipe cache + device-local `device_secret`, WebAuthn-PRF-wrapped material, and any OS keychain handles. The device becomes unregistered; re-onboarding via §3a of the design plan is required to use the maintainer again.
   - `full` — `cacheAndKeys` + invoke every available OS-level revocation hook (`navigator.credentials.preventSilentAccess()` on browsers, `ASCredentialIdentityStore.removeAllCredentialIdentities` on iOS, equivalent on Android) + clear extension/app storage.
4. Acknowledge by sending the `device/wipe/0.1#response` document (over whatever transport is still functional). Populate `diagnostics` truthfully — partial wipes are surfaced via `wipePartial`. **MUST NOT** suppress diagnostics to make the wipe appear cleaner than it was.
5. After acknowledging, the target **MUST NOT** perform any operation against the maintainer (it would fail anyway because the ACL is revoked).

A maintainer **MAY** issue subsequent wipes to the same `deviceId` at a wider `scope`. Targets MUST execute each wipe; idempotency is at the operation level (re-wiping already-clean state is a no-op).

## Delivery

In order of preference:

1. **Online (DIDComm push via mediator).** Target executes immediately.
2. **On next heartbeat.** If the target was offline at issuance, the maintainer queues the wipe; the target fetches on its next `device/heartbeat/0.1` and executes before any other op.
3. **Mediator-suppressed.** The maintainer also tells the mediator to refuse to forward subsequent inbound traffic to the wiped device's holder DID. Even if the target tries to reconnect, it cannot receive vault deltas.

## Payload

`deviceId`, `scope` (REQUIRED), `reason` (REQUIRED, audit-logged).

## Response

`deviceId`, `scope`, `completedAt`, `diagnostics` (optional).

## Examples

### Issue a full wipe for a lost laptop

```json
{
  "id": "wipe-1234",
  "type": "https://trusttasks.org/spec/device/wipe/0.2",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-26T14:00:00Z",
  "payload": {
    "deviceId": "dev_01HZX3MGRT…",
    "scope": "full",
    "reason": "lost laptop reported by holder via mobile companion",
    "issuedAt": "2026-05-26T14:00:00Z"
  },
  "proof": { "…": "…" }
}
```

### Target acknowledgement

```json
{
  "id": "wipe-1234-ack",
  "type": "https://trusttasks.org/spec/device/wipe/0.2#response",
  "threadId": "wipe-1234",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T14:00:01Z",
  "payload": {
    "deviceId": "dev_01HZX3MGRT…",
    "scope": "full",
    "completedAt": "2026-05-26T14:00:01Z",
    "diagnostics": {
      "cacheBytesWiped": 287304,
      "keysWiped": 4,
      "osHooksInvoked": [
        "navigator.credentials.preventSilentAccess",
        "chrome.storage.local.clear"
      ]
    }
  }
}
```

## Security & Privacy

**Defence in depth.** The wipe message is one of four mitigations; the others (ACL revoke, cache-key rotation, mediator suppression) execute regardless of whether the wipe is acknowledged. Implementers MUST NOT defer the server-side mitigations until acknowledgement — that creates a window in which a silently-dropped wipe leaves the device functional.

**Proof verification on the target side.** The target verifies the maintainer's proof before executing. This protects against a malicious mediator or man-in-the-middle attempting to silently wipe legitimate Companions. A wipe whose proof does not verify MUST be ignored (and logged locally if the target has logging).

**No replay neutralisation.** A captured wipe document is still a valid wipe of its named target — replaying it is harmless if the device is already wiped, and the maintainer-side mitigations cannot be re-triggered into a useful state by a replay.

**Audit reach.** Every wipe is logged with `{ who issued, when, deviceId, scope, reason }`. Acknowledgements (or absences) are also logged so an investigator can distinguish "device confirmed wipe" from "device may still hold cached material".

**Diagnostics privacy.** `diagnostics` is part of the target's signed surface (sent over an authenticated transport) but is not encrypted on the wire by default. Targets SHOULD NOT include personally-identifiable detail beyond byte counts and hook names.

**Recovery.** A user who wipes a device by mistake (e.g. confused with disable) MUST re-onboard the device via the normal provision-integration + acl/swap-key + device/register flow. There is no "undo wipe".
