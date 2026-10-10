---
slug: vtc/storage/configs/probe
version: "0.1"
title: "VTC Storage — Configs — Probe"
summary: A room-hosting administrator checks that a storage config works — authenticate, then put, read back and delete a canary object under a test prefix with the config's own credentials — and gets each step's result with the backend's error verbatim.
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
  requirement: RECOMMENDED
  rationale: >-
    The probe exercises the config's credentials against its backend, and on Walrus may spend the community's storage funds. A proof is recommended so it is attributable on every transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed probe repeats a write to the backend and, on Walrus, a payment.
sideEffects:
  level: mutating
  rationale: >-
    Writes one canary object under a test prefix and deletes it again; records the result in the config's health. No room's data is touched.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses whether each step succeeded and, when one failed, the backend's error text — which can name the account, role or bucket involved — to an administrator who already holds the config's settings. Never a credential.
retention:
  class: transient
  rationale: >-
    The canary is deleted at the end of the probe; the community keeps only the result as the config's last health.
errorCodes:
  - code: vtc/storage/configs/probe:notFound
    meaning: "The community has no storage config with this identifier."
    retryable: false
  - code: vtc/storage/configs/probe:credentialRequired
    meaning: "The config cannot authenticate: a `sealed` config whose credential has not been set."
    retryable: false
related:
  - vtc/storage/configs/get
  - vtc/storage/configs/create
---

## Abstract

The **VTC Storage — Configs — Probe** Trust Task checks a storage config end to end, with the credentials it actually uses: authenticate (for a `vta-account` config, by asking the community's VTA for a credential scoped to the test prefix), then put a small canary object, read it back and compare it, and delete it. Each step reports whether it worked, how long it took, and — when it failed — the backend's own error, verbatim, so an administrator setting up a bucket policy sees what the cloud said rather than a paraphrase.

A probe that passes says the community can store, read and delete on this config right now. It is what an administrator runs after creating a config, after changing the account behind it, and when uploads start failing.

On **Walrus** a write is a paid registration on a public ledger, so a probe writes only when `includeWrite` is true. Without it, a Walrus probe checks that the signer authenticates and that the relay and aggregator answer.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, an unknown config with `notFound`, and a config with no usable credential with `credentialRequired`.
2. **MUST** run the steps in order — `authenticate`, `put`, `get`, `delete` (on Walrus without a write: `authenticate`, `reachRelay`, `reachAggregator`) — stop at the first failure, and report every step it attempted.
3. **MUST** write the canary under a prefix used for nothing else (`probe/` beneath the config's prefix), never under a room's prefix, and **MUST** compare the bytes read back with those written.
4. **MUST** report a failed step's backend error verbatim, truncated to 2048 characters, and **MUST** first remove from it anything that is a credential or derived from one (a signature, a session token).
5. On a `walrus` config, **MUST NOT** register a blob unless `includeWrite` is true. On every other kind `includeWrite` is ignored.
6. **MUST** record the outcome as the config's last probe.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

## Definitions

- **Canary** — a small object of random bytes written, read back and deleted by the probe.
- **Step** — `authenticate`, `put`, `get`, `delete`, `reachRelay`, `reachAggregator`.

## Request

A room-hosting administrator (`issuer`) names one config to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Probe an S3 config

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/probe/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000bff",
  "payload": {
    "configId": "eu-s3-primary"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A failed probe is a successful response with `ok: false`; a refusal to probe at all is a `trust-task-error`.

### The bucket policy is missing DeleteObject

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/probe/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000bff",
  "payload": {
    "configId": "eu-s3-primary",
    "ok": false,
    "probedAt": "2026-10-10T10:00:01Z",
    "prefix": "probe/",
    "steps": [
      {
        "step": "authenticate",
        "ok": true,
        "durationMs": 412
      },
      {
        "step": "put",
        "ok": true,
        "durationMs": 88
      },
      {
        "step": "get",
        "ok": true,
        "durationMs": 41
      },
      {
        "step": "delete",
        "ok": false,
        "durationMs": 37,
        "error": "AccessDenied: User: arn:aws:sts::111122223333:assumed-role/northwind-rooms/eu-s3-primary is not authorized to perform: s3:DeleteObject on resource: arn:aws:s3:::northwind-rooms-eu/probe/c4a1"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A config identifier in; step results and, on failure, the backend's error text out. Backend errors routinely name the role, account and bucket involved; the community strips anything credential-like before returning one.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The backend sees one more write and delete from the community's account under a probe prefix; on Walrus with `includeWrite`, a registration attributable to the community's signer address on a public ledger.

### Retention

The canary is deleted by the probe — on Walrus, registered for one epoch and left to lapse. The community keeps the outcome as the config's last health.

### Consent/purpose

The purpose is checking that the community's storage works. A probe is not a way to exercise a credential against resources outside the config: it writes only beneath the config's own probe prefix.
