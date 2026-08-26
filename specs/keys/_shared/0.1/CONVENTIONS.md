# keys — category conventions

This document captures invariants that apply to **every** specification under
the `keys/` category. Individual specs reference this file rather than restating
the same prose; conformance is anchored here.

## 1. Family error codes

Three failure conditions mean the same thing in every `keys/*` specification
that can reach them, so they are named once here and carried by the `keys`
**family namespace** — the form [SPEC.md §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)
rule 2 permits for "a code whose meaning is defined once for the whole family".
A consumer can recognise all three from the document's `type` alone, without
enumerating the specs that emit them.

Every specification in this category that can reach one of these conditions
**MUST** list the corresponding code in its `errorCodes` declaration:

```yaml
- code: keys:notFound
  meaning: No key record on this custodian carries the named `keyId`.
  retryable: false
- code: keys:alreadyExists
  meaning: A key record already carries the target identifier; the custodian refuses rather than overwrite it.
  retryable: false
- code: keys:invalidArgument
  meaning: A payload member is well-formed against the schema but unusable for this request — an algorithm the key type cannot perform, or key material that is not a well-formed key of the declared `keyType`.
  retryable: false
```

Codes that only one specification can reach are **not** family codes. §8.5 rule 2
says a family namespace is "never to give a specification-specific code a broader
name than it has earned", so `keys/sign:failedPrecondition` — a key whose `status`
is not `active` — stays namespaced under its own slug.

## 2. Why these are extended codes and not §8.3 standard ones

`keys:alreadyExists` is deliberately **not** the standard `idConflict`. §8.3
defines `idConflict` over the *Trust Task document's* own `id`: it reports a
document whose `id` matches one the consumer has already accepted but whose
content differs, and it exists to keep that case distinguishable from a retry
(see [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 11 and
[§8.4](/SPEC.md#84-retry-semantics)). A `keyId` collision is a *domain*
identifier collision inside a perfectly well-formed, never-before-seen document.
Answering it with `idConflict` would tell the producer its envelope was a
duplicate submission, sending it to inspect `id` and `threadId` rather than to
choose a different key identifier.

`keys:notFound` and `keys:invalidArgument` have no §8.3 equivalent at all.
`malformedRequest` is the nearest neighbour to the second and is still wrong:
it reports a document that failed schema validation, whereas these payloads
validate and fail on semantics the schema cannot express — that this custodian
holds no such key, or that an `ed25519` key cannot perform the named algorithm.

Adding any of these to §8.3 would mean minting a new `trust-task-error` version,
since the standard vocabulary is the `code` enum in that spec's payload schema.
They are task-specific conditions, so the extension mechanism §8.5 provides is
the right one.

## 3. `permissionDenied` is not declared

Refusing a caller who is not authorised to act on the custodian's key set is the
framework's own `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)).
Specs in this category name it in prose and **MUST NOT** declare a namespaced
`keys:permissionDenied` or `keys/<task>:permissionDenied`: §8.5 forbids an
extended code from shadowing a standard one, and a shadowed duplicate is worse
than useless — a consumer switching on the standard code would not match it, and
§8.5's fallback rule would degrade it to `taskFailed`.

## 4. Wire form

Every code above is carried on a `trust-task-error` document per
[SPEC.md §8](/SPEC.md#8-error-responses), never on the task's own `#response`
variant. A bare local part with no namespace — `not_found`, `invalid_argument` —
is not a code at all: `trust-task-error`'s `code` member requires either one of
the §8.3 standard values or a namespaced extended code, so an unnamespaced token
fails payload validation and never reaches the wire.

## 5. Versioning

The category's first release is `0.1` across every constituent spec; versions
evolve independently after that. The category as a whole carries no version
number.
