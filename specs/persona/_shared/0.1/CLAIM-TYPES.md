# persona — the claim-type registry

This document defines the core vocabulary of
[`ClaimType`](persona-record.schema.json) tokens, and the per-token defaults
that every surface handling a holder's attributes reads. Its machine-readable
half is [`claim-types.json`](claim-types.json).

It is **descriptive of the core set, not exhaustive of what may exist.** The
`x:` namespace stays open, and an unregistered token is a valid token — it
simply resolves to the conservative defaults in §4.

## 1. Why a registry, when the pattern is already enforced

`ClaimType` constrains the *shape* of a token and says nothing about which
tokens exist. Nothing today distinguishes `phone.mobile` from `mobile.phone`:
both validate, both store, and two clients that pick differently produce two
pools that cannot be compared, queried or reasoned about together.

That alone would be worth fixing for discoverability. It became urgent for a
different reason: **four separate mechanisms need somewhere stable to key on.**

- Masking needs to know that `payment.card` is shown as `•••• 4242` and
  `email.work` as `a•••@example.com` — the format is a property of the type.
- A sensitivity default needs a token to attach to, or every holder must
  classify their own card number.
- A release requirement (§3) needs one, for the same reason.
- A verifier's request, and any mapping to an external vocabulary, can only
  name types that have names.

A registry is the cheapest of those five artefacts and the one the other four
depend on.

## 2. The token stays ours

[`ClaimType`](persona-record.schema.json) already settles the question this
registry might otherwise reopen:

> The token is the maintainer's own; no external vocabulary is primary.
> External vocabularies (vCard/jCard, OIDC standard claims, schema.org) are
> mappings applied at PRESENTATION by a renderer, not at rest.

This registry does not change that. It **records the mappings** rather than
adopting one — each entry carries an `oidc` array naming the OIDC standard
claims it answers, because that is the vocabulary relying parties actually ask
in. Others may be added the same way.

Two properties of the native token are load-bearing and no flat external
vocabulary has both:

- **It is hierarchical.** `payment.*` classifies as one family without
  enumerating its members, which is what makes a default table finite and a
  future token safe by construction.
- **It is extensible.** The `x:` namespace exists because the closest prior
  art — CardSpace's fifteen fixed claim types — failed on exactly that.

## 3. Three axes, and they are not the same axis

A great deal of confusion is avoided by keeping these apart. Each entry
declares the first two; the third is computed elsewhere and is listed here only
so it is not conflated with them.

| axis | the question it answers | where it lives |
|---|---|---|
| **sensitivity** | how carefully is this shown **to the holder** | this registry, overridable per attribute |
| **release** | what it takes to let this **leave** | this registry, overridable per attribute |
| **linkability** | what it **costs** the holder once it has left | computed — `persona/correlation/analyze` |

A payment card is highly sensitive, release-gated, and barely linkable: every
card number is unique, and knowing one tells a second verifier nothing about
the first. A passport number is all three. A self-asserted nickname may be
linkable and not sensitive at all.

Reading any one of these as a proxy for another produces a UI that hides the
wrong things and warns about the wrong things.

### 3.1 `sensitivity`

`normal` | `high`. `high` means the value is **withheld from a listing that did
not explicitly ask for sensitive values** — and, being withheld, masked when a
consumer holds it anyway.

That withholding is the half that is not cosmetic. Masking a value already
fetched defends a screen; it is no defence against a log, a crash dump, or the
memory of the process holding it. The control that matters is on the read path.

**Masking is not the same decision, and does not require `high`.** See §3.3.

### 3.2 `release`

`consent` | `stepUp`. Governs what it takes to disclose the value.

`consent` is the existing gate: `persona/disclosure/preview` renders what would
leave, and `persona/disclosure/present` releases it. A human sees it once.

`stepUp` requires a **fresh** authentication bound to *that* preview — not to
the session. Without the binding "each time" degrades into "once per login",
which is the failure this exists to prevent. The single-use `previewId` the
preview already mints is the natural thing to bind to.

`stepUp` is the default for `payment.*` and `gov.*`.

### 3.3 `mask`

One of the styles enumerated in `claim-types.json`, and **independent of
`sensitivity`**: any type whose style is not `none` is masked on screen,
whatever its sensitivity.

The two were tangled in the first draft — masking was defined as something
`high` meant — which left `email.*` carrying `emailLocal` that no rule could
ever apply, while §1 used `a•••@example.com` to motivate the registry. An email
address is worth hiding from the person behind you without being worth
withholding from every listing, and there was no way to say so.

So there are three strengths, not two:

| | shown | masked | withheld from a listing |
|---|---|---|---|
| `normal`, `mask: none` | ✓ | | |
| `normal`, `mask: …` | | ✓ | |
| `high` | | ✓ | ✓ |

The style is a property of the type because only the type knows which
characters are the recognisable ones: the last four of a card, the domain of an
email address, none of a display name.

A mask applies to a **rendering**, never to what is stored or sent. It only
becomes ambiguous for `valueType: object`, every one of which is `full` today.

## 4. How a default resolves

Per axis, independently:

1. If the holder set the value explicitly on the attribute, that wins.
2. Otherwise, if the token has an **exact** entry here, its entry supplies it,
   as written.
3. Otherwise, take the **longest registered prefix** of the token and the
   `defaults.unregistered` value, and use whichever is **more protective**
   (`strictness` in `claim-types.json` orders each axis).
4. Otherwise — no entry, no registered prefix, anything under `x:` — the
   conservative default: `high` / `consent` / `full`.

Rule 3 is what makes §2's claim about the hierarchy true rather than merely
appealing, and it was missing from the first draft of this document. Without
it, `payment.somethingNew` resolved to the unregistered default, whose
`release` is `consent` — **weaker than every registered member of the family it
plainly belongs to.** A gated family must not be leavable by inventing a token.

"More protective" rather than "the prefix wins" so a family entry can only ever
tighten. `name` as a prefix does not make an unregistered `name.somethingNew`
visible; it cannot make anything looser than the floor, only stricter. That
keeps one direction of surprise available and closes the other.

**Store the override; derive the default.** Only a holder's deliberate choice is
persisted. A resolved default is computed at read, so tightening this registry
protects existing attributes rather than only future ones — while a holder's
own decision never changes under them.

Rule 3 is where "absence is the most restrictive answer" belongs: applied to a
*vocabulary nobody has reasoned about*, not to an unset field on a known type.
Applied to the latter it would mask every legal name in every pool, which
teaches holders to reveal reflexively and leaves them less protected than
before.

## 5. The minimum set

`minimumSet: true` marks the types a community can ask for without it being
remarkable, and that a face is expected to be able to carry: currently
`name.given`, `name.family` and `name.display`.

Its relationship to the axes above is a rule, not a coincidence: **the minimum
set contains only `normal`-sensitivity, `consent`-release types.** A proposal to
add a `high` type to it is a proposal that should be refused — the signal that
it does not belong there.

## 6. Open questions

- **Should `release: "never"` exist?** A value the holder keeps for their own
  reference and can never disclose. Useful and honest, but it makes the pool
  partly a password manager, and that is easier to add later than to remove.
- **Should the agent serve this table?** Clients can ship it statically today.
  A `persona/claim-types/list` task — the shape `persona/renderers/list`
  already has — would let a client learn types its build predates. Worth doing
  when the first extension type ships, not before.
- **Which external vocabularies after OIDC?** vCard is the obvious second for
  contact-shaped types; schema.org is broad enough to map loosely and is
  probably not worth normative mappings.
