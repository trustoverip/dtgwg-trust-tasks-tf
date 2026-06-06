# Contributing a Trust Task specification

This guide walks through adding or updating a Trust Task specification under the registry.

For framework-level contributions (changes to `SPEC.md`, the build pipeline, or the website), see [`CONTRIBUTING.md`](CONTRIBUTING.md).

## TL;DR

```
specs/<slug>/<version>/
├── spec.md              # YAML front matter + prose + examples
└── payload.schema.json  # JSON Schema 2020-12 for the payload
```

1. Fork the repo and create a branch.
2. Create the folder `specs/<your-slug>/<your-version>/`. The slug may be hierarchical — `specs/acl/grant/0.1/` is a valid layout whose slug is `acl/grant`.
3. Add `spec.md` with the YAML front matter shape described below, prose for your specification, and at least one example Trust Task document under an `## Examples` section.
4. Add `payload.schema.json` describing your `payload` member. Its `$id` **MUST** equal `https://trusttasks.org/spec/<your-slug>/<your-version>` — note that the slug's `/` separators appear literally in the URL.
5. Run `npm install` then `npm run build` from the repo root to validate.
6. Open a PR. CODEOWNERS will route review to the right people.

You don't touch `website/` — the build copies your folder into the website tree automatically.

## File layout

The folder structure mirrors the canonical *Type URI* (per [SPEC.md §6.1](SPEC.md#61-type-uri)):

| URL | File on disk |
|---|---|
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` (HTML) | `specs/<slug>/<MAJOR.MINOR>/spec.md` |
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` (JSON Schema) | `specs/<slug>/<MAJOR.MINOR>/payload.schema.json` |
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` (JSON-LD context) | `specs/<slug>/<MAJOR.MINOR>/context.jsonld` (optional) |

Slugs may be single-segment (`kyc-handoff`) or hierarchical (`acl/grant`, `members/promote-to-admin`). In either case, the on-disk path matches the slug verbatim: a slug of `acl/grant` means `specs/acl/grant/<version>/`. Use hierarchical slugs to group related specifications under a namespace and keep the top of the `specs/` tree readable.

Versions live side-by-side in their own folders (`specs/acl/grant/0.1/`, `specs/acl/grant/1.1/`, …). Each version is independently editable; you never overwrite a published version.

## Slug rules (per SPEC §6.1)

- Each path segment is lowercase, hyphen-separated, and matches the regex `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` (no consecutive hyphens within a segment).
- Path segments are joined with `/`; consecutive slashes and leading/trailing slashes are not permitted.
- The slug `trust-task` and any slug whose first segment is `trust-task` or begins with `trust-task-` are **reserved** for framework-defined types (see SPEC §6.1 and §8).

## Version rules (per SPEC §5)

- `MAJOR.MINOR` only — no patch level.
- `MINOR` bump = backwards-compatible change.
- `MAJOR` bump = breaking change, reset `MINOR` to 0.
- See [SPEC.md §5.2](SPEC.md#52-compatibility-rules) for the precise compatibility rules consumers will apply to your version bump.

## `spec.md` front matter

Every `spec.md` begins with a YAML block delimited by `---` lines. The block is validated against [`specs/spec.meta.schema.json`](specs/spec.meta.schema.json) at build time. Required fields:

```yaml
---
slug: acl/grant                    # MUST match the folder path under specs/
version: "0.1"                     # MUST match the version-folder name
title: ACL — Grant
summary: One-sentence elevator pitch.
status: draft                      # draft | candidate | standard | retired
targetFrameworkVersion: "0.1"      # SPEC.md MAJOR.MINOR this spec targets
category: access-control           # must be one of the TT_CATEGORIES ids
keywords: [acl, access-control, grant]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Granting authority
    requirement: REQUIRED          # REQUIRED | RECOMMENDED | OPTIONAL
    member: issuer                 # issuer | recipient | (omit if the party is neither)
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED            # OPTIONAL | RECOMMENDED | REQUIRED
  rationale: A one-sentence reason your threat model needs this strength.
errorCodes: []                     # see "Task-specific error codes" below; [] is fine
related: []                        # slugs of related Trust Tasks (full slugs incl. /)
---
```

Notes:

- **Do not declare `vidSchemes` on parties.** Which VID schemes (`did:web`, `did:key`, `x509`, OIDC, …) a maintainer accepts is an implementation/trust-framework concern, not a spec one. Leaving it out keeps specs portable across maintainers with different verification preferences.

- **`bearer: true` flips off audience binding — do not set it casually.** The default for any spec is non-bearer ([SPEC §4.8.3](SPEC.md#483-bearer-specifications)). Adding `bearer: true` to your front matter does two coupled things: it declares that documents conforming to your spec are intended for unspecified consumption (any party that can verify the `proof` is a legitimate recipient), and it causes the codegen to emit `Payload::IS_BEARER = true`. That constant in turn suppresses the audience-binding rule of [SPEC §4.8.2](SPEC.md#482-audience-binding) in every conforming consumer pipeline — a `proof`-carrying document with no in-band `recipient` is accepted instead of rejected with `malformedRequest`. **Only set `bearer: true` when the audience-free property is intrinsic to the assertion your spec publishes** (public attestations, heartbeats, schema-publication announcements). A spec that should have been audience-bound but is mistakenly bearer-flagged is silently exposed to cross-recipient replay (SPEC §10.1) — there is no second check downstream. If `bearer: true` is set, the spec's `parties` declaration **MUST** also list `recipient` as `OPTIONAL`, and the prose **MUST** state what assertion the document conveys and why audience binding is inappropriate for it.

- **`proofRequirement.requirement` is runtime-enforceable, not advisory.** The three values map to consumer behaviour through `Payload::IS_PROOF_REQUIRED` (codegen-emitted): `REQUIRED` sets the const to `true` and causes every conforming consumer pipeline to reject a proofless document with `proofRequired` ([SPEC §7.2 item 7](SPEC.md#72-consumer-requirements)); `RECOMMENDED` and `OPTIONAL` leave the const at its trait default (`false`) and the pipeline accepts proofless documents (subject to the consumer's chosen `ProofPolicy`). Picking `REQUIRED` therefore commits every conforming consumer — including bindings without an in-band verifier — to reject proofless requests, which is the right outcome for evidentiary specs like `acl/grant` but makes the spec unreachable on bindings whose integrity guarantees are out-of-band until those bindings grow a verifier. **Pick `REQUIRED` only when the threat model genuinely needs transport-independent integrity** (audit replay, downstream corroboration, dispute resolution after the original transport has closed). For everyday request/response interactions whose integrity is already guaranteed by the transport, `RECOMMENDED` is the right default.

After the closing `---`, write the human-readable specification: Abstract, Status, Conformance, Definitions, Examples, Security & Privacy, plus anything else useful. Use `##` for the top-level sections you want to appear in the on-page sidebar TOC. The website auto-builds the TOC from your `##` headings.

- **Tag the party that fills each framework member** with `member: issuer` or `member: recipient`. A party named only in the `payload` (neither the document issuer nor recipient) omits `member`. This is what makes `requirement: REQUIRED` enforceable: the codegen emits `Payload::IS_RECIPIENT_REQUIRED` from the `member: recipient` party, and every conforming consumer then rejects a document with no in-band `recipient` ([SPEC §7.2 item 5](SPEC.md#72-consumer-requirements)). For a request the `recipient` is the `member: recipient` party; a response swaps parties, so its `recipient` requirement follows the `member: issuer` party.

## Request and Response sections

A specification that defines both a request document and a success-response document **MUST** organize its `spec.md` body around two H2 sections — `## Request` and `## Response` — placed immediately after `## Definitions` and before `## Security & Privacy`. The framework's `#request` / `#response` fragment convention (see [SPEC.md §4.4.1](SPEC.md#441-request-and-response-variants)) maps directly to these anchor names: clicking a Trust Task `type` URL like `https://trusttasks.org/spec/acl/grant/0.1#response` lands the reader on the rendered spec page at the **Response** section.

The shape:

- `## Request` — one-sentence intro that names the producer and recipient and points to the top-level schema in `payload.schema.json`. Follow with one or more `### Sub-heading` example blocks, each a complete *Trust Task document*.
- `## Response` — one-sentence intro that names the producer (the *recipient* of the request, now responding) and points to the sub-schema reachable via `$anchor: "response"` in `payload.schema.json`. Describe each payload member and note that failures use `trust-task-error`, not a `#response` document. Follow with at least one `### Sub-heading` example response — and, where helpful, an example showing the failure path with a paired `trust-task-error` reply.

A specification that defines a fire-and-forget task (no success response document) **MAY** omit the `## Response` section; in that case the `payload.schema.json` MUST NOT contain a `$defs.Response` sub-schema (the build script checks).

Every example block **SHOULD** be a complete JSON object — including framework members like `id`, `type`, `issuer`, `recipient`, `issuedAt`, and (where required) `proof` — so a reader can copy, modify, and use it directly. Pair request and response examples by `threadId` so the round trip is visible. Comment briefly before each example on what it demonstrates.

See `specs/acl/grant/0.1/spec.md` for a worked example.

## Naming conventions (per SPEC §4.10)

Member names and enumerated values use **lowerCamelCase**, so documents are consistent for both human readers and code generators:

- **Payload member names** — lowerCamelCase (`sessionId`, `wakeHandle`, `redactedFields`). Deviate only where you embed a member whose name is fixed by an external vocabulary (a field copied verbatim from a WebAuthn or JOSE structure), and confine the foreign naming to that sub-object.
- **Enumerated values you define** — statuses, kinds, decisions, event types: lowerCamelCase (`cacheAndKeys`, `stepUp`, `proxyLogin`).
- **Extended error `code` identifiers** — the local part after the slug is lowerCamelCase (`acl/change-role:lastAuthorityProtected`), matching the framework standard codes (`malformedRequest`, `proofInvalid`).
- **Externally-owned values** — carry **verbatim**, never re-cased: WebAuthn (`public-key`, `cross-platform`), JOSE (`EdDSA`, `ES256`), cookie `SameSite` (`Lax`, `Strict`), W3C Data Integrity (`DataIntegrityProof`, `assertionMethod`). The framework compares these by exact string equality.
- **Slugs and `ext` namespace keys** keep their own grammars (lowercase-hyphenated and reverse-DNS) — see [SPEC §6.1](SPEC.md#61-type-uri) and [§4.5.1](SPEC.md#451-the-ext-extension-member).

Casing is part of the wire contract: changing the casing of a member name or a value you define is a breaking change ([SPEC §5](SPEC.md#5-versioning)).

## `payload.schema.json`

Must be a JSON Schema 2020-12 document. The build script enforces:

- `$schema` = `https://json-schema.org/draft/2020-12/schema`
- `$id` = `https://trusttasks.org/spec/<slug>/<version>` (the Type URI of your spec, with the slug's `/` separators preserved in the URL, and **without** a fragment)
- `additionalProperties` declared explicitly (recommended: `false`).

The schema describes **only the `payload` member**, not the outer document. The outer structure (`id`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `proof`) is owned by the framework — see [SPEC.md §6.3](SPEC.md#63-schema-scope).

### Request and response in one schema file

Where the specification defines a success-response document, both shapes live in the same `payload.schema.json`. The top-level schema (or a sub-schema reachable via `$anchor: "request"`) describes the **request** payload; a sub-schema under `$defs.Response` with `$anchor: "response"` describes the **response** payload. Skeleton:

```jsonc
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://trusttasks.org/spec/acl/grant/0.1",
  "type": "object",
  "additionalProperties": false,
  "required": ["subject", "role", "after"],
  "properties": { /* request fields */ },
  "$defs": {
    "AclEntry": { /* shared shape */ },
    "Response": {
      "$anchor": "response",
      "type": "object",
      "additionalProperties": false,
      "required": ["entry"],
      "properties": { "entry": { "$ref": "#/$defs/AclEntry" } }
    }
  }
}
```

A consumer that receives a document with `type: ".../acl/grant/0.1#response"` resolves `#response` against the fetched schema, lands on `$defs.Response`, and validates the response `payload` against it. The build script verifies that any `$defs.Response` you publish declares `$anchor: "response"` and that no other `$defs` entry uses that anchor.

For a fire-and-forget task with no success response, omit `$defs.Response` entirely — the framework still gives you `trust-task-error` for failures.

### Opting into the framework `ext` extension slot

If your spec needs to accommodate ecosystem-defined fields without forking — the typical example is a maintainer that wants to require a custom policy field on every inbound document — opt into the framework's `ext` slot ([SPEC §4.5.1](SPEC.md#451-the-ext-extension-member)) rather than minting your own extension shape. Reference the framework `Ext` `$def` from your payload schema (and from any nested object that should carry per-instance ecosystem data):

```jsonc
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    /* … your declared members … */
    "ext": {
      "$ref": "../../../_framework/0.1/framework.schema.json#/$defs/Ext",
      "description": "Ecosystem-defined extension members per SPEC §4.5.1."
    }
  }
}
```

The codegen resolves the cross-file `$ref` by splicing the shared `Ext` `$def` into your spec's generated schema and Rust types. The producer/consumer rules around namespacing ("reverse-DNS lowercase keys", "consumers MUST ignore unrecognized namespaces", "MAY require their own namespace") are framework-wide and apply uniformly — your spec inherits them by referencing `Ext`.

Spec authors **MUST NOT** define cross-spec semantics for any `ext.*` key; ecosystem semantics belong to the namespace controller. Spec authors **MAY** include a non-normative example showing the `ext` shape an ecosystem might use, but the example should be clearly marked as illustrative.

## Task-specific error codes (optional)

If your task defines extension codes per [SPEC.md §8.5](SPEC.md#85-extension-by-individual-trust-task-specifications), declare them in front matter. The namespace before the colon **MUST** equal the spec's slug — including any `/` separators:

```yaml
errorCodes:
  - code: acl/grant:role_not_recognized    # namespace is the full slug
    meaning: Plain-text description.
    retryable: false
    detailsSchema:                          # optional JSON Schema fragment for `details`
      type: object
      additionalProperties: false
      required: [offendingRole]
      properties:
        offendingRole: { type: string }
        knownRoles:
          type: array
          items: { type: string }
```

### Extension authority: spec authors vs. consumer maintainers

Two parties may need to mint codes under `<slug>:<local>`:

- **Spec authors** declare canonical codes in `errorCodes` front matter when publishing or revising the spec. These are the codes every conforming consumer can rely on.
- **Consumer maintainers** **MAY** mint additional codes for invariants the spec did not enumerate (for example, a maintainer-specific authorization guard). The slug **MUST** be the slug of the spec **being processed** — never that of a related spec. A consumer handling `acl/change-role` that needs to surface a "last authority protected" rejection emits `acl/change-role:lastAuthorityProtected`, **not** `acl/revoke:lastAuthorityProtected` even though the related rule is canonically declared on `acl/revoke`. A client dispatching on `payload.code` expects the slug to identify the request's own type; cross-slug codes break that contract.

Per [SPEC.md §8.5](SPEC.md#85-extension-by-individual-trust-task-specifications), a consumer that does not recognize an extended `code` treats the error as `taskFailed`, so maintainer-minted codes degrade gracefully for clients that only know the canonical set.

## Build and validate locally

```sh
npm install            # one-time
npm run build          # validate + regenerate website/assets/tasks.generated.js + sync specs/ → website/specs/
npm run validate       # validate only, no website writes
```

If validation fails, the script prints the offending file and a specific reason. Fix and re-run.

## Submitting a PR

- Touch only your own spec folder (or namespace). CODEOWNERS routes review to that slug's editors; touching multiple folders requires multiple approvals and slows everyone down.
- Sign-off your commits (`git commit -s`) — this repository requires the DCO trailer.
- Run `npm run build` once before submitting. CI runs it on every PR; failing the build blocks the merge.

## Promoting maturity

Status transitions follow [SPEC.md §5.3](SPEC.md#53-maturity-levels):

- `draft` → `candidate`: change `status: candidate` in front matter once two independent, interoperable implementations exist.
- `candidate` → `standard`: change `status: standard` once your candidate has gone 90 days without a breaking change.

Maturity bumps are PRs like any other. The build script enforces the values; the editors' team verifies the transition criteria.

## Retiring a spec

`retired` is a **terminal** status — once a spec is retired it stays retired. Retirement signals "no longer recommended for new use" while keeping the spec's prose and schema available so already-issued documents remain verifiable.

Three cases trigger retirement in practice:

1. **Abandoning a `draft`** — the proposed task didn't pan out; leave a tombstone rather than deleting the folder. Change `status: retired` in place.
2. **Deprecating a `candidate`** — the schema froze but the working group chose not to standardize. Same change.
3. **Sunsetting a `standard` after a successor lands** — the most common case. The successor is typically a new `MAJOR.MINOR` of the same slug (e.g. `acl/grant/0.1` → `acl/grant/1.0`), but it can be a different slug entirely.

When you retire a spec, **strongly recommend** declaring `supersededBy` in the front matter:

```yaml
status: retired
supersededBy: acl/grant/1.0      # or just `acl/grant` for "latest non-retired version"
```

The bare-URL redirect (`https://trusttasks.org/spec/<slug>`) skips retired versions automatically. Consumer tooling reads `supersededBy` to direct implementers at the recommended replacement.

A retired spec **MUST NOT** transition back to `draft`, `candidate`, or `standard`. If you need to revive functionality, publish a fresh `MAJOR.MINOR` of the slug and let it progress through the lifecycle from `draft`.
