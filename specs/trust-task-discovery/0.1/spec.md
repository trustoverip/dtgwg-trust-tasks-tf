---
slug: trust-task-discovery
version: "0.1"
title: Trust Task Discovery
summary: Query a Trust Task party for the set of task types it supports.
status: draft
targetFrameworkVersion: "0.1"
category: framework
keywords:
  - discovery
  - framework
  - protocol-negotiation
  - capability-exchange
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Discoverer
    requirement: REQUIRED
    member: issuer
  - role: Responder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: A discovery exchange is non-authoritative metadata exchange between parties that have already authenticated through the transport. The responder's list is advisory — the discoverer still validates each subsequent task per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements). A `proof` adds little beyond what the transport binding already provides and may be omitted.
errorCodes: []
related: []
---

## Abstract

The **Trust Task Discovery** exchange lets one party ask another which *Trust Task specifications* it can act upon. The query carries an optional list of slug-glob patterns; the response carries the matching subset of *Type URIs* the responder supports.

Discovery is a framework-defined meta-task: it exists so that a *consumer* and a *producer* can negotiate a shared task vocabulary before committing to any single specification. The slug `trust-task-discovery` is reserved by [SPEC.md §6.1](../../../SPEC.md#61-type-uri) under the framework's `trust-task-` namespace, alongside `trust-task-error`, `trust-task-ok`, and `trust-task-next-step`.

## Status of this document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A *conforming responder* **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-discovery/0.1#response` and whose `payload` validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.
2. Populate `supportedTypes` with the subset of *Type URIs* the responder can act upon that match **at least one** of the request's patterns. When the request supplied no patterns (the array is absent or empty), the responder treats the query as if it had carried the single pattern `"*"`.
3. List bare *Type URIs* only — no `#request` or `#response` fragment. Each listed *Type URI* implicitly covers both directions of that specification's exchange.
4. Avoid duplicate entries by *Type URI* — regardless of whether the entry is in shorthand string form or expanded object form.

Each `supportedTypes` entry **MAY** be either:

* **Shorthand** — a bare *Type URI* string. Semantically equivalent to an expanded form with no capability annotations.
* **Expanded** — an object with `type` (the bare *Type URI*) and optional capability annotations. The 0.1 schema reserves one annotation, `requiredExt`, for declaring `ext` namespaces the responder requires on inbound documents per [SPEC.md §4.5.1](../../../SPEC.md#451-the-ext-extension-member) and [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements). A *responder* that enforces `ext` policy **SHOULD** advertise it here so *producers* can satisfy the policy before issuing the task.

A *conforming responder* **SHOULD**:

* Sort `supportedTypes` lexicographically by *Type URI* for deterministic output.
* Include the bare-string shorthand for every entry that has no capability annotations — the expanded form is only useful when at least one annotation is present.
* Set `frameworkVersion` to the MAJOR.MINOR of the framework specification ([SPEC.md](../../../SPEC.md)) the responder targets, so a discoverer at framework version X can reason about forward-minor compatibility per [SPEC.md §5.2](../../../SPEC.md#52-compatibility-rules). The field is optional in 0.1 and **RECOMMENDED** in future revisions.
* Include specifications whose `targetFrameworkVersion` is older than the discoverer's framework version where it has a choice — listing those permits forward-minor-compatible processing.

A *conforming discoverer* **MUST**:

* Treat the response as advisory: a Type URI's presence is a hint that the responder will accept a *Trust Task document* of that type, not a binding promise. The full [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) pipeline still applies to every subsequent exchange.

A *conforming discoverer* **MAY**:

* Issue further discovery queries with narrower patterns as its understanding of the responder's capabilities evolves.

## Pattern grammar

A pattern is a string. The matching rules are:

| Pattern              | Matches                                                                              |
|----------------------|--------------------------------------------------------------------------------------|
| `*`                  | Every slug.                                                                          |
| `<prefix>/*`         | Every slug that starts with `<prefix>/` (e.g. `acl/*` matches `acl/grant`, `acl/revoke`, `acl/grant/sub`). |
| `<slug>` (exact)     | The specific slug. No wildcards interpreted in the middle of the string.             |

A pattern in any other shape is treated as an exact slug match (no wildcards). Wildcards anywhere other than as the trailing segment of a `<prefix>/*` pattern are **not** interpreted; they are matched literally — so a pattern like `acl/*-handoff` matches no slug at all (since slugs cannot contain `*`).

Multiple patterns in the request `patterns` array are combined with **OR** semantics: a slug matches the query if it matches at least one pattern.

## Privacy considerations

A discovery response leaks information about which specifications the responder implements — useful for an honest discoverer planning an exchange, useful for an attacker fingerprinting deployments. Responders that consider their supported task set sensitive **SHOULD** authenticate the discoverer before responding, and **MAY** return a filtered subset of their true capabilities (or no response at all) when the discoverer is unknown or unauthenticated.

A discovery query that names specific patterns also leaks information about what the discoverer wants. The grammar is therefore deliberately coarse — a discoverer asking for `acl/*` does not commit to any particular version or specific task within the namespace.

## Request

A *request* document carries `type: https://trusttasks.org/spec/trust-task-discovery/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Discover everything

```json
{
  "id": "urn:uuid:6e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1",
  "issuer": "did:web:client.example",
  "recipient": "did:web:server.example",
  "issuedAt": "2026-06-20T10:00:00Z",
  "payload": {}
}
```

### Discover the ACL family

```json
{
  "id": "urn:uuid:7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1",
  "issuer": "did:web:client.example",
  "recipient": "did:web:server.example",
  "issuedAt": "2026-06-20T10:00:00Z",
  "payload": {
    "patterns": ["acl/*"]
  }
}
```

### Mixed query — one family plus one exact slug

```json
{
  "id": "urn:uuid:8e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1",
  "issuer": "did:web:client.example",
  "recipient": "did:web:server.example",
  "issuedAt": "2026-06-20T10:00:00Z",
  "payload": {
    "patterns": ["acl/*", "kyc-handoff"]
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/trust-task-discovery/0.1#response` with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

### Response to "discover everything"

```json
{
  "id": "urn:uuid:9e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1#response",
  "threadId": "urn:uuid:6e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:server.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-06-20T10:00:01Z",
  "payload": {
    "supportedTypes": [
      "https://trusttasks.org/spec/acl/change-role/0.1",
      "https://trusttasks.org/spec/acl/grant/0.1",
      "https://trusttasks.org/spec/acl/list/0.1",
      "https://trusttasks.org/spec/acl/revoke/0.1",
      "https://trusttasks.org/spec/acl/show/0.1"
    ]
  }
}
```

### Response to the ACL-only query

```json
{
  "id": "urn:uuid:ae2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1#response",
  "threadId": "urn:uuid:7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:server.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-06-20T10:00:01Z",
  "payload": {
    "supportedTypes": [
      "https://trusttasks.org/spec/acl/change-role/0.1",
      "https://trusttasks.org/spec/acl/grant/0.1",
      "https://trusttasks.org/spec/acl/list/0.1",
      "https://trusttasks.org/spec/acl/revoke/0.1",
      "https://trusttasks.org/spec/acl/show/0.1"
    ]
  }
}
```

### Response declaring framework version and `requiredExt` policy

A responder that requires the `vnd.affinidi.webvh` `ext` namespace on every `acl/grant` document advertises it via the expanded entry form. Discoverers see the requirement up front and can satisfy it on the next exchange:

```json
{
  "id": "urn:uuid:ce2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1#response",
  "threadId": "urn:uuid:7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:server.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-06-20T10:00:01Z",
  "payload": {
    "frameworkVersion": "0.1",
    "supportedTypes": [
      {
        "type": "https://trusttasks.org/spec/acl/grant/0.1",
        "requiredExt": ["vnd.affinidi.webvh"]
      },
      "https://trusttasks.org/spec/acl/list/0.1",
      "https://trusttasks.org/spec/acl/show/0.1"
    ]
  }
}
```

The shorthand and expanded forms coexist in the same array; `acl/grant` carries the `requiredExt` annotation because the responder enforces it, the other two specs have no annotations to declare so they ride as bare strings.

### Empty response — responder supports nothing in the queried namespace

```json
{
  "id": "urn:uuid:be2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.1#response",
  "threadId": "urn:uuid:8e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:server.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-06-20T10:00:01Z",
  "payload": {
    "supportedTypes": []
  }
}
```

An empty `supportedTypes` array is a valid response and means "I support nothing matching your query." It is **not** equivalent to an error — the exchange was processed successfully, the responder simply has nothing to offer.

## Security & Privacy

See the "Privacy considerations" section above. The responder's framework-level identity authentication ([SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)) still applies — a `recipient` mismatch or transport-identity-mismatch on either side of the exchange is a framework-level rejection per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements), unrelated to anything specific to this specification.

## References

* [SPEC.md](../../../SPEC.md) — Trust Tasks framework specification.
