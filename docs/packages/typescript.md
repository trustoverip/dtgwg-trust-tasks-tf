# TypeScript: `@openvtc/trust-tasks` and its sibling packages

Notes specific to the npm packages. The repository-wide guide is
[`docs/DEVELOPMENT.md`](../DEVELOPMENT.md).

## The hand-written TypeScript packages

`@openvtc/trust-tasks-proof` (dir `trust-tasks-ts-proof`) is the first sibling
package to the core `@openvtc/trust-tasks`. Notes:

- **Proof crypto is rolled on `@noble`**, not digitalbazaar — its
  jsonld-signatures/jsonld stack fits our bare JCS documents poorly and is
  heavy, and the ecosystem (vti-tsp-js, vti-didcomm-js, pnm-core) is @noble-based
  and browser-first. The verifier is byte-compatible with the Rust and Dart proof
  libraries (a test reproduces the Rust `proofValue`), and @noble is the place
  ML-DSA (PQC) lands later. eddsa-jcs-2022 + ecdsa-jcs-2019; did:key only.
- **Tests compile to `.test-build/` and import from `../src`** (the core's
  convention), run with `node --test`. The npm package ships `dist/` only.
- **The core is a peer dependency, imported as `@openvtc/trust-tasks/runtime`.**
  A regular dependency installs a second core — every prebuilt module in it —
  beside the application's, and the package root loads all of them. Until this
  changed every sibling declared `^0.19.0`, which npm reads as `<0.20.0`, while
  the core reached 0.27. When the core crosses a `0.x` minor, raise the
  siblings' `peerDependencies` (and `devDependencies`) floor in the PR right
  **after** that release publishes — not before: the siblings install the core
  from npm, so a floor naming an unpublished core breaks `npm ci` in every
  sibling job. `check-sibling-core-ranges` therefore only warns on the Release
  PR (it holds the siblings to the newest core on npm while the checkout's is
  unpublished), and fails every PR once the new core is on npm until the floors
  move. Releasing that PR releases the siblings.
- **Release wiring is per package.** Each npm package has a `publish-npm`
  matrix leg and a `TS_PACKAGES` entry in `scripts/release-pr.sh`; its first
  npm publish is manual (RELEASING.md).

`@openvtc/trust-tasks-tsp` (dir `trust-tasks-ts-tsp`) is the TSP binding on
`@openvtc/vti-tsp-js`, which is **key-based and does no DID resolution** — the
consumer takes the recipient's raw Ed25519/X25519 private keys and a
`resolveSender(vid)` callback for the sender's public keys (the Go-style model).
Its sealed `{type, document}` envelope matches the Rust/Dart/Go bindings; refusals
are `ErrorResponse` returns, not throws (the TS core's convention).

`@openvtc/trust-tasks-capability-client` (dir `trust-tasks-ts-capability-client`)
is the TypeScript port of the Rust `trust-tasks-capability-client` crate: **pure
wire logic, no crypto and no transport** — document builders for the
`governance/capability/*` and `git-trust/*` families, envelope parsing, and reply
classification. It computes a Type URI's slug/response-variant from the string
(the version is always the final path segment), rather than importing a parser.
Idempotent success is keyed on the SPEC §8.5 extended error **code**, never the
non-normative free-text `message` — the deprecated free-text path is opt-in via
`ReplyPolicy`. Sign built documents with `@openvtc/trust-tasks-proof`. Like the
other siblings, its first npm publish is manual and its publish wiring + website
"shipped" flip follow that (RELEASING.md).
