---
slug: "external/accounts/get"
version: "0.1"
title: "External Accounts — Get"
summary: "A manager, or a bound integration, reads one external account: settings, public material, bindings, state and last probe, never a key or secret."
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
  requirement: "RECOMMENDED"
  rationale: "Authority is the caller's standing at the custodian, which a verified proof or a transport-authenticated sender establishes. A proof is recommended so the read is attributable on every transport, relayed ones included."
sideEffects:
  level: "none"
  rationale: "Reads one account record. Persists nothing."
exposure:
  discloses: "metadata"
  ingests: "none"
  actsAsSubject: false
  rationale: "Discloses one account's provider settings, public material, bindings and state — to a bound integration, with only its own binding. No secret, key or credential."
retention:
  class: "transient"
  rationale: "A read; the custodian keeps nothing."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
related:
  - "external/accounts/list"
  - "external/accounts/setup"
  - "external/accounts/probe"
---

## Abstract

The **External Accounts — Get** Trust Task returns one external account as [`external/accounts/list`](../../list/0.1/spec.md) would show it. It is the read behind an account's page in a console: its model and settings, the public key or certificate its provider pins, who is bound to it and how far, whether it needs provider setup, and the last probe.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** return the account to a caller holding `external-accounts-manage` in its context.
2. **MUST** return it to a caller holding only `external-auth-use` there if, and only if, that caller is a binding's consumer, with every other binding omitted.
3. **MUST** answer every other caller, and an unknown id, with `external:notFound`, so that the read does not confirm an account exists to a caller with no standing.
4. **MUST NOT** return any private key, static secret or issued credential.

## Definitions

- **External account**, **binding**, **custodian** — as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json) and its [conventions](../../../_shared/0.1/CONVENTIONS.md) define them.
- **Manager** — a caller holding `external-accounts-manage` in the context. **Bound integration** — a caller holding `external-auth-use` there and named as a binding's consumer.

## Request

A manager or a bound integration (`issuer`) asks the custodian (`recipient`).

### One account

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/external/accounts/get/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000102",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary"
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The account

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/external/accounts/get/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000102",
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

A context and an account id in; one account record out. No task in this family returns a private key, a static secret or an issued credential in a read ([conventions §1](../../../../external/_shared/0.1/CONVENTIONS.md#1-five-rules) rule 1), so this response is safe to log and to show in a console.

### Correlation

Account settings name provider resources — an AWS account number, a GCP project, a bucket — and bindings name the integrations allowed to use them. Together they map a community's infrastructure. That is why a bound integration sees only its own binding, and why the read is restricted to managers.

### Retention

Nothing is stored by the read.

### Consent/purpose

Reading an account confers no use of it. The purpose is administering accounts, or, for a bound integration, discovering what it may ask for.
