---
slug: hcp/grants/revoke
version: "0.1"
title: "HCP — Revoke Grant"
summary: A Management Client, acting for the person, ends a Human Context Protocol grant at the provider; carries the HCP 2.0-draft-3 Revoke grant operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Revoke grant
category: access-control
keywords: [hcp, human-context-protocol, grant, revocation, consent-withdrawal]
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
    Revocation is irreversible and exercises the person's own authority, so the request is signed: the provider can
    show which Management Client ended the grant, and when, independently of the transport. The response proof is
    RECOMMENDED so the Management Client can show the provider acknowledged the revocation.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive and acts for the person, so it is consequential; a re-delivered revoke must be placeable
    in the provider's duplicate-execution window.
sideEffects:
  level: destructive
  rationale: >-
    Revocation cannot be undone: the grant ends at once (HCP 6.2.2), and restoring access needs a new grant, given
    by the person again.
exposure:
  discloses: none
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Authorizing Principal's authority over their own grants. The request carries
    the Management Client's access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider records a `grant.revoked` audit event (HCP 6.2.2), which is the person's record that access ended
    and when.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      The Management Client's access token is missing from the provider's records, expired, revoked, or otherwise not
      valid for this provider (HCP 5.5 `invalid-token`).
    retryable: false
related:
  - hcp/grants/revoked
  - hcp/entries/query
---

## Abstract

The **HCP — Revoke Grant** Trust Task carries the Revoke grant operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.2 and 5.4; `DELETE /v2/grants/{grantId}` in HCP 9.2). A *Management Client*, acting for the *Authorizing Principal*, tells an *HCP Provider* to end a grant. The grant ends at once, the provider records the revocation, and it notifies the affected client with [`hcp/grants/revoked`](../../revoked/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on revocation; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Authorization

The authority is **the Authorizing Principal's own authority over their grants**, which a Management Client exercises for them (HCP 6.1). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Only a Management Client may revoke: a Context Client token, or any other caller, is refused with `permissionDenied` (HCP 6.1.9). A Context Client that wants to give up its own grant uses HCP's separate Relinquish operation (HCP 5.2.7), which is not this task.

Under Delegated Authorization (HCP 17.5), a delegate acting as Authorizing Principal may revoke as the person could; the provider applies HCP's delegation rules to the token, unchanged.

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A provider that knows which VID it registered the Management Client under **SHOULD** check that the `issuer` is that client.

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Authorizing Principal** and **grant** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path), 9.4.6 and 6.2.1.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{grantId}` (`DELETE /v2/grants/{grantId}`, HCP 9.2) | `grantId` | Moved from the path into the payload, unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `204 No Content` | the empty `#response` payload | No body in HCP, so none here. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the revocation to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `grantId`, the grant to end, and `accessToken`, the Management Client's token. Both are REQUIRED.

### Revoking a booking agent's grant

```json
{
  "id": "urn:uuid:6a3c9e12-8b4d-4f7a-9c2e-1d0b3a5f7e31",
  "type": "https://trusttasks.org/spec/hcp/grants/revoke/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-24T11:00:00Z",
  "threadId": "urn:uuid:b8e1d4c2-7a3f-4e6b-9d5c-2f1a0e9b8c41",
  "payload": {
    "grantId": "urn:uuid:c4d2e1f0-9a8b-4c7d-8e6f-5a4b3c2d1e0f",
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-24T11:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5vgFc2hLmXyQ7d9kT3rNpB8sWj4aZe6uHq1oVcR2xYbM9nKf7tL3pD5gS8wJ2eA6iU4yO1mC9vX7zB3nQ5hT2kF"
  }
}
```

## Response

The provider answers when the grant is revoked. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries HCP's `204 No Content`. The response is declared, although it carries nothing, because the revocation is consequential and its completion is relied upon ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The grant is revoked

```json
{
  "id": "urn:uuid:1f7b3d5e-2c9a-4e8b-a0d6-4c3e2b1a9f52",
  "type": "https://trusttasks.org/spec/hcp/grants/revoke/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-24T11:00:01Z",
  "threadId": "urn:uuid:b8e1d4c2-7a3f-4e6b-9d5c-2f1a0e9b8c41",
  "payload": {}
}
```

## Errors

| HCP condition (5.5) | Trust Task error |
|---|---|
| `malformed-request` | `malformedRequest` |
| `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| `scope-denied`, including a caller that is not a Management Client (HCP 6.1.9) | `permissionDenied` |
| `subject-unavailable` | `unavailable` |
| `unsupported-version` | `unsupportedVersion` |

Revocation is **never rate limited** (HCP 6.2.1), so a provider **MUST NOT** refuse it with `unavailable` for throttling.

## Revocation rules carried from HCP

- **Immediate.** The grant is invalid from the moment the provider accepts the revocation: every later request under it fails with `hcp:invalidToken` (HCP 6.2.2). The provider appends the `grant.revoked` audit event within 60 seconds (HCP 6.2.2).
- **Notice is independent.** The provider then notifies the affected Context Client ([`hcp/grants/revoked`](../../revoked/0.1/spec.md)); the revocation takes effect whether or not that notice is delivered (HCP 6.2.3), and the response to this task does not wait for it.
- **No notice after relinquishment.** A client that relinquished its grant (HCP 5.2.7) is not notified.

## Repeated requests

HCP 2.0-draft-3 does not say what a revocation of an already-revoked grant, or of an unknown `grantId`, yields. This specification does not define it either ([SPEC §9.4](/SPEC.md#94-carried-protocols), "no departures"), and it is an open question for the HCP editors. A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) and is not executed twice.

## What HCP requires that a document cannot express

- The provider-held grant state, and the immediate invalidation of every token issued under the grant (HCP 6.2.2).
- The audit event and its 60-second deadline (HCP 6.2.2).
- Delegated-profile revocation rules (HCP 17.5).

## Security & Privacy

### Data carried

The request carries a grant identifier and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries nothing. A producer **MUST NOT** put the token or any context in `ext`.

### Correlation

The provider learns which Management Client revoked which grant and when; that is the record the person needs. The `grantId` is known to the provider, the Management Client and the affected Context Client, and appears in the revocation notice; it joins the three documents by design. `threadId` **SHOULD** be fresh per revocation.

### Retention

The provider keeps the `grant.revoked` audit event (HCP 6.2.2), readable by the Context Subject (HCP 8.3). The signed request is evidence of who ended the grant and when, and a provider **SHOULD** keep it with that event. The Management Client **MAY** keep the response as evidence that the provider accepted the revocation.

### Consent/purpose

Revocation withdraws the person's permission; the data in the document is used only to carry out and record that withdrawal. Whether a revocation needs any confirmation from the person before the Management Client sends it is the Management Client's policy, not a requirement of this specification ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy; a generous window suits this task, since refusing a late revocation leaves alive a grant the person meant to end.
