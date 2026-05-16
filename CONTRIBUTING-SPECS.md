# Contributing a Trust Task specification

This guide walks through adding or updating a Trust Task specification under the registry.

For framework-level contributions (changes to `SPEC.md`, the build pipeline, or the website), see [`CONTRIBUTING.md`](CONTRIBUTING.md).

## TL;DR

```
specs/<slug>/<version>/
├── spec.md              # YAML front matter + prose
└── payload.schema.json  # JSON Schema 2020-12 for the payload
```

1. Fork the repo and create a branch.
2. Create the folder `specs/<your-slug>/<your-version>/`.
3. Add `spec.md` with the YAML front matter shape described below and prose for your specification.
4. Add `payload.schema.json` describing your `payload` member.
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

Versions live side-by-side in their own folders (`specs/kyc-handoff/1.0/`, `specs/kyc-handoff/1.1/`, …). Each version is independently editable; you never overwrite a published version.

## Slug rules (per SPEC §6.1)

- Lowercase, hyphen-separated short name.
- Regex: `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`. No consecutive hyphens.
- The slug `trust-task` and any slug beginning with `trust-task-` are **reserved** for framework-defined types (see SPEC §6.1 and §8).

## Version rules (per SPEC §5)

- `MAJOR.MINOR` only — no patch level.
- `MINOR` bump = backwards-compatible change.
- `MAJOR` bump = breaking change, reset `MINOR` to 0.
- See [SPEC.md §5.2](SPEC.md#52-compatibility-rules) for the precise compatibility rules consumers will apply to your version bump.

## `spec.md` front matter

Every `spec.md` begins with a YAML block delimited by `---` lines. The block is validated against [`specs/spec.meta.schema.json`](specs/spec.meta.schema.json) at build time. Required fields:

```yaml
---
slug: kyc-handoff                  # must match the folder name
version: "1.0"                     # must match the version-folder name
title: KYC Handoff
summary: One-sentence elevator pitch.
status: standard                   # draft | candidate | standard
targetFrameworkVersion: "0.1"      # SPEC.md MAJOR.MINOR this spec targets
category: identity                 # must be one of the TT_CATEGORIES ids
keywords: [kyc, aml, onboarding]
authors:
  - DTGWG Identity Subgroup
parties:
  - role: Verifier (KYC provider)
    requirement: REQUIRED          # REQUIRED | RECOMMENDED | OPTIONAL
    vidSchemes: [did:web, did:key]
  - role: Relying party
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
proofRequirement:
  requirement: REQUIRED            # OPTIONAL | RECOMMENDED | REQUIRED
  rationale: A one-sentence reason your threat model needs this strength.
errorCodes: []                     # see "Task-specific error codes" below; [] is fine
related: []                        # slugs of related Trust Tasks
---
```

After the closing `---`, write the human-readable specification: Abstract, Status, Conformance, Definitions, Security & Privacy, plus anything else useful. Use `##` for the top-level sections you want to appear in the on-page sidebar TOC. The website auto-builds the TOC from your `##` headings.

## `payload.schema.json`

Must be a JSON Schema 2020-12 document. The build script enforces:

- `$schema` = `https://json-schema.org/draft/2020-12/schema`
- `$id` = `https://trusttasks.org/spec/<slug>/<version>` (the Type URI of your spec)
- `additionalProperties` declared explicitly (recommended: `false`).

The schema describes **only the `payload` member**, not the outer document. The outer structure (`id`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `proof`) is owned by the framework — see [SPEC.md §6.3](SPEC.md#63-schema-scope).

## Task-specific error codes (optional)

If your task defines extension codes per [SPEC.md §8.5](SPEC.md#85-extension-by-individual-trust-task-specifications), declare them in front matter:

```yaml
errorCodes:
  - code: kyc-handoff:document_revoked    # MUST be namespaced as <slug>:<local>
    meaning: Plain-text description.
    retryable: false
    detailsSchema:                         # optional JSON Schema fragment for `details`
      type: object
      additionalProperties: false
      required: [documentRef]
      properties:
        documentRef: { type: string }
        revokedAt:   { type: string, format: date-time }
```

## Build and validate locally

```sh
npm install            # one-time
npm run build          # validate + regenerate website/assets/tasks.generated.js + sync specs/ → website/specs/
npm run validate       # validate only, no website writes
```

If validation fails, the script prints the offending file and a specific reason. Fix and re-run.

## Submitting a PR

- Touch only your own spec folder. CODEOWNERS routes review to that slug's editors; touching multiple folders requires multiple approvals and slows everyone down.
- Sign-off your commits (`git commit -s`) — this repository requires the DCO trailer.
- Run `npm run build` once before submitting. CI runs it on every PR; failing the build blocks the merge.

## Promoting maturity

Status transitions follow [SPEC.md §5.3](SPEC.md#53-maturity-levels):

- `draft` → `candidate`: change `status: candidate` in front matter once two independent, interoperable implementations exist.
- `candidate` → `standard`: change `status: standard` once your candidate has gone 90 days without a breaking change.

Maturity bumps are PRs like any other. The build script enforces the values; the editors' team verifies the transition criteria.
