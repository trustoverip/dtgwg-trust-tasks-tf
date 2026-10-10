---
slug: "external/accounts/keys/rotate"
version: "0.1"
title: "External Accounts — Keys — Rotate"
summary: "A manager rotates an external account's key in three steps — stage a successor beside the current key, confirm once the provider trusts it and a probe succeeds, or abandon — so the account never stops working."
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
  rationale: "`stage` creates a successor key; `confirm` retires the current key and makes the successor current; `abandon` discards the successor."
exposure:
  discloses: "metadata"
  ingests: "none"
  actsAsSubject: false
  rationale: "Discloses the account, with the successor's public material, and the regenerated setup. No private key."
retention:
  class: "durable"
  rationale: "The rotation state persists on the account until confirmed or abandoned."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:notActive"
    meaning: "The account is `suspended` or `archived` and cannot be used."
    retryable: false
  - code: "external/accounts/keys/rotate:notKeyModel"
    meaning: "The account holds a secret, not a key; replace it with external/accounts/secret/set."
    retryable: false
  - code: "external/accounts/keys/rotate:rotationPending"
    meaning: "`stage` was asked while a successor is already staged. Confirm or abandon it first."
    retryable: false
  - code: "external/accounts/keys/rotate:noRotationPending"
    meaning: "`confirm` or `abandon` was asked with no successor staged."
    retryable: false
  - code: "external/accounts/keys/rotate:probeFailed"
    meaning: "`confirm` ran a probe with the successor key and it failed; the provider does not trust it yet. `details` is the probe report."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["report"], "properties": {"report": {"type": "object"}}}
related:
  - "external/accounts/setup"
  - "external/accounts/probe"
---

## Abstract

A pinned model cannot switch keys in one step: the provider holds the old public material until an administrator uploads the new. The **External Accounts — Keys — Rotate** Trust Task therefore rotates in phases:

1. **`stage`** — the custodian creates a successor key and returns the account (with `pendingKeyFingerprint`) and the regenerated setup, which carries **both** keys: both in the uploaded JWKS for `gcp-wif-pinned`, a second end-entity certificate under the same anchor for `aws-roles-anywhere`, a second certificate for `azure-cert`.
2. The administrator applies that setup at the provider.
3. **`confirm`** — the custodian probes with the successor; only if every step succeeds does it make the successor current and retire the old key.
4. **`abandon`** — discards a successor that is not wanted.

After a restore that could not carry a wrapped RSA key, `stage` then `confirm` is also how an account is made usable again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** refuse a static model with `external/accounts/keys/rotate:notKeyModel`.
2. On `stage`: **MUST** refuse with `rotationPending` if a successor exists; otherwise **MUST** generate one as `external/accounts/create` would, keep using the current key for issuance, and answer with the account and the regenerated setup.
3. On `confirm`: **MUST** refuse with `noRotationPending` if none is staged; **MUST** run a probe with the successor and refuse with `probeFailed` unless it succeeds; then **MUST** make the successor current, retire the old key so that it is never used or derived again for any account, and for `aws-roles-anywhere` **SHOULD** add the old end-entity certificate to the CRL that setup returns.
4. On `abandon`: **MUST** discard the successor.
5. For `sui-signer`, **MUST** refuse `stage`: a new key is a new address holding no funds, which is a new account, not a rotation.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

**Consent.** A custodian **SHOULD** subject this task to consent under its approvals policy ([conventions §4](../../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)): the request is held, and approvers — the context's administrators other than the requester — each decide with a `task-consent/decision` signed by their own DID. A decision counts for the DID that signed it, never for a relayer that delivered it. Without consent, one administrator, or one compromised console relaying for administrators, can replace the community's key at a provider with one created at their request. A custodian **SHOULD** gate `stage`; `confirm` is guarded by its probe, and `abandon` reduces authority.

## Definitions

- **Successor key** — the staged key, shown as `publicMaterial.pendingKeyFingerprint` until confirmed.

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### Stage a successor

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000019",
  "type": "https://trusttasks.org/spec/external/accounts/keys/rotate/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000110",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "phase": "stage"
  }
}
```

### Confirm after uploading it

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000020",
  "type": "https://trusttasks.org/spec/external/accounts/keys/rotate/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000111",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "phase": "confirm"
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the account, and on `stage` the setup to apply.

### Staged

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000021",
  "type": "https://trusttasks.org/spec/external/accounts/keys/rotate/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000110",
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
        "certificatePem": "-----BEGIN CERTIFICATE-----\nMIIBszCCAVmgAwIBAgIUQ29tbXVuaXR5IENBIGV4YW1wbGUwCgYIKoZIzj0EAwIw\n-----END CERTIFICATE-----\n",
        "pendingKeyFingerprint": "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc"
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
      "updatedAt": "2026-10-10T09:00:00Z"
    },
    "setup": {
      "model": "aws-roles-anywhere",
      "artifacts": [
        {
          "name": "trust-anchor.pem",
          "mediaType": "application/x-pem-file",
          "content": "-----BEGIN CERTIFICATE-----\nMIIBszCCAVmgAwIBAgIUQ29tbXVuaXR5IENBIGV4YW1wbGUwCgYIKoZIzj0EAwIw\n-----END CERTIFICATE-----\n"
        },
        {
          "name": "trust-policy.json",
          "mediaType": "application/json",
          "content": "{\"Version\":\"2012-10-17\",\"Statement\":[{\"Effect\":\"Allow\",\"Principal\":{\"Service\":\"rolesanywhere.amazonaws.com\"},\"Action\":[\"sts:AssumeRole\",\"sts:TagSession\",\"sts:SetSourceIdentity\"],\"Condition\":{\"StringEquals\":{\"aws:PrincipalTag/x509Subject/CN\":\"eu-s3-primary\"},\"ArnEquals\":{\"aws:SourceArn\":\"arn:aws:rolesanywhere:eu-west-1:123456789012:trust-anchor/0b2f7c9e-6a51-4d3e-9b1a-3c5d7e9f1a2b\"}}}]}"
        },
        {
          "name": "permissions-policy.json",
          "mediaType": "application/json",
          "content": "{\"Version\":\"2012-10-17\",\"Statement\":[{\"Effect\":\"Allow\",\"Action\":[\"s3:PutObject\",\"s3:GetObject\",\"s3:DeleteObject\"],\"Resource\":\"arn:aws:s3:::community-rooms-eu/rooms/*\"}]}"
        }
      ],
      "steps": [
        {
          "description": "Create the trust anchor from the custodian's CA certificate.",
          "command": "aws rolesanywhere create-trust-anchor --name community-eu-s3-primary --source sourceType=CERTIFICATE_BUNDLE,sourceData={x509CertificateData=file://trust-anchor.pem} --enabled"
        },
        {
          "description": "Create the role, trusting only certificates for this account under that anchor.",
          "command": "aws iam create-role --role-name community-rooms-storage --assume-role-policy-document file://trust-policy.json"
        },
        {
          "description": "Allow the role only object reads, writes and deletes under rooms/ in the bucket.",
          "command": "aws iam put-role-policy --role-name community-rooms-storage --policy-name rooms-objects --policy-document file://permissions-policy.json"
        },
        {
          "description": "Create the profile that maps the anchor to the role.",
          "command": "aws rolesanywhere create-profile --name community-eu-s3-primary --role-arns arn:aws:iam::123456789012:role/community-rooms-storage --enabled"
        }
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

A phase in; the account and, on `stage`, public setup material out.

### Correlation

None beyond the account itself.

### Retention

A staged successor persists until confirmed or abandoned. A retired key is never used again.

### Consent/purpose

Rotation limits how long a compromised key stays useful; consent keeps a rotation from being a quiet substitution.
