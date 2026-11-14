# Developing in this repository

The developer guide for the Trust Tasks registry, its build pipeline and the
four client libraries. Start with [`CONTRIBUTING.md`](../CONTRIBUTING.md) for
how to open a pull request; [`CONTRIBUTING-SPECS.md`](../CONTRIBUTING-SPECS.md)
covers writing a Trust Task specification and [`RELEASING.md`](../RELEASING.md)
covers releases. The notes specific to one library and its sibling packages are
in [`docs/packages/`](packages/): [Rust](packages/rust.md),
[TypeScript](packages/typescript.md), [Go](packages/go.md) and
[Dart](packages/dart.md).

## Repository layout

This repo holds the Trust Tasks framework spec, the per-task spec/schema registry
(`specs/<slug>/<version>/`), the transport bindings (`bindings/`), the registry
website (`website/`), and the generated client libraries (`trust-tasks-rs`,
`trust-tasks-ts`, `trust-tasks-go`, `trust-tasks-dart`) with their hand-written
binding and proof packages. `npm run build` (`scripts/build-registry.mjs`)
regenerates the website registry; `cargo run -p trust-tasks-codegen`,
`npm run build-ts-bindings`, `npm run build-go-bindings` and
`npm run build-dart-bindings` regenerate the libraries.

| Path | What it holds |
|---|---|
| `SPEC.md` | the framework specification, generated — see [`SPEC.md` is generated](#specmd-is-generated--never-edit-it-here) |
| `specs/<slug>/<version>/` | the per-task specification and schema registry ([`CONTRIBUTING-SPECS.md`](../CONTRIBUTING-SPECS.md)) |
| `bindings/<slug>/<version>/` | the transport bindings |
| `ceremonies/` | Trust Ceremony definitions (`npm run validate-ceremonies`, `npm run test-ceremonies`) |
| `website/` | the registry website served at trusttasks.org |
| `scripts/` | the registry build, the TS/Go/Dart generators and the CI checks |
| `trust-tasks-rs`, `trust-tasks` | the Rust runtime with its generated modules, and the facade crate |
| `trust-tasks-codegen`, `trust-tasks-build` | the Rust tree generator (never published) and the per-module generator an application's `build.rs` also calls |
| `trust-tasks-cli` | the `trust-tasks` command that writes and checks a `trust-tasks.lock` |
| `trust-tasks-https`, `-didcomm`, `-didcomm-v1`, `-tsp`, `-proof`, `-capability-client`, `-ceremony` | the hand-written Rust crates |
| `trust-tasks-ts`, `trust-tasks-ts-*` | the npm core and its sibling packages, including `trust-tasks-ts-gen` (consumer-side generation) |
| `trust-tasks-go` | the Go module, with its nested modules |
| `trust-tasks-dart`, `trust-tasks-dart-*` | the Dart core and its sibling packages |
| `examples/` | consumer applications CI builds |
| `docs/` | this guide, [`runtime-parity.md`](runtime-parity.md), design documents and ADRs |

## Build, validate and regenerate

```sh
npm install                       # one-time
npm run build                     # validate specs + regenerate website registry
npm run validate                  # validate only, no website writes
npm run build-ts-bindings         # regenerate TS bindings
npm run build-go-bindings         # regenerate Go bindings (gofmts itself)
npm run build-dart-bindings       # regenerate Dart bindings (dart formats itself)
cargo run -p trust-tasks-codegen  # regenerate Rust bindings (then `cargo fmt --all`)
npm run build-bundles             # registry bundles + index into website/specs/ (after `npm run build`)
npm run check-bundles             # verify them independently of the Rust generator
npm run serve                     # serve ./website on :8765, routed as CloudFront routes it (after `npm run build`)
```

The website deploys to S3/CloudFront via `.github/workflows/deploy.yml` on push to
`main`. Generated website outputs (`website/assets/tasks.generated.js`, `website/robots.txt`, `website/sitemap.xml`,
`website/specs/` including its `bundle.json`s, `index.json` and `detail.json`s,
`website/bindings/`) are `.gitignore`'d and rebuilt in CI — but
`website/assets/data.js` is **source**, committed by hand.

**`tasks.generated.js` is the registry index, not the registry** (#1145). It
carries only what the lists, nav, filters, search and counts read
(`INDEX_FIELDS` in `build-registry.mjs`), and the shared-schema catalogue
without schemas. The spec page fetches a version's full record from
`/specs/<slug>/<M.N>/detail.json`, and `/schema/<slug>` fetches the shared
schema's `detailPath` (its schema and the tasks using it); the build writes
both. A field the page reads outside those pages must be added to
`INDEX_FIELDS`, or it is `undefined` there.

**`website/index.html` is generated too** (#1147). The page has no in-browser
compiler: `scripts/build-website.mjs` (run by `npm run build`, or alone as
`node scripts/build-website.mjs` while editing JSX) compiles
`components.jsx`, `pages.jsx` and `app.jsx` with esbuild into one
content-hashed `website/assets/app.<hash>.js`, and writes `index.html` from
`website/index.template.html` naming it. Edit the template and the `.jsx`
files. The three are classic scripts sharing one global scope (pages.jsx calls
components.jsx's functions), so they are compiled one by one and concatenated,
never bundled as modules; a new `.jsx` file must be added to `JSX_FILES`
there, and the build fails until it is. React and `marked` stay UMD globals
from unpkg, pinned by `integrity` in the template.

The libraries publish via `.github/workflows/publish.yml`, but **not on merge** —
that workflow keeps a Release PR up to date, and merging the Release PR is the
release. See [`RELEASING.md`](../RELEASING.md) and [Merging is not releasing](#merging-is-not-releasing--read-releasingmd-before-touching-a-version).

### The Rust toolchain is pinned — `rust-toolchain.toml`

The root `rust-toolchain.toml` names the exact stable release (plus `rustfmt`
and `clippy`) that contributors and every CI job build with (#1142); CI installs
it through `.github/actions/rust-toolchain`, never `dtolnay/rust-toolchain@stable`.
`rust-toolchain-bump.yml` opens a PR each week a newer stable is out — new lints
and rustfmt changes are fixed on that PR, not discovered on every open one. The
crates' MSRV (`rust-version` in `Cargo.toml`) is separate; a job building at it
must set `RUSTUP_TOOLCHAIN`, since the file overrides `rustup default`. The root
`.npmrc` sets `engine-strict=true`, which applies to installs at the repository
root only (Node 24, as `.nvmrc` and `engines` say), not to the published packages.

## `SPEC.md` is generated — never edit it here

`SPEC.md` is the Trust Tasks framework specification, **generated** from the
canonical source, [trustoverip/dtgwg-trust-tasks-spec](https://github.com/trustoverip/dtgwg-trust-tasks-spec),
by `npm run generate-framework-spec` (`scripts/generate-framework-spec.mjs`).
A framework change is a PR to that repository; this copy follows it **at a
pinned commit**: `scripts/framework-spec-source.json` names the canonical
commit (full SHA) `SPEC.md` is generated from, and `npm run
generate-framework-spec` regenerates from that commit. The pin moves only in a
pull request — `.github/workflows/framework-spec-bump.yml` runs nightly and on
a `framework-spec-updated` dispatch from the canonical repository, runs `npm run
bump-framework-spec` (canonical `main` → pin + `SPEC.md`), and opens or
refreshes one "regenerate SPEC.md" PR when the text changed; when it fails it
opens (or comments on) a "Framework specification bump is failing" issue. The
website serves the **committed** copy: every run of `deploy.yml`, pull request
or deploy, runs `npm run check-framework-spec`, which fetches the pinned commit
by SHA, confirms the checkout is that commit, regenerates, and fails if the
committed file is not that output (#1138). So a canonical change reaches
trusttasks.org only by a reviewed merge here, and cannot turn an unrelated PR
red.

- **Section numbers are assigned by the generator**, in canonical order — the
  canonical text has none. When a canonical change moves a section, its number
  and anchor move with it.
- **The build fails on any `/SPEC.md#…` link that names no section** — from spec
  prose, binding prose, or the binding specs' absolute GitHub links. When that
  happens after a canonical change, either update the link, or add the old anchor
  to `scripts/framework-spec-legacy-anchors.json` (old anchor → canonical named
  anchor) so it keeps resolving as an alias.
- `SPEC §N` in prose is a plain number the build cannot check; when a section is
  renumbered, grep for it.

## Hand-maintained lists

Several lists in this repository are edited by hand and must agree with the
tree. Each one below has shipped a defect at least once; the build now guards
each of them, but the files are still edited by hand.

### Adding or changing a spec category — TWO files must stay in sync

The category taxonomy lives in **two hand-maintained places**. When you add or
rename a category you MUST update both:

1. `specs/spec.meta.schema.json` → `#/properties/category` enum (the build
   validates every spec's `category` against this).
2. `website/assets/data.js` → `window.TT_CATEGORIES` (the website's `id` / `name` /
   `color` for that category).

`data.js` is **not** regenerated by `npm run build` (only `tasks.generated.js` and
`website/specs/` are). If you add a category to the enum + a spec but forget the
`data.js` entry:

- the category is invisible in nav and on `/categories`, AND
- the spec detail page **crashes** — `pages.jsx` does
  `cat.name` on the result of `TT_CATEGORIES.find(...)`, which is `undefined` for an
  unknown category (`TypeError`, white screen).

This is exactly how `chat/message/1.0` shipped "broken" on the live site. Keep the
two lists in lockstep. `checkCategoryTaxonomy()` in `scripts/build-registry.mjs`
now **fails the build** on an enum entry missing from `data.js` (and warns on the
reverse), so CI catches this — but the two files are still edited by hand.

### Adding a framework-reserved slug — allowlist it in `spec.meta.schema.json`

The third instance of the same failure mode. SPEC §6.1 reserves every slug
matching `^trust-(task|ceremony)($|-|/)`, and `specs/spec.meta.schema.json`
enforces that with a hand-maintained **allowlist** at
`#/properties/slug/anyOf[1]/enum` naming the framework slugs permitted to use
the reserved namespace. Publishing a new one (`trust-task-control`,
`trust-task-ok` when it lands) means updating **both**:

1. The reserved-slug table (SPEC §6.1) — a change to the canonical framework
   repository (`SPEC.md` here is generated from it), so the slug is documented.
2. `specs/spec.meta.schema.json` — the `enum`, so the build accepts it.

Forget the second and `npm run build` fails with `/slug 'trust-task-control' is
in the framework-reserved namespace … and is not on the allowlist`, naming the
enum to edit. (The schema's own wording — `/slug must NOT be valid`, `must be
equal to one of the allowed values` — is replaced by the build for this case.)
The build does catch it — unlike the `data.js` trap above, nothing ships broken.

### Adding a transport binding — register it in `bindings.js` too

Same failure mode, different list. `npm run build` **copies** `bindings/` to
`website/bindings/` but never enumerates it; the registry site's list of bindings
is `website/assets/bindings.js` → `window.TT_BINDINGS`, hand-edited. A binding
can therefore be merged, published and implemented while remaining invisible on
the site — which is what happened to both `didcomm/0.2` and `didcomm-v1/0.1`.

When you add `bindings/<slug>/<version>/spec.md`, add the matching entry:

```js
{ id: "<slug>/<version>", slug, version, title, summary, bindingURI,
  envelopeType, status, accent, prosePath: "/bindings/<slug>/<version>/spec.md",
  implementations: [] }
```

`accent` must be one of the `--tt-*` tokens in `website/assets/tokens.css`
(`teal`, `violet`, `amber`, `coral`, `sky`). `checkBindingRegistry()` in
`scripts/build-registry.mjs` **fails the build** on an on-disk binding with no
entry, and on a `prosePath` that does not match its `id`; a listed binding with
no directory warns.

### Adding or changing a library — `libraries.js`

The fifth list: `website/assets/libraries.js` → `window.TT_LIBRARIES` and
`window.TT_LIBRARY_CAPABILITIES`, the site's library pages and capability
matrix. `checkLibraryRegistry()` in `scripts/build-registry.mjs` fails the build
when a library's `dir` or a package it names under `transports` is not in the
tree, when a `binding:<slug>` capability has no `transports` entry naming the
package that provides it, or when a binding under `bindings/` has no capability
row. It carries no version numbers on purpose, so it cannot state a stale one.

### Binding prose

The fourth instance of the hand-maintained-list failure mode was binding *prose*:
#308 moved `targetFrameworkVersion` to `0.5` in five bindings and updated none of
their "Targets **framework `X`**" lines, and three current bindings still named
`trust-task-error/0.2` after that spec reached `0.5` — one of them instructing an
implementer to emit a `0.2` document carrying `inResponseTo`, a member `0.2` does
not define. Both are guarded now (see [What the CI checks actually assert](#what-the-ci-checks-actually-assert)). **A binding does not pin the
error specification's version** — SPEC §8.1 routes to the current one — and a
binding's prose target must match its front matter.

## Changing a spec or payload schema — regenerate the libraries

The Rust, TS, Go and Dart client libraries are generated from the specs. When
you add or change anything under `specs/` (a new task, a schema edit, a new
category used by a task), you MUST regenerate **all four** in the same PR. Drift
CI guards all four — `codegen.yml` has `codegen-drift`, and `ts.yml`, `go.yml`
and `dart.yml` each have their own `bindings-drift` (the TS one was added after
PR #85 → #86, where only Rust was regenerated). All four workflows are
deliberately unfiltered so those checks always report — a path-filtered job
cannot be a required check.

1. **Regenerate Rust bindings:** `cargo run -p trust-tasks-codegen && cargo fmt --all`
   (commit the diff — CI fails on drift).
2. **Regenerate TS bindings:** `npm run build-ts-bindings`
   (updates `trust-tasks-ts/src/<slug>/...` + `src/index.ts` exports).
3. **Regenerate Go bindings:** `npm run build-go-bindings`
   (updates `trust-tasks-go/specs/<slug>/v<MAJOR>_<MINOR>/payload.go`).
4. **Regenerate Dart bindings:** `npm run build-dart-bindings`
   (updates `trust-tasks-dart/lib/specs/<slug>/v<MAJOR>_<MINOR>/payload.dart`).

   Neither of the last two needs a separate format command, unlike the Rust step:
   each generator runs `gofmt` / `dart format` on its own output and **fails** if
   the toolchain is not on PATH, because a contributor without it installed would
   otherwise commit unformatted output and discover it in CI.

**Do NOT bump a version or write a CHANGELOG entry.** That changed — see [Merging is not releasing](#merging-is-not-releasing--read-releasingmd-before-touching-a-version).
What you owe the release instead is a conventional-commit PR title: `feat(<slug>):`
when you ADD a spec family (additive API surface in every library), `spec(<slug>):`
for an edit inside one. **A `!` is only for a package's public API, scoped to the
package (`feat(rs)!:`, `fix(codegen)!:`) — never for a spec change.** One `!` makes
every crate, and the npm, Go and Dart packages, a breaking release; commit-lint
refuses a `!` whose scope is not a package.

Breaking schema changes require a new spec **version** folder (per SPEC §5), not an
in-place edit — and once a version has been **released**, *any* change its generated
types can see does, even one SPEC §5.2's draft allowance would permit in place. CI
(`published schemas change only additively`, `scripts/check-published-schemas.mjs`)
compares each changed spec with the last `trust-tasks-rs-v*` release: in place, a
released version may only gain an optional member, an appended enum value or
`oneOf`/`anyOf` branch, a loosened bound, a new `$defs` entry, or a description, and
may not change `proofRequirement` or drop a declared error code. A new version is a
new module — an additive patch for every library; an in-place break is a MINOR for
all of them.

## Merging is not releasing — read RELEASING.md before touching a version

**If you remember that "your job is just to bump the versions in the PR; the
merge to `main` releases them" — that is the OLD flow and it is gone.**

`.github/workflows/publish.yml` no longer publishes whatever version is newer
than the registry. Its `release-pr` job (`scripts/release-pr.sh`) keeps **one**
Release PR, on the `release` branch, up to date with the version bumps and
changelog entries the merged commits imply for **every** package — crates via
`release-plz update`, npm/Go/Dart via `scripts/release-bump-{ts,go,dart}.sh`,
because release-plz is Rust-only and cannot bump a `package.json` or a
`pubspec.yaml`, and the Go module has no manifest at all. Merging *that* PR is
the release. (It used to be one Release PR per package — up to thirteen, each
regenerated by every merge of another. A new publishable package now needs an
entry in `release-pr.sh`'s package lists as well as its publish matrix leg.)

Consequences for anything you do in this repo:

- **Never edit a `version = ` in a `Cargo.toml`, `"version"` in
  `trust-tasks-ts/package.json`, `const Version` in
  `trust-tasks-go/trusttasks/version.go`, or `version:` in
  `trust-tasks-dart/pubspec.yaml` (and its mirror `packageVersion` in
  `lib/src/runtime/version.dart`).** They are assigned by the Release PR. A
  version in a feature PR collides with every other open PR touching that
  package.
- **Never hand-write a `CHANGELOG.md` entry.** They are generated from
  conventional commits by `cliff.toml`. The commit body is included verbatim.
- **The PR title is load-bearing.** PRs squash-merge, so the title becomes the
  commit subject, which is both the published changelog entry and the signal that
  sizes the bump. `.github/workflows/commit-lint.yml` enforces the format.
  `spec(...)` is allowed but is **not** a standard type, so release-plz scores it
  as a patch — use `feat(<slug>):` when you add a spec family.
- **The per-PR version-bump gates are gone** from `rust.yml` and `ts.yml`. `main`
  is legitimately ahead of both registries between releases; those jobs would now
  fail every ordinary PR.
- `trust-tasks-codegen` is still `publish = false` and is never released.

Both registries use **OIDC trusted publishing** (no long-lived tokens). Every
crate's Trusted Publisher is registered against the workflow **filename**
`publish.yml` — renaming that file breaks the OIDC exchange before any release
logic runs.

Full details, including the one-time tag seeding the crate and npm releases
depend on (the Go module seeds its own — never tag it by hand):
**`RELEASING.md`**.

### Library versions are semver over the *library's* API, not the framework's

The libraries version independently of `SPEC.md`. A breaking change to what a
consumer compiles, links, or relies on bumps the leading non-zero component
(`0.2.x` → `0.3.0` below 1.0); anything additive is a patch.

**Behavioural breaks count even when the wire format is unchanged.** A spec that
starts declaring `proof` REQUIRED flips `IS_PROOF_REQUIRED`, and a consumer then
rejects documents it used to accept — that is a breaking change to the library
even though no schema moved. Judge the bump by what a consumer observes, not by
whether a `.json` file changed.

**Bumping the leading component is a workspace event, not a one-line edit.**
Every other crate in release-plz's `core` version group — the facade
`trust-tasks`, the transport bindings, `-proof` and `-capability-client` —
declares `trust-tasks-rs = { path = "…", version = "<leading>" }`. Moving the
leading component means updating every one of those requirements, and
release-plz releases those crates too. `cargo check` fails immediately if you
miss one, so the trap is discovering it mid-review rather than planning for it.
(`trust-tasks-ceremony`, `-build` and `-cli` do not depend on `trust-tasks-rs`
and are not part of the set.) **Do not count them from this note** — it has been
stale twice, once claiming five crates at `"0.2"` when the workspace was at
`0.6`, then six when the facade made seven. List them:

```sh
cargo metadata --no-deps --format-version 1 | jq -r '.packages[]
  | select(.publish != [])
  | select(any(.dependencies[]; .name == "trust-tasks-rs" and .kind == null))
  | .name'
```

The last such bump was `0.6.5` → `0.7.0` (#223), which is worth reading as a
worked example: it also carried `@openvtc/trust-tasks` to `0.7.0`, because the
same change widened a TypeScript union and broke exhaustive switches on that
side too. A break in `trust-tasks-rs` is usually a break in the other libraries too.

**`StandardCode` is `#[non_exhaustive]` as of 0.7.0.** Adding a framework
standard error code (SPEC §8.3) is therefore no longer breaking for downstream
`match` expressions. Go's `StandardCode` is a named string type and Dart's is an
`extension type` over `String`, both open by construction, so they sit with Rust
here; TypeScript's is a closed union and is the one side where adding a code is a
**breaking** change. It still requires a new `trust-task-error` spec version —
the code enum lives in that payload schema, so a document carrying a code the
declared version doesn't list will not validate — and all four libraries must be
pointed at the new version (`TRUST_TASK_ERROR_TYPE_URI` in `trust-tasks-rs`'s
`document.rs`, `TRUST_TASK_ERROR_TYPE_URI` in `trust-tasks-ts`,
`TrustTaskErrorTypeURI` in `trust-tasks-go`, `trustTaskErrorTypeUri` in
`trust-tasks-dart`; the error-URI check in `check-bindings-conformance.mjs`
fails until they agree). Note that `trust-task-error`
is in the codegen's `SKIP_SLUGS`: the Rust side is hand-modelled in `error.rs`,
so regenerating will not pick a new code up for you.

## What the CI checks actually assert

Worth knowing before trusting a green board, because the gaps have shipped real
defects three times.

| Check | Asserts | Cannot see |
|---|---|---|
| `codegen-drift`, `bindings-drift` (×3: TS, Go and Dart) | the generators were re-run | a generator that is **consistently wrong** — regenerating reproduces it and the diff is empty |
| `bindings match specs` (`npm run check-bindings`) | generated Rust, TS, Go and Dart agree with each spec's front matter, its `$defs.Response`, and each other; no withdrawn version is generated and every other one carries the same deprecation note in all four; Rust's `ERROR_CODES` matches each spec's `errorCodes`; and the Dart pubspec agrees with its own `packageVersion` | anything not expressible from front matter + schema |
| `node --test (runtime)`, `cargo test`, `go test`, `dart test` | the hand-written §7.2 pipelines behave | the generated types they operate on |
| `npm run smoke`, `go test ./smoke`, `dart test test/smoke_test.dart` | the built npm package imports as a consumer imports it; the Go and Dart packages drive a *generated* module through the real pipeline | |
| `test:infra` | the CloudFront negotiation function's routing decisions; that `response-headers-policy.json` sets every security header and its Content-Security-Policy allows each script, stylesheet and `fetch` the page makes, with no inline script | whether the deployed site actually serves them — the policy is applied by hand (`infra/cloudfront/apply-response-headers.sh`), and `verify.sh` checks the live headers |
| `test:website` (in `deploy.yml`) | prose rendered through `website/assets/markdown.js` and the `marked` the page loads (fetched from the URL in `index.template.html` and checked against its `integrity` hash; `MARKED_JS=<file>` to run offline) holds only allow-listed markup, escapes text after an unclosed inline tag, and links only to relative, fragment, `http(s):` and `mailto:` URLs; the page code compiles through the build's esbuild step, the template's production React pins match their files, and (after `npm run build`) the built bundle `index.html` names renders a set of routes through React's development server renderer against the real registry (the spec and schema pages' fetched detail read from `website/specs/`, and their loading state with none) (`REACT_JS`, `REACT_DOM_SERVER_JS`, `REACT_PROD_JS`, `REACT_DOM_PROD_JS` to run offline); `routes.js` maps prose links as `website-routes.test.mjs` pins | effects, real layout, and anything only a browser runs |
| `npm run smoke:website` (in `deploy.yml`, after `npm run build`) | the built site, served through the CloudFront routing function, loads `/`, a registry category, a spec, a binding, `/schema`, a shared schema and `/implementations/rust` in headless Chrome (over the DevTools protocol, no npm dependency; `CHROME_PATH` to choose the browser) with no console error, uncaught exception or failed/4xx request — under the response headers policy's Content-Security-Policy, which `serve-website.mjs` sends — an `<h1>`, no loading state or failed fetch once settled, and a canonical link naming the route | any other route, and the deployed CloudFront and S3 metadata (`immutable`, gzip encoding) |
| `checkCategoryTaxonomy`, `checkBindingRegistry` | hand-maintained lists match the tree | other hand-maintained lists nobody has guarded yet |
| `checkProseLinksAsRendered`, `checkSiteFrameworkLinks`, `checkBindingImplementations` | every link in spec and binding prose, mapped by `website/assets/routes.js` exactly as the page maps it, lands on a registered spec, binding, shared schema or real file; `SPEC_ANCHORS` in `components.jsx` names each section's *current* heading anchor (regenerate it when a canonical change renumbers); `bindings.js` `implementations` and `libraries.js` `transports` name the same packages | whether the prose around them is still true |
| `checkPayloadSchema` | every spec version has a `payload.schema.json` that parses, whose `$id` is its own URL and `$schema` is JSON Schema 2020-12, and whose `$defs.Response` (if any) declares `$anchor: "response"`; warns when the payload or response declares neither `additionalProperties` nor `unevaluatedProperties` (SPEC §7.3 item 7.4) | whether the schema says what the prose means |
| `checkProofFloor` | a spec's declared `proofRequirement` is no weaker than the floor its own front matter implies, per variant: a destructive side effect or `exposure.actsAsSubject` makes the request's proof REQUIRED, `exposure.discloses: secret` the response's (SPEC §7.3 item 8, read through items 13 and 14) | whether the side-effect and exposure declarations themselves are true |
| `checkIssuedAtFloor` | a consequential spec declares `issuedAtRequirement` REQUIRED for every variant it has; an undeclared or weaker one fails, except that a retired spec, which §5.3 freezes, is counted as exempt when it declares nothing | whether the handler really is consequential (§2 makes the handler authoritative) |
| `checkFreeTextBounds` | every free-text string member (named from a fixed vocabulary such as `note`, `reason` or `description`, with no `enum`, `const`, `format`, `pattern` or `$ref`) declares a `maxLength` (SPEC §7.3 item 19); retired specs are counted, never failed | a free-text member named outside the vocabulary, and identifiers, which §10.2 bounds at the transport |
| `checkErrorCodeNamespaces` | each declared `errorCodes` entry is rooted at the spec's own slug or a path prefix of it, and its local part does not shadow a standard code of the current `trust-task-error` (SPEC §8.5); a retired spec's declarations are counted, never failed | whether the code is the right one to emit |
| `checkGeneratorKeywords` | no schema carries a code generator's keyword (`tsType`, `tsEnumNames`), which the TypeScript generators would write into a module as source | other non-standard keywords a generator might read |
| `checkTargetFrameworkForm` | a non-retired spec's `targetFrameworkVersion` is written in three parts (`MAJOR.MINOR.PATCH`) when it names 0.5 or later (SPEC §5.1.1); two-part values naming 0.1–0.4 stay conformant | whether the spec follows the framework release it names |
| `checkKnownImplementationRoles` | a `knownImplementations` entry's `role` names one of the spec's own `parties`, since the spec page shows it beside the link | whether the implementation exists or plays that role |
| `checkIdentifierScopeJustification` | a spec declaring `parties[].identifierScope: public` discusses it in its prose (a warning: a lint can tell a justification from silence, not from a sentence) | whether the justification is sound |
| `checkOutcomeEvidence` | a declared `outcomeEvidence` names a registered response meeting SPEC §7.3 items 20.1–20.3; retention is `durable` (20.4, a warning); the initiating document's `recipient` is REQUIRED on a non-bearer spec; each binding pointer (20.5) names a member the payload schema declares, as far as the schema is transparent | the challenge form's issuer condition (20.5.2), which lives in prose |
| `checkCommittedRecordMirror` | `rooms/_shared` `CommittedRecord`'s members are exactly `rooms/records/get`'s response members minus the verification members a reader strips (`dataCommitment`, `recordCount`, `headVersion`, `trace`, `ext`), compared by name; a divergence would make every record trace fail to verify | requiredness, on which the two deliberately differ |
| `npm run check-sibling-core-ranges` (in `bindings match specs`) | every `trust-tasks-ts-*` sibling takes `@openvtc/trust-tasks` as a peer dependency, and every TS and Dart sibling's caret range admits the core version in the checkout (for TS, the newest on npm while the checkout's is unpublished — the Release PR — with a warning) | whether the sibling actually works with that core — its own CI job builds and tests against the newest core its range admits |
| `npm run check-dart-packages` | the five files naming the Dart packages (`publish.yml`'s `tag-dart` matrix, `release-pr.sh`'s `DART_PACKAGES`, `publish-dart.yml`, `release-bump-dart.sh`, `dart.yml`) name the same packages with the same dirs and tag prefixes | whether a package's first manual publish and trusted-publishing setup happened |
| `npm run check-release-lists` (in `Dart package lists agree`) | the same for npm (`publish.yml` matrix ↔ `TS_PACKAGES` ↔ `release-bump-ts.sh` arms ↔ a `ts.yml` job per sibling), Go (matrix ↔ `GO_MODULES` ↔ `release-bump-go.sh` arms, `go.mod` module paths, every nested module in both core `:(exclude)` lists and in a `go.yml` job) and crates (published library crates ↔ `check-rs-api-compat.sh` `CRATES`, `trust-tasks-rs` dependents ↔ release-plz's `core` group ↔ `CORE_SCOPES`, every member in `rust.yml`'s path filter); and commit-lint's `!` scope list equals the scopes the bump scripts, `check-rs-api-compat.sh` and the published crates recognise | whether a package's first manual publish and trusted-publishing setup happened |
| `documented facts match the tree` (`checks.yml`, `npm run check-doc-facts`, `scripts/check-doc-facts.mjs`) | the facts the docs still state in prose agree with the files that decide them: `RELEASING.md`'s "What gets published" table names exactly the published crates (manifests, not `cargo metadata`) and `release-pr.sh`'s `TS_PACKAGES`/`GO_MODULES`/`DART_PACKAGES` (a row saying "Not released yet" excepted); the crates this guide says do not depend on `trust-tasks-rs` are exactly the published ones that do not; every `trust-tasks*` workspace crate is in the layout table; the Node floor in `README.md`, `CONTRIBUTING.md` and this guide is `.nvmrc`'s and `engines`', and each TS package README's is its own `engines`; the current framework version in `CLAUDE.md`, this guide and `CONTRIBUTING-SPECS.md` is `CURRENT_FRAMEWORK_VERSION`. A sentence reworded out of its pattern fails rather than passing (#1123) | a fact it has no pattern for — a count or version written anywhere else; prefer prose that names no number |
| `versions change only in the Release PR` (`checks.yml`, `scripts/check-version-edits.mjs`) | a PR not from this repository's `release` branch changes no package version declaration (`[package] version` of a workspace member, `trust-tasks-ts*/package.json` `"version"`, a Go `const Version`, a pubspec `version:`, Dart `packageVersion`), compared as parsed values; a file the PR adds is not a change | a version moved by a commit pushed straight to `main` |
| `checkBindingFrameworkTarget`, `checkBindingErrorSpecPin` | a binding's prose agrees with its own front matter, and pins no `trust-task-error` version | prose that is stale in a way no other file contradicts |
| the member/value casing lint (`scripts/lib/member-casing.mjs`) | every schema's member names and enumerated values are lowerCamelCase (SPEC §4.10), or named in `member-casing-allowlist.json` — `external` with the vocabulary that owns it, or `frozen` for its own family's pre-lint deviations | whether an `external` entry's attribution is true |
| the wire-conventions lint (`scripts/lib/wire-conventions.mjs`) | the latest version of every non-retired spec — and every shared definition it `$ref`s — spells instants as RFC 3339 and names them `<event>At` (only a bound such as `since` or `validUntil` keeps a preposition; never a bare `timestamp` or `at`), pages with `cursor` + `limit` → `nextCursor` (never `null`, no `truncated`/`pageSize`/`offset`), uses `since` only as a time filter, never a boolean `created`, and names every integer duration's unit (`…Seconds`, not `…Secs` or a bare `ttl`); deliberate look-alikes are in `wire-conventions-allowlist.json` with a reason | a convention the lint has no rule for, or an instant whose name and description give no hint it is one |
| the stale-references check (`scripts/lib/stale-references.mjs`) | no current spec names another spec at a superseded version — in a schema, a description, a prose link or mention, or an example — unless it is history (a `Changes from` section), a version run ending at the latest, or listed in `stale-references-allowlist.json` with a reason; nor does any `*.md` outside `specs/` (READMEs, guides, `docs/`, the latest version of each binding — a link, a mention, a generated module path such as `acl::grant::v0_3`, or a Type URI) or any `examples/**/trust-tasks.toml` `[tasks]` pin, unless the document is history (`documents.history`: CHANGELOGs, `SPEC.md`, ADRs) or the reference is in `documents.exceptions` with a reason (#1129); `npm run fix-stale-references` rewrites the text references in both and lists the wire pins, normative values, examples and pins a person must move | whether the newer version is the one the referencing spec should follow, a version named in prose the heuristics read as history, or a version spelled any other way (`AclGrant_v0_2`, `acl/grant 0.2`) |
| the member-description ratchet (`scripts/lib/member-descriptions.mjs`) | the latest live version of each slug has no more payload members lacking both `description` and `$ref` than `member-descriptions-baseline.json` allows it (none for a slug not listed); fewer warns to run `--lower`, which never raises an entry | whether a description says anything |
| `spec lifecycle rules` (`npm run check-spec-lifecycle`, PRs only, against the merge base) and `checkCurrentFramework` | a spec version the PR adds targets `CURRENT_FRAMEWORK_VERSION` (`scripts/lib/spec-lifecycle.mjs`, `0.6.0`) or later; a status the PR raises is on the latest live version of its slug, since promoting an older one un-deprecates it in all four libraries; the build checks the current release is published and equals `DEFAULT_FRAMEWORK_VERSION` in `discovery.rs`, and reports how many latest versions lag it | whether a promotion meets CONTRIBUTING-SPECS.md's criteria (implementations, 90 days) |
| the spec-examples lint (`scripts/lib/spec-examples.mjs`) | the latest version of every non-retired spec has a fenced JSON example whose `type` is its own Type URI — and a `#response` one when the schema has `$defs.Response` — wherever it sits in `spec.md`, and each example's `payload` validates against the spec's own schema (`$defs.Response` for a response); offenders when it landed are in `spec-examples-allowlist.json`, keyed by version, which only shrinks (a stale entry fails too) | the envelope around the payload (`checkExampleDocuments` checks that); whether the Rust conformance tests round-trip the example — `extract_examples` still harvests only the `## Request` / `## Response` sections |
| the schema-bounds lint (`scripts/lib/schema-bounds.mjs`) | the latest version of every non-retired spec — and every shared definition it `$ref`s — gives every request-side array a `maxItems`, every request `limit` a `maximum` and every request `cursor` a `maxLength`, and every URL-named string member (either side) a scheme constraint (a scheme-anchored `pattern`, an `enum`/`const`, or a URI `format` whose description names the schemes); offenders when it landed are in `schema-bounds-allowlist.json`, keyed by version (or shared file), which only shrinks (a stale entry fails too) | whether a declared bound is the right size, a URL member whose name does not say it is one, or an integer or string bound other than `limit` and `cursor` |
| `published schemas change only additively` (`checks.yml`, unfiltered; `scripts/check-published-schemas.mjs`, `scripts/lib/schema-compat.mjs`) | a spec file this PR changed, deleted or renamed that was in the last `trust-tasks-rs-v*` release changed only additively — no removed, renamed or newly required/optional member, no type, `title`, pattern or format change, no enum value inserted before another, no narrowed or dropped bound, no dropped error code, no deleted or renamed schema or `spec.md`, and no change to the per-variant policy the front matter derives (`proofRequirement` and `issuedAtRequirement` in either form, `parties[].requirement`, `bearer` — parsed as YAML) | a semantic change carried only in prose, and anything already merged to main before the last release that was never compared |
| the API compatibility jobs — `rs API compatibility`, `ts API compatibility`, `go API compatibility`, `Dart API compatibility` | the published crates' (`trust-tasks-rs` over the runtime and the `acl` family only; a binding crate only when the PR changes it) and the core npm, Go and Dart packages' public API is compatible with their last published release, or the PR title declares the break with a `!` scoped to that package (or one merged since the release already did); an appended string-union value is compatible by policy | a behavioural break that changes no signature (a spec that starts requiring a proof), and the hand-written sibling packages, which are not checked yet |
| commit-lint's `!` rule | a `!` title scopes it only to an explicit list of package scopes (`pkg=` in `commit-lint.yml`, held equal to the release scripts' by `check-release-lists`) | whether the package API really broke |
| commit-lint's `breaking-trailer` job | no commit message in the PR, and not its description, has a line starting `BREAKING CHANGE`/`BREAKING-CHANGE` — which release-plz and every bump script read as an unscoped break of every package | a break declared by editing the squash message in the merge dialog |
| `check-bundles` (in `deploy.yml`, after `build-bundles`) | every registry bundle's digest, canonical form and index entry; that its schemas, and their `$defs` order, equal the TypeScript module's; and that each `defOrigins` entry names a shared schema in `specs/` defining it | whether a consumer's generator reads the bundle the way the in-repo generators read the specs |
| the error-URI check in `check-bindings-conformance.mjs` | `trust_task_error_type_uri()`, `TRUST_TASK_ERROR_TYPE_URI`, `TrustTaskErrorTypeURI` and `trustTaskErrorTypeUri` all name the same version | whether that version is the right one to have adopted |
| the runtime-defaults check in `check-bindings-conformance.mjs` | the four runtimes export the same default clock skew, consequential max age, expiry horizon and replay-guard capacity | the differences `docs/runtime-parity.md` marks *needs decision* (prune depth, `extends` chain bound, capacity 0) |

## Keep the generators and checks independent

`scripts/check-bindings-conformance.mjs` deliberately **re-implements** the
front-matter → policy derivation rather than importing the generator's. Do not
DRY it up: sharing the helper would make it assert that the generator agrees
with itself, which is the property that already held while both defects shipped.

`scripts/build-go-bindings.mjs` and `scripts/build-dart-bindings.mjs` are
duplicated from `build-ts-bindings.mjs` in the same deliberate way, and their
file headers say so. Their `$ref` inliners and policy derivations are third and
fourth independent implementations, and `check-bindings` compares the emitted
schema text of all four languages against each other. Four resolvers that agree
is evidence; one resolver quoted four times is decoration. **Do not factor the
generators together**, tempting as the overlap looks.

## Registry bundles — `trust-tasks-codegen --bundles`

Milestone 1 of `docs/design/consumer-side-generation.md`.
`npm run build-bundles` (`cargo run -p trust-tasks-codegen -- --bundles
website/specs`) writes one self-contained `bundle.json` per specification
version — schemas with `$ref`s inlined, front-matter policy, error codes,
lifecycle — and an `index.json` naming each by `sha256:` digest. It touches
nothing in `trust-tasks-rs`. `deploy.yml` runs it after `npm run build` (which
recreates `website/specs/`), so the site serves `/specs/<slug>/<M.N>/bundle.json`
and `/specs/index.json`; the bundles are not committed.

- **A bundle is built from `generate_one`'s own code paths** — `spec_schemas()`
  for the schemas and `read_spec_policy()` for the policy — so its `schema` /
  `responseSchema` are, as parsed JSON, the module's `PAYLOAD_SCHEMA`s before
  pruning. Change the inlining or a policy reader there and both move
  together; do not give `bundle.rs` a reader of its own.
- **The bundle carries both sides' full `$defs`; the generators prune what
  they emit** (#1098). Every schema constant in all four languages — prebuilt
  and consumer-side — keeps only the root `$defs` its root reaches (the request
  drops `Response` and what only it uses; the response document drops what
  only the request uses), and drops `$defs` when nothing is left. Pruning the
  bundle instead would change every digest a `trust-tasks.lock` pins. Seven
  independent copies of the rule (`prune_unreachable_defs` in
  `trust-tasks-build`, `pruneUnreachableDefs` in the three `build-*-bindings`
  scripts, `trust-tasks-ts-gen`, the Go `trust-tasks-gen` and the Dart
  generator), plus an eighth re-derivation in `check-bundles`; a local `$ref`
  other than `#/$defs/<name>…` keeps every definition in all of them. Change
  one, change all. `check-bindings` fails on a shipped schema with a dangling
  `#/$defs/` reference, which is what catches the copies agreeing on a wrong
  rule. Types are still generated from the whole schema.
- **`propertyOrder`, `defsOrder` and `defOrigins` restore what canonical form
  loses** — the authored member order, the `$defs` order the TS/Go/Dart
  inliners produce (a last-in-first-out `$ref` frontier, replayed in
  `trust-tasks-build/src/publish/order.rs`), and which shared component each
  spliced definition is. The builder refuses a bundle whose replayed order does
  not name exactly its `$defs`, and `check-bundles` compares `defsOrder` with
  the TypeScript module's `$defs` order. Change an inliner's walk and both
  must move.
- **The bytes are canonical (RFC 8785) and the digest is over them.** Any change
  to what a bundle contains changes every digest a consumer has pinned, so a
  shape change bumps `bundleVersion`.
- Withdrawn versions **are** bundled (`status: "withdrawn"`); `trust-task-error`
  is not (`SKIP_SLUGS`, listed in the index's `excluded`).
- `npm run check-bundles` (`scripts/check-bundles.mjs`) checks them without the
  Rust generator: digests, canonical form, the index against the tree, and every
  schema against the **TypeScript** module's. Like `check-bindings`, keep it
  independent — do not import from the generators.

## Superseded, retired and withdrawn versions in the generators

All four generators implement one lifecycle rule (SPEC "Superseded and Withdrawn
Versions"), each independently — do not factor it out, `check-bindings` derives it
a fifth time and compares:

- **`status: withdrawn`** — generated nowhere: no module, export, index entry or
  conformance test. Versions become withdrawn only in the quarterly batch
  (`.github/workflows/withdraw.yml` → `scripts/withdraw-due.mjs`), 90 days after
  being superseded or retired, because removing a module is a breaking release.
- **Deprecated** — `status: retired`, or superseded (a later live version of the
  slug at equal or higher maturity). Every generated module carries a constant
  (`DEPRECATION` in Rust and TS, `Deprecation` in Go, `deprecation` in Dart) that is
  "none" or the byte-exact note `superseded by <slug>/<M.N>` / `retired; use …` /
  `retired`; a deprecated one also marks its root types (`#[deprecated]`,
  `@deprecated`, `// Deprecated:`, `@Deprecated`).
- Rust CI denies warnings, so a hand-written use of a superseded generated type
  needs `#[allow(deprecated)]` or a move to the current version — and must move
  before that version's withdrawal batch, or the batch PR's CI fails.
  `Withdrawal readiness` (`scripts/check-withdrawal-readiness.mjs`) lists every
  hand-written import of a version due in the next batch — a warning until 30
  days before it, a failure inside that window. Move the use to the successor it
  names; a Type URI string or a JSON fixture naming the version is fine, since a
  withdrawn schema stays published.

## Every prebuilt specification module is deprecated (milestone 5)

Separate from the per-version lifecycle above: since milestone 5 of
`docs/design/consumer-side-generation.md`, the prebuilt modules **as a whole**
are deprecated in all four libraries, because an application generates the
tasks it pins instead. The framework's own specifications (`trust-task-*`,
`trust-ceremony-*`, `_framework/`) are not — the runtimes speak them. Each
generator marks only the prebuilt **entry point**, never the module body, since
the module body is also what the consumer-side generators emit:

- Rust — `#[deprecated]` on each gated family `pub mod` in `src/specs/mod.rs`
  (`PREBUILT_DEPRECATION` in trust-tasks-codegen). `trust_tasks_build`'s output
  is unmarked.
- TypeScript — the root `index.ts` re-exports each module as
  `import * as _X …; /** @deprecated … */ export import X = _X;`, because
  TypeScript ignores `@deprecated` on `export * as`.
- Go — a `Deprecated:` paragraph in each prebuilt package's doc.
- Dart — `@Deprecated` on each prebuilt library's `library` directive;
  `analysis_options.yaml` ignores `deprecated_member_use_from_same_package`
  so this package's own tests pass the 3.3 floor's `--fatal-infos`.

Consequences: hand-written Rust in this repo that uses a prebuilt module on
purpose — the `acl`-specimen test modules, the integration tests, the examples —
carries `#[allow(deprecated)]` (CI denies warnings); new runtime code must not
depend on a non-framework family at all. The TS, Go and Dart equivalence tests
mask exactly the mark and nothing else, so change its wording in the generator
and the test together.

**No family is on by default (#1102).** `trust-tasks-rs` and the `trust-tasks`
facade both have `default = []` — the generator writes it, and
`tests/spec_feature_manifest.rs` asserts it — so a plain dependency is the
runtime alone. A dev-dependency names the families its tests use
(`features = ["acl"]`), and a CI job that needs every family says so:
`cargo test --workspace --features trust-tasks/all-specs`,
`-p trust-tasks-rs --features validate,all-specs`. `--no-default-features` is
no longer needed anywhere to get the runtime alone.

## `extends`: routing a newer version to the version a consumer implements

A spec version may declare `extends: "<M.K>"` (SPEC §5.2 "Declared extension"): it
only adds to `M.K`. The build checks the promise (`scripts/lib/schema-compat.mjs`),
the stale-references check treats a reference along the chain as current, and all
four libraries act on it:

- **Every generated module carries the direct declaration** — `EXTENDS` (Rust, TS),
  `Extends` (Go), `extendsVersion` (Dart) — and each library generates an index
  from bare Type URI to it (`extends_index.rs`, `_extends.ts`, `trusttasks/extends_index.go`,
  `extends_index.dart`). `check-bindings` compares the constant with the front matter.
- **The runtime reads a later version as the one implemented** when the chain
  reaches it: Rust's `Dispatcher`/`AsyncDispatcher` route a miss to the highest
  registered version the chain reaches (and `extension::read_as`); TS
  `readForSpec`, Go `ReadForSpec`, Dart `readForVersion` do it for a caller that
  passes a `Spec`. The payload is **pruned** to the members the implemented schema
  knows; the document keeps its received `type`, so the answer is
  `<received type>#response`.
- **Proofs are verified over the document as received, never the pruned copy.**
  Rust's `ProofPolicy::VerifyReceived`, Go's and Dart's `received` option, and TS's
  internal pairing of the pruned copy with the received object all exist for this;
  item 11's digest is over the received document too, so two documents differing
  only in an extension member are an `idConflict`. Do not "simplify" any of these
  to verify the typed document.

`npm run extends-candidates` lists version pairs whose schemas already qualify;
only an author can declare one, because the promise also covers meaning.

## Instants the libraries write: whole seconds

Every `issuedAt` a library stamps — error responses, success responses,
requests the HTTPS client and the capability clients fill in — is UTC in whole
seconds, rounded down: `2026-10-28T05:14:12Z`. SPEC §13.1 item 3 calls a
full-precision timestamp a fingerprint, and readers with a narrower RFC 3339
profile refused Rust's nanoseconds (#970). Rust stamps through
`trust_tasks_rs::issued_at_now()`, TypeScript through `issuedAtNow()`, Go
through `SystemClock`, Dart through `systemClock`; never write `Utc::now()` or
`toISOString()` into a document field. **Readers take every RFC 3339
precision and offset**, and a received document is never re-stamped (its
proof and its item-11 digest cover what was sent). The same vectors are
pinned in each library: `tests/issued_at_precision.rs`,
`test/issued-at.test.ts`, `trusttasks/issued_at_test.go`,
`test/issued_at_test.dart` — change one, change all four.

## Consumers that route on the Type URI

A consumer that takes a document of any type and dispatches on `type` (#969)
needs the type-erased half of every binding:

- **Rust.** The binding `pack_trust_task` / `unpack_trust_task` functions take
  any `Serialize` / `DeserializeOwned` payload — `TrustTask<Value>` and
  `ErrorResponse` included — because none of them reads a `Payload` constant.
  `ErrorPayload` is a `Payload` whose `TYPE_URI` is the emitted
  `TRUST_TASK_ERROR_TYPE_URI`. The guarded path is `consume_dispatched` (and
  each consumer's `receive_dispatched` / `consume_dispatched`): it takes the
  spec policy and schema from the `AsyncDispatcher` registration the document
  routes to, refuses `unsupportedType` / `unsupportedVersion` / a payload `P`
  cannot hold **before** the item-11 record is written, and otherwise runs
  `consume_inbound`'s own body (`consume_core`). Do not give it a second copy
  of the checks. The synchronous `Dispatcher` has no such path.
  `consume_dispatched_answer` is the same function for a dispatcher whose
  handlers answer with something other than a response document — an
  `Option`, where `None` acknowledges a fire-and-forget task and is recorded
  as *no response*. **`trust-tasks-https`'s server is built on it** (#1089):
  the server keeps only the body limit, content type, bearer auth and status
  mapping, and every check it had alone is a `ConsumeChecks` option —
  `require_attribution`, `proof_did_methods`, `received_bytes` (the per-type
  `maxDocumentBytes` bound) — and the identity-first order with no
  authenticated sender is `consume_core`'s own. A check a binding needs goes
  there, not into the binding.
- **TypeScript, Go, Dart.** No dispatcher: their consume functions take the
  specification's policy as a value (`spec`), and their binding unpack is
  already untyped, so the caller unpacks, looks up the spec for `type`, and
  calls the consumer's `consume` — or answers `unsupportedType` with the
  runtime's `reject` and packs it, since nothing executes. That is the
  documented difference, not a gap to close by porting the dispatcher.

Every known cross-runtime difference — this one, Rust-only `ERROR_CODES`, TS's
closed `StandardCode`, and the open gaps — is tabled as yes / deliberately no /
todo in **`docs/runtime-parity.md`**; update it when you change a row.

## Declared error codes are generated — Rust only, so far

A spec's `errorCodes` front matter (SPEC §7.3 item 9) reaches `trust-tasks-rs`:
every generated module carries `pub const ERROR_CODES: &[DeclaredErrorCode]`
(empty when the spec declares none) and, when it declares any, a
`pub mod error_codes` with one `DeclaredErrorCode` constant per code, named for
the local part in SCREAMING_SNAKE_CASE (`withdraw:notFound` →
`error_codes::NOT_FOUND`). `schema_index::error_codes_for(type_uri)` serves the
same slice by bare request URI. The point is that an implementation emits the
code the spec declares instead of a string literal nothing checks, and can
derive a census of declared-vs-emitted codes rather than hand-keep one.

- The generator **refuses** a declaration that does not parse, whose namespace
  is not the slug or a path prefix of it, or whose local part collides with
  another's constant (`notFound` / `not_found`, or `keys:notFound` beside
  `keys/revoke:notFound`). `DeclaredErrorCode → TrustTaskCode` is infallible on
  the strength of that.
- `check-bindings` compares every Rust `ERROR_CODES` against the front matter,
  re-derived independently, as it does the policy constants.
- **TypeScript, Go and Dart do not emit them yet.** When they do, add them to the
  same comparison so the four agree with each other, not only with the front
  matter.

## `maxDocumentBytes` is generated in all four libraries

A spec's `maxDocumentBytes` front matter (per variant, only where declared; an
absent `response` never inherits the request's bound) reaches Rust as
`Payload::MAX_DOCUMENT_BYTES`, TypeScript as `SPEC.maxDocumentBytes` /
`RESPONSE_SPEC.maxDocumentBytes`, Go as `Spec.MaxDocumentBytes` /
`ResponseSpec.MaxDocumentBytes` (0: none) and Dart as
`SpecPolicy.maxDocumentBytes` on `spec` / `responseSpec`. In TypeScript, Go
and Dart it is emitted by **both** generators — the prebuilt one from front
matter (`build-{ts,go,dart}-bindings.mjs`) and the consumer-side one from the
bundle's `policy.maxDocumentBytes` (`trust-tasks-ts-gen/src/emit.ts`,
`trust-tasks-go/cmd/trust-tasks-gen`, `lib/src/generator/emit.dart`),
independently. `check-bindings` compares all four against the front matter.

Enforcement: the Rust and Dart HTTPS servers' pre-parse body limit is the
largest registered bound with a 256 KiB floor, then each document is held to
its route's bound (`malformedRequest`) — in Rust through
`ConsumeChecks::received_bytes`, which `consume_inbound` / `consume_dispatched`
check. Go's `ConsumeInbound` refuses a document of `Spec`'s own version over the
bound when given its bytes as `ConsumeOptions.Received`. A document read forward
along `extends` is never held to the implemented version's bound. The Rust
message bindings do not pass the bytes yet; the TypeScript runtime never sees
them, so its transport enforces the bound.
