---
slug: social-recovery/buddies/list
version: "0.1"
title: "Social Recovery — List Buddies"
summary: "A device owner reads the full roster of recovery buddies currently enrolled for their identity — an argument-free, owner-scoped read with no filter to narrow what the requester sees."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - social-recovery
  - guardian
  - key-recovery
parties:
  - role: device owner
    requirement: REQUIRED
    member: issuer
  - role: recovery maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The buddy roster is the durable, sensitive social-graph fact
    `social-recovery/buddies/add`'s own Correlation section names — who the
    owner trusts to hold recovery authority over their access. An
    unattributed read would let anyone reaching the maintainer's transport
    enumerate that roster for an identity that is not theirs.
sideEffects:
  level: none
  rationale: >-
    A read-only list (`category: "read"`, `requiresApproval: false`) over
    the resolved identity chain; the handler itself takes no arguments and
    reads nothing from the request beyond dispatching it.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Returns each enrolled buddy's identifying fields (DID, display label,
    enrolment timestamp) and the buddy's **public** Ed25519 key
    (`publicKeyHex`) — the same public key `social-recovery/buddies/add`
    accepted at enrolment, disclosed deliberately so a recipient can later
    verify a buddy's cooperation, never a private key or secret share.
    Also returns two roster-level device-topology fields
    (`activeDeviceCount`, `canBootstrap`) unrelated to any individual
    buddy. Never the reconstruction mechanism's own internal state (e.g.
    threshold-share material, if used).
errorCodes:
  - code: social-recovery/buddies/list:noIdentityChain
    meaning: >-
      The caller's own identity chain does not yet resolve (a fresh
      device that has not completed onboarding). There is no roster to
      read yet, distinct from an onboarded identity with a genuinely
      empty roster — a maintainer SHOULD distinguish the two so a new
      owner is not told "you have no buddies" when the real answer is
      "you have no identity chain yet to attach buddies to."
    retryable: true
related:
  - social-recovery/buddies/add
  - social-recovery/buddies/remove
  - social-recovery/status/get
---

## Abstract

The **Social Recovery — List Buddies** Trust Task returns the full roster
of recovery buddies currently enrolled for the requesting owner's identity.
It is genuinely argument-free: the reference implementation's own handler
(`handleRecoveryBuddyList`, formerly `handleGuardianList` before the
recovery-buddy rename) takes no per-request parameters and resolves the
roster entirely from the caller's own identity chain — this task answers
"who is enrolled," full stop, with no filter narrowing what comes back.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)).
It targets framework version 0.6.0 and may change without a version bump
while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents an operation a maintainer already implements
(`listRecoveryBuddies`, `src/packages/devices/manifest-operations-part-2.ts`),
written down so the shape stops being recoverable only by reading an
implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD
NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be
interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14)
when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy
[SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the
requirements stated here.

## Definitions

- **Recovery buddy roster** — every currently enrolled custodial recovery
  contact for the requesting owner's identity, as recorded via
  [`social-recovery/buddies/add`](../../add/0.1/spec.md). This task takes no
  arguments; there is no per-device or per-buddy filter to narrow the
  result.
- **`recoveryBuddies[]`** — the roster array itself, one entry per currently
  enrolled buddy. Each entry carries:
  - **`recoveryBuddyDid`** — the buddy's `did:*`.
  - **`keyId`** — the maintainer's own internal identifier for the buddy's
    enrolment key record (opaque to the wire; not itself key material).
  - **`publicKeyHex`** — the buddy's Ed25519 **public** key, 64 hex
    characters, the same value supplied on
    [`social-recovery/buddies/add`](../../add/0.1/spec.md)'s
    `recoveryBuddyPublicKeyHex`. It is a public key, disclosed deliberately —
    see Security & Privacy below.
  - **`addedAtIso`** — ISO-8601 timestamp of enrolment, or `null` if the
    maintainer did not record one.
  - **`name`** — the display label supplied at enrolment, or `null`.
  - **`bootstrap`** — `true` when the buddy was enrolled through the
    single-device bootstrap path (the only path this milestone supports);
    reserved for the multi-device FROST co-signing path.
- **`activeDeviceCount`** — the number of currently active devices on the
  requesting owner's identity chain.
- **`canBootstrap`** — `true` iff `activeDeviceCount <= 1`; whether the
  bootstrap (single-device) enrolment/removal path is currently available.

## Request

The device owner asks for the current roster. The top-level schema is in
[`payload.schema.json`](payload.schema.json).

### Listing enrolled recovery buddies

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000601",
  "type": "https://trusttasks.org/spec/social-recovery/buddies/list/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:recovery-maintainer",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000006ff",
  "payload": {}
}
```

## Response

The maintainer returns the current roster. The response shape below is normative
prose: `payload.schema.json` governs the REQUEST only and carries no response
sub-schema. Failures are `trust-task-error` documents.

### Roster returned

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000602",
  "type": "https://trusttasks.org/spec/social-recovery/buddies/list/0.1#response",
  "issuer": "did:example:recovery-maintainer",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000006ff",
  "payload": {
    "recoveryBuddies": [
      {
        "recoveryBuddyDid": "did:example:sister",
        "keyId": "g-key-example:sister",
        "publicKeyHex": "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
        "addedAtIso": "2026-01-01T00:00:00Z",
        "name": "My sister",
        "bootstrap": true
      }
    ],
    "activeDeviceCount": 1,
    "canBootstrap": true
  }
}
```

## Security & Privacy

### Data carried

The response carries, for each enrolled buddy: their DID, a key
identifier, **their recovery public key as hex**, the enrolment timestamp, an
optional display label, and whether they are a bootstrap buddy. It also carries
the responder's active device count and whether bootstrap is still available.

A previous revision of this section stated that the response carries "never a
public key". **That was false**: the implementation this specification documents
returns `publicKeyHex` for every enrolled buddy. The sentence is corrected here
rather than removed, because a reader who saw it may have relied on it.

The public key is enrolment material, not recovery-execution material: it
identifies the key a buddy would sign with, and does not by itself advance or
authorise a recovery. No share, no threshold parameter and no reconstruction
input is carried.

### Correlation

**This is the same substantive concern
[`social-recovery/buddies/add`](../../add/0.1/spec.md)'s Correlation section
names, restated for this read**: the roster is a durable, sensitive
social-graph fact — who the owner trusts to hold recovery authority over
their digital access. A maintainer **MUST NOT** expose one owner's roster
to another party, including to the enrolled buddies seeing each other's
identities, unless the owner's own recovery mechanism requires buddies to
coordinate directly.

### Retention

`durable` in effect: the roster is exactly the standing state
`social-recovery/buddies/add`/`social-recovery/buddies/remove` maintain;
this read does not itself change what is retained.

### Consent/purpose

Descriptive only, per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) — reading
one's own roster requires no approval, matching the reference
implementation's `requiresApproval: false`.

### Custody scope

A recovery maintainer typically serves more than one owner's identity. It
**MUST** scope this read strictly to the requesting owner's own roster,
and **MUST NOT** ever return a different owner's enrolled buddies through
this operation.
**MUST** scope this read strictly to the requesting owner's own roster,
and **MUST NOT** ever return a different owner's enrolled buddies through
this operation.

## Open Questions for Registry Review

🟡 NEEDS-REVIEW: In the reference implementation, this operation is a same-device, owner-local read (the daemon's own Settings → Recovery buddies screen listing the operator's own roster) rather than one party sending a request that a distinct "recovery maintainer" party answers about someone else's data — the handler takes no per-request identifier and always resolves the caller's own identity chain. We are documenting it as specified because that is what the shipping code does, but whether a same-device configuration read belongs in a peer-facing Trust Task registry at all, as opposed to staying purely local and out of scope for this registry, is a design question we cannot resolve ourselves and would welcome the maintainers' judgment on.
