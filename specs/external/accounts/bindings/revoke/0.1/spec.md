---
slug: "external/accounts/bindings/revoke"
version: "0.1"
title: "External Accounts — Bindings — Revoke"
summary: "A manager stops one integration using an external account, at once and alone; credentials already issued lapse at their expiry."
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
  rationale: "Removes a binding. Takes effect for the next request; nothing already issued is recalled."
exposure:
  discloses: "metadata"
  ingests: "metadata"
  actsAsSubject: false
  rationale: "A consumer DID in; the account out."
retention:
  class: "durable"
  rationale: "The removal persists; the audit trail keeps the binding that was removed."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external/accounts/bindings/revoke:noSuchBinding"
    meaning: "The account has no binding for this consumer."
    retryable: false
related:
  - "external/accounts/bindings/grant"
  - "external/accounts/suspend"
---

## Abstract

The **External Accounts — Bindings — Revoke** Trust Task removes one integration's binding. It is deliberately **not** consent-gated: taking authority away has to be fast and possible for one person alone ([conventions §4](../../../../_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)).

A revocation cannot recall a credential the integration already holds; that credential lapses at its expiry, which the binding capped at `maxTtlSeconds`. That cap is why it should be minutes. To stop every integration at once, suspend the account.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** refuse with `external/accounts/bindings/revoke:noSuchBinding` a consumer with no binding.
2. **MUST** refuse every issuance and signature to the consumer from the moment the revocation is stored, including one in flight that has not yet been signed.
3. **MUST NOT** require consent for this task.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

This task **MUST NOT** be consent-gated: withdrawal is the safe direction, and a withdrawal that waits for a quorum is one an attacker using the binding can outlast.

## Definitions

- **AccountBinding**, **CredentialScopeCeiling** — as [`external/_shared/0.1/accounts.schema.json`](../../../../_shared/0.1/accounts.schema.json) defines them.

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### Cut off a compromised host

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000013",
  "type": "https://trusttasks.org/spec/external/accounts/bindings/revoke/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000107",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "consumer": "did:example:community",
    "reason": "Host compromised; rotating."
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`.

### Revoked

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000014",
  "type": "https://trusttasks.org/spec/external/accounts/bindings/revoke/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000107",
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
      "bindings": [],
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

A consumer DID and a reason in; the account out.

### Correlation

A binding links an integration DID to a provider account.

### Retention

The binding is gone; the audit row keeps it and the reason.

### Consent/purpose

None is needed to take authority away.
