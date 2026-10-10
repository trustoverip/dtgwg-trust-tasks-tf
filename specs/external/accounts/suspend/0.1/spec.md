---
slug: "external/accounts/suspend"
version: "0.1"
title: "External Accounts — Suspend"
summary: "A manager suspends an external account — the kill switch: every issuance and signature is refused at once, by one person, without consent."
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
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor."
sideEffects:
  level: "mutating"
  rationale: "Moves the account to `suspended`. Every use is refused from that moment."
exposure:
  discloses: "metadata"
  ingests: "metadata"
  actsAsSubject: false
  rationale: "An account reference and a reason in; the account out. No secret."
retention:
  class: "durable"
  rationale: "The state change persists; the audit row keeps the reason."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:archived"
    meaning: "The account is `archived`; restore it before changing or using it. See the family conventions §2."
    retryable: false
  - code: "external/accounts/suspend:invalidTransition"
    meaning: "The account is not in a state this task applies to (`active`)."
    retryable: false
related:
  - "external/accounts/resume"
  - "external/accounts/archive"
  - "external/accounts/restore"
  - "external/accounts/delete"
---

## Abstract

The **External Accounts — Suspend** Trust Task is the kill switch. From the moment it is stored, the custodian refuses every [`external/credentials/issue`](../../../credentials/issue/0.1/spec.md) and [`external/sign`](../../../sign/0.1/spec.md) for the account, for every binding. Reads continue, so the account can be investigated; [`external/accounts/resume`](../../resume/0.1/spec.md) undoes it.

Suspension cannot recall a credential already issued: it lapses at its expiry, which bindings cap at an hour and should cap at minutes. For `aws-roles-anywhere`, the setup's CRL, pushed to the trust anchor, also stops the end-entity certificate being accepted at the provider.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** accept the task only for an account that is `active`. **MUST** refuse an `archived` account, where `archived` is not a starting state, with `external:archived`, and any other state with `external/accounts/suspend:invalidTransition`.
2. **MUST** refuse every issuance and signature for the account from the moment the change is stored, including any not yet signed.
3. **MUST NOT** require consent.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

This task **MUST NOT** be consent-gated: it reduces authority, and a reduction that waits for a quorum is one an attacker using the account can outlast.

## Definitions

- **Account state** — `AccountState` in [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json). The lifecycle is `active` ⇄ `suspended`, either → `archived`, `archived` → `suspended` (restore), `archived` → gone (delete).

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### Suspend

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000040",
  "type": "https://trusttasks.org/spec/external/accounts/suspend/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000130",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "reason": "Rooms storage moved to the new account."
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`.

### Done

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000060",
  "type": "https://trusttasks.org/spec/external/accounts/suspend/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000130",
  "payload": {
    "account": {
      "id": "eu-s3-primary",
      "label": "EU S3 — main account",
      "context": "community",
      "settings": {
        "model": "aws-roles-anywhere",
        "region": "eu-west-1",
        "trustAnchorArn": "arn:aws:rolesanywhere:eu-west-1:123456789012:trust-anchor/0b2f7c9e-6a51-4d3e-9b1a-3c5d7e9f1a2b",
        "profileArn": "arn:aws:rolesanywhere:eu-west-1:123456789012:profile/5e8a1c3d-2b4f-4a6e-8c0d-1f3a5b7c9e2d",
        "roleArn": "arn:aws:iam::123456789012:role/community-rooms-storage",
        "bucket": "community-rooms-eu"
      },
      "state": "suspended",
      "publicMaterial": {
        "algorithm": "ES256",
        "keyFingerprint": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "certificatePem": "-----BEGIN CERTIFICATE-----\nMIIBszCCAVmgAwIBAgIUQ29tbXVuaXR5IENBIGV4YW1wbGUwCgYIKoZIzj0EAwIw\n-----END CERTIFICATE-----\n"
      },
      "bindings": [
        {
          "consumer": "did:example:community",
          "scopeCeiling": {
            "prefixes": [
              "rooms/"
            ],
            "actions": [
              "put",
              "get",
              "delete"
            ]
          },
          "maxTtlSeconds": 900,
          "ratePerMinute": 120,
          "grantedAt": "2026-10-10T09:00:00Z"
        }
      ],
      "egressHosts": [
        "rolesanywhere.eu-west-1.amazonaws.com"
      ],
      "providerSetupRequired": false,
      "createdAt": "2026-10-10T08:00:00Z",
      "updatedAt": "2026-10-10T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An account reference and an optional reason in; the account out.

### Correlation

None beyond the account itself.

### Retention

The state change persists; the audit trail keeps who changed it, when and why.

### Consent/purpose

Reductions need no one else; restorations of authority are consented where the operator enforces consent (conventions §4).
