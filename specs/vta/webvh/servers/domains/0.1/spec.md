---
slug: vta/webvh/servers/domains
version: "0.1"
title: WebVH Servers — Domains
summary: A producer asks an agent which hosting domains it may use on one of the hosting servers it has registered; the agent relays the server's caller-scoped view.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - webvh
  - did-hosting
  - domain
  - discovery
  - relay
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer (operator or tooling)
    requirement: REQUIRED
    member: issuer
  - role: Agent holding the server registration
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The answer is a caller-scoped projection of public-by-domain-name configuration, and the agent authenticates the producer via the transport session. A proof matters when the listing is retained as evidence of what was reachable at a point in time.
sideEffects:
  level: none
  rationale: "Read-only. The agent may perform a network round trip to the hosting server, but nothing is created or changed on either side."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - did-management/me/domains
  - did-management/did/register
  - did-management/did/check-name
---

## Abstract

The **WebVH Servers — Domains** Trust Task answers "which hosting domains may I use on server *X*?" for an agent that holds registrations with one or more DID-hosting servers.

The agent does not own this information — the hosting server does. It authenticates to that server with its own credentials, calls the server's [`did-management/me/domains`](../../../../../did-management/me/domains/0.1/spec.md), and relays the result. The response items are therefore **the same `DomainEntry`** the `did-management` family defines, not a parallel shape: an operator comparing what the agent reports against what the server reports must not have to reconcile two spellings of one domain.

## Why this is not `did-management/me/domains`

They are two hops of one question, and the difference is `serverId`.

`me/domains` is addressed **to a hosting server**, and "me" is the authenticated caller — the server needs no parameter because there is only one server in the conversation. This task is addressed **to an agent that knows several servers**, so it must name which one. An agent asked without a `serverId` cannot answer generally: hosting servers do not share a domain namespace, and merging their answers would produce a list where no entry means anything without knowing where it came from.

The response is the same because the *answer* is the same object, simply relayed.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/webvh/servers/domains/0.1`, with itself as `issuer`, the agent as `recipient`, and `payload.serverId` naming a server the agent has registered.

A conforming **consumer** (the agent) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements).
2. Refuse with `not_found` where it holds no registration under `serverId`. An unknown server is not an empty domain list — those mean different things, and collapsing them tells a producer that a server it has not registered simply grants it nothing.
3. Relay the hosting server's caller-scoped view **without filtering it further**. The scoping decision belongs to the server, which holds the ACL; an agent that narrows the list again would report fewer domains than the producer may actually use.
4. Return each domain as a `DomainEntry`, preserving the members the server supplied — including `createdAt`.
5. Report `default` as the server reported it, and **omit** it where the server reported none. Consumers **MUST** treat an absent `default` and an explicit `null` as the same answer — the schema admits both, and a producer that distinguished them would be reading a difference no implementation intends.

An agent **SHOULD** return an empty `domains` array, rather than an error, for a server it can reach but holds no domain grant on. That is a true answer to the question asked.

Where the agent's transport to the server cannot express this query — an older server, or one reachable only over a protocol whose surface lacks it — the agent **SHOULD** return an empty list rather than fail, and producers **MUST NOT** read an empty list as "no domains exist". It means "none discoverable here", and the correct fallback is to let the server resolve the domain itself at creation time.

## Definitions

* **Producer.** The party asking; identified by `issuer`.
* **Agent.** The party holding the server registration and performing the relay; identified by `recipient`.
* **Hosting server.** The DID-hosting service that owns the domain registry; not a party to this document.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/webvh/servers/domains/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "2f1c8a44-6b21-4c3e-9f51-8a3c79e3d1a2",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/domains/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-01T09:00:00Z",
  "payload": {
    "serverId": "primary-host"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/webvh/servers/domains/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "3a2d9b55-7c32-4d4f-a062-9b4d8af4e2b3",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/domains/0.1#response",
  "threadId": "2f1c8a44-6b21-4c3e-9f51-8a3c79e3d1a2",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-01T09:00:01Z",
  "payload": {
    "domains": [
      {
        "name": "did.example.com",
        "label": "Production",
        "status": "active",
        "defaultDomain": true,
        "createdAt": "2026-03-01T00:00:00Z"
      },
      {
        "name": "staging.example.com",
        "status": "active",
        "createdAt": "2026-05-14T12:00:00Z"
      }
    ],
    "default": "did.example.com"
  }
}
```

### Reachable server, no grant

```json
{
  "id": "4b3e0c66-8d43-4e50-b173-0c5e9bf5f3c4",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/domains/0.1#response",
  "threadId": "2f1c8a44-6b21-4c3e-9f51-8a3c79e3d1a2",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-01T09:05:01Z",
  "payload": {
    "domains": []
  }
}
```

Failures (`not_found` for an unregistered `serverId`) use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

The listing is a caller-scoped projection of configuration that is public by domain name, so the disclosure is modest — but it does tell the producer which hosting relationships the agent holds, which is not otherwise observable. Agents **SHOULD** answer only for producers entitled to act through them.

**The scoping decision is the hosting server's, and relaying it unfiltered is a conformance requirement rather than a convenience.** An agent that applied its own narrowing would produce a list that is *shorter* than the truth, and the producer would conclude a domain is unavailable when the server would in fact accept it. Under-reporting here is not the safe direction: it silently removes valid choices instead of surfacing an error.

`createdAt` is preserved because a relay that quietly drops members turns "the agent did not tell me" into "the server does not know" — indistinguishable to the producer, and wrong.
