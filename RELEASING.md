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
`security` `spec` `revert`.

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

**Nine crates and one npm package.** `trust-tasks-codegen` sets
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

Adding a crate to the published set means setting `publish` back to the default
*and* checking everything it depends on is published; crates.io requires a
published crate's whole dependency closure to be published too.

---

## Cutting a release

### 1. Review the Release PR(s)

Two can be open at once. They are independent — merge either, both, or neither.

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

> ⚠️ **The npm side has no `cargo-semver-checks` equivalent.** Its bump is only
> as accurate as the commit subjects: if a change breaks the TypeScript API and
> nobody wrote `!` in the PR title, the package ships a patch. Check the diff
> before merging. This is the one place the two halves of a release differ in
> rigour.

The `release-ts` branch is **regenerated from `main` on every push**. Do not
commit to it — a force-push will take your work.

### 2. Merge it

That's the release.

Merging the crates PR triggers `release-plz-release`, which tags each crate
(`<crate>-v<version>`), publishes to crates.io in dependency order, and creates
a GitHub Release per crate carrying its changelog section.

Merging the npm PR triggers `publish-npm`, which builds the package, publishes
it with OIDC provenance, and pushes the `trust-tasks-ts-v<version>` tag that the
*next* npm Release PR measures from.

Nothing else publishes. An ordinary feature merge runs the same jobs and they do
nothing, because every version is already on its registry.

### 3. If it fails partway

Re-run the job. Both sides are idempotent: `cargo publish` skips a crate already
at that version, and `publish-npm` skips a version already on npm. A re-run
resumes rather than duplicating.

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
  token. **Not currently set.** GitHub suppresses workflow runs for events
  authored by the default `GITHUB_TOKEN`, so without it a Release PR opens with
  no CI on it — meaning the one commit that publishes would be the one commit CI
  never built. Until the token exists, **close and reopen the Release PR** to
  trigger CI before merging it. This applies to both Release PRs.

### One-time migration

release-plz anchors each crate's changelog to the tag of its last release, and
`scripts/release-ts-pr.sh` does the same for the npm package. No such tags exist
in this repo. Seed them once at the current `main` — every version there is
already published — **before trusting the first Release PR**:

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

Without these:

- the first crates Release PR bumps versions correctly but produces **empty
  changelog sections** — there is no range for it to read commits from;
- the `release-ts-pr` job **fails loudly** with "No trust-tasks-ts-v\* tag
  exists", by design, rather than proposing a bump from nothing.

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

## Why the npm package is released separately

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

`release-ts-pr` runs `needs: publish-npm` — **not** beside it. Both fire on the
same push, and the tag `release-ts-pr` measures from is written by
`publish-npm`. In parallel, the run that merges a TS release would compute its
next bump against the *previous* tag and immediately reopen a Release PR for the
release that had just gone out.

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
