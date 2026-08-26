---
slug: device/wipe
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — Wipe
summary: The maintainer issues a wipe to a Companion or Service. Target destroys its cache (and optionally device-local keys); the maintainer additionally revokes ACL access and rotates the device's cache-key derivation root so defence in depth neutralises non-compliant targets.
status: draft
targetFrameworkVersion: "0.5"
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
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: Wipe is destructive and irreversible from the target's perspective. The maintainer's authority MUST be verifiable so the target can confirm the wipe is genuine before executing it (defence against an attacker who has captured the transport channel attempting to silently wipe legitimate Companions).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A wipe destroys the device's local state irreversibly. A wipe document that cannot be placed in an acceptance window stays armed forever, and the consumer must otherwise remember every wipe it has ever executed in order not to run one twice.
sideEffects:
  level: destructive
  rationale: "Destroys the target's cache and optionally its device-local keys, revokes access, and rotates the cache-key root."
consequences:
  - "Erases the device's local cache and may destroy device-local keys; the wipe cannot be undone on the target."
subjectPath: /deviceId
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Nothing is returned to the issuer beyond an acknowledgement, so `discloses`
    stays `none`. Inbound is the unusual half: `reason` is a REQUIRED free-text
    member the schema exists to feed an audit log's record of intent, and the
    circumstances that prompt a wipe — a device lost, a person departed, a
    compromise suspected — make the natural sentence one that names an
    identifiable person and what happened to them. That text is delivered to
    the device being wiped, which is the party the maintainer has just declared
    it no longer trusts.
retention:
  class: durable
  rationale: >-
    The two halves of this task retain in opposite directions. The target
    destroys what it holds — that is the point — while the maintainer keeps the
    wipe as a security-incident record: who issued it, when, against which
    `deviceId`, at what `scope`, and with what `reason`, alongside whether an
    acknowledgement ever arrived. `wipedAt` is stamped on the DeviceBinding
    rather than the row being deleted. A consumer that discarded the record
    would lose the only evidence distinguishing a device that confirmed its
    wipe from one that may still hold cached vault material.
errorCodes:
  - code: device/wipe:notFound
    meaning: No DeviceBinding with this id.
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

### Data carried

The request is four members: `deviceId`, a `scope` of `cache`, `cacheAndKeys`,
or `full`, a self-contained `issuedAt` repeated so an offline-queued delivery
still carries its own timestamp, and `reason`.

`reason` is the member that deserves care, and its position in this task is
unusual enough to state plainly. It is **required**, it is free text up to 256
characters, and the schema is explicit that it is required "because every wipe
is consequential and the audit log must capture intent". But the document it
travels in is delivered *to the device being wiped* — the one party the
maintainer has, by issuing this task, formally declared it no longer trusts, and
which in the lost-or-stolen case is physically in someone else's hands. An
operator writing "laptop stolen from Anna at the Barcelona office" or "issued on
dismissal" has written a sentence for the audit log and transmitted it to a
stranger.

Nothing here needs that sentence to reach the target. `scope` already says what
the device must do; the audit entry already records who issued the wipe and
when. A producer **SHOULD** write a `reason` that satisfies the audit
requirement without narrating the incident, and **MUST NOT** place credentials,
third-party personal data, or investigation detail in it.

The response carries `deviceId`, `scope`, `completedAt`, and optional
`diagnostics`: byte and key counts, the `osHooksInvoked` naming which OS-level
revocation APIs the target managed to call, and free-form `partialReasons`
strings. `diagnostics` is part of the target's signed surface and travels over
an authenticated transport, but it is not encrypted on the wire by default, so
targets **SHOULD NOT** put personally-identifiable detail there beyond byte
counts and hook names.

### Correlation

`deviceId` ties the wipe to the DeviceBinding and, through it, to every
heartbeat, capability grant, and audit line that device ever produced. The wipe
itself becomes part of that record: `wipedAt` is stamped on the binding rather
than the row being removed, and `deviceId` is never re-used, so "this device was
wiped, on this date" is a permanent property of the identifier.

`osHooksInvoked` is more identifying than it looks. Naming
`ASCredentialIdentityStore.removeAllCredentialIdentities` rather than
`navigator.credentials.preventSilentAccess` distinguishes an Apple platform from
a browser extension, and the set of hooks a target succeeded in calling
fingerprints its OS version and its permission state — on a response that is not
encrypted by default.

Replay does not create a correlation problem here, only a non-problem worth
recording: a captured wipe document remains a valid wipe of its named target, so
replaying it against an already-wiped device achieves nothing, and the
maintainer-side mitigations cannot be re-triggered into a useful state by one.

The device party declares `identifierScope: pairwise`. The wipe must name a
device its own maintainer can resolve, and no third party is asked to recognise
that identifier; a device reusing one identifier across maintainers would let
each of them learn that it had been wiped by another.

### Retention

Destructive on one side and durable on the other, which is the whole shape of
the task. The target destroys its cache and, at `cacheAndKeys` or `full`, its
device-local key material; there is no undo, and a user who issues a wipe when
they meant `device/disable` **MUST** re-onboard through the full
provision-integration → `acl/swap-key` → `device/register` sequence.

The maintainer keeps the opposite. Every wipe is logged with who issued it,
when, against which device, at what scope, and for what reason; acknowledgements
and their absence are logged too, so an investigator can tell "device confirmed
wipe" from "device may still hold cached material". That means the `reason` text
persists in the audit trail indefinitely — a second, independent reason to write
it as a record rather than as a narrative.

### Consent/purpose

The purpose is neutralisation, and the design assumes the target will not
cooperate. The wipe message is one of four mitigations; ACL revocation,
cache-key-derivation-root rotation, and mediator suppression all execute
regardless of whether it is acknowledged, and implementers **MUST NOT** defer
those server-side steps until an acknowledgement arrives — doing so opens a
window in which a silently-dropped wipe leaves the device fully functional.

Authority runs the other way to everywhere else in this family. The target
verifies the maintainer's proof *before* executing, because the instruction is
destructive and a malicious mediator or an intercepting party would otherwise be
able to wipe legitimate Companions at will; a wipe whose proof does not verify
**MUST** be ignored, and logged locally where the target has logging.

`reason` is collected for the audit log, and that is the limit of its purpose. A
consumer that renders it in the target's own interface is showing incident
detail to whoever currently holds the device, which is precisely the population
the wipe exists to defend against.
