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
status: draft                      # draft | candidate | standard
targetFrameworkVersion: "0.1"      # SPEC.md MAJOR.MINOR this spec targets
category: permission               # must be one of the TT_CATEGORIES ids
keywords: [acl, access-control, grant]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Granting authority
    requirement: REQUIRED          # REQUIRED | RECOMMENDED | OPTIONAL
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED            # OPTIONAL | RECOMMENDED | REQUIRED
  rationale: A one-sentence reason your threat model needs this strength.
errorCodes: []                     # see "Task-specific error codes" below; [] is fine
related: []                        # slugs of related Trust Tasks (full slugs incl. /)
---
```

Notes:

- **Do not declare `vidSchemes` on parties.** Which VID schemes (`did:web`, `did:key`, `x509`, OIDC, …) a maintainer accepts is an implementation/trust-framework concern, not a spec one. Leaving it out keeps specs portable across maintainers with different verification preferences.

After the closing `---`, write the human-readable specification: Abstract, Status, Conformance, Definitions, Examples, Security & Privacy, plus anything else useful. Use `##` for the top-level sections you want to appear in the on-page sidebar TOC. The website auto-builds the TOC from your `##` headings.

## Request and Response sections

A specification that defines both a request document and a success-response document **MUST** organize its `spec.md` body around two H2 sections — `## Request` and `## Response` — placed immediately after `## Definitions` and before `## Security & Privacy`. The framework's `#request` / `#response` fragment convention (see [SPEC.md §4.4.1](SPEC.md#441-request-and-response-variants)) maps directly to these anchor names: clicking a Trust Task `type` URL like `https://trusttasks.org/spec/acl/grant/0.1#response` lands the reader on the rendered spec page at the **Response** section.

The shape:

- `## Request` — one-sentence intro that names the producer and recipient and points to the top-level schema in `payload.schema.json`. Follow with one or more `### Sub-heading` example blocks, each a complete *Trust Task document*.
- `## Response` — one-sentence intro that names the producer (the *recipient* of the request, now responding) and points to the sub-schema reachable via `$anchor: "response"` in `payload.schema.json`. Describe each payload member and note that failures use `trust-task-error`, not a `#response` document. Follow with at least one `### Sub-heading` example response — and, where helpful, an example showing the failure path with a paired `trust-task-error` reply.

A specification that defines a fire-and-forget task (no success response document) **MAY** omit the `## Response` section; in that case the `payload.schema.json` MUST NOT contain a `$defs.Response` sub-schema (the build script checks).

Every example block **SHOULD** be a complete JSON object — including framework members like `id`, `type`, `issuer`, `recipient`, `issuedAt`, and (where required) `proof` — so a reader can copy, modify, and use it directly. Pair request and response examples by `threadId` so the round trip is visible. Comment briefly before each example on what it demonstrates.

See `specs/acl/grant/0.1/spec.md` for a worked example.

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
