---
slug: keys/export-secret
version: "0.1"
title: "Keys Export Secret"
summary: "Release the private half of one named key to a caller entitled to it."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords: [keys, export, secret, custody, key-management]
parties:
  - role: "The party asking for the key material"
    requirement: REQUIRED
    member: issuer
  - role: "The custodian holding the key"
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is the sole evidence of who asked for a private key, and RECOMMENDED
    would make an unsigned ask sufficient on a task whose answer is key material. The
    response is the custodian's own record of what it released, to whom and when — the
    only durable account of a secret leaving, and the thing an incident review reads.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request to release key material is exactly the document worth replaying, and one
    with no issuedAt cannot be placed in the bounded window SPEC §7.2 item 11 needs to
    absorb a duplicate. It is also what dates the release in the custodian's record.
sideEffects:
  level: none
  rationale: >-
    Reads existing key material and changes nothing: no key is created, rotated, revoked
    or re-derived, and a second identical request returns the same material. The
    consequence of this task is disclosure, which `exposure` carries — not mutation.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The response is a private key in the clear. Whatever receives one can act as that key
    until it is rotated. The request carries only an identifier.
retention:
  class: durable
  rationale: >-
    A caller asks for key material because it intends to hold and use it, so what it
    receives is durable by design rather than by omission. Naming the class is what makes
    the obligation concrete: durable custody of somebody else's secret, held to the
    standard the custodian holds its own.
errorCodes:
  - code: keys:notFound
    meaning: No key record on this custodian carries the named `keyId`.
    retryable: false
  - code: keys/export-secret:notExportable
    meaning: The key exists and the caller is entitled to it, but it is marked as not releasable. A refusal about the key rather than the caller — retrying with more authority does not change it, though a party with authority beyond the one that imposed the restriction can lift it first.
    retryable: false
  - code: keys/export-secret:neverExportable
    meaning: The key's private half is never released by this custodian under any circumstances or any authority — key material that exists only inside it, and is usable but not extractable. Distinct from notExportable, which names a decision that can be reversed; this one names a property that cannot.
    retryable: false
related:
  - keys/set-exportability
  - keys/show
  - keys/create
---

## Abstract

A custodian holds private keys and uses them on request — signing, key agreement — so
that the material never has to leave. Sometimes it does have to leave: an operator taking
a backup of one key, a service being handed the identity it will run as, a migration to a
different custodian. This task is that release, one key at a time.

It exists as its own specification because the alternative is worse in a specific way.
Where releasing a key is a side effect of some larger operation, there is no single point
at which the custodian decided to release it — nothing to refuse, nothing to record as one
act, and nothing for a policy about key custody to attach to. Naming the release makes it
a decision.

One key per request, deliberately. A batch form would make a hundred releases one
decision, and the whole value of this shape is that each one is its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the invariants in [the category conventions](../../_shared/0.1/CONVENTIONS.md).

## Authorization

The authority is **standing over the key's scope**: the custodian has recorded this
producer as entitled to act on the scope the key belongs to. A producer without it is
refused with the framework's `permissionDenied`.

**Entitlement is necessary and not sufficient.** A conforming consumer applies two further
checks that no amount of authority satisfies:

- A key whose material is **never extractable** — generated inside the custodian and
  reproducible nowhere — is refused with `neverExportable`, to every caller including the
  most privileged. A custodian that made an exception for a sufficiently authorised caller
  would be describing a different key than the one it stored.
- A key marked **not releasable** (see [`keys/set-exportability`](../../set-exportability/0.1/spec.md))
  is refused with `notExportable`. That refusal is about the key, not the asker: a caller
  cannot satisfy it by presenting more authority, only by having the restriction lifted
  first — which is a separate act, deliberately harder than imposing it was.

Both are stated as consumer requirements rather than left to policy because the point of
each is that it holds *regardless* of policy. A conforming consumer **MUST** perform them
after establishing entitlement and before assembling a response.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the `issuer`, the
transport identity or the `proof` establishes *who asked* and *that the request is
unaltered*; none of it establishes that this asker may hold this key.

## Definitions

**`keyId`** — the key to release. Chosen by the producer from what it can already see via
`keys/list` or `keys/show`. Custodian-scoped; there is no globally meaningful form.

**`keyType`** — what the key is for, from the shared enumeration: `ed25519` signs,
`x25519` performs key agreement and never signs, `p256` signs with ES256.

**`publicKeyMultibase`** — the public half. Chosen by the custodian, and returned so a
consumer can confirm the pair agrees before installing it.

**`privateKeyMultibase`** — the private half, multibase-encoded over multicodec-prefixed
bytes. The prefix identifies the material, so a consumer decodes by prefix rather than
trusting `keyType` alone: the two disagreeing is a custodian bug, and silently believing
the wrong one installs a key that will fail at its first use rather than here.

## Request

The producer is the party asking; the recipient is the custodian holding the key. The
payload names one key and carries nothing else — see the top-level schema in
[`payload.schema.json`](payload.schema.json).

### Asking for one key by identifier

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/keys/export-secret/0.1#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "keyId": "did:example:issuer#key-0"
  }
}
```

## Response

The producer of the response is the recipient of the request. The payload is the key
material, reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` rather than
a `#response` document — including both refusals above, which are **not** empty successes.

A consumer **MUST NOT** answer a refused request with a response document carrying an
absent or empty `privateKeyMultibase`. A producer reading such a document has no way to
tell a withheld key from a released one, and the shape it would most likely take — an
empty string where a key belongs — is the shape most likely to be installed and used.

### The key material

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/keys/export-secret/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "keyId": "did:example:issuer#key-0",
    "keyType": "ed25519",
    "publicKeyMultibase": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "privateKeyMultibase": "z3u2en7t5LR2WtQH5PfFqMqtVcSdd7ELrcFtnP63HKq4KLg"
  }
}
```

## Security & Privacy

### Data carried

The request carries an identifier. The response carries **a private key in the clear**,
which is the whole of this task's risk.

- The exchange **MUST** be carried over a channel confidential to the two parties and
  authenticating both. There is no minimisation available in the payload — a private key
  is either disclosed or it is not — so the protection is entirely in the carrier and in
  the checks above.
- A response **MUST NOT** be logged, cached in a shared store, or included in a diagnostic
  bundle. `privateKeyMultibase` is the member most likely to reach a log by accident,
  because it is a string in an otherwise routine-looking document.
- A producer **MUST NOT** put key material in `keyId` or `ext`.

### Correlation

The custodian learns which keys a party wants held outside it, and when. To an observer of
the channel, a sequence of these requests names the keys being extracted, in order —
which, for a custodian holding keys for several identities, is a map of which identities
are being moved or backed up. The `keyId` cannot be varied: it is the request. What a
producer can vary is *timing* and *batching* — exporting a set of keys in one burst
publishes the shape of that set more clearly than exporting them as each is needed.

### Retention

The consumer keeps the material for as long as it needs the key, and
`retention.class: durable` says so. Durable custody of somebody else's secret carries the
obligations the custodian's own storage carries: at-rest protection, no plaintext copies
outside it, and destruction when the key is rotated or the need ends. A consumer that
exported a key for a one-off migration **MUST** destroy its copy when the migration is
done rather than keep it against a future need.

The custodian **SHOULD** retain the request and response documents — without the key
material — as the record of what was released to whom and when. That record is why
`proofRequirement.response` is REQUIRED.

### Consent/purpose

The purpose is the one the producer asked for: holding or moving this key. Material
obtained here **MUST NOT** be re-disclosed — a consumer that has received a key is not
thereby a source of it for anything else, and re-export is the step that turns one
authorised release into an uncontrolled one. Descriptive only: per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT**
declare that consent, approval or a step-up is required before a custodian releases a key.
