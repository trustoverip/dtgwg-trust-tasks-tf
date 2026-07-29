---
slug: did-management/agent-name/set
version: "0.1"
title: DID Management — Set Agent Name
summary: Bind a human-memorable agent name (`/@alice`) to a hosted DID, or update an existing binding. The submitted DID document must claim the name via `alsoKnownAs`.
status: retired
supersededBy: did-management/agent-name/update
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, alsoKnownAs, redirect]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID owner
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Binding a name is an evidentiary state transition — the name becomes publicly resolvable to the DID — so the maintainer retains a signed record. The submitted `didData` is itself signed, but the request that instructs the host to serve the redirect is a separate authorization worth retaining.
sideEffects:
  level: mutating
  rationale: "Adds or updates a name→DID binding and begins serving a redirect; reversible via agent-name/remove or agent-name/disable."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/set:not_owner
    meaning: The caller is not the DID slot's current owner or an admin.
    retryable: false
  - code: did-management/agent-name/set:name_reserved
    meaning: The requested name is on the host's reserved list (e.g. `admin`, `support`) and cannot be claimed by a tenant.
    retryable: false
  - code: did-management/agent-name/set:name_taken
    meaning: The name is already bound to a different DID on this hosting domain.
    retryable: false
  - code: did-management/agent-name/set:also_known_as_mismatch
    meaning: The submitted `didData` does not claim the requested name via `alsoKnownAs`. A host MUST refuse to serve a name the document does not claim.
    retryable: false
  - code: did-management/agent-name/set:invalid_did_data
    meaning: The submitted `didData` failed proof or structural validation for the target DID.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/remove, did-management/agent-name/enable, did-management/agent-name/disable, did-management/did/publish]
---

## Abstract

The **DID Management — Set Agent Name** Trust Task binds a human-memorable **agent name** to a hosted DID, or updates an existing binding. An agent name is a URL whose path begins with `/@` (`example.com/@alice`) and which resolves — by HTTP redirect — to a DID. This task is the hosting side of establishing that redirect.

Idempotent: setting a name already bound to the same DID succeeds and refreshes it. A name already bound to a *different* DID on the same hosting domain is rejected.

## Status of this Document

**Retired.** This task is superseded by [`did-management/agent-name/update`](../../../agent-name/update/0.1/spec.md) with `state: active` (bind or refresh). Consumers SHOULD NOT accept new documents of this type; the specification is retained for auditability of previously-issued documents.

## The `alsoKnownAs` invariant

An agent name is only meaningful if the DID it points at claims it back. The specification's anti-spoofing rule (Layer 1) is that a resolved DID Document's `alsoKnownAs` MUST contain the agent name; a resolver that finds otherwise rejects the name. Anyone can publish a redirect at any DID, so the redirect alone proves nothing — only the DID's controller can add the `alsoKnownAs` entry.

This task therefore carries the **new signed DID document** (`didData`) that claims the name, and the two effects land together:

- the host verifies that `didData`'s `alsoKnownAs` contains the requested name (canonicalised — see below) and rejects with `also_known_as_mismatch` if not;
- the host publishes `didData` as a new version of the DID **and** records the name binding in the same commit.

There is no window in which the name is served but the document does not claim it, and none in which the document claims a name the host is not serving. A host MUST NOT record a binding whose `didData` it did not accept.

## Canonicalisation

The name in `payload.name` and the entry in `didData`'s `alsoKnownAs` are compared after canonicalisation: the authority lowercased, the default port dropped, a trailing slash removed. The **local part is compared case-sensitively** — `@Alice` and `@alice` are different names, because folding them would merge identities a host may have issued separately. A host and a resolver MUST canonicalise identically or a validly-bound name fails to resolve.

## Request

```json
{ "id": "set-1", "type": "https://trusttasks.org/spec/did-management/agent-name/set/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "mnemonic": "alice", "name": "alice",
    "didData": "<jsonl: a new signed log entry whose document alsoKnownAs contains example.com/@alice>" } }
```

`payload.name` is the name's **local part**, without the leading `@` — the `alice` in `/@alice`. The hosting domain comes from the DID, not this field.

## Response

```json
{ "id": "set-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/set/0.1#response",
  "threadId": "set-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-07-01T10:00:00Z", "updatedAt": "2026-07-20T09:00:01Z",
    "versionCount": 2, "domain": "did.example.com", "disabled": false } } }
```

## Security & Privacy

- **Reserved names.** A host MUST maintain a reserved list and reject names on it with `name_reserved`. Names such as `admin`, `support`, `security`, `abuse` are what a victim expects to belong to the operator; letting a tenant claim them is a ready-made phishing primitive.
- **Cross-DID collision.** A host MUST reject a name already bound to another DID on the same domain (`name_taken`) rather than silently re-pointing it.
- **The document is the authority.** A conformant resolver re-verifies `alsoKnownAs` itself; this host is a cache of that authorisation, not the authority for it. A host that serves a name whose current document does not claim it is non-conformant.
