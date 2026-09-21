/// The released version of this package.
///
/// Mirrors `version` in `pubspec.yaml`, which is the value pub.dev publishes
/// under. Unlike Go — where a module has no manifest at all and the constant IS
/// the manifest — this is a convenience: a consumer can log which binding
/// version produced a document without reading the package metadata.
///
/// DO NOT EDIT BY HAND. `scripts/release-dart-pr.sh` rewrites it alongside the
/// pubspec, and `npm run check-bindings` fails if the two disagree. See
/// RELEASING.md — a version in a feature PR collides with every other open PR.
library;

/// The released version of `package:trust_tasks`.
const String packageVersion = '0.1.13';
