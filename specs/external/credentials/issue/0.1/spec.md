---
slug: "external/credentials/issue"
version: "0.1"
title: "External Credentials — Issue"
summary: "A bound integration obtains a short-lived provider credential, downscoped to one prefix or object, from an external account a key custodian holds; the custodian performs the provider exchange itself and returns the credential sealed to the caller."
status: "draft"
targetFrameworkVersion: "0.6.0"
category: "key-management"
keywords:
  - "cloud"
  - "federation"
  - "credentials"
  - "downscoping"
  - "aws"
  - "gcp"
  - "azure"
  - "s3"
  - "presigned-url"
  - "workload-identity"
parties:
  - role: "Bound integration"
    requirement: "REQUIRED"
    member: "issuer"
    identifierScope: "any"
  - role: "Key custodian"
    requirement: "REQUIRED"
    member: "recipient"
proofRequirement:
  requirement: "REQUIRED"
  rationale: "The binding is checked against the request's proven signer, never a header or a payload claim: the proof is what makes \"the caller is the binding's consumer\" true on every transport. The response carries a secret, sealed, and its proof is what lets the consumer attribute the credential to the custodian whatever relayed it (SPEC §7.3 item 8)."
issuedAtRequirement:
  requirement: "REQUIRED"
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor. A replayed request would obtain another credential acting as the community at the provider."
sideEffects:
  level: "mutating"
  rationale: "Creates a session at the provider (or presigns a URL), consumes the binding's rate, and writes an audit row. Nothing about the account changes."
exposure:
  discloses: "secret"
  ingests: "metadata"
  actsAsSubject: true
  rationale: "The custodian authenticates to the provider as the account and returns a usable credential — a secret for its lifetime — sealed to the caller's key-agreement key. The request carries an account reference and a scope."
retention:
  class: "transient"
  rationale: "The credential is the caller's to hold in memory until it expires; the custodian keeps only an audit row, never the credential."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:notActive"
    meaning: "The account is `suspended` or `archived` and cannot be used."
    retryable: false
  - code: "external:notBound"
    meaning: "The caller is not the consumer of any binding on this account."
    retryable: false
  - code: "external/credentials/issue:scopeOutsideCeiling"
    meaning: "The requested scope is not inside the binding's ceiling, or is not a scope the account's model can express (no prefix for a storage model, more than one action for a presigned URL)."
    retryable: false
  - code: "external/credentials/issue:ttlTooLong"
    meaning: "`ttlSeconds` exceeds the binding's `maxTtlSeconds`. `details.maxTtlSeconds` states it."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["maxTtlSeconds"], "properties": {"maxTtlSeconds": {"type": "integer", "minimum": 60}}}
  - code: "external:rateLimited"
    meaning: "The binding's rate is exhausted. `details.retryAfterSeconds` says when to try again."
    retryable: true
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["retryAfterSeconds"], "properties": {"retryAfterSeconds": {"type": "integer", "minimum": 1}}}
  - code: "external:providerSetupRequired"
    meaning: "The account cannot be used until its provider-side setup is redone: after a restore that could not carry its key or secret, or while a rotation awaits confirmation."
    retryable: false
  - code: "external:providerRefused"
    meaning: "The provider refused the custodian's request. `details.providerError` carries the provider's words, verbatim, truncated, with any credential removed."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["providerError"], "properties": {"providerError": {"type": "string", "maxLength": 2048}, "providerRequestId": {"type": "string", "maxLength": 256}}}
  - code: "external:providerUnavailable"
    meaning: "The provider could not be reached or did not answer in time."
    retryable: true
  - code: "external/credentials/issue:notBrokered"
    meaning: "The account is `sui-signer`, which issues no credentials; use external/sign."
    retryable: false
  - code: "external/credentials/issue:noKeyAgreement"
    meaning: "The caller's DID offers no X25519 key-agreement key to seal the credential to: it is not a `did:key`, and its resolved DID document has no X25519 `keyAgreement` verification method."
    retryable: false
related:
  - "external/sign"
  - "external/accounts/bindings/grant"
  - "external/accounts/get"
---

## Abstract

Integrations need to reach cloud storage and third-party APIs, and the usual answer is a long-lived secret pasted into their configuration — copied, logged, backed up, and usable by everyone who ever saw it. The **External Credentials — Issue** Trust Task replaces that with a credential that lives for minutes and does one thing.

A **bound integration** names an account, a scope and a lifetime. The custodian:

1. checks the binding ([Conformance](#conformance), in order);
2. **performs the provider exchange itself** — signs a Roles Anywhere CreateSession, an ID token for Google's STS, a client assertion for Entra or another token endpoint; or presigns one S3 request;
3. **downscopes** the result to the request: an inline session policy on one bucket prefix, a GCS Credential Access Boundary, the single object a presigned URL names;
4. returns it **sealed to the caller's key-agreement key**, inside a response the custodian signs.

The consumer never sees a signed assertion. If the custodian returned one for the consumer to exchange, the consumer would choose the session policy, and the narrowest layer of least privilege would be its promise rather than the custodian's enforcement ([conventions §1](../../../_shared/0.1/CONVENTIONS.md#1-five-rules)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`) checks, **in this order**, before anything is signed:

1. **The caller is a binding's consumer on the named account** — the DID the request's proof, or the transport's sender authentication, established; never a DID named in the payload or a header. Every caller that is not, **including every caller naming an account that does not exist**, gets the same `external:notFound`, so the task is no oracle for which accounts exist or what state they are in.
2. The caller holds `external-auth-use` in the account's context, through its entry's act scope. Else `permissionDenied`.
3. Only now, to a bound consumer: the account is `active` (else `external:notActive`); its model is brokered (else `external/credentials/issue:notBrokered`); its provider setup is complete (else `external:providerSetupRequired`).
4. **The scope is inside the binding's ceiling**: `prefix` begins with one of `scopeCeiling.prefixes`, every action is in `scopeCeiling.actions`, every OAuth scope in `scopeCeiling.scopes`; for `s3-static-presign`, exactly one action and an `objectKey`. Else `external/credentials/issue:scopeOutsideCeiling`.
5. `ttlSeconds` does not exceed the binding's `maxTtlSeconds`. Else `external/credentials/issue:ttlTooLong`.
6. The binding's rate is not exhausted. Else `external:rateLimited`.

Then it:

7. **MUST** build every provider policy from validated values with the provider language's own encoder — the IAM policy as JSON, the access boundary condition through a CEL builder — and **MUST NOT** interpolate a caller string into one.
8. **MUST** downscope to exactly the requested scope: for AWS, `s3:PutObject`/`GetObject`/`DeleteObject` as requested on `arn:aws:s3:::<bucket>/<prefix>*` only, plus `aws:SourceIp` when the binding names `sourceCidrs`; for GCS, a Credential Access Boundary on `resource.name.startsWith('projects/_/buckets/<bucket>/objects/<prefix>')`; for a presigned URL, one method on one key.
9. **MUST** request a provider lifetime no longer than `ttlSeconds`, and report the provider's actual expiry as `expiresAt`.
10. **MUST** return the credential only inside a sealed-transfer bundle whose plaintext is an `ExternalCredentialPayload`, sealed to the caller's **key-agreement key**: for a `did:key`, the X25519 derivation of its Ed25519 key; otherwise the first X25519 `keyAgreement` verification method of the caller's resolved DID document. With neither, **MUST** refuse with `external/credentials/issue:noKeyAgreement` before contacting the provider. **MUST NOT** log, store or audit the credential.
11. **MUST** use the `PinnedOnly` producer assertion for the bundle, and **MUST** sign the response document (its proof is REQUIRED): the response proof covers `sealedCredential` and so the bundle's digest, and it is the anchor the consumer trusts the bundle by.
12. **SHOULD** audit the issuance (account, binding, consumer, scope, expiry, provider request id), and **SHOULD** raise a security alert on a refusal at step 1 for a caller holding `external-auth-use`, or at step 4.

A **consumer** **MUST** verify the response's proof against the custodian's DID before opening the bundle, **MUST** keep issued credentials in memory only, re-issue before expiry, and drop them on shutdown.

### The sealed credential

The plaintext of the sealed bundle is an `ExternalCredentialPayload` ([`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json)), discriminated by `kind`:

- **`awsSession`** — AWS session credentials (`accessKeyId`, `secretAccessKey`, `sessionToken`, `expiration`, optional `region`), from `aws-roles-anywhere`;
- **`bearerToken`** — an access token (`token`, `tokenType`, `expiresAt`, optional `scope`), from `gcp-wif-pinned`, `azure-cert` or `oauth2-private-key-jwt`;
- **`presignedRequest`** — one request (`method`, `url`, optional `headers`, `expiresAt`), from `s3-static-presign`;

each with the provider's request id when there is one. Producer assertion and sealing key: [conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-sealed-payloads).

### Retries

Issuance is idempotent in effect: a second credential for the same scope is harmless and expires. A custodian **MAY** answer a duplicate document ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11) with its original response; a consumer that receives `external:providerUnavailable` retries with backoff.

## Authorization

The authority this task presupposes is twofold and both parts are required: **`external-auth-use`** in the account's context, held by the caller's access-control entry at the custodian and read at execution time; and **a binding on the account naming the caller as consumer**, which also fixes how far the caller may go. The capability says the caller may consume accounts in the context; the binding says which account, how far, how long and how often. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Issuance is not consent-gated: the consent that governs it was given when the binding was granted. An agent host approving tool calls (an MCP host, for instance) **SHOULD** treat this task as needing approval per call, never once for all, since a standing approval lets an agent mint provider credentials indefinitely.

## Definitions

- **AccountBinding**, **CredentialScope**, **SealedTransferBundle**, **ExternalCredentialPayload** — as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json).
- **Key-agreement key** — the X25519 key a credential is sealed to; see Conformance item 10.
- **Brokered model** — every model but `sui-signer`: `aws-roles-anywhere`, `gcp-wif-pinned`, `azure-cert`, `oauth2-private-key-jwt`, `s3-static-presign`, `static-secret`.

## Request

A bound integration (`issuer`) asks the custodian (`recipient`).

### Storage credentials for one room's prefix

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000070",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000170",
  "payload": {
    "context": "community",
    "account": "eu-s3-primary",
    "scope": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "actions": [
        "put",
        "get",
        "delete"
      ]
    },
    "ttlSeconds": 900
  }
}
```

### A presigned URL to read one object

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000071",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000171",
  "payload": {
    "context": "community",
    "account": "r2-rooms",
    "scope": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "actions": [
        "get"
      ],
      "objectKey": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "ttlSeconds": 300
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the sealed credential, and its expiry in the clear so the consumer can schedule renewal without opening it. A refusal is a `trust-task-error`.

### Issued

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000072",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000170",
  "payload": {
    "sealedCredential": "-----BEGIN VTA SEALED BUNDLE-----\nBundle-Id: 2b7e…\nDigest-Algo: sha-256\nChunk: 0/1\n\nRXh0ZXJuYWxDcmVkZW50aWFsQ2lwaGVydGV4dEV4dGVybmFsQ3JlZGVudGlhbA\n=Q1JD\n-----END VTA SEALED BUNDLE-----",
    "expiresAt": "2026-10-10T10:15:00Z",
    "scope": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "actions": [
        "put",
        "get",
        "delete"
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

An account reference, a scope and a lifetime in. A credential out, sealed to the caller's key-agreement key: whatever transport carried the document, and whatever relayed, terminated or logged it, the credential is readable only by the caller. A caller with no binding learns nothing about the account, not even that it exists.

### Correlation

The custodian and the provider learn which prefix the integration works in and when. Prefixes are chosen by the consumer to say nothing about their contents (a digest, not a name).

### Retention

The custodian keeps an audit row and never the credential. The consumer holds the credential in memory until it expires.

### Consent/purpose

Use is authorized by the binding, which was consented when granted. Each issuance is narrowed to the one purpose it names.
