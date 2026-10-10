---
slug: "external/accounts/update"
version: "0.1"
title: "External Accounts — Update"
summary: "A manager changes an external account's label or provider settings; the model and the key cannot change."
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
  rationale: "Rewrites the account's label or settings. Retargeting settings changes which provider identity the community's integrations act as."
exposure:
  discloses: "metadata"
  ingests: "metadata"
  actsAsSubject: false
  rationale: "Provider configuration in, the updated account out. No secret either way."
retention:
  class: "durable"
  rationale: "The change persists with the account."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:archived"
    meaning: "The account is `archived`; restore it before changing or using it. See the family conventions §2."
    retryable: false
  - code: "external:invalidSettings"
    meaning: "The settings validate against the schema but are unusable: a model change, an inconsistent value, or a driver or network this custodian does not implement. `details.member` names the offending member."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["member"], "properties": {"member": {"type": "string", "maxLength": 128}}}
related:
  - "external/accounts/get"
  - "external/accounts/probe"
  - "external/accounts/setup"
---

## Abstract

The **External Accounts — Update** Trust Task changes an account's `label` or `settings` — a bucket, a role, a pool provider, a Sui allow-list. It cannot change the account's model, which would make it a different account with the same name, and it cannot touch the key, which [`external/accounts/keys/rotate`](../../keys/rotate/0.1/spec.md) does.

Retargeting settings is the most dangerous quiet change an account can undergo: an integration keeps asking for "the account" and gets a different bucket. So it is consent-gated by recommendation, and a custodian marks the account as needing a fresh probe.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** refuse an `archived` account with `external:archived`, and with `external:invalidSettings` (`details.member: "model"`) a `settings.model` different from the account's.
2. **MUST** replace `settings` whole when present — never merge — so that the stored settings are exactly what was approved.
3. **MUST** recompute `egressHosts`, and **SHOULD** set `providerSetupRequired` when a setting the provider-side setup encodes has changed (a role, a pool provider, a bucket), until a complete probe succeeds.
4. **MUST NOT** change bindings, key or state.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

**Consent.** A custodian **SHOULD** subject this task to consent under its approvals policy ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)): the request is held, and approvers — the context's administrators other than the requester — each decide with a `task-consent/decision` signed by their own DID. A decision counts for the DID that signed it, never for a relayer that delivered it. Without consent, one administrator, or one compromised console relaying for administrators, can point every integration bound to the account at a different provider resource.

## Definitions

- Shapes as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json). The model is `settings.model`; there is no separate member to disagree with it.

## Request

A manager (`issuer`) asks the custodian (`recipient`). At least one of `label` and `settings`.

### Move the account to another bucket

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000007",
  "type": "https://trusttasks.org/spec/external/accounts/update/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000104",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "settings": {
      "model": "aws-roles-anywhere",
      "region": "eu-west-1",
      "trustAnchorArn": "arn:aws:rolesanywhere:eu-west-1:123456789012:trust-anchor/0b2f7c9e-6a51-4d3e-9b1a-3c5d7e9f1a2b",
      "profileArn": "arn:aws:rolesanywhere:eu-west-1:123456789012:profile/5e8a1c3d-2b4f-4a6e-8c0d-1f3a5b7c9e2d",
      "roleArn": "arn:aws:iam::123456789012:role/community-rooms-storage",
      "bucket": "community-rooms-eu-2"
    }
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the account as updated.

### Updated

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000008",
  "type": "https://trusttasks.org/spec/external/accounts/update/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000104",
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
        "bucket": "community-rooms-eu-2"
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
      "providerSetupRequired": true,
      "createdAt": "2026-10-10T08:00:00Z",
      "updatedAt": "2026-10-10T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

Provider configuration in, the account out. No secret, no key.

### Correlation

Settings name provider accounts and resources, which links this community to them for anyone who can read the account.

### Retention

The new settings replace the old; the audit trail keeps the change.

### Consent/purpose

The purpose is to let named integrations use a provider without holding its credentials. Consent from other administrators is what keeps that authority from being one person's decision.
