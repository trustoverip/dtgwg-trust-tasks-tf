---
slug: hcp/entries/hard-delete
version: "0.1"
title: "HCP — Hard Delete Entry"
summary: A Management Client, acting for the Authorizing Principal, permanently removes a Human Context Protocol entry; carries the HCP 2.0-draft-3 Hard delete operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Hard delete
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, erasure, hard-delete, governance]
parties:
  - role: Management Client (HCP), acting for the Authorizing Principal
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Hard deletion is irreversible and exercises the person's own authority, so the request is signed: the provider
    can show which Management Client asked for the erasure, and when, independently of the transport. The response
    proof is RECOMMENDED so the Management Client can show the provider confirmed the erasure, which is what an
    erasure obligation is evidenced by.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive and acts for the person, so it is consequential; a re-delivered hard deletion must be
    placeable in the provider's duplicate-execution window.
sideEffects:
  level: destructive
  rationale: >-
    The entry, every version of it and its unencrypted members are removed, or made unrecoverable by destroying
    every copy of the keys that decrypt it, backups included (HCP 5.4.2). It is never returned again, by export
    under any option either (HCP 5.4.1). Nothing restores it.
exposure:
  discloses: none
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Authorizing Principal's authority over their own context. The request
    carries the Management Client's access token, a bearer secret, and an entry identifier; the response carries
    only that identifier back.
retention:
  class: durable
  rationale: >-
    The provider appends a `context.hardDeleted` audit event (HCP 8.1), which is the person's record that the
    erasure happened. The entry itself is not retained: that is the point of the task.
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/entries/query
  - hcp/export
  - hcp/audit/read
---

## Abstract

The **HCP — Hard Delete Entry** Trust Task carries the Hard delete operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.4.2 and 5.4; `DELETE /v2/entries/{entryId}?hardDelete=true` in HCP 9.2 and 9.4.5). A *Management Client*, acting for the *Authorizing Principal*, tells an *HCP Provider* to remove an entry permanently. The provider removes it, so that it is unrecoverable and never returned again, and records a `context.hardDeleted` audit event.

Hard deletion is not withdrawal. HCP's withdraw (HCP 5.2.5) is a context operation that sets the entry's status to `deleted` and hides it from Context Clients while the provider still holds it in full; it is a different task. Hard deletion removes an entry in any status, including one already withdrawn (HCP 4.2.4).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on hard deletion; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Authorizing Principal's own authority over their context**, which a Management Client exercises for them (HCP 5.4.2, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. `accessToken` is OPTIONAL. Where the request carries none, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose (HCP 3.2.3), acting for the Authorizing Principal who enrolled it, and refuses any other issuer with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal. Where it carries one, the token rules apply unchanged, including the token binding of [The access token](#the-access-token). Both are *pending HCP confirmation* ([question 56](#pending-hcp-confirmation)). Hard deletion is a governance operation: a Context Client cannot perform it, and an attempt by any caller other than a Management Client acting for the Authorizing Principal **MUST** fail with not authorized (HCP 5.4.2, 6.1.9), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a Management Client may act for the Context Subject or for a delegate (HCP 6.1.9). A delegate's governance operations reach only entries in categories within the delegation's scope (HCP 17.4); an entry outside it is refused with `permissionDenied`, exactly as an entry that does not exist.

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). Where the request carries `accessToken`, the proof is not the authorization: the token is. Where it carries none, the proof is the Management Client's authentication (see above). A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate** and **entry** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path) and 9.4.5 (the withdraw result, whose shape the hard-delete result takes).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{entryId}` (`DELETE /v2/entries/{entryId}`, HCP 9.2) | `entryId` | Moved from the path into the payload, unchanged. A Management Client names the entry by its `id` (HCP 9.4.5). |
| query parameter `hardDelete=true` (HCP 9.4.5) | the Type URI | This task is the hard deletion; there is no member for it. |
| `Idempotency-Key` header (HCP 9.6) | the document `id` | See [Repeated requests](#repeated-requests). |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `entryId` | `entryId` | Unchanged. |
| result `hardDeleted` | `hardDeleted` | Always `true`. |
| result `status` | none | Absent: the entry is no longer held, so it has no status, *pending HCP confirmation* ([question 11](#pending-hcp-confirmation)). |
| result `timestamp` | `respondedAt` | **Renamed**, as in [`hcp/entries/query`](../../query/0.1/spec.md#hcp-field-mapping). Same type and meaning. |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the hard deletion to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `entryId`, the `id` of the entry to remove, and `accessToken`, the Management Client's token. `entryId` is REQUIRED; `accessToken` is OPTIONAL; where it is absent, the request's `proof` authenticates an enrolled Management Client (see [Authorization](#authorization)).

### Erasing a withdrawn health entry at the person's request

```json
{
  "id": "urn:uuid:485078e7-b932-42eb-8b8d-da1c2ee735c3",
  "type": "https://trusttasks.org/spec/hcp/entries/hard-delete/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:00:00Z",
  "threadId": "urn:uuid:024298c2-5a31-4e81-af88-ba288b28730f",
  "payload": {
    "entryId": "urn:uuid:d29f48ce-af52-4f6d-b36c-102222532293",
    "accessToken": "hcp_mt_5c1e9a7d3b2f4086"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4Hn7wQ2pLcX9vB3kT6mR1sY8dF5gJ2aZe7uN4oVbC1iM9kD3tH6yP2xS8wE4rA7jU1fG5lO9pK3vX6zN8bT2cW5"
  }
}
```

## Response

The provider answers when the entry is removed. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json):

- `entryId` names the entry as the caller named it (HCP 9.4.5).
- `hardDeleted` is always `true`.
- `respondedAt` is HCP's `timestamp`, renamed.

HCP 9.4.5 shows the result of a withdraw but not of a hard deletion. This specification takes the withdraw result's shape, with `hardDeleted` `true` and no `status`, *pending HCP confirmation* ([question 11](#pending-hcp-confirmation)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The entry is erased

```json
{
  "id": "urn:uuid:72458ae8-ceb2-46c1-b955-417d0d4bb700",
  "type": "https://trusttasks.org/spec/hcp/entries/hard-delete/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:00:01Z",
  "threadId": "urn:uuid:024298c2-5a31-4e81-af88-ba288b28730f",
  "payload": {
    "entryId": "urn:uuid:d29f48ce-af52-4f6d-b36c-102222532293",
    "hardDeleted": true,
    "respondedAt": "2026-11-25T09:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Rt5mK8vNcQ1wB7hX4pL9sY3dF6gJ0aZe5uT2oVnC8iM1kB4tH7yD3xS9wE6rA2jU5fG8lO4pK7vX1zN3bQ6cY8"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Authorizing Principal (HCP 5.4.2, 6.1.9), a delegate names an entry outside the delegation's scope (HCP 17.4), or the entry does not exist, including one already hard-deleted (see [Repeated requests](#repeated-requests)) | `permissionDenied`, with one fixed `message` whatever the reason, which never discloses whether the entry exists (HCP 5.5, 9.7, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` (HCP 9.6) | `idConflict` (see [Repeated requests](#repeated-requests)) |
| version conflict | `version-conflict` | Does not arise: hard deletion names no version and removes every version (HCP 5.4, 5.4.2). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6), and nothing is written. |
| subject unavailable | `subject-unavailable` | `unavailable`. Under end-to-end encrypted storage, the request needs the Context Subject's trust domain, which cannot be reached (HCP 4.8); the Management Client may retry later. |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **Who.** Only a Management Client acting for the Authorizing Principal may hard-delete; any other caller receives not authorized (HCP 5.4.2, 6.1.9). Implementations **MUST** support hard deletion at the Context Subject's request (HCP 5.4.2, 18.4).
- **How.** Hard deletion **MAY** be carried out by destroying every copy of the keys that decrypt the entry, in backups as well, provided the entry's unencrypted members are also removed (HCP 5.4.2).
- **Outcome.** The entry is unrecoverable and is never returned again, by any operation, and by export under any option (HCP 5.4.1, 5.4.2). Every version goes with it.
- **Any status.** An entry in any status may be hard-deleted; a `deleted` (withdrawn) entry is held in full until it is (HCP 4.2.4).
- **Audit.** The provider appends `context.hardDeleted` (HCP 8.1), durable before this task's response is sent (HCP 8.4).

## Repeated requests

An entry that does not exist, including one already hard-deleted, is refused with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal: HCP 5.5 makes "the entry does not exist" a not-authorized condition and forbids saying which, so a caller cannot tell an erased entry from one it may not reach.

HCP's HTTP binding asks a provider to accept an idempotency key on `DELETE /v2/entries/{entryId}` (a SHOULD, HCP 9.6), the path a hard deletion takes with `hardDelete=true` (HCP 9.4.5). As for [`hcp/entries/withdraw`](../../withdraw/0.1/spec.md#repeated-requests), this task uses the document `id` as that key, under [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): a re-delivered document with the same `id` and content is not executed twice, produces no second `context.hardDeleted` event, and is answered with the original response; a reused `id` with different content is refused with `idConflict`. The provider **MUST** retain its duplicate-execution record, with the response, for at least 24 hours (HCP 9.6), and for at least the acceptance window it applies to `issuedAt`.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 11 — the hard-delete result.** HCP 9.4.5 shows only the withdraw result. The hard-delete result takes its shape, with `hardDeleted` `true` and no `status`, since the entry is no longer held; `timestamp` is renamed `respondedAt` (see [Response](#response)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 56 — a Management Client authenticated without a token.** `accessToken` is OPTIONAL. Where it is absent, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose, reading HCP 3.2.3's "credentials" as including an enrolled key proved by a signature on each request, and refuses any other issuer with `permissionDenied`, with the one fixed `message`. Where a token is present, the token rules apply unchanged, including the binding of question 22. The request `proof` stays REQUIRED whether or not a token is present (question 53), and a provider keeps a record of the Management Clients each person enrolled. This is not question 37, which asks whether governance may travel off the HTTP Binding at all; it asks how the caller is authenticated once it does (see [Authorization](#authorization)).

## What HCP requires that a document cannot express

- The erasure itself: removing the entry and every version, or destroying every copy of its keys, backups included, together with its unencrypted members (HCP 5.4.2).
- Never returning the entry again, by any operation or export option (HCP 5.4.1).
- The `context.hardDeleted` audit event and its durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries an entry identifier and the Management Client's access token where it carries one, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries the entry identifier back and nothing of the entry. A producer **MUST NOT** put the token or any context in `ext`, and a provider **MUST NOT** put any of the erased entry's content in a `message`.

### The access token

`accessToken`, where a request carries it, is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which Management Client erased which entry and when; that is the record the person needs. The entry's `id` is never disclosed to a Context Client (HCP 7.4), so it is known only to the provider and the person's Management Clients. `threadId` **SHOULD** be fresh per deletion.

### Retention

The provider keeps the `context.hardDeleted` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3); it records that an entry was erased, not what it said. The signed request is evidence of who asked for the erasure and when, and a provider **SHOULD** keep it with that event, with `accessToken` replaced by its digest and a record that its proof was verified at receipt (see [The access token](#the-access-token)). Neither the provider nor the Management Client keeps any of the entry's content because of this task. A Context Client's copies, disclosed earlier under a grant, are governed by their `useUntil` (HCP 7.6), which hard deletion does not change.

### Consent/purpose

Hard deletion cannot be undone. A Management Client **SHOULD** show the person the entry, and say that it will be erased permanently and never returned again, before sending the request; whether it asks for a further confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). A person who only wants an entry hidden from Context Clients can withdraw it instead (HCP 5.2.5).
