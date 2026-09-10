---
slug: vta/contexts/secrets
version: "1.0"
title: "VTA Context Secrets"
summary: "Fetch the private key material for the DID of one trust context, so a service holding that context can operate as it."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords: [vta, contexts, secrets, provisioning, bootstrap, integration]
parties:
  - role: "The service operating the context, asking for the keys of the DID it was provisioned with"
    requirement: REQUIRED
    member: issuer
  - role: "The agent holding the context and its key material"
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is the sole evidence that the party asking is the party the context was
    granted to, and the response hands over key material. RECOMMENDED on the request
    would make an unsigned ask sufficient on a task whose answer is a private key, so the
    floor is REQUIRED on both. The response is evidentiary in the other direction too:
    it is the recipient's own record of what it released, to whom, and when — which is
    the only durable account of a secret leaving.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request to release key material is exactly the document worth replaying, and a
    document with no issuedAt cannot be placed in the bounded window SPEC §7.2 item 11
    needs to absorb a duplicate. It is also what dates the release in the recipient's
    audit record.
sideEffects:
  level: none
  rationale: >-
    Reads existing key material and changes nothing: no key is created, rotated,
    revoked or re-derived, and a second identical request returns the same bundle. The
    consequence of this task is disclosure, which `exposure` carries — not mutation.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The response carries private keys in the clear. A consumer that receives one can
    sign, decrypt and authenticate as the context's DID until those keys are rotated —
    the whole authority of that DID, not a scoped subset of it. The request carries only
    a context id.
retention:
  class: durable
  rationale: >-
    The point of the exchange is that the service keeps the keys and goes on operating
    as the DID across restarts, so what it receives is durable by design rather than by
    omission. Naming it here is what makes the obligation in Security & Privacy →
    Retention concrete: durable custody of a secret, held to the same standard as the
    recipient's own storage.
errorCodes:
  - code: vta/contexts/secrets:notFound
    meaning: No context with this id exists within the caller's entitlement. A caller not entitled to the id is refused for that reason instead, and because entitlement is checked before existence, "not found" is only ever said to a caller already entitled to hear it — so the pair leaks nothing about ids the caller may not reach.
    retryable: false
  - code: vta/contexts/secrets:notReleasable
    meaning: The context is reachable and has key material, but the recipient's policy marks that material as non-releasable. A refusal about the key, not about the caller — retrying as a more privileged caller does not change it.
    retryable: false
related:
  - vta/contexts/get
  - vta/contexts/update-did
  - vta/backup/initiate-export
---

## Abstract

A service that has been provisioned with a trust context — a mediator, a data-room host,
any long-running integration — needs the private keys of that context's DID before it can
do anything as that DID. This task is how it asks for them: one request naming the
context, one response carrying the DID and its key material.

It is a Trust Task rather than an API call because the request is an instruction a service
gives *its own* agent, authorised by the agent having granted that service the context —
and because releasing a secret is the exchange most worth having signed evidence of on
both sides. A bare endpoint would leave the recipient with a log line where it needs a
document it can produce later.

The alternative shapes are both worse. Provisioning the keys into the service out of band
means an operator handling private keys by hand, and a service that can never be
re-provisioned without repeating that. Having the service generate its own keys and
register the public halves means the agent no longer knows what it is delegating to, and
cannot rotate the DID's keys without the service's cooperation. Here the agent generates,
holds and can rotate; the service fetches.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **the producer's standing in the named context**: the recipient has
recorded this producer as a party to that context, and that record is what entitles it to
the context's DID and to nothing else. A consumer **MUST** resolve the producer's
entitlement against the context named in `id`, and **MUST NOT** infer it from the producer
being a known or previously served party — an entitlement to one context says nothing
about another, and a task that returns key material is where that distinction is load
bearing.

This is not identity checking. Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements),
verifying the `issuer`, the transport identity or the `proof` establishes *who asked* and
*that the request is unaltered*; none of it establishes that this asker may hold this
context's keys. Both checks are required and the second is the one this task turns on.

The entitlement is **narrow rather than privileged**, and that is the design. A service
authorised for its own context reaches exactly the DID it was provisioned with; a party
with wide authority over the recipient is not thereby entitled to any context's keys under
this task, because the check is scope, not rank. A consumer that implements the
entitlement as a privilege level rather than as a scope will get this backwards and hand a
broadly-authorised caller every context's key material.

A consumer **MAY** refuse to release key material it holds — see
`vta/contexts/secrets:notReleasable`. Whether any given key is releasable, and what it
takes to change that, is the consumer's policy; per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) this specification does not
declare that consent, human approval or a step-up is required.

## Definitions

**`id`** — the context whose DID's key material is being asked for. Chosen by the
producer, from the context it was provisioned with. For a nested context this is the full
path (`parent/leaf`). A consumer resolves it in its own namespace; there is no globally
meaningful form.

**`did`** — the DID the returned secrets belong to. Chosen by the recipient: it is the DID
recorded on the named context, not something the producer asks for. A producer that
already knows which DID it expects **SHOULD** compare the two and stop if they differ,
because a mismatch means the context's DID has been changed under it (see
`vta/contexts/update-did`) and the keys it is about to install belong to a different
identity than the one it believes it is.

**`secrets`** — one entry per verification method of `did` whose secret the recipient
holds and is willing to release. Chosen by the recipient. The array **MAY** be empty; see
Response.

**`keyId`** — the verification method the key backs, as an absolute DID URL under `did`.
Chosen by the DID document, which is what makes it usable: an inbound message names the
recipient key it was encrypted to by this identifier, so a consumer that installs key
material under a locally-invented name will hold the right key and fail to find it.

**`keyType`** — what the key is for: `ed25519` signs, `x25519` is a key-agreement key and
signs nothing, `p256` is an ECDSA key for ES256.

**`privateKeyMultibase`** — the private key itself, multibase-encoded (Base58BTC) over its
multicodec-prefixed bytes. The prefix identifies the key material, so a consumer decodes
by prefix rather than trusting position or ordering.

## Request

The producer is the service operating the context; the recipient is the agent holding it.
The payload names one context and carries nothing else — see the top-level schema in
[`payload.schema.json`](payload.schema.json).

### A service asks for the keys of the context it was provisioned with

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/contexts/secrets/1.0#request",
  "issuer": "did:example:room-host",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "rooms/host-1"
  }
}
```

## Response

The producer of the response is the recipient of the request. The payload is the sub-schema
reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `did`,
and `secrets` as an array of `{ keyId, keyType, privateKeyMultibase }`. Failures use
`trust-task-error` rather than a `#response` document.

`secrets` **MAY** be empty. A context whose DID has no releasable key material is not a
failure, and a consumer that reports an empty array as an error will misreport a context
that is simply not provisioned yet. The distinguishable cases are: no such
context within the producer's entitlement (`notFound`), a context whose material exists
but is withheld (`notReleasable`), and a context that has none (an empty array). A producer
not entitled to the id it named is refused for *that* reason rather than told the context
does not exist; a consumer **MUST** check entitlement before existence, so that `notFound`
distinguishes nothing for a producer who was never going to be told either way.

A consumer **MUST** check every `keyId` is a verification method of `did` before
installing anything, and **MUST** discard an entry that is not. The two members arrive in
one document from one party, so this is not a defence against the recipient — it is what
keeps a recipient's storage-side naming mistake from being installed as though the DID
document had said it.

### A bundle for a DID with a signing key and a key-agreement key

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/contexts/secrets/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:room-host",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "did": "did:webvh:QmExample:example.com:rooms:host-1",
    "secrets": [
      {
        "keyId": "did:webvh:QmExample:example.com:rooms:host-1#key-0",
        "keyType": "ed25519",
        "privateKeyMultibase": "z3u2en7t5LR2WtQH5PfFqMqtVcSdd7ELrcFtnP63HKq4KLg"
      },
      {
        "keyId": "did:webvh:QmExample:example.com:rooms:host-1#key-1",
        "keyType": "x25519",
        "privateKeyMultibase": "z3wei82HkeQBoBJXbtEbnH3ohvHYqRJmDwtjEZ7Bn5UWMwZ"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a context id and nothing else. A producer **MUST NOT** put key
material, credentials or personal data in it; there is no free-form member for it to
appear in, and `ext` is not one.

The response carries **private keys in the clear**, which is the whole of this task's risk.
Anything that reads one can act as the context's DID until the keys are rotated. So:

- The exchange **MUST** be carried over a channel that is confidential to the two parties
  and authenticates both. There is no minimisation available in the payload itself — a
  private key is either disclosed or it is not — so the protection is entirely in the
  carrier and in the authorization check above.
- A response **MUST NOT** be logged, cached in a shared store, or included in a diagnostic
  bundle. This is the member most likely to reach a log by accident, because it is a
  string in a routine-looking document.
- The smallest response that answers the task is the keys of the named context's DID only.
  A recipient **MUST NOT** include key material belonging to another context or to another
  DID, however convenient it would be for a caller that operates several.

### Correlation

The recipient necessarily learns which context a service is operating and when it started
or restarted, since a fetch is what a cold start looks like. That is unavoidable and
unobjectionable: the recipient is the party that provisioned the context.

To an observer or intermediary, the request and response are correlatable to each other by
`threadId`, and a sequence of them is correlatable to one service by the `issuer`. Neither
identifier can be varied here — the `issuer` is the entitlement, so a producer that varied
it would not be authorised. What a producer can vary is *timing*: a service that fetches
on every restart publishes its restart pattern to anyone watching the channel, and one
that fetches once and holds does not.

### Retention

The consumer keeps the key material for as long as it operates as the DID — that is the
purpose, and `retention.class: durable` says so. Durable custody of a secret carries the
obligations the recipient's own key storage carries: at rest protection, no plaintext
copies outside it, and destruction when the service is decommissioned or the keys are
rotated. A consumer that no longer operates the context **MUST** destroy what it holds
rather than keep it against a future need.

The recipient **SHOULD** retain the request and response documents themselves — without
the key material — as the record of what was released to whom and when. That record is
the reason `proofRequirement.response` is REQUIRED, and it is what makes a later rotation
decision reviewable.

### Consent/purpose

The purpose is operating the named context's DID. Key material obtained here **MUST NOT**
be reused to act as that DID outside that purpose, and **MUST NOT** be re-disclosed — a
consumer that has fetched a bundle is not thereby a source of it for anything else. This
is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a
specification **MUST NOT** declare that consent, approval or a step-up is required, and
whether a recipient demands any of those before releasing is its policy.
