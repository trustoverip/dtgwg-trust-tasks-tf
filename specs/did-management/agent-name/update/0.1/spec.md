---
slug: did-management/agent-name/update
version: "0.1"
title: DID Management — Update Agent Name
summary: Set the binding state of a human-memorable agent name (`/@alice`) on a hosted DID — `active` binds, refreshes, or resumes it; `parked` stops it resolving while keeping the reservation. Supersedes the separate set / enable / disable verbs.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, alsoKnownAs, redirect, state, park]
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
  rationale: A binding-state transition is an evidentiary change — the name becomes (or ceases to be) publicly resolvable to the DID — so the maintainer retains a signed record. The submitted `didData` is itself signed, but the request that instructs the host to serve or park the redirect is a separate authorization worth retaining.
sideEffects:
  level: mutating
  rationale: "Transitions a name→DID binding between `active` and `parked`. Both directions are reversible with another update; the reservation itself is never released by this task (that is agent-name/remove, which stays destructive)."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/update:notOwner
    meaning: The caller is not the DID slot's current owner or an admin.
    retryable: false
  - code: did-management/agent-name/update:nameReserved
    meaning: The requested name is on the host's reserved list (e.g. `admin`, `support`) and cannot be claimed by a tenant.
    retryable: false
  - code: did-management/agent-name/update:nameTaken
    meaning: The name is already bound to a different DID on this hosting domain.
    retryable: false
  - code: did-management/agent-name/update:notFound
    meaning: "`state: parked` was requested for a name that is not bound to this DID. Parking requires an existing binding; there is no reservation to keep."
    retryable: false
  - code: did-management/agent-name/update:alsoKnownAsMismatch
    meaning: "The submitted `didData`'s `alsoKnownAs` does not match the requested state: it must claim the name for `active` and must not claim it for `parked`."
    retryable: false
  - code: did-management/agent-name/update:invalidDidData
    meaning: The submitted `didData` failed proof or structural validation for the target DID.
    retryable: false
  - code: did-management/agent-name/update:stepUpRequired
    meaning: The operation requires a higher authentication assurance level (operator step-up) that has not been satisfied. Parking a name takes it out of service and is a consumer-gated operation.
    retryable: true
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/remove, did-management/agent-name/check, did-management/agent-name/list, did-management/did/register]
---

## Abstract

The **DID Management — Update Agent Name** Trust Task sets the binding state of a human-memorable **agent name** on a hosted DID. An agent name is a URL whose path begins with `/@` (`example.com/@alice`) and which resolves — by HTTP redirect — to a DID.

The task is declarative: the payload names the **desired state**, not a transition.

- **`state: active`** — the name is bound to the DID and resolves. This one value covers what were previously three situations with two tasks: binding a free name, refreshing an existing binding (the old `agent-name/set`), and resuming a parked one (the old `agent-name/enable`).
- **`state: parked`** — the name stops resolving, but the host keeps it reserved to this DID so nobody else can claim it (the old `agent-name/disable`).

Requesting the state the binding is already in succeeds as an idempotent refresh — there is no `alreadyEnabled` / `notDisabled` error class, because a declarative request for the current state is not a fault.

This task supersedes [`agent-name/set`](../../set/0.1/spec.md), [`agent-name/enable`](../../enable/0.1/spec.md), and [`agent-name/disable`](../../disable/0.1/spec.md), whose payloads were field-for-field identical and differed only in the verb.

## Why `remove` is not a state of this task

[`agent-name/remove`](../../remove/0.1/spec.md) remains its own task, deliberately. Releasing a name is **destructive**: once free, the name may be claimed by a different DID, and the previous holder cannot unilaterally get it back. Both states of *this* task are `mutating` — each is reversible by another update. Folding `remove` in as a third state (`released`) would force the merged task to carry the maximum side-effect class of its members, making every routine rename or park look destructive to a consent surface, and would erase the audit distinction between "the owner parked their name" and "the owner released the name into the free pool". The framework's side-effect classification exists precisely so that consumers can gate the destructive class differently; keeping the destructive verb on its own slug keeps that gate — and its audit trail — separately addressable.

## Status of this Document

Draft.

## The `alsoKnownAs` invariant

An agent name is only meaningful if the DID it points at claims it back: a resolved DID Document's `alsoKnownAs` MUST contain the agent name, and a resolver that finds otherwise rejects the name. The submitted `didData` must therefore agree with the requested state, and the two effects land in one commit:

- **`active`** — `didData`'s `alsoKnownAs` MUST contain the requested name (canonicalised — see [`agent-name/set` 0.1 §Canonicalisation](../../set/0.1/spec.md#canonicalisation), which this task inherits unchanged); the host publishes `didData` as a new version of the DID **and** records/refreshes the binding in the same commit.
- **`parked`** — `didData`'s `alsoKnownAs` MUST NOT contain the name; the host publishes `didData` and marks the binding parked in the same commit, keeping the reservation.

Either direction of disagreement is rejected with `alsoKnownAsMismatch`. There is no window in which the served state and the document's claim diverge.

## Request

### Bind (or refresh, or resume) a name

```json
{ "id": "upd-1", "type": "https://trusttasks.org/spec/did-management/agent-name/update/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "mnemonic": "alice", "name": "alice", "state": "active",
    "didData": "<jsonl: a new signed log entry whose document alsoKnownAs contains example.com/@alice>" } }
```

### Park a name

```json
{ "id": "upd-2", "type": "https://trusttasks.org/spec/did-management/agent-name/update/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-21T09:00:00Z",
  "payload": { "mnemonic": "alice", "name": "alice", "state": "parked",
    "didData": "<jsonl: a new signed log entry whose document alsoKnownAs no longer contains example.com/@alice>" } }
```

`payload.name` is the name's **local part**, without the leading `@` — the `alice` in `/@alice`. The hosting domain comes from the DID, not this field.

## Response

```json
{ "id": "upd-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/update/0.1#response",
  "threadId": "upd-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-07-01T10:00:00Z", "updatedAt": "2026-07-20T09:00:01Z",
    "versionCount": 2, "domain": "did.example.com", "disabled": false } } }
```

Note `record.disabled` refers to the **DID slot**, not the name — a DID with an active slot can carry a parked name. Per-name state is read back via [`agent-name/list`](../../list/0.1/spec.md).

## Security & Privacy

- **Reserved names.** A host MUST maintain a reserved list and reject `active` requests for names on it with `nameReserved`. Names such as `admin`, `support`, `security`, `abuse` are what a victim expects to belong to the operator; letting a tenant claim them is a ready-made phishing primitive.
- **Cross-DID collision.** A host MUST reject an `active` request for a name already bound to another DID on the same domain (`nameTaken`) rather than silently re-pointing it.
- **Parking disrupts dependents.** `state: parked` takes a live name out of service. A consumer SHOULD gate the parking direction behind operator step-up and respond `stepUpRequired` until satisfied — a consumer policy derived from the operation's impact, not a registry-enforceable flag.
- **Parked names occupy the namespace.** A parked name remains reserved indefinitely; a host applying quotas SHOULD count parked names against a tenant's allocation.
- **The document is the authority.** A conformant resolver re-verifies `alsoKnownAs` itself; the host is a cache of that authorisation, not the authority for it.
