---
slug: "external/accounts/secret/set"
version: "0.1"
title: "External Accounts — Secret — Set"
summary: "A manager sets or replaces the secret of a static external account, sealed in the manager's own client to a single-use wrapping key the custodian issued; the value is never returned, only a keyed fingerprint."
status: "draft"
targetFrameworkVersion: "0.6.0"
category: "key-management"
parties:
  - role: "Account manager"
    requirement: "REQUIRED"
    member: "issuer"
    identifierScope: "any"
  - role: "Key custodian"
    requirement: "REQUIRED"
    member: "recipient"
proofRequirement:
  requirement: "REQUIRED"
  rationale: "The task changes an authority the community's integrations rely on, and where consent applies it is counted against the request's own digest. Both need the request to be attributable to its signer whatever transport, or relayer, carried it."
issuedAtRequirement:
  requirement: "REQUIRED"
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor. An approval is bound to the request, and a request that could be replayed outside a window could be presented again after its approvers had moved on."
sideEffects:
  level: "mutating"
  rationale: "Stores or replaces the account's secret. A replacement takes effect for the next issuance."
exposure:
  discloses: "none"
  ingests: "secret"
  actsAsSubject: false
  rationale: "Ingests a provider secret — an access-key secret or an API token — inside a sealed-transfer bundle sealed to a single-use wrapping key only the custodian holds. Returns a keyed fingerprint and a time, never the value."
retention:
  class: "durable"
  rationale: "The secret is kept, wrapped, until replaced or the account is deleted. It is excluded from the custodian's backups."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:archived"
    meaning: "The account is `archived`; restore it before changing or using it. See the family conventions §2."
    retryable: false
  - code: "external/accounts/secret/set:notStaticModel"
    meaning: "The account's model holds a key, not a secret (only `s3-static-presign` and `static-secret` take one)."
    retryable: false
  - code: "external/accounts/secret/set:unsealFailed"
    meaning: "The bundle is not sealed to an unexpired, unused wrapping key of this custodian, fails its producer assertion, does not carry an ExternalSecretPayload, or names a different context, account or access key id than the request and the account."
    retryable: false
related:
  - "keys/import-wrapping-key"
  - "external/accounts/create"
  - "external/accounts/probe"
---

## Abstract

Two models hold a **secret** instead of a key: `s3-static-presign` (an S3-compatible store that cannot federate) and `static-secret` (an API key used by a custodian driver). The **External Accounts — Secret — Set** Trust Task is the only way such a secret reaches the custodian, and nothing ever brings it back out.

The secret travels as a **sealed-transfer bundle**, sealed **in the manager's client** — the administrator's browser, say — so a console relaying the request, a terminating proxy, or a request log never holds it. It is sealed to a **single-use wrapping key** the client first obtains from the custodian with [`keys/import-wrapping-key`](../../../../../keys/import-wrapping-key/0.1/spec.md), not to the custodian's DID: the wrapping key's private half lives in the custodian's memory for minutes and opens one bundle, so a captured bundle cannot be replayed, and a later compromise of the custodian's DID key opens nothing recorded earlier. The custodian answers with a fingerprint: enough to confirm which value is set, nothing that recovers it.

So no administrator, however many there are, can recover the secret after setting it, through any surface the custodian offers.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../../external/_shared/0.1/CONVENTIONS.md).

A **producer** (the manager's client) **MUST**:

1. Obtain a wrapping key with [`keys/import-wrapping-key`](../../../../../keys/import-wrapping-key/0.1/spec.md) and verify that response's proof against the custodian's DID before sealing to it, as that task requires.
2. Seal an `ExternalSecretPayload` naming this request's `context` and `id` to the wrapping key, in the client where the secret was entered, with a producer assertion, and **MUST NOT** pass the cleartext to any other party first. Sealing to the custodian's DID is not conforming.

A conforming **custodian** (`recipient`):

1. **MUST** refuse an `archived` account with `external:archived`, and with `external/accounts/secret/set:notStaticModel` an account whose model is not `s3-static-presign` or `static-secret`.
2. **MUST** open the bundle only with the wrapping key `wrappingKeyId` names, which it issued, has not used and has not let expire; **MUST** discard that key whether or not the bundle opens; **MUST** refuse with `external/accounts/secret/set:unsealFailed` an unknown, used or expired `wrappingKeyId`; and verify the bundle's producer assertion.
3. **MUST** refuse with `external/accounts/secret/set:unsealFailed`, storing nothing, a bundle that does not open, fails its assertion, does not carry an `ExternalSecretPayload`, names a `context` or `account` other than the request's, or (for `s3-static-presign`) names an `accessKeyId` other than the account's.
4. **MUST** store the secret wrapped, exclude it from backups, and **MUST NOT** return, log, audit or export it through any task, ever.
5. **MUST** answer with a `SecretFingerprint`, keyed under a custodian-held key, so that the fingerprint cannot be used to test guesses offline.
6. **SHOULD** clear `providerSetupRequired` only after a complete probe with the new secret succeeds.

The authority [`keys/import-wrapping-key`](../../../../../keys/import-wrapping-key/0.1/spec.md) checks before issuing a wrapping key is, for this purpose, the authority to set a secret: a custodian **MUST** issue one to a caller holding `external-accounts-manage`.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

**Consent.** A custodian **SHOULD** subject this task to consent under its approvals policy ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)): the request is held, and approvers — the context's administrators other than the requester — each decide with a `task-consent/decision` signed by their own DID. A decision counts for the DID that signed it, never for a relayer that delivered it. Without consent, one administrator, or one compromised console relaying for administrators, can swap the community's credential for one they chose, or for one that belongs to someone else's account.

## Definitions

- **Sealed bundle** — `SealedTransferBundle` in [`external/_shared/0.1/accounts.schema.json`](../../../../_shared/0.1/accounts.schema.json). The plaintext of the sealed bundle is an `ExternalSecretPayload` from the same file: the secret, and the context and account it is for, so a bundle captured on its way to one account cannot be replayed into another.
- **Wrapping key** — as [`keys/import-wrapping-key`](../../../../../keys/import-wrapping-key/0.1/spec.md) defines it: fresh, single-use, held in memory for minutes. See [conventions §6](../../../../_shared/0.1/CONVENTIONS.md#6-sealed-payloads).
- **Fingerprint** — `SecretFingerprint` from the same file.

## Request

A manager (`issuer`) sends the bundle to the custodian (`recipient`).

### An R2 access-key secret

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000009",
  "type": "https://trusttasks.org/spec/external/accounts/secret/set/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000105",
  "payload": {
    "context": "community",
    "id": "r2-rooms",
    "wrappingKeyId": "wk-7c1e9a",
    "sealedSecret": "-----BEGIN VTA SEALED BUNDLE-----\nBundle-Id: 9f3c…\nDigest-Algo: sha-256\nChunk: 0/1\n\nU2VhbGVkU2VjcmV0Q2lwaGVydGV4dFNlYWxlZFNlY3JldENpcGhlcnRleHQ\n=Q1JD\n-----END VTA SEALED BUNDLE-----"
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the fingerprint and the time it was set.

### Set

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000010",
  "type": "https://trusttasks.org/spec/external/accounts/secret/set/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000105",
  "payload": {
    "fingerprint": "hmacsha256:q3Vx9Lr2mB7cT0nY8wKe4A",
    "setAt": "2026-10-10T10:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

A sealed secret in; a keyed fingerprint out. The cleartext exists only in the client that sealed it and inside the custodian.

### Correlation

The fingerprint is keyed per custodian, so the same secret set at two custodians does not produce matching fingerprints. The wrapping key is fresh per request and belongs to no DID document, so it links nothing.

### Retention

Wrapped at rest until replaced or the account is deleted; never in a backup. After a restore the account reports `providerSetupRequired` until the secret is set again.

### Consent/purpose

The purpose is to let named integrations use a provider without holding its credentials. Consent from other administrators is what keeps that authority from being one person's decision.
