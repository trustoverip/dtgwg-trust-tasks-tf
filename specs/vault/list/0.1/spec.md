---
slug: vault/list
version: "0.1"
title: Vault — List
summary: A vault consumer queries a vault maintainer for the metadata view of stored credentials, filtered by context, binding target, secret kind, tag, last-used time, expiry, or breach status; secrets are never returned by this task.
status: draft
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - vault
  - credentials
  - password-manager
  - inventory
  - search
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: List is read-only and the maintainer already authenticates the consumer (typically via the transport's session). A proof on the document is recommended for non-session-bound transports (e.g. a single Trust Task delivered over DIDComm with no prior handshake) so the maintainer can attribute the request to a specific consumer key even if the transport layer cannot. Maintainers MAY require a proof unconditionally as a policy choice.
sideEffects:
  level: none
  rationale: "Read-only metadata query over stored entries; secrets are never returned."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/list:context_not_found
    meaning: The `contextId` filter does not match any context the maintainer knows about. Distinguished from an empty result so consumers can tell "no entries" from "wrong context id".
    retryable: false
  - code: vault/list:permission_denied
    meaning: The consumer is authenticated but lacks the VaultRead capability for the requested context (or for any context if `contextId` was omitted).
    retryable: false
  - code: vault/list:filter_conflict
    meaning: The supplied filter combination is invalid (e.g. both `usedSince` and `neverUsed` set).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum: ["used_since_with_never_used", "page_size_above_ceiling", "cursor_invalid"]
  - code: vault/list:cursor_invalid
    meaning: The supplied `cursor` cannot be honoured (expired, malformed, or issued by a maintainer state the current maintainer no longer recognises). Consumers SHOULD retry from the first page without a cursor.
    retryable: true
---

## Abstract

The **Vault — List** Trust Task returns the **metadata view** of credential entries stored on a vault maintainer. Consumers use it to render a password-manager UI, drive search and filtering, answer questions like *"what are all my logins?"*, *"when did I last use facebook.com?"*, *"what's about to expire?"*, and *"which of my passwords are in known breaches?"*, and bootstrap a local cache before subscribing to incremental sync (`vault/sync/0.1`).

This task **never returns secret material**. Even when the consumer has `VaultRead` capability for a returned entry, the secret is only released by `vault/release/0.1` (where policy is re-evaluated, recent user verification can be required, and the release is recorded in the audit log). List exists to enumerate; release exists to use.

Filters are AND-combined. Pagination is opaque-cursor based. The maintainer chooses ordering; the recommended default is most-recently-used first.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the vault consumer) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vault/list/0.1`, with itself as `issuer` and the vault maintainer as `recipient`.
2. Populate filter fields only as needed; an empty payload is valid and means "all entries the consumer can read".
3. Treat `cursor` as opaque — never construct, decode, mutate, or assume meaning for a cursor returned by the maintainer.
4. **MUST NOT** populate both `usedSince` and `neverUsed`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements). When a `proof` is present, verify it.
2. Authenticate and authorise the requesting consumer against its ACL. If the consumer lacks `VaultRead` for the requested scope → `vault/list:permission_denied`.
3. If `contextId` is supplied and unknown → `vault/list:context_not_found`. (The maintainer SHOULD NOT silently degrade to "all contexts" when a `contextId` is supplied — explicit feedback prevents the consumer from believing it queried a narrower scope than it actually did.)
4. If `usedSince` and `neverUsed` are both present → `vault/list:filter_conflict` with `details.reason = "used_since_with_never_used"`.
5. Apply the filter set as the intersection of all populated criteria. For target filters (`targetOriginPrefix`, `targetDid`, `targetIosBundleId`, `targetAndroidPackage`), an entry matches when AT LEAST ONE of its `targets[]` entries satisfies the corresponding criterion. Return the resulting `entries` in metadata-only view (per the `VaultEntry` shared schema). **MUST NOT** populate any field that would carry secret material.
6. Set `truncated: true` and supply a `cursor` when more pages are available AND the maintainer supports pagination from this point. Set `truncated: true` with no `cursor` when more entries exist but pagination is unsupported beyond this page.
7. When redacting fields from `VaultEntry` per the consumer's policy (e.g. `lastUsedAt` for a low-trust Service consumer), list the redacted field names in `redactedFields` so the consumer's UI can correctly differentiate "absent" from "redacted".
8. Treat retries (same `id`, same filters) as idempotent; the result MAY change between requests as the underlying vault changes, but a retry MUST NOT cause side effects.

A conforming consumer **SHOULD** order returned entries by `lastUsedAt` descending (most-recently-used first), with `null` lastUsedAt sorted last, when no other order is implied by the filters.

## Definitions

* **Vault consumer.** Any party that holds a `VaultRead` capability on the maintainer's ACL — a Companion (browser plugin, mobile app), a Service (AI agent, sync daemon), or a CLI session.
* **Vault maintainer.** The party storing the entries and enforcing the policy; the VTA in the canonical deployment.
* **Metadata view.** A `VaultEntry` with `secretKind` populated but no secret material; defined in [`_shared/0.1/vault-entry.schema.json`](../../_shared/0.1/vault-entry.schema.json).
* **Binding target.** One element of a `VaultEntry.targets[]` array: a web origin, a DID, an iOS bundle id, or an Android package + signing fingerprints. A single credential MAY have multiple targets (typical: web origin + iOS bundle + Android package for the same service).
* **Context.** A trust context (persona) — a logical partition under the maintainer that bundles a DID + key set and a vault scope.

## Payload

`payload.contextId` (optional) — restrict to a single trust context.

`payload.targetOriginPrefix` (optional) — restrict to entries with any web-origin target whose origin starts with this prefix.

`payload.targetDid` (optional) — restrict to entries with any DID target exactly equal to this value.

`payload.targetIosBundleId` (optional) — restrict to entries with any iOS-app target whose `bundleId` exactly equals this value.

`payload.targetAndroidPackage` (optional) — restrict to entries with any Android-app target whose `packageName` exactly equals this value.

`payload.secretKind` (optional) — restrict to one secret kind.

`payload.tag` (optional) — restrict to entries whose `tags` array contains this value.

`payload.usedSince` (optional) — restrict to entries with `lastUsedAt >= usedSince`.

`payload.neverUsed` (optional, mutually exclusive with `usedSince`) — restrict to entries with no `lastUsedAt`.

`payload.expiresBefore` (optional) — restrict to entries with `expiresAt < this`. "What's about to expire" queries.

`payload.breached` (optional) — `true` returns only breached entries; `false` returns only non-breached.

`payload.pageSize` (optional, 1–1000) — caller's upper bound.

`payload.cursor` (optional, opaque) — continuation from a prior response.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

The full JSON Schema is in [`payload.schema.json`](payload.schema.json).

## Examples

### List everything in a context (first page)

```json
{
  "id": "vlist-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/vault/list/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T10:00:00Z",
  "payload": {
    "contextId": "ctx_work",
    "pageSize": 50
  }
}
```

### "What credential does the GitHub iOS app need?"

```json
{
  "id": "vlist-2345-6789-01bc-def234567890",
  "type": "https://trusttasks.org/spec/vault/list/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T10:01:00Z",
  "payload": {
    "targetIosBundleId": "com.github.stwalkerster.codehub"
  }
}
```

### "What's expiring this week?"

```json
{
  "id": "vlist-3456-7890-12cd-ef3456789012",
  "type": "https://trusttasks.org/spec/vault/list/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T10:02:00Z",
  "payload": {
    "expiresBefore": "2026-06-02T00:00:00Z"
  }
}
```

### "Show me only breached credentials"

```json
{
  "id": "vlist-4567-8901-23de-f4567890123",
  "type": "https://trusttasks.org/spec/vault/list/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T10:03:00Z",
  "payload": {
    "breached": true
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vault/list/0.1#response`. Payload: `{ entries, truncated, cursor?, redactedFields? }`.

### Successful list with mixed targets and pagination

```json
{
  "id": "vlist-resp-5678-9012-34ef-56789012345",
  "type": "https://trusttasks.org/spec/vault/list/0.1#response",
  "threadId": "vlist-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-26T10:00:00Z",
  "payload": {
    "entries": [
      {
        "id": "vault_01HZX2QY8E0F4Q3V5W7B9N2K6S",
        "contextId": "ctx_work",
        "targets": [
          { "kind": "web-origin", "origin": "https://github.com" },
          { "kind": "ios-app", "bundleId": "com.github.stwalkerster.codehub", "teamId": "VEKTX9H2N7" },
          { "kind": "android-app", "packageName": "com.github.android",
            "sha256CertFingerprints": ["12:34:56:78:9A:BC:DE:F0:12:34:56:78:9A:BC:DE:F0:12:34:56:78:9A:BC:DE:F0:12:34:56:78:9A:BC:DE:F0"] }
        ],
        "label": "Work GitHub",
        "secretKind": "passkey",
        "tags": ["work", "engineering"],
        "selectors": ["recent_uv_required"],
        "createdAt": "2026-02-14T09:30:00Z",
        "createdBy": "did:peer:2.Ez6LSc…",
        "updatedAt": "2026-04-02T14:15:00Z",
        "lastUsedAt": "2026-05-25T22:11:00Z",
        "passwordChangedAt": "2026-04-02T14:15:00Z",
        "version": 7
      },
      {
        "id": "vault_01HZX2R0F1G5R4W6X8C0P3L7T",
        "contextId": "ctx_work",
        "targets": [
          { "kind": "web-origin", "origin": "https://aws.amazon.com" }
        ],
        "label": "Work AWS — root",
        "secretKind": "password",
        "tags": ["work", "high-value"],
        "notes": "Recovery email: ops@example.com",
        "selectors": ["step_up_push", "high_value"],
        "createdAt": "2026-01-08T11:00:00Z",
        "updatedAt": "2026-05-01T08:00:00Z",
        "lastUsedAt": "2026-05-26T09:30:00Z",
        "passwordChangedAt": "2026-05-01T08:00:00Z",
        "breachedAt": "2026-04-22T00:00:00Z",
        "version": 3
      }
    ],
    "truncated": true,
    "cursor": "opaque-maintainer-cursor-string"
  }
}
```

### Empty result (no matches but filters were valid)

```json
{
  "id": "vlist-resp-6789-0123-45f0-678901234567",
  "type": "https://trusttasks.org/spec/vault/list/0.1#response",
  "threadId": "vlist-4567-8901-23de-f4567890123",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-26T10:03:01Z",
  "payload": {
    "entries": [],
    "truncated": false
  }
}
```

## Security & Privacy

**No secret leakage.** The shared `VaultEntry` schema deliberately omits secret material. Maintainers MUST verify their implementation cannot accidentally populate secret fields when serialising for this task — a common bug pattern is a single `VaultEntry` type used for both internal storage and external responses, with the secret optional. The recommended implementation is a distinct *metadata projection* type that cannot syntactically carry a secret.

**Mobile binding integrity.** When matching `targetIosBundleId` or `targetAndroidPackage`, the maintainer is only matching the identifier the requesting Companion claims. Production deployments SHOULD additionally verify the requesting Companion's device-level attestation (App Attest on iOS, Play Integrity on Android) at the ACL layer before granting `VaultRead` — see `device/register/0.1`. Android matching MUST verify the signing fingerprint, not just the package name, because package names alone are forgeable on rooted devices.

**Enumeration as signal.** A consumer that can list entries learns the existence, count, and timing of every credential in scope. Maintainers issuing `VaultRead` to a Service consumer (AI agent, sync daemon) SHOULD narrow the consumer's scope to a specific context or target set rather than grant blanket read.

**Cursor stability.** Cursors MUST NOT encode secret state; the maintainer MUST assume a cursor may be persisted by the consumer and replayed weeks later. Recommended implementation: opaque pointer to a server-side query snapshot with a short TTL, returning `cursor_invalid` after expiry.

**Timing-data exposure.** `lastUsedAt`, `passwordChangedAt`, and `breachedAt` are side-channels revealing behaviour patterns and risk posture. When the requesting consumer is lower-trust (e.g. a third-party Service), maintainers SHOULD redact these to hour or day precision, or omit them entirely and list the omissions in `redactedFields`.

**Replay.** This task is read-only and idempotent; replay is benign. Maintainers SHOULD nevertheless apply the framework's general `issuedAt` freshness check to limit the window in which a captured request can be replayed.

**Audit.** Maintainers SHOULD record list calls with the requesting consumer VID, the filter set, and the result count (NOT the entry ids). This supports incident investigation ("which AI agent enumerated my vault and when") without re-leaking secret presence.

The optional `ext` extension is part of the producer's signed surface; producers MUST NOT place data in `ext` that they would not be comfortable signing.
