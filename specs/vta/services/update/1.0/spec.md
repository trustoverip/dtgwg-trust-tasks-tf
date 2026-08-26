---
slug: vta/services/update
version: "1.0"
title: VTA Services — Update
summary: An operator changes the settings of a transport the agent already advertises.
status: draft
targetFrameworkVersion: "0.2"
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
  "type": "https://trusttasks.org/spec/vta/services/update/1.0",
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

## Response

```json
{
  "id": "00000004-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/update/1.0#response",
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

The advertised surface is public by construction — it lives in the agent's DID
document, which anyone may fetch. So the disclosure risk here is not the
*content* of a service entry but the *ability to change it*: an attacker who can
enable a transport can point the agent's traffic at infrastructure they control,
and every client that resolves the DID afterwards will believe it.

That is why every mutation in this family is super-admin only, and why each one
writes a signed log entry rather than flipping a runtime flag. The log is what
makes a change attributable after the fact.

Two failure modes are worth stating plainly:

- **`serverless: true` means nobody else can see the change yet.** The entry is
  written locally but not published; a consumer that reports success without
  surfacing this tells the operator a change is live when no verifier can
  observe it.
- **A drain is not a completed disable.** While `drainUntil` is in the future the
  old mediator is still accepting delivery. Reporting "disabled" at that point is
  wrong in the direction that loses messages.

