---
slug: hcp/grants/request
version: "0.1"
title: "HCP — Request Grant"
summary: A Context Client asks an HCP Provider for a grant over a person's context; the provider holds the request with a next step while the person decides, then returns the grant and its access token.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [hcp, human-context-protocol, grant, authorization-request, consent]
parties:
  - role: Context Client (HCP), the HCP Client asking for the grant
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider, acting through its Authorization Service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The client has no token yet, so the request's proof is what authenticates it: the provider shows the person which
    client is asking, and records it in the grant. The response carries a bearer token and the grant the person
    approved, so it is signed: the client can show which provider issued the grant, and a token cannot be injected
    into the exchange by anyone else.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task creates a grant, so it is consequential; a re-delivered request must be placeable in the provider's
    duplicate-execution window, and must never ask the person twice.
sideEffects:
  level: mutating
  rationale: >-
    Approval creates a grant and appends a `grant.issued` audit event (HCP 6.1.6, 8.1). Nothing changes until the person
    acts, and a decline creates nothing (HCP 6.1.7).
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: personal
  rationale: >-
    The response carries an access token, a bearer secret. The request carries the categories and purposes asked for and
    may carry a hint identifying the person, which is personal data about them.
retention:
  class: durable
  rationale: >-
    An approval is recorded as a `grant.issued` audit event and a grant object the person can inspect and revoke (HCP
    6.1.5, 8.1). A decline need not be recorded (HCP 6.1.7).
    The response's access token is kept in no record of the exchange (see Security & Privacy, The access token).
related:
  - hcp/entries/query
  - hcp/grants/revoke
  - hcp/grants/revoked
  - trust-task-next-step
---

## Abstract

The **HCP — Request Grant** Trust Task lets a *Context Client* ask an *HCP Provider* for a grant over a person's context, under the grant model of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.1). The client names what it needs as HCP authorization details: actions, categories, purposes and, optionally, a narrowing (HCP 6.1.5). The provider puts the request to the person. While the person decides, the provider answers with a [`trust-task-next-step`](../../../../trust-task-next-step/0.1/spec.md) and the client resubmits on the same thread. When the person approves, the response carries the grant as approved and the access token that [`hcp/entries/query`](../../../entries/query/0.1/spec.md) and the other `hcp/*` tasks carry as their authorization evidence. A decline is `permissionDenied`.

**This task carries no HCP operation.** HCP Core defines no grant request: the request that starts a grant is defined by HCP's authorization profiles (HCP 6.1.2, 6.1.7), as an OAuth 2.0 authorization request (HCP 13.3) or through an authenticated channel of the provider's choosing (HCP 12.2). A specification that declared `carries: HCP` for it would define an operation HCP does not define ([SPEC §9.4](/SPEC.md#94-carried-protocols) item 1), so this specification declares no `carries` member. It is a Trust-Task-native authenticated channel in the sense of HCP 6.1.6, and it meets the outcome HCP 6.1.6 and 6.1.7 require of every channel. It lives in the `hcp/` namespace beside the carried tasks because it exists to obtain the evidence they carry, *pending HCP confirmation* ([question 21](#pending-hcp-confirmation)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this task ([SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). It is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution. Where it restates an HCP requirement and differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** meet HCP 6.1.4, 6.1.5, 6.1.5.1, 6.1.5.2, 6.1.6 and 6.1.7 in full for every grant it issues through it; the requirements that bear on the document are restated below.

Serving this task does not change which HCP authorization profile a provider claims ([SPEC §9.4](/SPEC.md#94-carried-protocols)). A provider claiming the OAuth Interop Profile still accepts the OAuth request of HCP 13.3, and offers this task in addition to it; a provider claiming the First-Party Authorization Profile may offer this task as its authenticated channel (HCP 12.2), only to the clients that profile admits. A provider **MUST NOT** list an identifier for this task in its HCP `conformsTo`, and advertises it through `trust-task-discovery`, *pending HCP confirmation* ([questions 1 and 21](#pending-hcp-confirmation)).

## Authorization

No grant exists yet, so no token is presented. The authority to create one is **the person's own**: the grant exists only once the Authorizing Principal performs an affirmative act, distinct from any act by which they authenticated (HCP 6.1.6), and nothing in the request is authority to issue it.

The REQUIRED `proof` authenticates the client. The provider **MUST** refuse a request whose `issuer` is not a client it has registered, with `permissionDenied`, and **MUST** show the person the client's registered name and its Client Operator, never a name the request supplies (HCP 6.1.5 `client.name`, 6.1.6). The `issuer` VID serves as the client's HCP client identifier, recorded in the grant's `client.id`, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). A client registered at the provider by another identifier is mapped to it by the provider's registration, which this specification does not define.

The person reaches the decision through the provider's own modality, never through the client: a management app, a web page, a message. The provider **MUST** tell the person, before they act, everything HCP 6.1.6 lists — the categories as the provider describes them, the actions, the purposes, the longest time a disclosed copy may be used (HCP 7.6), whether the grant is temporary or persistent and when it expires, the client and its operator, the requesting party where it differs from the person, any declared legal retention, any standing rule that excludes a requested category, a `provenance` action as a separate item, whether the client claims to follow directives, and any onward-sharing request.

## Definitions

- **Context Client**, **HCP Provider**, **Authorization Service**, **Authorizing Principal**, **Context Subject**, **grant** and **authorization detail** have their HCP 2.0-draft-3 meanings.
- **Hold**: the period between the provider receiving a request and the person's decision, during which the provider answers with a next step ([SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs)).

### HCP field mapping

HCP has no request schema for this task. The request members follow an OAuth authorization request with RFC 9396 authorization details, as HCP 13.3 names it, and the response members follow HCP's grant object (HCP 6.1.5, `schemas/2.0-draft-3/grant.json`).

| HCP / OAuth field | Member | Notes |
|---|---|---|
| `authorization_details` of type `https://hcp.example/spec/authorization/context-access` (HCP 13.3; RFC 9396) | `access` | Each detail carries `type`, `actions`, `categories`, and optionally `purposes`, `narrowedTo` and `onwardSharing`, as HCP 6.1.5 defines them. |
| `scope` (HCP 6.1.1, 13.3) | not carried | Scope strings are a compact view of `access`; a client names details. |
| `resource` (RFC 8707, HCP 13.3) | `recipient` | The provider the grant is for. |
| `client_id` | `issuer` | See [Authorization](#authorization). |
| `login_hint` (OpenID Connect) | `subjectHint` | Not in HCP. See [Finding the person](#finding-the-person). |
| grant `grantType` (HCP 6.1.4) | `grantType` | As asked in the request; as elected in the response. |
| grant `expiresAt`, temporary default and maximum (HCP 6.1.3) | `expiresIn` (request), `grantExpiresAt` (response) | `grantExpiresAt` is absent for a persistent grant, where HCP writes `null`. |
| grant `requestingParty` (HCP 6.1.5) | `requestingParty` | Unchanged. |
| token response `access_token`, `expires_in`, `authorization_details` (HCP 13.3) | `accessToken`, `tokenExpiresAt`, `access` | The token is opaque. `access` is what the person approved. |
| grant `grantId`, `issuedAt` (HCP 6.1.5) | `grantId`, `grantedAt` | `issuedAt` is renamed so it is not read as the document's own `issuedAt`. |
| refresh token (HCP 13.4) | not carried | Version 0.1 issues no refresh token; a client asks again, or the person makes the grant persistent. |

### Parties

| Role | Trust Task member |
|---|---|
| Context Client | `issuer` of the request and every resubmission, `recipient` of the next step and the response |
| HCP Provider (Authorization Service) | `recipient` of the request, `issuer` of the next step and the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), the provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and the client **MUST** check that binding before it relies on a token the response carries.

## Request

The client sends the request to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). `access` is REQUIRED; `grantType`, `expiresIn`, `subjectHint` and `requestingParty` are OPTIONAL.

- **Purposes.** A detail that names no `purposes` is allowed: the provider **MUST** then ask the person to choose one or more before approving (HCP 6.1.5.2), and the response tells the client which were approved.
- **Persistence.** `grantType: persistent` asks that the person be offered persistent access. Temporary is the default outcome of every request (HCP 6.1.4): the provider **MUST** offer persistence only as a separate affirmative act after the person confirms the grant, **MUST NOT** pre-select it, and issues a temporary grant unless the person elects it, *pending HCP confirmation* ([question 24](#pending-hcp-confirmation)).
- **Duration.** `expiresIn` asks for a temporary grant's lifetime, in seconds. Absent means HCP's default of 4 hours; the schema caps it at HCP's 24-hour maximum (HCP 6.1.3). The provider **MAY** grant less, and the person **MAY** choose less.
- **Narrowing and onward sharing.** A `narrowedTo` key the provider does not support, or an `onwardSharing` request in a detail that lacks `read` or covers a category whose default sensitivity is not Low, is refused with `malformedRequest` at once, before the person is asked (HCP 6.1.5, 6.1.5.1 rule 2).

### A booking agent asks for travel preferences

```json
{
  "id": "urn:uuid:3f1a7c2e-9b4d-4e8a-a1c6-5d2e0b9f7a14",
  "type": "https://trusttasks.org/spec/hcp/grants/request/0.1#request",
  "issuer": "did:web:agent.booking.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T10:00:00Z",
  "threadId": "urn:uuid:3f1a7c2e-9b4d-4e8a-a1c6-5d2e0b9f7a14",
  "payload": {
    "access": [
      {
        "type": "https://hcp.example/spec/authorization/context-access",
        "actions": ["read"],
        "categories": ["general_preferences", "travel_preferences"],
        "purposes": [
          {
            "code": "https://purposes.example/travel-booking",
            "description": "Book trips that match how you like to travel"
          }
        ]
      }
    ],
    "expiresIn": 14400,
    "subjectHint": "acct:alex@context.provider.example"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:00:00Z",
    "verificationMethod": "did:web:agent.booking.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4hT9mWqXc3HnRb7eLfYv2pDsA8uJgTzQ4oNiMxE6yCrUkVhB1aFw3SjP9LdGq7ZnT2vXe5YsRb8KuMhCf4AiWp6o"
  }
}
```

## Holding while the person decides

The person's decision is an act that has not yet occurred, so a request awaiting it is **held**, and a hold is a next step ([SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs)). The act belongs to a third party, the person, and the provider offers no task by which the client could learn the outcome, so:

1. Where the transport carries a reply, the provider **MUST** answer a held request with a `trust-task-next-step` whose `expects` names this specification's request Type URI, with `continuation: resubmit`, and **MUST NOT** answer it with an error response.
2. The client resubmits on the same thread: a new document with a fresh `id`, the same `threadId` and the same payload. The provider answers a resubmission with the outcome once the person has decided, or with a further next step while they have not.
3. The provider **MUST NOT** put a second request to the person for a thread it already holds. A resubmission whose payload differs from the held request is refused with `malformedRequest`; the held request is unaffected.
4. The provider **SHOULD** say in `hint.retryAfter`, in seconds, how long the client should wait before resubmitting, and the client **SHOULD NOT** resubmit sooner.
5. A hold lasts as long as the provider is willing to wait; how long is provider policy. A hold the provider abandons leaves nothing pending, and a later resubmission is a new request, put to the person again.

A next step is not a decision and confers no authority ([SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs)). The person's decision never travels through the client.

### The provider holds while the person decides

```json
{
  "id": "urn:uuid:7c2e9b4d-1a3f-4e8a-b6c5-0d9e2f7a14b3",
  "type": "https://trusttasks.org/spec/trust-task-next-step/0.1",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:agent.booking.example",
  "issuedAt": "2026-11-25T10:00:01Z",
  "threadId": "urn:uuid:3f1a7c2e-9b4d-4e8a-a1c6-5d2e0b9f7a14",
  "payload": {
    "expects": [
      {
        "typeUri": "https://trusttasks.org/spec/hcp/grants/request/0.1#request",
        "reason": "The request is with the person, who has not decided yet.",
        "hint": { "retryAfter": 5 }
      }
    ],
    "continuation": "resubmit",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/hcp/grants/request/0.1#request",
      "id": "urn:uuid:3f1a7c2e-9b4d-4e8a-a1c6-5d2e0b9f7a14"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3Kt9mWqXc3HnRb7eLfYv2pDsA8uJgTzQ4oNiMxE6yCrUkVhB1aFw3SjP9LdGq7ZnT2vXe5YsRb8KuMhCf4AiWp6o"
  }
}
```

## Response

Once the person approves, the provider answers the latest resubmission on the thread with this task's response. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json):

- `accessToken` and `tokenExpiresAt`: the grant evidence. The client presents the token unchanged as `accessToken` in [`hcp/entries/query`](../../../entries/query/0.1/spec.md) and every other `hcp/*` task it sends under the grant. Its lifetime follows HCP 13.4 and never outlasts the grant.
- `grantId`, `access`, `grantType`, `grantedAt` and, for a temporary grant, `grantExpiresAt`: the grant as the person approved it, which may be narrower than asked. `grantExpiresAt` is REQUIRED for a temporary grant and absent for a persistent one.

The provider records the approval as a `grant.issued` audit event, with the modality and the `acr` of the person's authentication, and the grant object of HCP 6.1.5, before it sends the response (HCP 6.1.6). The response carries the token and the grant as HCP's token response and read-own-grant would, *pending HCP confirmation* ([question 25](#pending-hcp-confirmation)).

### The person approves

```json
{
  "id": "urn:uuid:9e4b2d1a-7c3f-4a8e-b5d6-2c0e9f1a7b48",
  "type": "https://trusttasks.org/spec/hcp/grants/request/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:agent.booking.example",
  "issuedAt": "2026-11-25T10:01:30Z",
  "threadId": "urn:uuid:3f1a7c2e-9b4d-4e8a-a1c6-5d2e0b9f7a14",
  "payload": {
    "accessToken": "hcp_at_4d9a2c7e1b8f3a60",
    "tokenExpiresAt": "2026-11-25T11:01:30Z",
    "grantId": "urn:uuid:c4d2e1f0-9a8b-4c7d-8e6f-5a4b3c2d1e0f",
    "access": [
      {
        "type": "https://hcp.example/spec/authorization/context-access",
        "actions": ["read"],
        "categories": ["travel_preferences"],
        "purposes": [
          {
            "code": "https://purposes.example/travel-booking",
            "description": "Book trips that match how you like to travel"
          }
        ]
      }
    ],
    "grantType": "temporary",
    "grantedAt": "2026-11-25T10:01:30Z",
    "grantExpiresAt": "2026-11-25T14:01:30Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:01:30Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2mDfC3vL8nQe7sYxKp4WuTb1RhA9GjZc6VoNiXdE5yUtMqFkS3aPrHw8LJgB2zCn4TvQe6YsDxKu7MhRf9AiWp3o"
  }
}
```

In this example the person approved `travel_preferences` and not `general_preferences`; the client learns that from `access`.

## Errors

| Condition | Trust Task error |
|---|---|
| The payload fails the schema; an unsupported `narrowedTo` key; an `onwardSharing` request HCP 6.1.5 does not allow; a resubmission whose payload differs from the held request | `malformedRequest` |
| The `issuer` is not a registered client | `permissionDenied` |
| The person declined (HCP 6.1.7) | `permissionDenied` |
| The provider cannot find or reach the person named by `subjectHint` | `permissionDenied`, after a hold (see [Finding the person](#finding-the-person)) |
| The person's instance is unavailable (HCP 5.5 `subject-unavailable`) | `unavailable` |
| Throttling | `unavailable` with `retryAfter` |

Every `permissionDenied` carries the same fixed `message`, whatever the reason ([SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)), so a client cannot tell a decline from an unknown person. A decline is a refusal and is answered as one when the client next resubmits, never with a further next step ([SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs)). HCP does not require a decline to be logged (HCP 6.1.7), and the provider **MAY** record it.

`hcp:invalidToken` does not arise: the request carries no token.

## Finding the person

An HCP instance belongs to one person, but a provider may host many. HCP's OAuth flow finds the person when they sign in at the Authorization Service; a Trust Task request reaches the provider without them, so `subjectHint` names them — an account identifier, a handle, or another value the person gave the client, in the provider's own syntax. A provider serving one person ignores it. This is not in HCP, *pending HCP confirmation* ([question 23](#pending-hcp-confirmation)).

A request is a way to probe whether a person holds an account at a provider. To hide that, the provider **MUST** answer a request whose `subjectHint` it cannot resolve, or whose person it cannot reach, exactly as it answers a request the person has not yet decided — with a next step — and **MUST** later refuse it with the same `permissionDenied` a decline gets. It **SHOULD** make the hold last long enough that the timing does not tell the two apart.

## Repeated requests

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, it does not put the request to the person again, and it is answered with what the first delivery was answered with. A resubmission is not a re-delivery: it has a fresh `id` and is answered as [Holding while the person decides](#holding-while-the-person-decides) says. A request on a new thread is a new request, even with the same payload.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this task in its HCP `conformsTo`, and advertises it with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md).
- **Question 21 — a Trust Task grant request.** HCP Core defines no grant request (HCP 6.1.7). This task is read as an authenticated channel meeting HCP 6.1.6, which a First-Party provider may use as its channel (HCP 12.2) and an OAuth Interop provider may offer beside HCP 13.3. It declares no `carries`.
- **Question 22 — the client's identity.** The request's `issuer` VID, authenticated by its proof, serves as the HCP client identifier the grant records in `client.id`.
- **Question 23 — naming the person.** `subjectHint` names the person, like OpenID Connect `login_hint`. An unknown or unreachable person is held and then refused exactly as a decline, so a request cannot probe for accounts.
- **Question 24 — asking for persistence.** `grantType: persistent` lets the provider offer persistence as HCP 6.1.4's separate act; it never pre-selects it, and the outcome is temporary unless the person elects it.
- **Question 25 — the token in a Trust Task response.** The response carries the access token and the grant as approved, as HCP's OAuth token response does (HCP 13.3), with no refresh token in this version. A token obtained this way is accepted by the `hcp/*` tasks as any HCP access token is.

## What HCP requires that a document cannot express

- The person's affirmative act, its separation from authentication, and everything they must be told before it (HCP 6.1.6).
- Persistence and onward sharing as separate, never pre-selected acts (HCP 6.1.4).
- The `grant.issued` audit event and the grant object (HCP 6.1.5, 8.1).
- Whether a narrowing key is supported and enforced (HCP 6.1.5.1).

## Security & Privacy

### Data carried

The request carries the categories, actions and purposes asked for, and may carry a `subjectHint` that identifies the person to the provider; a client **MUST** send it only to the provider the person named. The response carries a bearer access token, which the client **MUST** keep as it keeps any HCP token and present only to the provider that issued it. Neither carries context. A producer **MUST NOT** put a token, a hint or any context in `ext`.

The response is signed (REQUIRED), so the grant it describes is a third-party-verifiable assertion by the provider. That is its purpose: the client can show what it was granted. It does not authorize anyone but the client: the token is bound to the client by the provider's records, and HCP forbids passing it on (HCP 3.11).

### The access token

The response's `accessToken` is a bearer secret: whoever presents it exercises the grant. It **MUST** travel only to the client that asked for the grant, in the response addressed to it. The client keeps the token to use the grant but **MUST NOT** log it, and a record of the response it keeps as evidence of the grant keeps the document with `accessToken` removed, or a digest of the document, and never the token. The provider **MUST NOT** log the token, or keep it in any record of the exchange. Every `hcp/*` task that presents the token refuses it with `permissionDenied` when its request carries a `proof` from a VID other than this request's `issuer` (see [`hcp/entries/query`](../../../entries/query/0.1/spec.md#the-access-token)), *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)).

### Correlation

The provider learns which client asked for which categories, for which purposes, and about whom. The client learns the person's decision and, through `grantId`, a stable identifier for the grant, which it also sees in [`hcp/grants/revoked`](../../revoked/0.1/spec.md). `threadId` **SHOULD** be fresh per request. Every refusal looks the same, so a client cannot learn whether a person holds an account.

### Retention

The provider keeps the grant object and its `grant.issued` event for as long as HCP requires; the person can read both (HCP 6.1.5, 8.3). The signed request is evidence of what the client asked for and **SHOULD** be kept with the event. The client keeps the token only as long as it is valid, and **MAY** keep the signed response as evidence of the grant.

### Consent/purpose

This task *is* the consent request. The provider **MUST NOT** treat the request, its proof, the client's authentication, or the person's sign-in as the person's consent (HCP 6.1.6). Purposes bind what the client may say it is doing with what it reads (HCP 6.1.5.2), and every query under the grant names one.
