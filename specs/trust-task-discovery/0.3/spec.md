---
slug: trust-task-discovery
version: "0.3"
title: Trust Task Discovery
summary: Query a Trust Task party for the set of task types it supports, and the acceptance window it applies to them.
status: draft
targetFrameworkVersion: "0.6.0"
category: framework
keywords:
  - discovery
  - framework
  - protocol-negotiation
  - capability-exchange
  - freshness
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
  requirement: RECOMMENDED
  rationale: >-
    A discovery response steers everything the discoverer does next — which task
    it issues, to which party, and which `ext` namespaces it attaches — so an
    unattributable one is a redirection surface in the same sense
    [trust-task-next-step](../../trust-task-next-step/0.1/spec.md) describes. The
    transport does not close that on its own, and the earlier claim that the
    parties "have already authenticated through the transport" is not true of
    every binding: the HTTPS binding's own security profile records that it gives
    no end-to-end integrity and that re-origination by any TLS-terminating
    intermediary is possible and undetectable, so a response arriving over it may
    have been rewritten between responder and discoverer. RECOMMENDED rather than
    REQUIRED because the exchange is consumed immediately and retained by nobody,
    so the [SPEC.md §4.7.1](/SPEC.md#471-when-to-include-a-proof) condition that
    makes a proof mandatory does not arise — and because the Conformance rules
    below forbid acting on a response whose origin is authenticated neither
    in-band nor by the transport, which is what a discoverer over such a binding
    is otherwise left holding.
sideEffects:
  level: none
  rationale: "Read-only query for the task types a party supports."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    The response enumerates `supportedTypes` — every Trust Task specification
    the responder implements, optionally with the `ext` namespaces it enforces,
    the framework version it targets, and the acceptance window it applies as a
    consumer. Privacy considerations below already
    calls that "useful for an attacker fingerprinting deployments"; a capability
    enumeration is exactly what `metadata` names, and `none` under-stated it.
errorCodes: []
related: []
---

## Abstract

The **Trust Task Discovery** exchange lets one party ask another which *Trust Task specifications* it can act upon. The query carries an optional list of slug-glob patterns; the response carries the matching subset of *Type URIs* the responder supports and, optionally, the acceptance window the responder applies to the `issuedAt` of documents it receives.

Discovery is a framework-defined meta-task: it exists so that a *consumer* and a *producer* can negotiate a shared task vocabulary before committing to any single specification. The slug `trust-task-discovery` is reserved by [SPEC.md §6.1](/SPEC.md#61-type-uri) under the framework's `trust-task-` namespace, alongside `trust-task-error`, `trust-task-ok`, and `trust-task-next-step`.

## Status of this document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A *conforming responder* **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-discovery/0.3#response` and whose `payload` validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.
2. Populate `supportedTypes` with the subset of *Type URIs* the responder can act upon that match **at least one** of the request's patterns. When the request supplied no patterns (the array is absent or empty), the responder treats the query as if it had carried the single pattern `"*"`.
3. List bare *Type URIs* only — no `#request` or `#response` fragment. Each listed *Type URI* implicitly covers both directions of that specification's exchange.
4. Avoid duplicate entries by *Type URI* — regardless of whether the entry is in shorthand string form or expanded object form.

Each `supportedTypes` entry **MAY** be either:

* **Shorthand** — a bare *Type URI* string. Semantically equivalent to an expanded form with no capability annotations.
* **Expanded** — an object with `type` (the bare *Type URI*) and optional capability annotations. The schema defines two. `acceptanceWindow` states the window the responder applies to documents of that *Type URI* where it differs from the response-level one ([Acceptance window](#acceptance-window)). `requiredExt` is for declaring `ext` namespaces the responder requires on inbound documents per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member) and [SPEC.md §7.2](/SPEC.md#72-consumer-requirements). A *responder* that enforces `ext` policy **SHOULD** advertise it here so *producers* can satisfy the policy before issuing the task. Advertising it asks for nothing: `requiredExt` is a claim about the responder's own policy and confers no entitlement to the data, so a discoverer that cannot already populate the named namespace under its own policy declines rather than gathers (see the discoverer rules below).

A *conforming responder* **SHOULD**:

* Sort `supportedTypes` lexicographically by *Type URI* for deterministic output.
* Include the bare-string shorthand for every entry that has no capability annotations — the expanded form is only useful when at least one annotation is present.
* Set `frameworkVersion` to the MAJOR.MINOR.PATCH of the framework specification ([SPEC.md](/SPEC.md)) the responder targets — the same three-part value a *Trust Task specification* declares as its target framework version ([SPEC.md §5.1.1](/SPEC.md#511-versioning-of-this-framework-specification)) — so a discoverer can reason about compatibility per [SPEC.md §5.2](/SPEC.md#52-compatibility-rules) and knows the exact framework release, `PATCH` included, the responder's documents are validated against.
* Include specifications whose `targetFrameworkVersion` is older than the discoverer's framework version where it has a choice — listing those permits forward-minor-compatible processing.
* Advertise the acceptance window it applies, as [Acceptance window](#acceptance-window) describes, where it applies one to `issuedAt`.

A *conforming discoverer* **MUST**:

* Apply the discoverer rules of [Acceptance window](#acceptance-window) to any `acceptanceWindow` it acts on.

* Treat the response as advisory: a Type URI's presence is a hint that the responder will accept a *Trust Task document* of that type, not a binding promise. The full [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) pipeline still applies to every subsequent exchange.
* **MUST NOT** act on a discovery response whose origin it can authenticate neither in-band (via `proof`) nor from the transport. This mirrors [trust-task-next-step](../../trust-task-next-step/0.1/spec.md) consumer rule 3, and for the same reason: an unauthenticated response that *narrows* what the discoverer will send next is indistinguishable from an injected one, and a discoverer that acts on it has had its whole subsequent behaviour chosen for it. Acting on it means letting the response influence what is issued next — reading it and discarding it is not acting on it.
* Treat `requiredExt` as **untrusted input**. It is a responder's assertion about its own local policy, carried on a document that may be unproven, and a discoverer **MUST NOT** let it cause a producer to populate an `ext` namespace with data it would not otherwise have sent. A `requiredExt` naming a namespace the producer does not already populate under its own policy is a reason to decline the exchange, never a reason to gather and attach the data.

A *conforming discoverer* **MAY**:

* Issue further discovery queries with narrower patterns as its understanding of the responder's capabilities evolves.

## Acceptance window

A *consumer* refuses a document whose `issuedAt` lies outside its acceptance window ([SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 13), and the framework leaves that window to the consumer's own policy. A *producer* that sends a document at once does not need to know it. A *producer* that holds a document before delivery does need to know it. That includes store-and-forward delivery through a mediator, a retry with backoff, an escalation to another transport, and a queue replayed after a restart. Without the window, the producer cannot tell a document the consumer will still accept from one it will refuse as `expired`, and so cannot tell when to issue a new attempt ([SPEC.md §8.4](/SPEC.md#84-retry-semantics)) in its place. The only other way is for both ends to share a constant, which holds until the first consumer configured differently.

The `acceptanceWindow` member lets a *responder* state its window. It carries two members, both in whole seconds:

| Member | Meaning |
|---|---|
| `maxAgeSeconds` | The greatest age of a document's `issuedAt`, before any tolerance, at which the responder still accepts it. At least 1. |
| `clockSkewSeconds` | The clock-skew tolerance the responder applies, in both directions ([SPEC.md §4.2](/SPEC.md#42-top-level-members)). Zero when it applies none. |

The responder refuses a document received at `now` when `now > issuedAt + maxAgeSeconds + clockSkewSeconds`, and when `issuedAt > now + clockSkewSeconds`. The window governs `issuedAt` only. `expiresAt`, where present, is honoured independently ([SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 4), and an advertised window never extends acceptance past it.

The member appears in two places. At the top level of the response, it is the window for every listed *Type URI* whose entry carries none of its own. On an expanded `supportedTypes` entry, it is the window for documents of that *Type URI*, and it takes precedence over the top-level one. The window a discoverer applies to a *Type URI* is therefore the entry's, if the entry carries one, and otherwise the response's.

A *responder* that includes `acceptanceWindow`:

1. **MUST** state a window no wider than the one it applies. Neither `maxAgeSeconds` nor `clockSkewSeconds` may exceed the value the responder enforces for the documents it covers. A wider claim makes producers deliver documents the responder then refuses. A narrower one only costs producers the new attempts they did not need, so a responder that applies different windows to different documents it covers **SHOULD** state the narrowest of them, or state each on its own entry.
2. **MUST NOT** treat what it advertised as binding on itself. The window is a description of current policy. A document is accepted or refused under the window the responder applies when the document arrives.

The member is optional, and a responder that considers its window sensitive **MAY** omit it (see [Correlation](#correlation)).

A *discoverer* that acts on an `acceptanceWindow`:

1. **MUST** have authenticated the response's origin, as the discoverer rules above require of any response it acts on.
2. **SHOULD** use the window to decide whether a document it holds can still be accepted. It **SHOULD NOT** send a document after `issuedAt + maxAgeSeconds`, and **MUST NOT** send one after `issuedAt + maxAgeSeconds + clockSkewSeconds`. It **SHOULD** issue a new attempt in its place: a new document with a fresh `id` and a fresh `issuedAt`, carrying the original's `threadId` (or the original's `id` where the original opened the thread) and a new `proof` where the original carried one.
3. **MUST NOT** re-send the original with a re-stamped `issuedAt` under the original's `id`. That is a different document under a reused `id`, which [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 11 requires the consumer to reject with `idConflict` ([SPEC.md §8.4](/SPEC.md#84-retry-semantics)).
4. **MUST NOT** issue a new attempt of a *consequential Trust Task* unless a second execution is harmless, or unless the consumer will recognize the new attempt as the same operation, for example by an idempotency mechanism the ecosystem defines and the consumer honours. A new attempt has a fresh `id`, so item 11 does not recognize it as a repeat. For a task whose repeat leaves a second effect, a new attempt without such a mechanism is a second request.
5. **MUST NOT** set `issuedAt` to anything but the time the document is produced (coarsened as [SPEC.md §13.1](/SPEC.md#131-identifier-correlation-and-linkability) permits) in order to fit a window. Backdating or post-dating a document to a window is not a new attempt; it misstates when the document was made.
6. **SHOULD** bound the number of new attempts it issues for one operation, whatever the advertised window. A window that is small, or that a responder claims falsely, would otherwise cost the discoverer a signature per window for as long as it keeps trying.
7. **MUST** treat a refusal as the answer. A document the consumer refuses as `expired` is refused whatever was advertised. A discoverer that receives such a refusal for a document it expected to be in the window **SHOULD** ask for discovery again before relying on a cached window.

**When the member is absent** from both the entry and the response, the discoverer has learnt nothing about the window for that *Type URI*. It **MUST NOT** read the absence as the absence of a window, or as a window without bound. It falls back to the framework's defaults. The framework fixes no maximum age, which [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 13 leaves to the consumer, so the discoverer uses the window the task's own specification states under [SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 17 where it states one, and its own configured assumption otherwise. For skew it uses the tolerance [SPEC.md §4.2](/SPEC.md#42-top-level-members) names as typical, sixty seconds or less. A 0.2 or 0.1 response never carries the member, so the same fallback applies to it.

Whole seconds, not an [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) duration, because the only use of the value is arithmetic on an instant. A duration admits months, whose length depends on the instant, and it has to be parsed before it can be compared.

## Relationship to 0.2 and 0.1

0.3 adds one thing: the optional `acceptanceWindow` member, at the top level of the response and on an expanded `supportedTypes` entry ([Acceptance window](#acceptance-window)). 0.2's response is closed (`additionalProperties: false`), so a 0.2 discoverer that validates the response would reject one carrying the member. That, and [SPEC.md §5.2](/SPEC.md#52-compatibility-rules)'s rule that an added optional member is a `MINOR` increment, is why this is a new version rather than an edit. The request payload, the pattern grammar, `supportedTypes`, `frameworkVersion` and the other conformance rules are unchanged.

0.2 changed one thing from 0.1: `frameworkVersion` in the response is the framework's three-part `MAJOR.MINOR.PATCH` ([SPEC.md §5.1.1](/SPEC.md#511-versioning-of-this-framework-specification)), where 0.1's schema admits only `MAJOR.MINOR`.

A *responder* **SHOULD** answer each version it supports in the version it was asked. A 0.1 request gets a 0.1 response, whose `frameworkVersion` is the `MAJOR.MINOR` of the release it targets. A 0.2 request gets a 0.2 response carrying the full three-part value and no `acceptanceWindow`. A 0.3 request gets a 0.3 response. A *discoverer* **SHOULD** ask in 0.3, and **MAY** fall back to 0.2, then 0.1, for a responder that does not list or rejects the later version.

## Pattern grammar

A pattern is a string. The matching rules are:

| Pattern              | Matches                                                                              |
|----------------------|--------------------------------------------------------------------------------------|
| `*`                  | Every slug.                                                                          |
| `<prefix>/*`         | Every slug that starts with `<prefix>/` (e.g. `acl/*` matches `acl/grant`, `acl/revoke`, `acl/grant/sub`). |
| `<slug>` (exact)     | The specific slug. No wildcards interpreted in the middle of the string.             |

A pattern in any other shape is treated as an exact slug match (no wildcards). Wildcards anywhere other than as the trailing segment of a `<prefix>/*` pattern are **not** interpreted; they are matched literally — so a pattern like `acl/*-handoff` matches no slug at all (since slugs cannot contain `*`).

Multiple patterns in the request `patterns` array are combined with **OR** semantics: a slug matches the query if it matches at least one pattern.

The array is bounded at **16** entries. The bound is not a capability limit — a single `*` already returns everything the responder will admit to — it is parser hardening in the manner of [SPEC.md §12.2](/SPEC.md#122-parser-hardening): an unbounded array of patterns, each matched against every published slug, is work a responder performs on an unauthenticated request. A discoverer that genuinely wants more than sixteen namespaces asks for `*` and filters locally.

## Privacy considerations

A discovery response leaks information about which specifications the responder implements — useful for an honest discoverer planning an exchange, useful for an attacker fingerprinting deployments. Responders that consider their supported task set sensitive **SHOULD** authenticate the discoverer before responding, and **MAY** return a filtered subset of their true capabilities (or no response at all) when the discoverer is unknown or unauthenticated.

A discovery query that names specific patterns also leaks information about what the discoverer wants. The grammar is therefore deliberately coarse — a discoverer asking for `acl/*` does not commit to any particular version or specific task within the namespace.

## Request

A *request* document carries `type: https://trusttasks.org/spec/trust-task-discovery/0.3` with a payload that validates against the top-level schema in `payload.schema.json`.

### Discover everything

```json
{
  "id": "urn:uuid:6e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3",
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
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3",
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
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3",
  "issuer": "did:web:client.example",
  "recipient": "did:web:server.example",
  "issuedAt": "2026-06-20T10:00:00Z",
  "payload": {
    "patterns": ["acl/*", "consent/request"]
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/trust-task-discovery/0.3#response` with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

### Response to "discover everything"

```json
{
  "id": "urn:uuid:9e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3#response",
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
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3#response",
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
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3#response",
  "threadId": "urn:uuid:7e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:server.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-06-20T10:00:01Z",
  "payload": {
    "frameworkVersion": "0.6.0",
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

### Response declaring an acceptance window

A responder that accepts a document up to ten minutes after its `issuedAt`, with sixty seconds of skew either way, states that once at the top level. It applies a tighter window to `acl/grant`, and states that on the entry:

```json
{
  "id": "urn:uuid:de2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3#response",
  "threadId": "urn:uuid:6e2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:server.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-06-20T10:00:01Z",
  "payload": {
    "frameworkVersion": "0.6.0",
    "acceptanceWindow": {
      "maxAgeSeconds": 600,
      "clockSkewSeconds": 60
    },
    "supportedTypes": [
      {
        "type": "https://trusttasks.org/spec/acl/grant/0.1",
        "acceptanceWindow": {
          "maxAgeSeconds": 120,
          "clockSkewSeconds": 30
        }
      },
      "https://trusttasks.org/spec/acl/list/0.1",
      "https://trusttasks.org/spec/acl/show/0.1"
    ]
  }
}
```

A discoverer holding an `acl/grant` document issued at `10:00:00Z` sends it until `10:02:00Z`. After `10:02:30Z` the responder refuses it whatever its clock reads within tolerance. Between the two, delivery is uncertain, which is why the rule above says **SHOULD NOT** for the first bound and **MUST NOT** for the second. An `acl/list` document issued at the same instant is good until `10:10:00Z`. After that instant, a discoverer that still has the operation to deliver issues a new attempt instead.

### Empty response — responder supports nothing in the queried namespace

```json
{
  "id": "urn:uuid:be2c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-discovery/0.3#response",
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

### Data carried

The request carries at most sixteen slug-glob patterns and nothing about the discoverer beyond the envelope's own `issuer`. The response carries the matching subset of the responder's supported *Type URIs*, optionally the `ext` namespaces it enforces, optionally the framework release it targets, and optionally the acceptance window it applies. None of it is personal data, but all of it is capability metadata about the responder's deployment — which is why `exposure.discloses` is `metadata` — and the responder chooses how much of it to reveal: it **MAY** return a filtered subset, or nothing, as [Privacy considerations](#privacy-considerations) sets out. Nothing in either direction needs `ext`; a discoverer **SHOULD NOT** attach any.

### Correlation

`frameworkVersion` is now three-part, so it narrows a responder to one framework release, `PATCH` included — finer than 0.1 could express. A responder that considers its exact release sensitive **MAY** omit the member, which is optional in the schema; the RECOMMENDED value in the responder rules above is for a discoverer that needs it. The `supportedTypes` list is itself a fingerprint of the deployment, and a discoverer's patterns reveal what it intends to do next. An `acceptanceWindow` adds to the fingerprint: a window configured away from an implementation's default distinguishes one deployment of it from another, and a per-entry window shows which tasks the operator treated as sensitive enough to narrow. A responder **MAY** omit the member, or state only the response-level window, when that matters more than its producers' delivery. Omitting it costs those producers the ability to re-issue against the real window, and nothing else. The responder's framework-level identity checks ([SPEC.md §4.8.1](/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)) still apply: a `recipient` mismatch or transport-identity mismatch on either side is a framework-level rejection per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements), unrelated to anything in this specification.

### Retention

The exchange is consumed at once and retained by nobody, which is why `proofRequirement` is RECOMMENDED rather than REQUIRED. A discoverer may cache a response to avoid asking again; the response is advisory, so a cached copy is no more authoritative than a fresh one, and a discoverer **SHOULD** re-ask, rather than keep acting on a cached list, when an exchange it chose on the strength of one fails. That includes a document refused as `expired` inside a cached acceptance window, which means the window has changed or was misstated.

### Consent/purpose

The response exists to let the discoverer choose which *Trust Task* to send, how to shape it, and when a document it holds is too old to send. A discoverer **MUST NOT** use it for any other purpose — in particular, `requiredExt` is never a reason to gather data (see the discoverer rules above). Whether to answer an unauthenticated discoverer at all is the responder's policy, not this specification's.

## References

* [SPEC.md](/SPEC.md) — Trust Tasks framework specification.
