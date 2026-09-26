---
slug: vta/services/update
version: "1.1"
title: VTA Services — Update
summary: An operator changes the settings of a transport the agent already advertises.
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - services
  - transport
  - did-document
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Operator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Re-points live traffic; the request should be as attributable as the log entry it produces.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An update replaces the service's running configuration or version wholesale, so an out-of-order copy redeploys a superseded release as though it were current.
sideEffects:
  level: mutating
  rationale: "Edits the agent's DID document and republishes its signed log."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/services/update:notFound
    meaning: The transport is not currently advertised, so there is nothing to update. Use enable.
    retryable: false
  - code: vta/services/update:validationFailed
    meaning: The config is not valid for the named service — wrong member, or a URL that is not https:// with no fragment and no userinfo.
    retryable: false
  - code: vta/services/update:notAuthorized
    meaning: The caller is not a super-admin.
    retryable: false
related:
  - vta/services/enable
  - vta/services/rollback
---

## Abstract

**VTA Services — Update** replaces the settings on a transport that is already
advertised, and republishes the log.

It is refused when the transport is not currently enabled. That refusal is the
point: `enable` and `update` differ in what they assume already exists, and a
single permissive verb would let a mistyped `service` create an advertisement
where the operator meant to correct one.

## Changes from 1.0

1.1 adds one optional request member, `drainTtlSecs`, for the **mediated**
transports — `didcomm` and `tsp`, the two whose `config` names a `mediatorDid`.
Replacing a mediator puts the old one in a drain: it keeps accepting delivery
for correspondents still holding the previous DID document, and a DIDComm
sender and a TSP sender with a cached document are stranded in exactly the same
way when it stops. 1.0 gave the operator no say in how long the drain lasts, so
a recipient could only apply its own default. An operator who knows its
correspondents refresh slowly needs a longer window, and one decommissioning a
mediator it no longer trusts needs the shortest it is allowed. Nothing else
changes; a 1.0 request is a valid 1.1 request.

`drainTtlSecs` is refused for a non-mediated transport (`rest`, `webauthn`), as
a `url` is for a mediated one: a member that does not apply to the named
service makes the request malformed rather than being ignored. A recipient
**MAY** raise a value below its floor to the floor — over a request that
arrived through the mediator being replaced it **MUST**, since tearing that
mediator down discards the reply — and reports the window it applied in the
result's `drainUntil`.

Where one mediator carries both mediated transports, replacing it for either
drains it for both: the old mediator keeps accepting DIDComm and TSP delivery
until the one `drainUntil` the result reports. A recipient **MUST NOT** run two
drains of different lengths on one mediator.

## Replacement, not merge

`config` is stored as given. A member omitted is not "left unchanged" — send the
whole configuration for the kind you are updating.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** reject a payload whose `config` members do not belong to the named `service`, rather than ignoring the surplus member. It **MUST** write a did:webvh log entry for every accepted change, and **MUST** report `serverless: true` when that entry was not published.

## Authorization

Authority is **super-admin**. Every task in this family edits — or reads — what
the agent tells the world about reaching it, and the ACL has no finer capability
for a subset of transports.

A `proof`, where present, establishes that the producer authored the request. It
is not the authorization; the caller's super-admin role is.

## Request

```json
{
  "id": "00000004-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/update/1.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {
    "service": "rest",
    "config": {
      "url": "https://vta.example/api/v2"
    }
  }
}
```

Replacing the mediator with a two-day drain for the old one:

```json
{
  "id": "00000004-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vta/services/update/1.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:12:00Z",
  "payload": {
    "service": "didcomm",
    "config": {
      "mediatorDid": "did:web:new-mediator.example"
    },
    "drainTtlSecs": 172800
  }
}
```

## Response

```json
{
  "id": "00000004-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/update/1.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000004-0000-4000-8000-000000000001",
  "payload": {
    "result": {
      "logEntryVersionId": "4-zQmLogEntry",
      "effectiveAt": "2026-08-19T09:10:01Z",
      "vtaDid": "did:webvh:QmAgent:vta.example",
      "serverless": false
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries the transport to change and its new setting — an endpoint
URL or a mediator DID — and, for a mediated transport, the drain window. The response
carries the log entry that made the change. Everything here ends up in, or
describes, the agent's public DID document.

### Correlation

The advertised surface is public by construction: anyone may fetch the agent's
DID document. The change introduces no identifier that was not about to be
published in it.

### Retention

The signed log entry is the durable record, kept for as long as the DID's log
is; the exchange itself carries nothing beyond it.

### Consent/purpose

The operator changes how its own agent is reached. No other party's data is
involved; correspondents learn of the change by resolving the DID, which the
drain gives them time to do.

### Threats

The disclosure risk here is not the *content* of a service entry but the
*ability to change it*: an attacker who can repoint a transport can point the
agent's traffic at infrastructure they control, and every client that resolves
the DID afterwards will believe it. That is why every mutation in this family is
super-admin only, and why each writes a signed log entry rather than flipping a
runtime flag.

- **`serverless: true` means nobody else can see the change yet.** The entry is
  written locally but not published; a consumer that reports success without
  surfacing this tells the operator a change is live when no verifier can
  observe it.
- **A drain is not a completed update.** While `drainUntil` is in the future the
  old mediator is still accepting delivery. A short `drainTtlSecs` strands
  senders that have not re-resolved the DID; the floor exists so that at least
  the request's own reply is not one of them.
