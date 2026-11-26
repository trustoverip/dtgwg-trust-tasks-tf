# hcp — family conventions

This document records what every specification in the `hcp/*` *protocol family*
([SPEC.md §9.4](/SPEC.md#94-carried-protocols)) shares and states once. The
specifications reference it rather than restating it; where a specification and
this document differ, this document governs the definitions below. The question
numbers are those of [the list of questions put to the HCP editors](QUESTIONS.md).

## 1. Family error codes

HCP 2.0-draft-3 defines nine Core errors (HCP 5.5). Six map to a standard Trust
Task error code, or do not arise over the carriage; each specification's
`## Errors` table says which. Three have no standard equivalent, and mean the same
thing in every `hcp/*` specification that can reach them, so they are named once
here and carried by the `hcp` **family namespace**, the form
[SPEC.md §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)
rule 2 permits for "a code whose meaning is defined once for the whole family".
A consumer can recognise them from the document's `type` alone, without
enumerating the specifications that emit them.

Every `hcp/*` specification that can reach one of these conditions **MUST** list
the corresponding code in its `errorCodes` declaration, with the `meaning` and
`retryable` value given here, unchanged:

```yaml
- code: hcp:invalidToken
  meaning: >-
    HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the access token
    the request presents. A token issued under a revoked grant, a relinquished one included (HCP 5.2.7), fails here
    too: HCP requires that no request presenting evidence issued under a revoked grant succeeds, but names no error
    (HCP 6.2.2), and this family reads that evidence as invalid, pending HCP confirmation (question 2). Such a
    failure is itself a notice of revocation to the client (HCP 6.2.2). Defined once for the family in
    [the hcp family conventions](/specs/hcp/_shared/0.1/CONVENTIONS.md#1-family-error-codes).
  retryable: false
- code: hcp:versionConflict
  meaning: >-
    HCP `version-conflict` (HCP 5.5): `expectedVersion` is not the entry's current `record.version` (HCP 5.2.4,
    5.4). Nothing changes, and the same document never succeeds: the client reads the entry again, and a Management
    Client shows the person the current version, before deciding whether to ask again. Defined once for the family
    in [the hcp family conventions](/specs/hcp/_shared/0.1/CONVENTIONS.md#1-family-error-codes).
  retryable: false
- code: hcp:credentialInvalid
  meaning: >-
    HCP `credential-invalid` (HCP 5.5): a credential that is an entry's content, as created or revised, fails
    verification, in its proof, its securing mechanism, its verification method or its status (HCP 14.6); or, for
    an `issuer-attested` entry in `health_context`, no credential the request carries, content or supporting, passes
    verification (HCP 16.2). The `message` says which check failed, and reveals no status-list internals. Defined
    once for the family in [the hcp family conventions](/specs/hcp/_shared/0.1/CONVENTIONS.md#1-family-error-codes).
  retryable: false
```

| Code | HCP error | Declared by |
|---|---|---|
| `hcp:invalidToken` | unauthenticated, `invalid-token` | every specification whose request carries `accessToken` (26 of the 28) |
| `hcp:versionConflict` | version conflict, `version-conflict` | `hcp/entries/revise`, `confirm`, `set-categories`, `restrict-purposes` |
| `hcp:credentialInvalid` | credential invalid, `credential-invalid` | `hcp/entries/create`, `revise` |

The two specifications that declare none are
[`hcp/grants/request`](../../grants/request/0.1/spec.md), whose request carries no
token, and [`hcp/grants/revoked`](../../grants/revoked/0.1/spec.md), which the
provider sends to a client.

A code that only one specification can reach is not a family code: §8.5 rule 2
says a family namespace is "never to give a specification-specific code a broader
name than it has earned", so such a code is namespaced under that specification's
own slug, as `hcp/export:tooLarge` is.

## 2. Why these are extended codes and not §8.3 standard ones

`hcp:invalidToken` is not `permissionDenied`. HCP keeps unauthenticated
(`invalid-token`) apart from not authorized (`scope-denied`), and a client acts
differently on each: on the first it obtains new authorization evidence, and on
the second it does not ask again. The carriage sends not authorized as
`permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5,
[SPEC.md §8.2.1](/SPEC.md#821-what-a-message-may-not-say)), so it needs a code of
its own for unauthenticated.

`hcp:versionConflict` is not `idConflict`, which §8.3 defines over the *Trust Task
document's* own `id`; an entry version is a domain value inside a well-formed,
never-before-seen document. `hcp:credentialInvalid` is not `proofInvalid`, which
reports the document's own `proof`; it reports a credential the document carries
as an entry's content.

A consumer that does not recognise one of these codes reads it as `taskFailed`
([SPEC.md §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications))
and still honours `retryable: false`.
