---
slug: "external/accounts/create"
version: "0.1"
title: "External Accounts — Create"
summary: "A manager creates an external account; the key custodian generates its key itself and returns the public material and the provider-side setup to perform."
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
  rationale: "Creating an account mints a key the provider will trust as an identity of this community, and consent is counted against the request's own digest. Both need the request to be attributable whatever transport carried it."
issuedAtRequirement:
  requirement: "REQUIRED"
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor. An approval is bound to the request, and a request that could be replayed outside a window could be presented again after its approvers had moved on."
sideEffects:
  level: "mutating"
  rationale: "Stores an account record and creates its key in the custodian. The provider is not touched: the account is unusable until an administrator performs the setup it returns."
exposure:
  discloses: "metadata"
  ingests: "metadata"
  actsAsSubject: false
  rationale: "Ingests provider configuration (account numbers, role names, bucket names). Discloses the account record, including its public key or certificate. No secret crosses in either direction: static models take theirs later, through external/accounts/secret/set."
retention:
  class: "durable"
  rationale: "The account persists until deleted; it is an authority the community relies on."
errorCodes:
  - code: "external:notFound"
    meaning: "The named context does not exist, or the caller has no standing in it. Conflated so that a context's existence is not confirmed to a stranger. See the family conventions §2."
    retryable: false
  - code: "external:alreadyExists"
    meaning: "An account in the context already carries this id."
    retryable: false
  - code: "external:invalidSettings"
    meaning: "The settings validate against the schema but are unusable: a model change, an inconsistent value, or a driver or network this custodian does not implement. `details.member` names the offending member."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["member"], "properties": {"member": {"type": "string", "maxLength": 128}}}
  - code: "external/accounts/create:modelUnsupported"
    meaning: "The custodian does not implement `settings.model`. A model it implements but cannot use as configured is `external:invalidSettings` instead."
    retryable: false
related:
  - "external/accounts/setup"
  - "external/accounts/probe"
  - "external/accounts/bindings/grant"
  - "external/accounts/secret/set"
  - "task-consent/request"
---

## Abstract

The **External Accounts — Create** Trust Task creates an external account in a context: a name, a provider model, and the provider-side identifiers the model needs. The custodian **generates the account's key itself** — a caller never supplies one — and returns the account with its public material.

A new account can do nothing yet. It has no bindings, so no integration may use it, and its provider has not been told to trust it. The next steps are [`external/accounts/setup`](../../setup/0.1/spec.md) (what to configure at the provider), [`external/accounts/probe`](../../probe/0.1/spec.md) (prove it works), and [`external/accounts/bindings/grant`](../../bindings/grant/0.1/spec.md) (who may use it).

Every model pins the custodian's verification material at the provider, so creating an account never requires the custodian to be reachable from the internet ([conventions §3](../../../_shared/0.1/CONVENTIONS.md#3-network-exposure)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** refuse with `external:notFound` a `context` it does not have; with `external/accounts/create:modelUnsupported` a model it does not implement; with `external:alreadyExists` an id already used in the context; and with `external:invalidSettings` settings it cannot use (a driver or network it does not implement, a missing `bucket` it will later need), naming the member. Settings that do not validate against the payload schema never reach this check: they are refused with `malformedRequest` before the custodian reads them. `external:invalidSettings` is for settings the schema accepts that the custodian cannot use.
2. **MUST** generate the account's key itself: a P-256 key derived in the context's key space at a path it records on the account and never takes from the caller, or, where the model or provider needs RSA, a 3072-bit key generated and stored wrapped. For `aws-roles-anywhere` it **MUST** also ensure the context's certificate authority exists and certify the account's key with a short-lived end-entity certificate. Static models get no key.
3. **MUST** derive `egressHosts` from the settings — the hosts issuance exchanges with and the hosts a probe dials — and **MUST NOT** later connect anywhere else on the account's behalf.
4. **MUST** create the account `active`, with no bindings, and **SHOULD** set `providerSetupRequired` until a complete probe succeeds.
5. **MUST NOT** contact the provider.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

**Consent.** A custodian **SHOULD** subject this task to consent under its approvals policy ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)): the request is held, and approvers — the context's administrators other than the requester — each decide with a `task-consent/decision` signed by their own DID. A decision counts for the DID that signed it, never for a relayer that delivered it. Without consent, one administrator, or one compromised console relaying for administrators, can give the community a new identity at a provider, under settings nobody else has seen.

## Definitions

- Shapes as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json). The model is `settings.model`; there is no separate member to disagree with it.

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### An AWS account through IAM Roles Anywhere

The trust anchor and profile do not exist yet when the account is created; the request names the ARNs they will have, from the setup the administrator is about to run.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/external/accounts/create/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000103",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "label": "EU S3 — main account",
    "settings": {
      "model": "aws-roles-anywhere",
      "region": "eu-west-1",
      "trustAnchorArn": "arn:aws:rolesanywhere:eu-west-1:123456789012:trust-anchor/0b2f7c9e-6a51-4d3e-9b1a-3c5d7e9f1a2b",
      "profileArn": "arn:aws:rolesanywhere:eu-west-1:123456789012:profile/5e8a1c3d-2b4f-4a6e-8c0d-1f3a5b7c9e2d",
      "roleArn": "arn:aws:iam::123456789012:role/community-rooms-storage",
      "bucket": "community-rooms-eu"
    }
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the account as created. A refusal is a `trust-task-error`; a request held for consent is answered as the consent ceremony defines.

### Created, awaiting provider setup

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/external/accounts/create/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000103",
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
      "providerSetupRequired": true,
      "createdAt": "2026-10-10T08:00:00Z",
      "updatedAt": "2026-10-10T08:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

Provider identifiers in; the account and its public material out. No key moves: the custodian generates it ([conventions §5](../../../_shared/0.1/CONVENTIONS.md#5-keys-and-backup)).

### Correlation

Settings name provider accounts and resources, which links this community to them for anyone who can read the account.

### Retention

The account is durable. Its record is in the custodian's backups; a wrapped RSA key is not.

### Consent/purpose

The purpose is to let named integrations use a provider without holding its credentials. Consent from other administrators is what keeps that authority from being one person's decision.
