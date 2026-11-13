<!--
The title is the changelog entry and sizes the release: a conventional commit,
e.g. `feat(<slug>): …` for a new spec family, `spec(<slug>): …` for an edit,
`fix(rs,ts): …` for library code. A `!` marks a package's public API break and
must be scoped to the package (`feat(rs)!: …`), never to a spec.
-->

## What and why

## Checklist

- [ ] Every commit is DCO signed (`git commit -s`).
- [ ] The title is a conventional commit; any `!` is scoped to the package whose API breaks.
- [ ] If `specs/` changed: all four libraries are regenerated (`npm run regen`) and committed.
- [ ] No package version and no `CHANGELOG.md` entry is edited — the Release PR does both.
- [ ] `npm run check` passes, and the tests for the code touched were run.

See [CONTRIBUTING.md](../CONTRIBUTING.md) and [docs/DEVELOPMENT.md](../docs/DEVELOPMENT.md).
