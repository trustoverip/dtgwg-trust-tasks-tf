---
slug: persona/attribute/put
version: "1.0"
title: Persona Attribute — Put
summary: A holder writes one atomic fact about themselves into their agent's attribute pool, declaring where the value came from, and receives back an advisory count of how many of their other profiles already present the same value.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - persona
  - attribute
  - provenance
  - correlation
  - optimistic-concurrency
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A write mutates the holder's own identity data, which every later disclosure projects from. Attribution must survive the transport that carried the request, so an audit record read later names the key that wrote rather than the session it arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A put replaces the value at an address, so two copies applied out of order leave the older value in place. Only the issue time lets the agent refuse the older one.
sideEffects:
  level: mutating
  rationale: "Creates or replaces one attribute. Recoverable — the prior value is overwritten but the address remains, and a conditional write cannot clobber a version it did not see."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: persona/attribute/put:versionConflict
    meaning: The `expectedVersion` precondition failed. The details carry the maintainer's current version and value, so the caller can resolve without a re-read.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["reason"]
      properties:
        reason:
          type: string
          enum: ["versionMismatch", "recordExists", "recordAbsent"]
        currentVersion:
          type: integer
          minimum: 1
        currentValue: {}
  - code: persona/attribute/put:valueTypeMismatch
    meaning: The supplied `value` does not agree with the declared `valueType`.
    retryable: false
  - code: persona/attribute/put:credentialNotFound
    meaning: A `credentialBacked` provenance names a credential the vault does not hold, or holds in a state it cannot be derived from. The attribute is not written — an attribute whose backing cannot be resolved at write time would read back stale forever.
    retryable: false
---

## Abstract

**Persona Attribute — Put** writes one atomic fact a holder keeps about
themselves — a name, a number, an address — into their agent's attribute pool.

Three things distinguish it from a key/value write, and each exists because a
store without it cannot be the source of a controlled disclosure.

The first is **provenance**. Every attribute declares where its value came
from: the holder typed it, a credential in the vault backs it, or it is minted
per verifier at disclosure time. Provenance survives to the verifier, so a
recipient can tell — per field — an assertion from an attestation. A store that
dropped it would make every disclosure equally worthless.

The second is **scope**. The pool is **agent-scoped**, not context-scoped: one
person keeps one set of facts about themselves, above every trust context they
operate in. That is what lets a correlation check see across contexts, which is
the risk it most needs to report and the one a per-context store cannot see by
construction. It is also why this task is holder-authorized and unreachable from
inside a context — see [Authorization](#authorization).

The third is the **advisory correlation count** on the response. Composing an
identity is the moment a holder accidentally links two personas by reusing one
value, and it is also the moment they are best placed to choose otherwise.
Returning the count on the write means a builder can say so while the holder is
still composing, rather than after a second round trip they may not make.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the holder's tooling) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/persona/attribute/put/1.0`, with the holder as `issuer` and the agent as `recipient`.
2. Populate `payload.type`, `payload.valueType`, `payload.value` and `payload.provenance`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming producer **SHOULD** supply `expectedVersion` when the write is a
read-modify-write, and **SHOULD** surface a returned `correlation.severity` of
`high` to the holder before the composition it belongs to is completed.

A conforming **maintainer** (the agent) **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped** — see [Authorization](#authorization).
2. Verify that `value` agrees with `valueType`, and emit `persona/attribute/put:valueTypeMismatch` when it does not.
3. For a `credentialBacked` provenance, resolve the named credential and the `claimPath` within it at write time, and emit `persona/attribute/put:credentialNotFound` when it cannot.
4. Assign an `attributeId` when the producer omitted one, and return it.
5. Encrypt `value` at rest.
6. Apply `expectedVersion` as a precondition when supplied, and on failure emit `persona/attribute/put:versionConflict` carrying the current version and value.

A conforming maintainer **MUST NOT** refuse a write because of its correlation
result. The check is advisory and the holder decides; a store that refused would
be substituting its judgment for the holder's on a question only the holder can
answer.

A conforming maintainer **MUST NOT** index the plaintext of `value`. The
correlation check requires exact-match lookup only, which a keyed hash provides
without a plaintext index; prefix and substring search over values are therefore
outside this family by construction, and that trade is deliberate.

## Authorization

**Holder-authorized and unscoped.** The pool sits above the context boundary, so
a caller holding a context-scoped session — of any role, including an
administrative one — **MUST** be refused.

This is worth stating as a rule about *direction* rather than a permission,
because the permission form invites the wrong implementation. A guard written as
"is this caller an administrator" passes for an administrator scoped to a single
context, who then reads and writes identity data belonging to every other
context. An administrator in one context **MUST** be as powerless over the pool
as an application in that context.

The intended flow is that a holder composes in their own tooling and **pushes** a
materialised projection into a context; a context never pulls from the pool. An
access-control failure over a readable pool discloses everything, whereas a pool
no context can address has nothing to disclose — so the maintainer's obligation
here is structural, not merely a check.

## Request

The payload carries the attribute. `attributeId` is omitted on a create and
supplied on a replace; a producer that means to create and wants the write to be
safe under retry **SHOULD** supply both a generated `attributeId` and
`expectedVersion: 0`, so that a duplicated request fails the precondition rather
than writing twice.

`value` is bounded by the maintainer's per-record cap, which it **MUST** state
and **MUST** enforce loudly. Refusing at a knowable limit is the whole
requirement; the number matters less than that a producer can discover it.

## Response

Returns the assigned `attributeId`, the new `version`, and whether the write
created or replaced.

`correlation` is advisory and **MAY** be omitted by a maintainer that does not
run the check. It carries a severity and a **count** of other profiles already
presenting this exact value — not their identifiers. A producer that needs to
render remedies calls `persona/correlation/analyze`, which is where that
reasoning belongs; returning identifiers here would make every write a
disclosure of the holder's other profiles to whatever tool made it.

## Security & Privacy

### Data carried

The request moves one personal fact: `value`, typed by `valueType` and named by
`type`. `label` is the holder's own note and is personal too. `provenance`
carries a vault identifier and a JSON Pointer when the value is credential-backed
— neither is the credential, but together they name one, so a reader of the
request learns *which* credential backs a claim without being able to read it.

The smallest payload that answers the task is `type`, `valueType`, `value` and
`provenance`. Everything else is the producer's choice. A producer **MUST NOT**
place secret material — a key, a password, a recovery phrase — in `value`, `label`
or `ext`: the credential vault and the secrets vault exist for material that must
never be disclosed, and this store is designed to disclose under control.

### Correlation

The stored value is the correlation surface, and it is the reason this family
exists. A value reused across two profiles links the personas presenting them,
permanently, to anyone who sees both. That risk is created at composition and is
invisible to the holder unless something reports it, which is what the advisory
`correlation` member on the response is for.

The response's count is itself information about the holder's other profiles. It
is a count rather than a list precisely so that a producer learns the risk
without learning which profiles carry it; identifiers require a separate,
equally holder-authorized call. A maintainer computes it over a keyed hash of
the value rather than a plaintext index, so the correlation surface is not
enlarged by the mechanism that reports on it.

The document itself carries no verifier, no counterparty and no disclosure — a
put is a private write, and an observer learns nothing about who the holder
intends to present to.

### Retention

An attribute is durable holder data with no natural expiry: it is retained until
the holder deletes it, and it belongs in a maintainer's backed-up partition,
because an agent restored without it comes back without the identity the restore
was for.

A replaced value is not evidence and a maintainer is under no obligation to keep
it; a maintainer that keeps prior versions to support `pinVersion` **SHOULD**
state that it does, since a holder who overwrites a value may reasonably believe
the old one is gone.

### Consent/purpose

The purpose is composition: a holder assembles facts about themselves so they can
later project selected subsets of them to chosen counterparties. The data is
collected to be disclosed *deliberately*, which is the opposite of collected to
be disclosed.

Reuse beyond that purpose is what the rest of the family constrains. A value
written here is disclosed only through a disclosure task, never by a context
reading the pool, and the record of each disclosure is queryable by the holder.
What gate a producer places in front of a write, or in front of a later
disclosure, is the consumer's policy and is deliberately not specified here.
