---
slug: "external/accounts/bindings/grant"
version: "0.1"
title: "External Accounts — Bindings — Grant"
summary: "A manager allows one integration to use an external account, up to a scope ceiling, a credential lifetime and a rate; a grant for an already-bound integration replaces its binding."
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
  rationale: "Adds or replaces a binding: from the moment it lands, the named integration can obtain credentials for the account."
exposure:
  discloses: "metadata"
  ingests: "metadata"
  actsAsSubject: false
  rationale: "A consumer DID and its limits in; the account out."
retention:
  class: "durable"
  rationale: "The binding persists until revoked or the account is deleted."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external/accounts/bindings/grant:ceilingNotApplicable"
    meaning: "The ceiling does not fit the model: prefixes on an account with no bucket, `scopes` outside the account's own, or a ceiling on `sui-signer`, whose ceiling is the account's allow-list."
    retryable: false
related:
  - "external/accounts/bindings/revoke"
  - "external/credentials/issue"
  - "external/sign"
---

## Abstract

An account is useless until something may use it, and dangerous if the wrong thing may. The **External Accounts — Bindings — Grant** Trust Task names **one integration** — by DID — that may use the account, and how far:

- **`scopeCeiling`**: the widest prefixes, actions or OAuth scopes it may request;
- **`maxTtlSeconds`**: the longest credential it may get (15 minutes is the recommendation, an hour the ceiling);
- **`ratePerMinute`**: how often;
- optionally **`sourceCidrs`**, so that credentials are useless off the integration's network where the provider supports it.

Each issuance is downscoped again, within this ceiling, to what that one request needs.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** refuse with `external/accounts/bindings/grant:ceilingNotApplicable` a ceiling the model cannot honour.
2. **MUST** replace an existing binding for the same consumer rather than add a second one.
3. **MUST** record `grantedAt`.
4. **SHOULD NOT** require the consumer to hold `external-auth-use` at grant time, since provisioning often runs in the other order, but **MUST** require it at every use.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

**Consent.** A custodian **SHOULD** subject this task to consent under its approvals policy ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)): the request is held, and approvers — the context's administrators other than the requester — each decide with a `task-consent/decision` signed by their own DID. A decision counts for the DID that signed it, never for a relayer that delivered it. Without consent, one administrator, or one compromised console relaying for administrators, can let an integration of their choosing — or one they control — act as the community at a provider.

## Definitions

- **AccountBinding**, **CredentialScopeCeiling** — as [`external/_shared/0.1/accounts.schema.json`](../../../../_shared/0.1/accounts.schema.json) defines them.

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### Let the community's host store room files

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/external/accounts/bindings/grant/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000106",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "binding": {
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
      "ratePerMinute": 120
    }
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the account with its bindings.

### Granted

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/external/accounts/bindings/grant/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000106",
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
      "state": "active",
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
      "updatedAt": "2026-10-10T09:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A binding in; the account out.

### Correlation

A binding links an integration DID to a provider account.

### Retention

Durable until revoked.

### Consent/purpose

A binding is the grant of a provider identity's use; consent from other administrators keeps it from being one person's decision.
