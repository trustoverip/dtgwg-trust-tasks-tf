---
slug: did-management/agent-name/check
version: "0.1"
title: DID Management — Check Agent Name
summary: A prospective claimant asks a hosting service whether an agent name (`/@alice`) is free to claim on a hosting domain — a read-only availability probe that distinguishes taken, reserved, and free.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, availability, check]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Prospective claimant
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: A read-only availability probe consumed over an authenticated transport; the answer is advisory and immediately stale, so there is no evidentiary record worth retaining. The evidentiary record of a claim is the subsequent agent-name/update.
sideEffects:
  level: none
  rationale: "Read-only probe; no reservation is made and no state changes. Unlike did/check-name, this task has no reserve mode — claiming happens only via agent-name/update."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/check:invalidName
    meaning: "The submitted `name` violates the agent-name grammar (length bounds, character set). Grammatical invalidity is an error, not unavailability — a reserved-list hit is reported in-band via `reserved: true`."
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/update, did-management/agent-name/list, did-management/did/check-name]
---

## Abstract

The **DID Management — Check Agent Name** Trust Task asks a hosting service whether an agent name is free to claim on a hosting domain. It is the agent-name analogue of [`did/check-name`](../../../did/check-name/0.1/spec.md)'s pure probe mode — but unlike that task it has **no reserve mode**: an agent name is only ever claimed via [`agent-name/update`](../../update/0.1/spec.md), whose `didData` requirement cannot be satisfied by a probe.

The response distinguishes three outcomes a UI needs to explain, not merely report:

- **free** — `available: true, reserved: false`; the caller may proceed to claim it.
- **taken** — `available: false, reserved: false`; bound to a DID on this domain (including *parked* bindings, which keep their reservation).
- **reserved** — `available: false, reserved: true`; on the host's reserved list (`admin`, `support`, …) — a well-formed name no tenant may claim, distinct from a grammar error (which is rejected with `invalidName`).

The community name (the domain's bare `/@`) reports as reserved: the probe carries no mnemonic, so it cannot know whether the asker is the root DID — and the honest answer to "can I claim this" is no for every caller but one, whose binding happens at provisioning rather than through this surface.

Availability is **domain-scoped**: the same name may be free on one domain and taken on another, so the response echoes the domain the probe was evaluated against after [domain resolution](../../../_shared/0.1/CONVENTIONS.md#1-domain-resolution).

## Status of this Document

Draft.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/agent-name/check/0.1` with `payload.name` (bare local part; a consumer strips one leading `@` from a lenient client before evaluation) and optional `payload.domain`. The consumer resolves the domain, validates the name's grammar (rejecting with `invalidName`), and answers from the current binding registry without mutating any state. The answer is advisory — it carries no reservation, and the name may be claimed by another party between the probe and a subsequent claim.

## Request

```json
{ "id": "chk-1", "type": "https://trusttasks.org/spec/did-management/agent-name/check/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "name": "alice", "domain": "did.example.com" } }
```

## Response

```json
{ "id": "chk-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/check/0.1#response",
  "threadId": "chk-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "name": "alice", "domain": "did.example.com", "available": true, "reserved": false } }
```

## Security & Privacy

- **Enumeration.** The probe necessarily discloses whether a name is bound on a domain — the same fact the public `/@name` redirect discloses to anyone. A host SHOULD nonetheless rate-limit probes per caller, since bulk enumeration of the bound-name namespace is a reconnaissance primitive the redirect surface does not offer at volume.
- **No TOCTOU guarantee.** `available: true` is not a hold. A consumer MUST NOT treat a probe as authorisation to serve the name; the claim is arbitrated solely by `agent-name/update`.
