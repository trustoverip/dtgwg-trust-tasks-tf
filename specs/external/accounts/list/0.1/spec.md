---
slug: "external/accounts/list"
version: "0.1"
title: "External Accounts — List"
summary: "A manager, or a bound integration, lists the external accounts a key custodian holds in a context: settings, public material, bindings and state, never a key or secret."
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
  rationale: "Reads account records. Persists nothing."
exposure:
  discloses: "metadata"
  ingests: "none"
  actsAsSubject: false
  rationale: "Discloses, to a manager, the accounts in a context — their provider settings, public keys, bindings and state — or, to a bound integration, only the accounts it is bound to. Provider configuration and public material only; no secret, key or credential is ever returned."
retention:
  class: "transient"
  rationale: "A read; the custodian keeps nothing."
errorCodes: []
related:
  - "external/accounts/get"
  - "external/accounts/create"
  - "external/credentials/issue"
---

## Abstract

A key custodian can hold **external accounts**: identities at third parties — a cloud provider, an object store, a blockchain, a SaaS API — that integrations bound to them use without ever holding the account's key or secret. The **External Accounts — List** Trust Task pages through the accounts in a context.

It returns what a console needs to show an account and what an administrator needs to configure its provider: the settings, the public key or certificate, the bindings, the state, the hosts it connects to and the last probe. It never returns a private key, a static secret, or a credential issued to anyone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** answer a caller holding `external-accounts-manage` in a context with every account in that context, restricted by `state` and `model` when present.
2. **MUST** answer a caller holding only `external-auth-use` with just the accounts on which it is the consumer of a binding, and in each of those **MUST** omit every binding but its own.
3. **MUST** refuse any other caller with `permissionDenied`.
4. **MUST** omit `archived` accounts unless `state` asks for them.
5. **MUST** return at most `limit` accounts per page, with `nextCursor` present exactly when more remain.
6. **MUST NOT** return any private key, static secret or issued credential.

## Definitions

- **External account**, **binding**, **custodian** — as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json) and its [conventions](../../../_shared/0.1/CONVENTIONS.md) define them.
- **Manager** — a caller holding `external-accounts-manage` in the context. **Bound integration** — a caller holding `external-auth-use` there and named as a binding's consumer.

## Request

A manager or a bound integration (`issuer`) asks the custodian (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Every active account in a context

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/external/accounts/list/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000101",
  "payload": {
    "context": "community",
    "state": "active",
    "limit": 50
  }
}
```

## Response

The custodian answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/external/accounts/list/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000101",
  "payload": {
    "accounts": [
      {
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
    ]
  }
}
```

## Security & Privacy

### Data carried

A context and filters in; account records out. No task in this family returns a private key, a static secret or an issued credential in a read ([conventions §1](../../../../external/_shared/0.1/CONVENTIONS.md#1-five-rules) rule 1), so this response is safe to log and to show in a console.

### Correlation

Account settings name provider resources — an AWS account number, a GCP project, a bucket — and bindings name the integrations allowed to use them. Together they map a community's infrastructure. That is why a bound integration sees only its own binding, and why the read is restricted to managers.

### Retention

Nothing is stored by the read.

### Consent/purpose

Reading an account confers no use of it. The purpose is administering accounts, or, for a bound integration, discovering what it may ask for.
