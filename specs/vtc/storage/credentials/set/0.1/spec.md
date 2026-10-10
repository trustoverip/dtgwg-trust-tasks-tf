---
slug: vtc/storage/credentials/set
version: "0.1"
title: "VTC Storage — Credentials — Set"
summary: The discouraged fallback for a community with no VTA account — a room-hosting administrator stores a storage config's long-lived credential, sealed in their browser to the community and never readable again. Parked for consent.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request installs the credential every room on a config will be stored with, replacing any before it. Its proof covers the sealed block, so who supplied it is attributable after the transport has closed, and a relay cannot swap in another.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would restore a credential administrators have since replaced — possibly one that was replaced because it leaked.
sideEffects:
  level: mutating
  rationale: >-
    Stores or replaces the config's credential. Reversible only by setting another; the replaced credential is destroyed.
subjectPath: /configId
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: >-
    The request carries a long-lived storage credential — an access-key pair, a service-account key or a Sui private key — sealed end to end to the community, so no transport, relay or log between the administrator's browser and the community can read it. The response carries the config's credential state — a mode, a keyed fingerprint and when it was set — and never the credential.
retention:
  class: durable
  rationale: >-
    The credential is kept, encrypted at rest, until replaced, until the config's authentication changes away from `sealed`, or until a restore — which never carries it.
errorCodes:
  - code: vtc/storage/credentials/set:notFound
    meaning: "The community has no storage config with this identifier."
    retryable: false
  - code: vtc/storage/credentials/set:notSealedMode
    meaning: "The config's `auth.mode` is not `sealed`. Credentials for a `vta-account` config are the VTA's, set there; an `ambient` config has none."
    retryable: false
  - code: vtc/storage/credentials/set:cannotOpen
    meaning: "The sealed block is not sealed to this community, is malformed, or its digest does not match `digestMultibase`. Nothing was stored."
    retryable: false
  - code: vtc/storage/credentials/set:invalidCredential
    meaning: "The block opened, but what it carries is not a credential for the config's kind (see Definitions). Nothing was stored."
    retryable: false
related:
  - vtc/storage/configs/create
  - vtc/storage/configs/update
  - vtc/storage/configs/probe
  - vtc/admin/actions/show
---

## Abstract

The community is meant to hold **no** long-lived storage credential. A config authenticated by `vta-account` gets short-lived, room-scoped credentials from the community's VTA, and no administrator, however many there are, can take anything home. The **VTC Storage — Credentials — Set** Trust Task is the fallback for a community with no runtime link to a VTA: it stores one long-lived credential on the community for a config whose `auth.mode` is `sealed`. It is discouraged, and a community **SHOULD** offer it only for evaluation setups.

The credential travels **sealed**: the administrator's browser seals it with the workspace's one secret-bearing wire format — HPKE to the community's DID, framed in ASCII armor — before it leaves the page. The community stores it encrypted at rest under a key derived from its own secret, shows it afterwards only as a fingerprint, never returns it through any task, never exports it, and leaves it out of every backup.

**What that protects against, and what it does not.** It protects the credential from every administrator through every surface the community offers: no task, console page, CLI, log, backup or audit row yields it, so several administrators can share a community without any of them being able to take the key. It does **not** protect it from whoever operates the machine the community runs on. The process must be able to use the key, so an operator with access to the process or its secret store can reconstruct it, and the community has no enclave to prevent that. A deployment where administrators and operators are the same people gains little from this mode and **SHOULD** treat it as unavailable.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** seals the credential in the administrator's own client, to the community's DID, and never sends it unsealed through any channel. It sets `digestMultibase` to the digest of the armored block it sends.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, an unknown config with `notFound`, and a config whose `auth.mode` is not `sealed` with `notSealedMode`.
2. **MUST** check `digestMultibase` against `sealed` and open it with its own key, refusing with `cannotOpen` on any failure, and **MUST** refuse with `invalidCredential` a payload that is not a credential for the config's kind.
3. **MUST** store the credential only encrypted at rest under a key derived from its own secret, **MUST NOT** write it in plaintext to any log, audit row, telemetry or backup, and **MUST** exclude it from its backup and from any export.
4. **MUST NOT** return the credential, or anything derived from it but a keyed fingerprint, through this or any other task; the response's `fingerprint` is computed with a community-held key, never a bare hash of the secret.
5. **MUST** destroy any credential this one replaces.
6. **SHOULD** probe the config with the new credential before reporting success, and **MUST** report the probe's outcome when it does.
7. **MUST** require the credential to be set again after a restore, reporting the config's credential as `required` until it is.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

**Second-administrator consent.** This operation changes where the community keeps its members' files or what it uses to reach them, so a community implementing the administrator action list **parks** it rather than executing it: it answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../../../vtc/admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked payload itself, exactly once, when its threshold is met, re-running every check in this specification against the community as it is then, and answers on the original `threadId` with this task's `#response` or a `trust-task-error`. An approval is signed by the approver's own DID; nobody approves their own request. Where the community has no other administrator to consent and runs in a single-administrator mode, it may waive the consent on its own stated terms. Whether, and by how many, an operation must be approved is the community's policy; this specification describes the model it is designed for. The parked action carries the sealed block, which only the community can open; approvers see its digest and the config it is for, never the credential.

## Definitions

- **Sealed block** — an ASCII-armored `sealed_transfer` envelope (HPKE base mode, X25519-HKDF-SHA256, HKDF-SHA256, ChaCha20-Poly1305, info string `vta-sealed-transfer/v1`) whose recipient is the X25519 derivation of the community DID's Ed25519 key, the same format [`provision/integration`](../../../../../provision/integration/0.3/spec.md) delivers bundles in.
- **Credential, by kind** — what the sealed payload must carry: for `s3`, an access key id and secret access key; for `gcs`, a service-account key; for `walrus`, the Sui private key that pays for storage. A `local` config has none.
- **Fingerprint** — a keyed digest that identifies which credential is stored without revealing it.

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Set an R2 access key

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/credentials/set/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000cff",
  "payload": {
    "configId": "r2-archive",
    "sealed": "-----BEGIN VTA SEALED BUNDLE-----\nBundle-Id: 4f1c0a9b2d3e4f5061728394a5b6c7d8\nChunk: 1/1\nDigest-Algo: sha256\n\nhQEMA0xJexampleCiphertextOnly4mR0Y=\n=Zc3a\n-----END VTA SEALED BUNDLE-----",
    "digestMultibase": "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc",
    "reason": "R2 cannot federate; evaluation setup only."
  }
}
```

## Response

When the change executes, the community answers on the request's `threadId` with the sub-schema reachable via `$anchor: "response"`: the config's credential state, by fingerprint, and the probe's outcome when it ran one. Before that, a parking community answers with `trust-task-next-step` (see [Authorization](#authorization)). A refusal is a `trust-task-error`.

### Stored

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/credentials/set/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000cff",
  "payload": {
    "configId": "r2-archive",
    "credentialState": {
      "status": "sealed",
      "fingerprint": "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
      "setAt": "2026-10-10T10:30:00Z"
    },
    "probeOk": true
  }
}
```

## Security & Privacy

### Data carried

A config identifier, a sealed credential, its digest and a reason in; a fingerprint and a probe outcome out. The credential is readable only by the community. The digest and fingerprint are not secrets, and identify the block and the stored credential without revealing them.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The fingerprint lets administrators tell when a credential changed; because it is keyed, it cannot be matched against a guessed or leaked key by anyone outside the community.

### Retention

The credential is kept until replaced, until the config stops using `sealed`, or until a restore. A replaced credential is destroyed. The parked action holding the sealed block **MUST** be discarded once the action closes, whichever way it closes.

### Consent/purpose

The credential is supplied solely so the community can store and read its rooms' files on the config. It **MUST NOT** be used against any resource outside that config, and the community **SHOULD** tell administrators, wherever the config is shown, that a `sealed` credential is recoverable by whoever operates the machine.
