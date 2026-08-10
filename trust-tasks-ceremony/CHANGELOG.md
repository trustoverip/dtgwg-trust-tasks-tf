# Changelog

All notable changes to `trust-tasks-ceremony` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Versioning follows [Semantic Versioning](https://semver.org/) over this
library's own API — not the framework's.**

## [0.1.1] - 2026-08-10

### Changed

- Dropped the `trust-tasks-rs` dependency, which was never used. `verify()`
  takes envelope facts (the recorder) as parameters rather than a `TrustTask`,
  deliberately — the party that signed a receipt is an envelope fact and a
  payload could claim anything — so the crate never needed the envelope type.
  It is therefore unaffected by envelope version changes.

## [0.1.0] - 2026-08-10

### Added

- Initial release. Verification for Trust Ceremonies: the `Digester` trait and
  its default JCS/SHA-256 backend, the completion-predicate evaluator, and
  `verify()` implementing the verifier conformance rules of
  `trust-ceremony-receipt/0.1`.
- `Outcome::Unverifiable` is distinct from a failure: a verifier that cannot
  resolve or match the definition has learned nothing, which is not the same as
  having learned the receipt is bad.
- The recorder's own `complete` flag is never consulted — completion is decided
  by evaluating the definition's rule, and a receipt with no terminal step is
  reported as a prefix however that flag is set.
