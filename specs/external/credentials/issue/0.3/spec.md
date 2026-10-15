---
slug: "external/credentials/issue"
version: "0.3"
title: "External Credentials — Issue"
summary: "A bound integration obtains a short-lived provider credential from an external account: a session or token downscoped to one prefix, or up to 64 presigned requests on one object, each write bound to its length and SHA-256, multipart included, sealed to the caller."
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
  - "multipart-upload"
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
  rationale: "Creates a session at the provider (or presigns requests), consumes the binding's rate, and writes an audit row. Nothing about the account changes."
exposure:
  discloses: "secret"
  ingests: "metadata"
  actsAsSubject: true
  rationale: "The custodian authenticates to the provider as the account and returns a usable credential — a secret for its lifetime — sealed to the caller's key-agreement key. The request carries an account reference and a scope, or one object's name with the lengths and checksums of the bytes to be written to it."
retention:
  class: "transient"
  rationale: "The credential is the caller's to hold in memory until it expires; the custodian keeps only an audit row, never the credential."
maxDocumentBytes:
  request: 65536
  response: 294912
  rationale: "A request's largest member is a batch of 64 operations on one object, each a few dozen bytes, with the object's name and upload id once: well inside 64 KiB. The response's size is set by `sealedCredential`, which is bounded at 256 KiB, not by the batch: a sealed-transfer bundle's overhead depends on how its cleartext is encoded and can be several times the cleartext, and a long object key or upload id lengthens every URL in the set, so even 64 requests can need more. A custodian refuses such a batch with `responseTooLarge` rather than answer over the bound. 288 KiB holds a bundle at that bound, the batch echoed beside it (at most about 12 KiB), the envelope and the proof. That exceeds the 64 KiB a party applies to a Trust Task by default, which is why the bound is declared; a `scope` response stays far below it."
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
  - code: "external/credentials/issue:requestShapeMismatch"
    meaning: "The request uses the shape the account's model does not take: `presign` for a model that issues sessions or tokens, which can bind neither an operation, a length nor a checksum into what it issues; or `scope` for `s3-static-presign`, which in this version issues only presigned requests. `details.expected` names the shape the model takes."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["expected"], "properties": {"expected": {"type": "string", "enum": ["scope", "presign"]}}}
  - code: "external/credentials/issue:invalidBatch"
    meaning: "The presign batch breaks a rule its schema cannot state: an `uploadId` missing for an operation that needs one, present beside `create-multipart`, two identical operations, or two `upload-part` operations naming the same part."
    retryable: false
  - code: "external/credentials/issue:scopeOutsideCeiling"
    meaning: "The requested scope is not inside the binding's ceiling: a prefix no ceiling prefix begins, an action the ceiling lacks, or — for `presign` — an operation whose required ceiling action the ceiling lacks."
    retryable: false
  - code: "external/credentials/issue:ttlTooLong"
    meaning: "`ttlSeconds` exceeds the binding's `maxTtlSeconds`. `details.maxTtlSeconds` states it."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["maxTtlSeconds"], "properties": {"maxTtlSeconds": {"type": "integer", "minimum": 60}}}
  - code: "external/credentials/issue:responseTooLarge"
    meaning: "The presign batch is valid, but its sealed answer would exceed the bound on `sealedCredential` (262144 characters). The custodian refuses it rather than truncate the set or answer over the bound, and it does not count against the binding's rate. `details.maxOperations`, when the custodian can compute it, is how many of the batch's operations, taken in order, would fit. The caller splits the batch and asks again."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"maxOperations": {"type": "integer", "minimum": 1, "maximum": 63}}}
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
  - "rooms/blobs/upload/begin"
  - "rooms/blobs/upload/commit"
---

## Abstract

Integrations need to reach cloud storage and third-party APIs, and the usual answer is a long-lived secret pasted into their configuration — copied, logged, backed up, and usable by everyone who ever saw it. The **External Credentials — Issue** Trust Task replaces that with a credential that lives for minutes and does one thing.

A **bound integration** names an account, a lifetime, and either a **scope** or a **presign batch**. The custodian:

1. checks the binding ([Conformance](#conformance), in order);
2. **performs the provider exchange itself** — signs a Roles Anywhere CreateSession, an ID token for Google's STS, a client assertion for Entra or another token endpoint; or presigns S3 requests;
3. **downscopes** the result to the request: an inline session policy on one bucket prefix, a GCS Credential Access Boundary, or — for presigned requests — one method on one object, and for a write the exact length and SHA-256 of the bytes, signed into the request so that storage itself refuses any others;
4. returns it **sealed to the caller's key-agreement key**, inside a response the custodian signs.

The consumer never sees a signed assertion. If the custodian returned one for the consumer to exchange, the consumer would choose the session policy, and the narrowest layer of least privilege would be its promise rather than the custodian's enforcement ([conventions §1](../../../_shared/0.1/CONVENTIONS.md#1-five-rules)).

### Direct uploads

A host that lets its users write straight to object storage — the **direct** transfer mode of [`rooms/blobs/upload/begin/0.2`](../../../../rooms/blobs/upload/begin/0.2/spec.md) — holds no storage credential of its own: it asks the custodian for every request its users and it will make. That needs more than one GET or PUT on one object. The host must hand out writes that storage enforces byte for byte, write a large object in parts, check what storage holds before it records anything ([`rooms/blobs/upload/commit/0.2`](../../../../rooms/blobs/upload/commit/0.2/spec.md) item 3), and do it for many parts at once — up to 64 operations in one issuance, so a begin answer with more part targets than that takes more than one. Since `0.2`, this task gives `s3-static-presign` that vocabulary:

| Operation | S3 request | Bound into the signature | Ceiling action |
|---|---|---|---|
| `put` | `PUT` PutObject | `Content-Length`, `x-amz-checksum-sha256` | `put` |
| `create-multipart` | `POST ?uploads` CreateMultipartUpload | `x-amz-checksum-algorithm: SHA256` | `put` |
| `upload-part` | `PUT ?partNumber&uploadId` UploadPart | part number and upload id (query); `Content-Length`, `x-amz-checksum-sha256` | `put` |
| `list-parts` | `GET ?uploadId[&part-number-marker]&max-parts=1000` ListParts | upload id, marker (query) | `put` |
| `complete-multipart` | `POST ?uploadId` CompleteMultipartUpload | upload id (query); `x-amz-mp-object-size` | `put` |
| `get` | `GET` GetObject | — | `get` |
| `head` | `HEAD` HeadObject | `x-amz-checksum-mode: ENABLED` | `get` |
| `delete` | `DELETE` DeleteObject | — | `delete` |
| `abort-multipart` | `DELETE ?uploadId` AbortMultipartUpload | upload id (query) | `delete` |

**The custodian presigns every one of them; it performs none.** CreateMultipartUpload, CompleteMultipartUpload and HeadObject are calls whose answer matters — an upload id, a completed object, a size and a checksum — and the custodian could make them itself. It does not, for three reasons. Presigning is a computation, so issuance keeps needing **no egress** ([conventions §3](../../../_shared/0.1/CONVENTIONS.md#3-network-exposure)) and the custodian never becomes a proxy on the storage path. The secret never leaves the custodian either way, so performing the call would protect nothing a presigned request exposes. And the party that must act on the answer — record the upload id, decide that an object is what was committed to — is the host, which is where [`rooms/blobs/upload/commit/0.2`](../../../../rooms/blobs/upload/commit/0.2/spec.md) puts that duty; a custodian that verified the object on the host's behalf would be asserting a fact it has no standing to act on.

**The binding's ceiling is unchanged.** Its `actions` still name what may happen to an object — `put`, `get`, `delete` — and each operation is authorized by one of them, as the table says. Writing in parts is a way to write, so `put` grants it; reading an object's length and checksum is less than reading it, so `get` grants `head`; aborting an upload destroys bytes storage holds, so it takes `delete`, as removing an object does. A ceiling that named protocol steps would let an operator grant single-request writes without multipart ones, which buys nothing once every write is bound to its length and checksum, and it would change the shape of every account record in the family.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.2

`0.2` let a batch hold up to 128 operations and bounded the sealed answer at 256 KiB, on the estimate that 128 presigned requests were about 128 KiB of cleartext and a third more once armored. That estimate does not hold. A sealed-transfer bundle's overhead depends on how its cleartext is encoded inside the seal, and can be several times the cleartext; a long object key and a long upload id — each may be 1024 characters, and an upload id grows when percent-encoded — lengthen every URL in the set. A batch of 128 could therefore produce an answer the schema forbids, and `0.2` said nothing about what a custodian does then.

- `PresignBatch.requests` holds at most **64** operations, down from 128. The bound on `sealedCredential` is unchanged.
- A new error, **`responseTooLarge`**: a custodian whose sealed answer to a valid batch would exceed that bound refuses the batch — it neither truncates the set nor answers over the bound — before it counts the issuance against the binding's rate ([Conformance](#conformance) item 7). The caller splits the batch and asks again.

The cap is not raised to fit, and the bound is not raised to fit the cap, for three reasons. Presigned requests are bearer authority until they expire — for direct-upload targets, five minutes at most — so presigning more than a consumer can use before they expire only widens what a leak exposes. Sixty-four parts of S3's 5 MiB minimum is 320 MiB in one batch; a larger upload writes larger parts or asks again, and a multipart upload already takes several issuances ([consumer](#conformance) item 4). And a larger answer must cross whatever carries it — a mediator, a DIDComm envelope whose nesting multiplies base64 expansion, a custodian or host with little memory — where every party pays for its size.

Narrowing `maxItems` is a breaking change to a `draft`, and a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). The shared `PresignedRequestSet` keeps its own bound of 128 for `0.2`; this version never produces more than 64.

### What 0.2 changed from 0.1

`0.1` could presign one GET, PUT or DELETE on one object per issuance, with nothing bound into the PUT but the object's name. A direct upload could therefore neither make storage enforce what was written nor use storage's multipart upload, and a host answering one upload with 128 part URLs would spend 128 issuances of its binding's rate on it.

- The payload takes **either** `scope`, as before, **or** `presign`: one object, at most one multipart upload of it, and up to 128 operations from the table above. `s3-static-presign` takes only `presign`; every other brokered model only `scope`. `CredentialScope.objectKey` is no longer used.
- A `put` or `upload-part` **must** carry the exact `contentLength` and `checksumSha256`, and `complete-multipart` the assembled `contentLength`; the custodian signs them into the request.
- The response echoes `presign` in place of `scope` for a batch, and its sealed cleartext is a `PresignedRequestSet` — one presigned request per operation, in order.
- Two errors are new: `requestShapeMismatch` and `invalidBatch`.

The request's shape changes, so this is a breaking change to a `draft` and a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). The binding, the account record and every other task of the family are unchanged: `PresignOperation` and `PresignedRequestSet` are additions to [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`) checks, **in this order**, before it returns anything — and, but for the sealing item 7 needs, before anything is signed:

1. **The caller is a binding's consumer on the named account** — the DID the request's proof, or the transport's sender authentication, established; never a DID named in the payload or a header. Every caller that is not, **including every caller naming an account that does not exist**, gets the same `external:notFound`, so the task is no oracle for which accounts exist or what state they are in.
2. The caller holds `external-auth-use` in the account's context, through its entry's act scope. Else `permissionDenied`.
3. Only now, to a bound consumer: the account is `active` (else `external:notActive`); its model is brokered (else `external/credentials/issue:notBrokered`); its provider setup is complete (else `external:providerSetupRequired`).
4. **The request has the model's shape**: `presign` for `s3-static-presign`, `scope` for every other brokered model. Else `external/credentials/issue:requestShapeMismatch`, with `details.expected`. A batch obeys the rules of `PresignBatch`: `uploadId` present exactly when an operation needs it (`upload-part`, `list-parts`, `complete-multipart`, `abort-multipart`) and absent when the batch holds `create-multipart`; no two operations identical; no two `upload-part` operations naming the same `partNumber`. Else `external/credentials/issue:invalidBatch`.
5. **The scope is inside the binding's ceiling**: the prefix (`scope.prefix` or `presign.prefix`) begins with one of `scopeCeiling.prefixes`; every `scope` action, and for `presign` the ceiling action each operation needs (the table in [Direct uploads](#direct-uploads), and `PresignOperation`), is in `scopeCeiling.actions`; every OAuth scope is in `scopeCeiling.scopes`. Else `external/credentials/issue:scopeOutsideCeiling`.
6. `ttlSeconds` does not exceed the binding's `maxTtlSeconds`. Else `external/credentials/issue:ttlTooLong`.
7. **For `presign`, the sealed answer fits**: the sealed-transfer bundle holding the batch's `PresignedRequestSet` is no longer than the bound on `sealedCredential` (262144 characters). Else `external/credentials/issue:responseTooLarge`, with `details.maxOperations` when the custodian can compute it. A custodian **MUST NOT** truncate the set to make it fit and **MUST NOT** answer over the bound. It learns the size by building the requests and sealing them (items 9 to 13), which contacts no provider; it therefore resolves the caller's key-agreement key first, and a caller with none is refused with `noKeyAgreement` (item 13) before this check. A set built here and then refused, here or at item 8, **MUST** be discarded unreturned, and is subject to the same rule against logging as one returned.
8. The binding's rate is not exhausted. Else `external:rateLimited`. One issuance counts once against `ratePerMinute`, whatever the size of its batch. A request refused at any earlier item, `responseTooLarge` included, **MUST NOT** count against the rate: a caller that splits a refused batch has obtained nothing yet.

Then it:

9. **MUST** build every provider policy and request from validated values with the provider's own encoder — the IAM policy as JSON, the access boundary condition through a CEL builder, a SigV4 canonical request through a SigV4 encoder that percent-encodes the object key, the upload id and every query value — and **MUST NOT** interpolate a caller string into one.
10. **MUST** downscope to exactly the requested scope: for AWS, `s3:PutObject`/`GetObject`/`DeleteObject` as requested on `arn:aws:s3:::<bucket>/<prefix>*` only, plus `aws:SourceIp` when the binding names `sourceCidrs`; for GCS, a Credential Access Boundary on `resource.name.startsWith('projects/_/buckets/<bucket>/objects/<prefix>')`; for `presign`, each request with the one method the table names on the one object `prefix` + `objectKey` of the account's bucket.
11. **MUST**, for `presign`, sign into each request what the table's third column names, so that storage refuses the request with anything else: for `put` and `upload-part`, `Content-Length` equal to `contentLength` and `x-amz-checksum-sha256` equal to `checksumSha256` as signed headers; for `upload-part`, `partNumber` and `uploadId` in the signed query; for `create-multipart`, `x-amz-checksum-algorithm: SHA256` as a signed header, so every part of the upload must carry a SHA-256 checksum; for `complete-multipart`, `uploadId` in the signed query and `x-amz-mp-object-size` equal to `contentLength` as a signed header; for `list-parts`, `uploadId`, `max-parts=1000` and, when given, `part-number-marker` in the signed query; for `abort-multipart`, `uploadId` in the signed query; for `head`, `x-amz-checksum-mode: ENABLED` as a signed header. It **MUST** return every signed header in the request's `headers`, and **MUST NOT** sign a header or query parameter the table does not name.
12. **MUST** request a provider lifetime no longer than `ttlSeconds` — for a presigned request, `X-Amz-Expires` — and report the provider's actual expiry as `expiresAt`: for `presign`, the earliest expiry in the batch.
13. **MUST** return the credential only inside a sealed-transfer bundle whose plaintext is an `ExternalCredentialPayload` for `scope` or a `PresignedRequestSet` for `presign`, holding one request per operation, in the order requested and carrying its index, sealed to the caller's **key-agreement key**: for a `did:key`, the X25519 derivation of its Ed25519 key; otherwise the first X25519 `keyAgreement` verification method of the caller's resolved DID document. With neither, **MUST** refuse with `external/credentials/issue:noKeyAgreement` before contacting the provider or presigning anything. **MUST NOT** log, store or audit the credential, a presigned URL, or anything from which one can be rebuilt.
14. **MUST** use the `PinnedOnly` producer assertion for the bundle, and **MUST** sign the response document (its proof is REQUIRED): the response proof covers `sealedCredential` and so the bundle's digest, and it is the anchor the consumer trusts the bundle by.
15. **SHOULD** audit the issuance (account, binding, consumer, scope or batch — the object, the upload id and each operation with its part number and length, never a URL — expiry, provider request id), and **SHOULD** raise a security alert on a refusal at step 1 for a caller holding `external-auth-use`, or at step 5.

A **consumer**:

1. **MUST** verify the response's proof against the custodian's DID before opening the bundle, **MUST** keep issued credentials and presigned requests in memory only, re-issue before expiry, and drop them on shutdown.
2. **MUST** send each presigned request with exactly the method and the headers it was returned with, and **MUST NOT** add a header storage would treat as signed. `complete-multipart` carries the CompleteMultipartUpload body, which a presigned request does not sign: the consumer names in it only parts it uploaded under the same `uploadId`, with the ETags and checksums storage returned.
3. **MUST NOT** treat a storage provider's acceptance of a signed header as proof that the provider enforces it. An S3-compatible store that ignores `x-amz-checksum-sha256`, `x-amz-checksum-algorithm` or `x-amz-mp-object-size` still accepts the signature, and the custodian cannot see the difference. A consumer that relies on what was written — as [`rooms/blobs/upload/commit/0.2`](../../../../rooms/blobs/upload/commit/0.2/spec.md) item 3 does — **MUST** still verify the object at storage (`list-parts` before completion, `head` after) before it relies on it.
4. **SHOULD**, for a multipart upload, ask in one issuance for `create-multipart` alone, then for each batch of `upload-part` operations, then for the checks and the completion together (`list-parts`, `complete-multipart`, `head`, and `abort-multipart` for the disagreeing case), so that one upload costs a handful of issuances, not one per part.
5. **SHOULD**, on `external/credentials/issue:responseTooLarge`, split the batch — to `details.maxOperations` operations when given, else in half — and ask again for each part. A refused batch obtained nothing and spent none of the binding's rate.

### The sealed credential

The plaintext of the sealed bundle is defined in [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json):

- for `scope`, an `ExternalCredentialPayload`, discriminated by `kind` — **`awsSession`** (from `aws-roles-anywhere`) or **`bearerToken`** (from `gcp-wif-pinned`, `azure-cert` or `oauth2-private-key-jwt`), each with the provider's request id when there is one. Its `presignedRequest` kind is `0.1`'s and is not issued by this version;
- for `presign`, a **`PresignedRequestSet`** (`kind: "presignedRequests"`): one entry per operation — its `index` in the request, `operation`, `method`, `url`, the signed `headers` and `expiresAt` — and the set's `expiresAt`.

Producer assertion and sealing key: [conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-sealed-payloads).

### Retries

Issuance is idempotent in effect: a second credential for the same scope, or a second set of presigned requests for the same batch, is harmless and expires. A custodian **MAY** answer a duplicate document ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11) with its original response; a consumer that receives `external:providerUnavailable` retries with backoff. A presigned `create-multipart` that is used twice creates two uploads; the consumer aborts the one it does not continue.

## Authorization

The authority this task presupposes is twofold and both parts are required: **`external-auth-use`** in the account's context, held by the caller's access-control entry at the custodian and read at execution time; and **a binding on the account naming the caller as consumer**, which also fixes how far the caller may go. The capability says the caller may consume accounts in the context; the binding says which account, how far, how long and how often. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Issuance is not consent-gated: the consent that governs it was given when the binding was granted. An agent host approving tool calls (an MCP host, for instance) **SHOULD** treat this task as needing approval per call, never once for all, since a standing approval lets an agent mint provider credentials indefinitely.

## Definitions

- **AccountBinding**, **CredentialScope**, **SealedTransferBundle**, **ExternalCredentialPayload**, **PresignOperation**, **PresignedRequestSet** — as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json).
- **Presign batch** — `PresignBatch` in [`payload.schema.json`](payload.schema.json): one object, at most one multipart upload of it, and 1 to 64 operations.
- **Signed header** — a header the presigned request's SigV4 signature covers, so that storage refuses the request when it is absent or different.
- **Key-agreement key** — the X25519 key a credential is sealed to; see Conformance item 13.
- **Brokered model** — every model but `sui-signer`: `aws-roles-anywhere`, `gcp-wif-pinned`, `azure-cert`, `oauth2-private-key-jwt`, `s3-static-presign`, `static-secret`.

## Request

A bound integration (`issuer`) asks the custodian (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Storage credentials for one room's prefix

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000270",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.3#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000370",
  "payload": {
    "context": "community",
    "account": "eu-s3-primary",
    "scope": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "actions": ["put", "get", "delete"]
    },
    "ttlSeconds": 900
  }
}
```

### A single-request upload: one write, then the check

A small blob is one part. The host asks for the write it will hand the member, and the `head` it will make at commit, in one issuance.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000271",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.3#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000371",
  "payload": {
    "context": "community",
    "account": "r2-rooms",
    "presign": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "objectKey": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
      "requests": [
        {
          "operation": "put",
          "contentLength": 300000,
          "checksumSha256": "4f3rT8yq1n0pYxXg2j3QmR0b8gq2sR5uJ6vXl0yH7aE="
        },
        { "operation": "head" }
      ]
    },
    "ttlSeconds": 300
  }
}
```

### A multipart upload: the parts

Having created the upload with a `create-multipart` issuance of its own, the host asks for the first parts' writes, each bound to its number, length and checksum. A batch holds at most 64; a begin answer carrying more part targets takes more than one issuance.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000272",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.3#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000372",
  "payload": {
    "context": "community",
    "account": "r2-rooms",
    "presign": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "objectKey": "7a1c2e3d-4b5f-4a6e-8d7c-9b0a1f2e3d4c",
      "uploadId": "VXBsb2FkSWQtZXhhbXBsZS1tdWx0aXBhcnQ",
      "requests": [
        {
          "operation": "upload-part",
          "partNumber": 1,
          "contentLength": 8388608,
          "checksumSha256": "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="
        },
        {
          "operation": "upload-part",
          "partNumber": 2,
          "contentLength": 1611392,
          "checksumSha256": "LCa0a2j/xo/5m0U8HTBBNBNCLXBkg7+g+YpeiGJm564="
        }
      ]
    },
    "ttlSeconds": 300
  }
}
```

### A multipart upload: check and complete

At commit, the host lists the parts storage holds, completes the upload bound to the assembled size, reads back the object's checksum, and holds an abort for the case in which anything disagrees.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000273",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.3#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:20:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000373",
  "payload": {
    "context": "community",
    "account": "r2-rooms",
    "presign": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "objectKey": "7a1c2e3d-4b5f-4a6e-8d7c-9b0a1f2e3d4c",
      "uploadId": "VXBsb2FkSWQtZXhhbXBsZS1tdWx0aXBhcnQ",
      "requests": [
        { "operation": "list-parts" },
        { "operation": "complete-multipart", "contentLength": 10000000 },
        { "operation": "head" },
        { "operation": "abort-multipart" }
      ]
    },
    "ttlSeconds": 300
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the sealed credential, its expiry in the clear so the consumer can schedule renewal without opening it, and the scope or batch granted. A refusal is a `trust-task-error`.

### Presigned

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000274",
  "type": "https://trusttasks.org/spec/external/credentials/issue/0.3#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000371",
  "payload": {
    "sealedCredential": "-----BEGIN VTA SEALED BUNDLE-----\nBundle-Id: 6c1d…\nDigest-Algo: sha-256\nChunk: 0/1\n\nUHJlc2lnbmVkUmVxdWVzdFNldENpcGhlcnRleHRQcmVzaWduZWRSZXF1ZXN0U2V0\n=Q1JD\n-----END VTA SEALED BUNDLE-----",
    "expiresAt": "2026-10-10T10:05:00Z",
    "presign": {
      "prefix": "rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/",
      "objectKey": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
      "requests": [
        {
          "operation": "put",
          "contentLength": 300000,
          "checksumSha256": "4f3rT8yq1n0pYxXg2j3QmR0b8gq2sR5uJ6vXl0yH7aE="
        },
        { "operation": "head" }
      ]
    }
  }
}
```

Opened, the bundle holds a `PresignedRequestSet` like this one (URLs shortened: a real one also carries `X-Amz-Credential` and `X-Amz-Date`):

```json
{
  "kind": "presignedRequests",
  "requests": [
    {
      "index": 0,
      "operation": "put",
      "method": "PUT",
      "url": "https://storage.example/community-rooms/rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Expires=300&X-Amz-SignedHeaders=content-length%3Bhost%3Bx-amz-checksum-sha256&X-Amz-Signature=5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7",
      "headers": {
        "Content-Length": "300000",
        "x-amz-checksum-sha256": "4f3rT8yq1n0pYxXg2j3QmR0b8gq2sR5uJ6vXl0yH7aE="
      },
      "expiresAt": "2026-10-10T10:05:00Z"
    },
    {
      "index": 1,
      "operation": "head",
      "method": "HEAD",
      "url": "https://storage.example/community-rooms/rooms/3f9a1c0e7b2d4a6f8e1c3b5d7a9f0e2c/3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Expires=300&X-Amz-SignedHeaders=host%3Bx-amz-checksum-mode&X-Amz-Signature=5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7",
      "headers": { "x-amz-checksum-mode": "ENABLED" },
      "expiresAt": "2026-10-10T10:05:00Z"
    }
  ],
  "expiresAt": "2026-10-10T10:05:00Z"
}
```

### Refused: a sealed answer that would not fit

A batch of 64 `upload-part` operations on an object whose key and upload id are long enough that the sealed set would exceed 256 KiB. Nothing was returned and nothing counted against the rate; the host asks again for the first 40 parts, then for the rest.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000276",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000376",
  "payload": {
    "code": "external/credentials/issue:responseTooLarge",
    "message": "The sealed answer to this batch would exceed 262144 characters.",
    "retryable": false,
    "details": { "maxOperations": 40 }
  }
}
```

### Refused: a write the binding does not allow

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000275",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000373",
  "payload": {
    "code": "external/credentials/issue:scopeOutsideCeiling",
    "message": "The binding does not allow delete, which abort-multipart needs.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An account reference, a lifetime, and a scope or a batch in. A batch names one object, at most one upload id, and the lengths and SHA-256 checksums of the ciphertext to be written — values that describe bytes and reveal nothing of what they encode. A credential out, sealed to the caller's key-agreement key: whatever transport carried the document, and whatever relayed, terminated or logged it, the credential is readable only by the caller. A caller with no binding learns nothing about the account, not even that it exists.

### What a presigned request discloses

A presigned URL is a **bearer credential** until it expires: whoever holds it may make that one request, and the custodian's checks are behind it. It carries in the clear the store's host, the bucket, the object key, the access key's identifier (not a secret), the signing date, the lifetime and the signature, and for the multipart operations the upload id and part number. Each operation discloses or allows exactly this:

- `put`, `upload-part` — writing the exact bytes committed to (length and checksum signed), into the one object, or the one part of the one upload. A leaked write can only write what the consumer was going to write.
- `create-multipart` — opening an upload of the one object. An upload that is never completed or aborted holds its parts in storage, and is billed, until a bucket lifecycle rule (`AbortIncompleteMultipartUpload`) reclaims it; an administrator **SHOULD** configure one on every bucket an `s3-static-presign` account writes to multipart.
- `complete-multipart` — assembling the upload into the object. The part list it sends is not signed, but every part in it was bound to its length and checksum when written, and the assembled size is signed; storage refuses any other total.
- `list-parts`, `head` — each part's number, length, ETag and checksum, or the object's length, checksum, ETag and metadata. Nothing of the content.
- `get` — the object's bytes; ciphertext, where the consumer seals before it writes.
- `delete`, `abort-multipart` — destroying the object or the upload's parts. This is why both take the ceiling's `delete`.

Because each of these is usable by whoever holds it until it expires, the size of a batch is the size of what one leak exposes. A batch is therefore held to what a consumer can use within the requests' lifetime (see **Batches** below).

**Storage is the enforcer, where it implements what is signed.** Because the length and checksum are in the signature, the custodian need not see the bytes, and the consumer cannot substitute others: storage computes the checksum of what arrives and refuses a mismatch. A store that does not implement a signed header still verifies the signature over it, but does not check the value; the consumer's own verification at storage ([consumer item 3](#conformance)) is what holds then.

**Lifetime.** Every request in a batch lives no longer than `ttlSeconds`, which never exceeds the binding's `maxTtlSeconds` nor one hour. A consumer handing requests on — a host giving its members direct-upload targets — **SHOULD** ask for the shortest lifetime it needs: [`rooms/blobs/upload/begin/0.2`](../../../../rooms/blobs/upload/begin/0.2/spec.md) item 17 caps its targets at 5 minutes, so such a host asks for `ttlSeconds` of 300 or less. Storage checks expiry when a request begins, so a transfer that outlasts it still completes.

**Batches.** A batch is bounded at 64 operations on one object and one upload, and counts once against the binding's rate. The bound is set by use, not by what fits: every request in a batch is bearer authority until it expires — five minutes at most for a direct-upload target — so presigning more than the consumer can use in that time only widens what a leaked answer exposes. Sixty-four parts of S3's 5 MiB minimum is 320 MiB in one batch; a larger upload writes larger parts or asks again. The bound also keeps the answer small enough to cross what carries it: a mediator, a DIDComm envelope whose nesting multiplies base64 expansion, a custodian or host with little memory to spare.

The rate bounds how often a consumer obtains authority; a batch obtains authority over one object only, every write in it bound to bytes the consumer has already committed to, so counting its operations separately would bound nothing more and would make a large upload cost a binding's whole minute.

**A batch whose sealed answer would not fit is refused, not trimmed.** A sealed-transfer bundle's overhead depends on how its cleartext is encoded and can be several times the cleartext, and a long object key or upload id lengthens every request, so 64 requests can exceed the bound on `sealedCredential`. The custodian refuses such a batch with `responseTooLarge` and does not count it against the rate. A custodian that silently dropped requests to fit would leave a consumer holding fewer targets than it asked for and handing its members an upload that cannot complete; one that answered over the bound would be refused, or worse, partly read, by a consumer that holds to the declared size.

### Correlation

The custodian and the provider learn which prefix and object the integration works on and when, and with a batch the sizes of what it writes. Prefixes and object keys are chosen by the consumer to say nothing about their contents (a digest or a random id, not a name). Whoever uses a presigned request shows the storage provider their network address: where a host hands direct-upload targets to its members, the provider sees the member, which [`rooms/blobs/upload/begin/0.2`](../../../../rooms/blobs/upload/begin/0.2/spec.md) weighs for each room tier.

### Retention

The custodian keeps an audit row and never the credential or a URL. The consumer holds them in memory until they expire.

### Consent/purpose

Use is authorized by the binding, which was consented when granted. Each issuance is narrowed to the one purpose it names: a prefix, or one object with the bytes to be written to it.
