# Dart: `trust_tasks` and its sibling packages

Notes specific to the Dart packages. The repository-wide guide is
[`docs/DEVELOPMENT.md`](../DEVELOPMENT.md).

## The Dart package — where it differs from the other three

`trust-tasks-dart` publishes to pub.dev as `trust_tasks`. It follows the
TypeScript shape — one library per schema, each free to declare its own `Ext`,
because a Dart library is a file and not a directory. Four things are its own:

1. **The barrel exports the runtime ONLY.** Dart's `export` is flat; there is no
   `export * as Foo`. Exporting 500 generated libraries from
   `lib/trust_tasks.dart` would collide on the first shared definition. A
   consumer imports one specification directly and prefixes it. Do not "fix"
   this by adding spec exports to the barrel.

2. **Closed value sets are `extension type`s, not `enum`s.** A Dart `enum`
   throws on a value it does not know, so a peer on a newer MINOR would crash the
   parse — squarely against SPEC §5.2. An extension type over `String` is
   zero-cost, erases to `String`, and carries an unrecognised value through. This
   puts Dart with Rust's `#[non_exhaustive]` and Go's named string type.

3. **Two version declarations must move together** — `version:` in
   `pubspec.yaml` and `packageVersion` in `lib/src/runtime/version.dart`.
   Nothing at runtime reads both, so a release that moved one and not the other
   would ship a package misreporting its own version. `npm run check-bindings`
   fails when they disagree; the first draft of the rewrite in
   `release-bump-dart.sh` did exactly that, because its regex could not match a
   version containing dots.

4. **pub.dev only publishes from a TAG-TRIGGERED workflow.** It checks that in
   the OIDC token's claims, so unlike the other three there is no publishing from
   the push-to-`main` trigger. `tag-dart` in `publish.yml` writes
   `trust-tasks-dart-v<version>`, and `publish-dart.yml` — triggered by that tag
   — publishes. ⚠️ A tag pushed by the default `GITHUB_TOKEN` does **not**
   trigger a workflow, so without `RELEASE_PLZ_TOKEN` the tag lands and the
   publish does not; `tag-dart` warns when that happens.

Paths are snake_cased for Dart's `file_names` lint, so the slug
`acl/change-role` becomes `lib/specs/acl/change_role/v0_1/payload.dart`.

### The hand-written Dart packages beside it

`trust-tasks-dart-proof` publishes as `trust_tasks_proof`: a Data Integrity
`ProofVerifier` and `signTrustTask` on Affinidi's `package:ssi`. Things that
are not obvious:

- **`pubspec_overrides.yaml` is committed** and points `trust_tasks` at
  `../trust-tasks-dart`, the Dart form of a Rust `path =` dependency. pub never
  publishes it. `verify-dart` deletes it before verifying a release, so the release
  is checked against the *published* core.
- **A sibling's `trust_tasks` constraint must admit the current core.** pub
  reads `^0.1.0` as `<0.2.0`, so until #895 every sibling declared a range that
  excluded the core it was developed against: consumers got core 0.1.19, and an
  app on core 0.9 could not add a sibling at all. `tag-dart` cannot see this —
  it verifies against the newest core the constraint admits — so when the core
  crosses a `0.x` minor, the floors must move in the same release.
  `release-pr.sh` does it: it passes the core's new version to each sibling's
  `release-bump-dart.sh`, which raises a floor that no longer admits it
  (`scripts/raise-dart-core-floor.mjs`) and releases the sibling for it even
  with no other change, and it raises the unreleased `-tsp` floor too.
  `npm run check-sibling-core-ranges` fails if one is missed. (Unlike npm this
  works inside the Release PR, because the siblings build against the in-tree
  core; on merge their `verify-dart` legs wait for the core to reach pub.dev —
  re-run them.)
- **`ssi: ">=3.9.6 <5.0.0"` spans two majors on purpose.** `package:didcomm`
  pins ssi 3, so requiring ssi 4 would make this package impossible to use
  beside a DIDComm transport. The `packages` job in `dart.yml` tests with ssi
  3.9.6 pinned as well as with whatever resolves. Do not narrow the range to get
  ML-DSA without deciding that trade explicitly. The floor is 3.9.6 because
  3.9.0–3.9.5 no longer compile against `x25519` 0.1.2.
- **`created` without a zone designator is refused**, as `trust-tasks-proof`
  refuses it. ssi's own generators emitted exactly that until ssi 4.3.0
  (affinidi-ssi-dart#305), so `signTrustTask` builds the proof itself rather
  than calling them — the package still accepts ssi 3.9. It writes `created`
  the way chrono does (no `.000`), and a test holds it to reproducing the Rust
  fixture's `proofValue` byte for byte; keep that test when touching either
  side.
- **Expect 150/160 on pub.dev, not 160.** `pqcrypto` 0.4.1, a dependency of
  ssi 4, ships a `dartdoc_options.yaml` naming a file it does not publish, so
  dartdoc fails for every package that depends on it — ssi itself included.
  The lost 10 are the documentation check, and nothing here can recover them.
- A new Dart package needs entries in five places and a manual first publish;
  RELEASING.md lists them, and `npm run check-dart-packages` fails until the
  five agree.

`trust-tasks-dart-https` publishes as `trust_tasks_https`, the HTTPS binding.

- **`HttpsServer.handle` is framework-agnostic** (`HttpsRequest` in,
  `HttpsReply` out); `dart:io` lives only in `lib/io.dart`. Keep it that way:
  importing `dart:io` from the main library would take the web away from the
  client, which is the half a Flutter web app wants.
- **Its request pipeline mirrors the Rust HTTPS server's**, including running
  expiry, freshness and identity (in §7.2 order, as `consumeInbound` does — except that with no authenticated sender identity
  runs first, so a misaddressed document stays suppressed per §8.1) *before*
  the attribution gate — `consumeInbound`
  repeats them afterwards, harmlessly. A reorder in one server changes which
  code a document that fails two checks is refused with, and the servers would
  then disagree with each other and with `consumeInbound`. The Rust server
  now runs `consume_dispatched_answer` (see [Consumers that route on the Type
  URI](../DEVELOPMENT.md#consumers-that-route-on-the-type-uri)); #1242 gave this one the same forward routing along `extends`,
  `unsupportedVersion` and per-type `maxDocumentBytes` bound. Change either
  server's order and the other in the same release.
- **Proofs are verified over the JSON as received**, not the typed re-encoding
  `consumeInbound` would hand the verifier, which drops payload members the
  generated type does not model and so breaks valid signatures.
- It deliberately has no dependency on `trust_tasks_proof`: its SDK floor is
  3.3, the proof package's is 3.6, and a server takes any `ProofVerifier`.

`trust-tasks-dart-didcomm` publishes as `trust_tasks_didcomm`, the DIDComm v2.1
binding on Affinidi's `package:didcomm`.

- **The sender is the `skid` of the ECDH-1PU layer**, found in the layers
  `unpackToPlainTextMessage` reports; the wrapping is judged from those layers,
  not from didcomm's error text. A fragment-less `skid` is an error, never a
  silent "no sender".
- **`allowedSenders` is checked before decryption.** Opening an envelope
  resolves the sender's DID, which for `did:web` is a fetch from a host the
  sender picked.
- **An authenticated body that does not deserialise is answered**
  (`malformedRequest`, §4) rather than thrown, because the sender is known.
  Only an envelope-layer failure throws.
- `package:didcomm` 2.3 pins ssi 3, so this package resolves ssi 3.9.x whatever
  its own range says; the range matches `trust_tasks_proof`'s so the two share
  one graph.

`trust-tasks-dart-tsp` publishes as `trust_tasks_tsp`, the TSP binding on
`affinidi_tsp`. It is the odd one out in two ways:

- **It cannot be published yet.** `affinidi_tsp` is not on pub.dev, so
  `pubspec_overrides.yaml` supplies it from a pinned git commit and the pinned
  `pubspec.yaml` names the hosted `^0.1.0` it will resolve against once
  published. Because of that it is deliberately **absent from the release
  lists** (`tag-dart` in `publish.yml`, `DART_PACKAGES` in `release-pr.sh`), its tag pattern
  and case arm in `publish-dart.yml`, and the `release-bump-dart.sh` case — a
  release would delete the override and fail `dart pub get`. `dart.yml` tests it
  (via the override), and `check-dart-packages` permits a tested-but-unreleased
  package. When affinidi_tsp publishes: drop the git override, add those four
  release entries, and flip the website matrix cell.
- **TSP signing is Ed25519 only** in `affinidi_tsp` (the HPKE key agreement is
  derived from the Ed25519 key), so VIDs are built from Ed25519 keys. The sealed
  payload is the same `{type, document}` envelope object the Rust crate seals —
  a test pins the two shapes equal.

`trust-tasks-dart-capability-client` publishes as `trust_tasks_capability_client`,
the Dart port of the Rust `trust-tasks-capability-client` crate: **pure wire
logic, no crypto** — document builders for the `governance/capability/*` and
`git-trust/*` families, envelope parsing, and reply classification. Notes:

- **Outcomes and replies are sealed hierarchies** (`WriteOutcome`,
  `CapabilityReply`), like the core's, so a `switch` is exhaustive. Idempotent
  success is keyed on the SPEC §8.5 extended error **code** (both snake_case and
  lowerCamelCase spellings), never the non-normative free-text `message` — the
  free-text path is opt-in via `ReplyPolicy`, matching the Rust/TS/Go ports.
- **It reuses the core's `slugFromTypeUri`** (which throws on a non-Trust-Task
  URI, so `slugOf` wraps it and returns `null`); the response-variant check is a
  `#response` suffix test. Sign built documents with `package:trust_tasks_proof`.
- **Released like the other siblings** — a `tag-dart` matrix leg, a
  `DART_PACKAGES` entry, a `publish-dart.yml` tag pattern and a
  `release-bump-dart.sh` case, which `check-dart-packages` holds in agreement.
  Its first pub.dev publish (`0.1.0`) was the manual one.
