# Releasing

**Merging is not releasing.** Anything merged to `main` sits unpublished until a
release is cut. Releases are cut by merging a **Release PR** — one kept up to
date by [release-plz](https://release-plz.dev) for the nine Rust crates, and one
kept up to date by `scripts/release-ts-pr.sh` for the npm package.

Contributing rather than releasing? You only need
[What this means for contributors](#what-this-means-for-contributors).

---

## What this means for contributors

**Two rules.**

1. **Never edit a `version = ` field in a `Cargo.toml`, or `"version"` in
   `trust-tasks-ts/package.json`.** Versions are assigned by the Release PRs,
   not by you. A version in a feature PR collides with every other PR touching
   that package.
2. **Write a conventional-commit PR title.** PRs squash-merge, so the title
   becomes the commit subject — and that subject is both the changelog entry
   published to crates.io and npm, and the signal that decides how far the
   version moves. CI lints it (`.github/workflows/commit-lint.yml`).

```
feat(rs): add a trust-tasks facade crate
fix(https): send the response payload the spec pairs with the request
spec(vault): tighten the secretKind enum
feat(rs)!: generated types are #[non_exhaustive]   <- ! marks a breaking change
```

Types: `feat` `fix` `docs` `test` `ci` `build` `perf` `refactor` `chore`
`security` `spec` `site` `revert`. (`site` is a change to the registry website
under `website/`; it reaches no published package.)

**`spec` is ours, not the standard's**, and release-plz does not know it — an
unrecognised type is scored as a **patch**. That is right for an in-place schema
edit and wrong for a **new spec family**, which is additive API surface in both
generated libraries. Use `feat(<slug>): add the … specification` when you add a
family; keep `spec(<slug>):` for edits inside one.

**Write a real commit body.** It is included in the changelog verbatim, so the
explanation you write for reviewers is the same text an external consumer reads
on crates.io. There is nothing else to do for the changelog — no fragment files,
no hand-written entries.

> **Changed from the old flow.** You no longer bump a version or write a
> `CHANGELOG.md` entry in your PR, and CI no longer requires you to. The two
> jobs that did require it — `rust.yml`'s *"Require a version bump for every
> changed publishable crate"* and `ts.yml`'s *"version bumped with the
> bindings"* — are gone. They were correct only while merging *was* publishing;
> under this model `main` is legitimately ahead of both registries between
> releases, so they would fail every ordinary PR.

---

## What gets published

**Nine crates, one npm package, one Go module and four Dart packages.**
`trust-tasks-codegen` sets
`publish = false` in its own `Cargo.toml` — it is the internal generator.

| Package | Registry |
|---|---|
| `trust-tasks` | crates.io — the facade; re-exports the other eight behind features |
| `trust-tasks-rs` | crates.io — the generated types and the §7.2 consumer pipeline |
| `trust-tasks-https`, `trust-tasks-didcomm`, `trust-tasks-didcomm-v1`, `trust-tasks-tsp` | crates.io — transport bindings |
| `trust-tasks-proof` | crates.io — proof verification backend |
| `trust-tasks-ceremony` | crates.io — Trust Ceremony helpers |
| `trust-tasks-capability-client` | crates.io |
| `@openvtc/trust-tasks` | npm — the TypeScript bindings |
| `trust-tasks-go` | none — a Go module is published by pushing a `trust-tasks-go/vX.Y.Z` tag, after which `proxy.golang.org` serves it |
| `trust-tasks-go/tsp` | none — a **separate** nested Go module for the TSP binding, published by pushing a `trust-tasks-go/tsp/vX.Y.Z` tag. **Not released yet**: its dependency `affinidi-tsp-go` has no tag, so it is required at a pseudo-version and a release waits until that is tagged. `go get` of the module already resolves the pseudo-version from the public repo. It is a separate module so the core stays dependency-free. |
| `trust_tasks` | pub.dev — the Dart bindings, published by pushing a `trust-tasks-dart-vX.Y.Z` tag, which triggers the publishing workflow |
| `trust_tasks_proof` | pub.dev — Data Integrity proof verification for `trust_tasks`, published the same way from a `trust-tasks-dart-proof-vX.Y.Z` tag |
| `trust_tasks_https` | pub.dev — the HTTPS transport binding, from a `trust-tasks-dart-https-vX.Y.Z` tag |
| `trust_tasks_didcomm` | pub.dev — the DIDComm v2.1 transport binding, from a `trust-tasks-dart-didcomm-vX.Y.Z` tag |

Adding a crate to the published set means setting `publish` back to the default
*and* checking everything it depends on is published; crates.io requires a
published crate's whole dependency closure to be published too.

---

## Cutting a release

### 1. Review the Release PR(s)

Four can be open at once. They are independent — merge any, all, or none.

**`chore: release` (release-plz, label `release`)** — the crates. It updates on
every merge to `main` and contains the version bump for each changed crate and
the changelog entries those commits produced.

The bump levels are **derived, not guessed**:
[`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks)
compares each crate's public API against the version on crates.io, so a genuine
API break moves the compatibility field whether or not anyone remembered to say
so. Every crate here is `0.x`, where cargo treats the **minor** field as the
compatibility boundary: `0.14.0` → `0.14.1` is compatible, `0.14.0` → `0.15.0`
is not.

**`chore: release @openvtc/trust-tasks <version>`** — the npm package, on the
`release-ts` branch. Same shape, computed by `scripts/release-ts-pr.sh` from the
conventional commits since the `trust-tasks-ts-v*` tag that touched `specs/`,
`trust-tasks-ts/` or `scripts/build-ts-bindings.mjs`.

**`chore: release trust-tasks-go <version>`** — the Go module, on the
`release-go` branch. Same shape again, computed by `scripts/release-go-pr.sh`
from the conventional commits since the `trust-tasks-go/v*` tag that touched
`specs/`, `trust-tasks-go/` or `scripts/build-go-bindings.mjs`.

Go has no manifest to bump, so the PR moves `const Version` in
`trust-tasks-go/trusttasks/version.go`, which stands in for one. The tag is what
actually publishes.

**`chore: release trust_tasks <version>`** — the Dart package, on the
`release-dart` branch, computed by `scripts/release-dart-pr.sh` from the
conventional commits since the `trust-tasks-dart-v*` tag that touched `specs/`,
`trust-tasks-dart/` or `scripts/build-dart-bindings.mjs`. It moves two
declarations that must stay equal: `version:` in `pubspec.yaml` and
`packageVersion` in `lib/src/runtime/version.dart`.

**`chore: release trust_tasks_proof <version>`** — the same script run as
`scripts/release-dart-pr.sh trust_tasks_proof`, on the `release-dart-proof`
branch, measured from the `trust-tasks-dart-proof-v*` tag and watching only
`trust-tasks-dart-proof/`. It moves `version:` alone; the package has no version
constant. **`chore: release trust_tasks_https <version>`** and
**`chore: release trust_tasks_didcomm <version>`** are the same again, on
`release-dart-https` and `release-dart-didcomm`, from their own tags.

> ⚠️ **Only the crates have a `cargo-semver-checks` equivalent.** The npm, Go and
> Dart bumps are only as accurate as the commit subjects: if a change breaks one
> of those APIs and nobody wrote `!` in the PR title, the package ships a patch.
> Check the diff before merging. This is the one place the four parts of a
> release differ in rigour.

> ⚠️ **A Go release cannot be unpublished.** `proxy.golang.org` caches a tag
> permanently and by design, so a tag pushed over a tree that does not build is
> served to every consumer forever and can only be superseded, never retracted.
> `publish-go` runs `go build`, `go vet` and `go test` *before* tagging for
> exactly this reason. npm and crates.io both allow a short retraction window;
> Go does not.

The `release-ts`, `release-go`, `release-dart` and `release-dart-proof` branches are **regenerated
from `main` on every push**. Do not commit to them — a force-push will take your
work.

### 2. Merge it

That's the release.

Merging the crates PR triggers `release-plz-release`, which tags each crate
(`<crate>-v<version>`), publishes to crates.io in dependency order, and creates
a GitHub Release per crate carrying its changelog section.

Merging the npm PR triggers `publish-npm`, which builds the package, publishes
it with OIDC provenance, and pushes the `trust-tasks-ts-v<version>` tag that the
*next* npm Release PR measures from.

Merging the Dart PR triggers `tag-dart`, which verifies the package analyses,
tests and dry-run-publishes, then pushes `trust-tasks-dart-v<version>`. That tag
triggers `publish-dart.yml`, which publishes to pub.dev over OIDC. **pub.dev only
accepts a publish from a tag-triggered workflow** — it checks that in the OIDC
token's claims — which is why Dart needs two workflows where the others need one.

`tag-dart` is a matrix with one entry per Dart package, run one at a time with
`trust_tasks` first. Each package is verified with its `pubspec_overrides.yaml`
**deleted** — in CI that file points a package at the in-tree `trust_tasks`, but
a consumer gets the published one. So a `trust_tasks_proof` release that relies
on core API nobody has released yet fails verification and is not tagged;
release the core first. `publish-dart.yml` looks up the tag's version on
pub.dev before publishing and does nothing when it is already there, which is
what keeps the tag written after a hand-published first version from turning
into a failed run.

> ⚠️ A tag pushed by the default `GITHUB_TOKEN` does **not** trigger a workflow.
> Without `RELEASE_PLZ_TOKEN` set, `trust-tasks-dart-v<version>` lands and
> nothing publishes. `tag-dart` prints a warning saying exactly that; the fix is
> to delete and re-push the tag from a workstation, which is an ordinary
> authenticated push and does trigger the workflow.

Merging the Go PR triggers `publish-go`, which verifies the module builds and
its tests pass, pushes the `trust-tasks-go/v<version>` tag, and asks
`proxy.golang.org` to index it. That tag is both the publication and the anchor
the *next* Go Release PR measures from — unlike the other two, there is no
separate release tag, because the go tooling requires this exact spelling for a
module in a subdirectory.

Nothing else publishes. An ordinary feature merge runs the same jobs and they do
nothing, because every version is already on its registry.

### 3. If it fails partway

Re-run the job. All four sides are idempotent: `cargo publish` skips a crate
already at that version, `publish-npm` skips a version already on npm,
`publish-go` skips a version already tagged, `tag-dart` skips a version whose
tag exists, and `publish-dart.yml` skips a version already on pub.dev. A re-run resumes rather than duplicating.

If the crates release dies mid-way with

```
failed to select a version for the requirement `trust-tasks-proof = "^0.13"`
```

that is a **publication-order** failure, and it is the exact failure the
`publication order is satisfiable` job in `rust.yml` exists to warn about — read
its log, which prints the order the dependency graph implies (including
versioned dev-dependencies, which `cargo publish` *does* resolve). Re-running
usually fixes it, because the missing crate is on crates.io by then.

---

## Setup this depends on

- **Trusted Publishing on both registries** — already configured. crates.io and
  npm each mint a short-lived token per run from the workflow's OIDC identity;
  no registry token is stored in this repo.
  ⚠️ **Every crate's Trusted Publisher is registered against the workflow
  filename `publish.yml`.** Renaming that file breaks the OIDC exchange before
  any release logic runs. See the comment at the top of it.
- **`RELEASE_PLZ_TOKEN`** — a PAT (contents + pull-requests write) or GitHub App
  token. **Set**, as of the `trust-tasks-dart-v0.1.0` tag on 2026-09-16: that tag
  was pushed by `tag-dart` and *did* trigger `publish-dart.yml`, which the
  default token cannot do. (This entry previously said "Not currently set"; that
  was stale.)

  Why it matters: GitHub suppresses workflow runs for events authored by the
  default `GITHUB_TOKEN`. Without this token a Release PR opens with no CI on it
  — meaning the one commit that publishes would be the one commit CI never built
  — and, on the Dart side, the release tag would not trigger the publish workflow
  at all. If it is ever unset, **close and reopen the Release PR** to trigger CI
  before merging, and re-push the Dart tag from a workstation to publish.
- **pub.dev automated publishing** — a one-time setup on the package's Admin
  tab: "Enable publishing from GitHub Actions", repository
  `trustoverip/dtgwg-trust-tasks-tf`, tag pattern
  `trust-tasks-dart-v{{version}}`.

  ⚠️ **The first version must be published by hand**, and this is not a guess:
  merging #470 tagged `trust-tasks-dart-v0.1.0`, `publish-dart.yml` triggered and
  ran the whole pipeline correctly — dry run clean, 853 KB archive validated,
  upload attempted — and pub.dev refused with

  ```
  Message from server: Only users are allowed to upload new packages.
  ```

  So run `dart pub publish` from `trust-tasks-dart/` signed in as a user, then
  configure the Admin tab. A package that does not exist has no Admin tab to
  configure, which is the chicken-and-egg the message is describing.

  Note that the first *automated* publish will therefore be the version after the
  one published by hand: `trust-tasks-dart-v0.1.0` already exists and pub.dev
  will not accept a second upload of a version it already has, so re-pushing that
  tag proves nothing. `0.1.1` is the first release the automation actually
  performs end to end.

  **The same applies to every new Dart package.** For `trust_tasks_proof` and
  `trust_tasks_https` (both done 2026-09-16) and `trust_tasks_didcomm`: once its PR has merged, run
  `dart pub publish` from the package's directory
  signed in as a user. Leave `pubspec_overrides.yaml` where it is — pub never
  uploads it, and says so with a hint rather than a warning. Then enable
  publishing from GitHub Actions on its Admin tab with its tag pattern
  (`trust-tasks-dart-<name>-v{{version}}`, e.g.
  `trust-tasks-dart-didcomm-v{{version}}`).
  Merging also writes the package's `-v0.1.0` tag; if that runs after your manual upload,
  `publish-dart.yml` finds the version already on pub.dev and does nothing, and
  if it runs before, it fails with the message above and your upload follows.

  Adding another Dart package means: an entry in `tag-dart` and in
  `release-dart-pr` in `publish.yml`, a tag pattern and a `case` arm in
  `publish-dart.yml`, a `case` arm in `scripts/release-dart-pr.sh`, a matrix
  entry in the `packages` job in `dart.yml`, and the manual first publish above.
  `npm run check-dart-packages` (a required-able job in `dart.yml`) fails until
  the five agree.

  **A package whose dependency is not yet on pub.dev cannot be released.**
  `trust_tasks_tsp` depends on `affinidi_tsp`, which is not published, so it is
  tested in `dart.yml` (against a git-pinned override) but left out of the four
  release entries above — a release deletes the override and `dart pub get`
  would fail on the unpublished hosted dependency. When `affinidi_tsp` reaches
  pub.dev: remove the git override from `trust-tasks-dart-tsp/pubspec_overrides.yaml`,
  add the four release entries, and do the manual first publish.
- **Nothing at all for Go.** A Go module is published by pushing a tag to a
  public repository; `proxy.golang.org` does the rest. There is no account to
  own, no token to rotate and no Trusted Publisher to misconfigure. The one
  thing it does require is that the repository stay **public** — the proxy
  cannot fetch a private one, and `go get` would fall back to a direct clone
  that most consumers cannot authenticate.

### One-time migration

release-plz decides *whether* a crate changed by comparing its packaged files
against the tarball on crates.io, so it will find the right bump with no tags at
all. It decides *what the changelog says* from the commits since that crate's
`<crate>-v<version>` tag — and `scripts/release-ts-pr.sh` uses its tag for both.
No such tags exist in this repo. Seed them once at the current `main` — every
version there is already published — **before trusting the first Release PR**:

```bash
git switch main && git pull
for c in trust-tasks trust-tasks-rs trust-tasks-https trust-tasks-didcomm \
         trust-tasks-didcomm-v1 trust-tasks-proof trust-tasks-tsp \
         trust-tasks-capability-client trust-tasks-ceremony; do
  v=$(grep -m1 '^version = ' "$c/Cargo.toml" | cut -d'"' -f2)
  git tag -s "$c-v$v" -m "$c $v"
done
v=$(node -p "require('./trust-tasks-ts/package.json').version")
git tag -s "trust-tasks-ts-v$v" -m "@openvtc/trust-tasks $v"
git push origin --tags
```

**Do not hand-seed the Go tag.** It is not in the list above on purpose — see
the next section.

Without these:

- the first crates Release PR bumps versions correctly but produces **empty
  changelog sections** — there is no range for it to read commits from;
- the `release-ts-pr` job **fails loudly** with "No trust-tasks-ts-v\* tag
  exists", by design, rather than proposing a bump from nothing.

### The Go module seeds itself — do not tag it by hand

`trust-tasks-go` needs no seeding step, and adding one is actively dangerous.

`publish-go` does not measure from a prior tag the way `release-go-pr` does. It
reads `const Version` from `trust-tasks-go/trusttasks/version.go`, asks whether
`trust-tasks-go/v<Version>` exists, and tags when it does not — so the very
first push to `main` that carries the module publishes it and writes the anchor
in one step. `release-go-pr` runs `needs: publish-go`, so by the time it looks
there is always a tag to measure from. This is exactly what happened on #468:
`publish-go` verified the tree, pushed `trust-tasks-go/v0.1.0`, and
`release-go-pr` then correctly reported nothing to release.

⚠️ **A Go tag is itself a publication, and it cannot be retracted.** Pushing
`trust-tasks-go/v0.1.0` is what makes `go get …/trust-tasks-go@v0.1.0` resolve,
and `proxy.golang.org` caches it permanently by design. The crate and npm tags
are inert anchors by comparison — a wrong one can simply be moved. A Go one can
only be superseded by a higher version.

That asymmetry is why `publish-go` runs `go build`, `go vet` and `go test`
*before* tagging, and why tagging by hand is the wrong instinct here: doing it
from a local checkout skips those checks and can publish a tree that was never
verified. If a release is genuinely stuck, fix `Version` and let the workflow
tag it.

At the time of writing the tree and both registries agree exactly, which is what
makes this migration clean:

| | manifest | registry |
|---|---|---|
| `trust-tasks` | 0.2.0 | 0.2.0 |
| `trust-tasks-rs` | 0.14.0 | 0.14.0 |
| `trust-tasks-https` | 0.16.0 | 0.16.0 |
| `trust-tasks-didcomm` | 0.15.0 | 0.15.0 |
| `trust-tasks-didcomm-v1` | 0.14.0 | 0.14.0 |
| `trust-tasks-proof` | 0.13.0 | 0.13.0 |
| `trust-tasks-tsp` | 0.14.0 | 0.14.0 |
| `trust-tasks-capability-client` | 0.14.0 | 0.14.0 |
| `trust-tasks-ceremony` | 0.2.0 | 0.2.0 |
| `@openvtc/trust-tasks` | 0.15.0 | 0.15.0 |

---

## Why the npm package, the Go module and the Dart package are released separately

release-plz manages Rust and only Rust. It has **no pre- or post-release hook**
and will not write a non-Rust manifest, so there is no supported way to make it
bump `trust-tasks-ts/package.json` inside the crates Release PR.

The options were to bolt an extra commit onto release-plz's own release branch —
which release-plz force-pushes on every run, so the commit would be repeatedly
dropped and re-applied — or to give the package its own Release PR built the
same way. This repo does the second. `scripts/release-ts-pr.sh` mirrors
release-plz deliberately: previous release is a tag, bump comes from
conventional commits since it, changelog comes from the same `cliff.toml`,
merging the PR is the release. The cost is one more PR to merge; the benefit is
that a TypeScript-only change (a fix in the hand-written `src/_runtime`
pipeline, which touches no crate) still gets a release, which it could not if it
depended on release-plz having found something to do.

The Go module is separate for a stronger reason still: it has no manifest at
all. A Go module's version *is* its git tag, so there is nothing for release-plz
or for a manifest-bumping script to write, and `scripts/release-go-pr.sh` moves
a `const Version` that exists precisely to give the Release PR something to
carry. See the comment on that constant.

The Dart package is separate for a third reason on top of both: pub.dev will
not publish from a push-to-`main` workflow at all, so the release is necessarily
two steps — tag, then publish on the tag.

`release-ts-pr` runs `needs: publish-npm`, `release-go-pr` runs
`needs: publish-go`, and `release-dart-pr` runs `needs: tag-dart` — **not**
beside them. Both fire on the same push, and the
tag each Release-PR job measures from is written by the publish job before it.
In parallel, the run that merges a release would compute its next bump against
the *previous* tag and immediately reopen a Release PR for the release that had
just gone out.

---

## Reference

| | |
|---|---|
| `release-plz.toml` | what release-plz does; the published set lives in the manifests |
| `cliff.toml` | how commits become changelog entries, for both halves |
| `scripts/release-ts-pr.sh` | the npm Release PR |
| `.github/workflows/publish.yml` | all four release jobs. **Do not rename this file** |
| `.github/workflows/commit-lint.yml` | PR title must be a conventional commit |
| `rust.yml` → `publication order is satisfiable` | tripwire for a dev-dependency edge that makes publication order impossible |
