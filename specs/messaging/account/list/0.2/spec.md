---
slug: messaging/account/list
version: "0.2"
title: Messaging — List Accounts
summary: A requester lists the accounts served by the mediator, paginated by an opaque cursor and a bounded page limit.
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - read
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Listing accounts is a non-mutating query; a proof is RECOMMENDED to bind the request to its requester for authorization and audit, but is not required for integrity of any change since none is made.
sideEffects:
  level: none
  rationale: "Read-only listing of served accounts."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - messaging/account/get
  - messaging/account/add
  - messaging/account/remove
---

## Abstract

The **Messaging — List Accounts** Trust Task enumerates the accounts served by the mediator. The requester **MAY** supply an opaque `cursor` to continue a previous enumeration, a `limit` to bound the page size, and an `accountType` filter to restrict the enumeration to one role; the mediator returns an array of [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) views and, when more results remain, a `nextCursor` to fetch the next page.

The task is read-only; it makes no change. The `cursor` value is opaque to the requester — it **MUST** be treated as an unstructured continuation token and echoed back verbatim. The mediator applies its own authorization policy; typically only an administrator may list accounts.

The `accountType` filter with `admin` or `rootAdmin` enumerates the mediator's administrators; this subsumes the retired `messaging/admin/list` task, which was this enumeration restricted to the admin roles.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.1

The two activity instants on [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) — each entry of the response's `accounts` — change type; their names do not:

- **`lastReceivedAt`**: integer Unix seconds → RFC 3339 string (`type: "string"`, `format: "date-time"`), e.g. `"2026-10-23T09:30:00Z"`.
- **`lastAuthenticatedAt`**: integer Unix seconds → RFC 3339 string (`type: "string"`, `format: "date-time"`).

Both remain optional, and present only under the same conditions as in `0.1`. The registry-wide convention is that an instant is an RFC 3339 string whose member name ends in `At`; these names already did, so only the type moves.

The change arrives through the shared component: this version re-pins its `$ref`s from `messaging/_shared/0.1` to `messaging/_shared/0.2`, which is why it is a new version at all ([SPEC.md §5.4](/SPEC.md#54-migrating-between-versions) couples the two). Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published and pinned to `messaging/_shared/0.1`.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the requester) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/list/0.2`, with itself as `issuer` and the mediator as `recipient`.
2. Where continuing a previous enumeration, set `payload.cursor` to the `nextCursor` returned by the prior page, echoed verbatim.
3. **SHOULD** include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof).
4. Set `payload.includeActivity` to `true` only to receive the activity timestamps (`lastReceivedAt`, `lastAuthenticatedAt`). A mediator that predates this member rejects the request as a schema violation, so a requester **SHOULD** set it only where it knows the mediator supports it.
5. Set `payload.includeStats` to `true` only to receive the account's lifetime counters (`stats`). A mediator that predates this member rejects the request as a schema violation, so a requester **SHOULD** set it only where it knows the mediator supports it.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and, where a `proof` is present, verify it.
2. Enforce its own authorization policy and respond with the framework's `permissionDenied` where the requester may not list accounts.
3. Where `accountType` is present, return only accounts holding that role.
4. Return at most `limit` accounts where `limit` is present, otherwise a mediator-chosen default page size.
5. Include `nextCursor` in the response when, and only when, further accounts remain beyond the returned page; omit it on the final page.
6. Include `lastReceivedAt` and `lastAuthenticatedAt` in each returned account when, and only when, `payload.includeActivity` is `true`, and then only the timestamps it has recorded. It **MUST NOT** include them otherwise: a requester that did not ask may validate the response against a schema without them.
7. Include `stats` in each returned account when, and only when, `payload.includeStats` is `true`, and then only the counters it keeps. It **MUST NOT** include them otherwise: a requester that did not ask may validate the response against a schema without them.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/list/0.2` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/list/0.2",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-22T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg..."
  }
}
```

This example requests the first page of up to 50 served accounts.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/list/0.2#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ accounts, nextCursor? }`, an array of account views and an optional continuation cursor.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/list/0.2#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "accounts": [
      {
        "did": "did:web:alice.example",
        "accountType": "standard",
        "acl": {
          "blocked": false,
          "sendMessages": true,
          "receiveMessages": true,
          "didcommEnabled": true,
          "tspEnabled": true
        }
      },
      {
        "did": "did:web:bob.example",
        "accountType": "admin",
        "acl": {
          "blocked": false,
          "sendMessages": true,
          "receiveMessages": true,
          "didcommEnabled": true,
          "tspEnabled": true
        }
      }
    ],
    "nextCursor": "eyJvIjoxMDB9"
  }
}
```

Because `nextCursor` is present, more accounts remain; the requester re-issues the request with `payload.cursor` set to `"eyJvIjoxMDB9"` to fetch the next page. Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A mediator **MUST** enforce its own authorization independent of the document, returning the framework's `permissionDenied` where the requester may not enumerate accounts, even though `proof` is only **RECOMMENDED** for this read-only task.

### Data carried

A listing discloses the full roster of served accounts, their roles, and their capabilities — a sensitive enumeration of the mediator's clientele. Filtering on `admin` or `rootAdmin` enumerates the mediator's administrators. The page is point-in-time and **MAY** be inconsistent under concurrent account changes.

The activity timestamps are returned only when `includeActivity` is set, and the lifetime counters (`stats`) only when `includeStats` is set. The lifetime counters are operational metadata about the served party: how much it has sent and received, and over which protocols. They disclose volume rather than content, and a mediator **SHOULD** apply the same authorization to them as to the rest of the account view.

The optional `ext` extension (see [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.

### Correlation

The activity timestamps (`lastReceivedAt`, `lastAuthenticatedAt`) disclose when a party last received traffic and last connected — a behavioural signal beyond the account's configuration. A mediator **SHOULD** apply the same authorization to them as to the rest of the account view, and **MAY** record `lastReceivedAt` coarsely (it may lag by up to 60 seconds) so recording it does not add a write to every message.

The `cursor` is an opaque continuation token; a mediator **SHOULD** make it stateless and unforgeable so a requester cannot enumerate beyond its authorization by crafting cursor values. Each returned account DID is the identifier the other `messaging/account/*` tasks accept, and `threadId` joins the request to its response.

### Retention

The response is a point-in-time snapshot of the mediator's roster. This specification sets no retention rule for it.

### Consent/purpose

The data is returned so an authorized requester can enumerate the mediator's served accounts. This specification places no further limit on its reuse.
