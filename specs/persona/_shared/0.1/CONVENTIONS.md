# persona — category conventions

This document records the conventions shared by every specification under the
`persona/` category. It is descriptive; where it could conflict with a specific
spec, **that spec's own front matter and payload schema are authoritative.**

## 1. Shared schema components

Persona payloads reference two shared components
([SPEC.md §6.6](/SPEC.md#66-shared-schema-components)):

[`persona-record.schema.json`](persona-record.schema.json) — `$defs`:
`ClaimType`, `ValueType`, `Provenance`, `ProofRung`, `Attribute`,
`ProfileEntry`, `Sensitivity`, `ReleaseRequirement` and their neighbours.

[`claim-types.json`](claim-types.json) — the core claim-type registry, with
[`CLAIM-TYPES.md`](CLAIM-TYPES.md) as its prose.

## 2. The boundary, and which side a task sits on

The attribute pool and the profiles built over it are **agent-scoped**: they sit
above every trust context and read across all of them. Bindings, contacts,
disclosure records and context-local profiles are **context-scoped**.

The holder pushes a materialised projection *down* into a context. **A context
never pulls.** Every task in this category is on exactly one side of that line,
and a task's authority requirement follows from which side it is on rather than
from how sensitive it feels: an administrator scoped to one context is refused
the pool exactly as an application would be, because the pool is not that
context's to read.

A context receives **a copy**. Editing an attribute refreshes the copies
projected from it — a write initiated above the boundary — which is what makes
"edit once, everywhere" true without opening a read path in the other direction.

## 3. A response may name identity; it may not carry identity

**A persona's label, a face's name, an attribute's `type` and the holder's own
`label` for it are names** — they say what exists, and a consumer needs them to
render a picker, a history or an audit line.

**A value is content.** Only three tasks return it, each because returning it is
the entire point:

- `persona/attribute/list` with `includeValues` (and `includeSensitive` for the
  values that resolve to `sensitivity: high`);
- `persona/profile/get` with `resolve`;
- `persona/disclosure/preview`, whose response is as sensitive as the
  disclosure it describes.

Every other response — in this category and outside it — returns names,
identifiers, counts and versions. `auth/whoami` is the case worth stating
because it is the one a maintainer is most tempted to extend: it answers *what
may I do*, never *who am I made of*. A convenience member carrying a display
name would make an authority introspection endpoint an identity endpoint, and
every consumer that logs its response would begin logging identity.

The holder's `label` is a special case worth naming: it is *their own words*,
returned to them freely, and **never disclosed to a verifier**. Two audiences,
two answers.

## 4. Sensitivity, release and linkability are three axes

Set out in full in [CLAIM-TYPES.md §3](CLAIM-TYPES.md). Briefly:

| axis | question | where it lives |
|---|---|---|
| sensitivity | how carefully is it shown **to the holder** | registry default, holder override |
| release | what it takes to let it **leave** | registry default, holder override |
| linkability | what it **costs** once it has left | computed — `persona/correlation/analyze` |

A payment card is highly sensitive, release-gated and barely linkable. A
passport number is all three. Reading any one as a proxy for another produces a
consumer that hides the wrong things and warns about the wrong things.

**Store the override; derive the default.** `Attribute.sensitivity` and
`Attribute.release` are optional, and their absence means *the holder did not
decide*, not *normal*. A consumer resolves absence from the registry, so
tightening the registry protects attributes already written — while a holder's
own decision never changes under them.

## 5. Two calls, on purpose

`persona/disclosure/preview` signs nothing and sends nothing. It returns a
single-use `previewId` that `persona/disclosure/present` consumes. There is no
single-call form, and adding one would remove the only point at which a human
sees what is about to be handed over.

The preview is therefore **not read-only** despite disclosing nothing: it mints
an authorisation, so a replayed preview hands out a second one.

That token is also what a `release: stepUp` attribute binds a fresh approval to.
Binding to the session instead would turn "each time" into "once per login",
which is the failure the requirement exists to prevent.

## 6. Recipient party

Every `persona/` task is addressed to the holder's **agent** — the party tagged
`member: recipient`, declared `REQUIRED`. The producer is the holder, or a
consumer acting with the holder's authority. Per
[SPEC.md §7.2 item 5](/SPEC.md#72-consumer-requirements) the `recipient` is
enforced in-band.
