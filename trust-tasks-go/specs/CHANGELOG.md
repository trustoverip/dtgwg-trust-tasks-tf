# Changelog

All notable changes to `trust-tasks-go/specs` (the generated specification
packages) are documented here. Entries are generated from conventional commits
by the release automation (see RELEASING.md).

## 0.1.1 — 2026-11-17


### Added

- **vtc/community/logo**: Add vtc/community/logo/update/0.1 and vtc/community/logo/show/0.1 (#1324)

* feat(vtc/community/logo): add vtc/community/logo/update/0.1

  An administrator hands the community a new logo, which the community
  re-encodes, strips of metadata and stores in place of the one it holds;
  `image: null` removes it. The answer is the stored re-encoding's digest,
  media type, size and dimensions.

