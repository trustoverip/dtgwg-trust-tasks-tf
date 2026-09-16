/// The Trust Tasks framework runtime for Dart — the SPEC.md §7.2 consumer
/// pipeline, the document envelope, and the transport seam.
///
/// Hand-written, unlike everything under `lib/specs/`, which is generated from
/// the spec registry by `scripts/build-dart-bindings.mjs`. Shipped in the same
/// package so a consumer gets the types and the checks that make them meaningful
/// from one `dart pub add`, as trust-tasks-rs, @openvtc/trust-tasks and
/// trust-tasks-go do.
///
/// The entry point is [consumeInbound].
///
/// ## Importing a specification
///
/// This library deliberately exports the **runtime only**. Dart's `export` is
/// flat — there is no namespaced re-export — so exporting 500 generated
/// libraries here would collide on the first shared definition anyone calls
/// `Ext`. Import the one specification you need directly, with a prefix:
///
/// ```dart
/// import 'package:trust_tasks/trust_tasks.dart';
/// import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
/// ```
///
/// ## Parity
///
/// This package mirrors trust-tasks-rs, @openvtc/trust-tasks and trust-tasks-go
/// check for check. A Dart consumer must reach the same verdict on the same
/// document as any of them, or the reference implementations disagree about what
/// conforms. Where a signature differs it is because Dart's type system forced
/// it, and the reason is written at the difference.
///
/// ## Dependencies
///
/// None, deliberately: the cryptosuite and the JSON Schema engine are the
/// consumer's to choose, so [ProofVerifier] and [PayloadValidator] are
/// interfaces this package does not implement.
library;

export 'src/runtime/canonical.dart';
export 'src/runtime/codes.dart';
export 'src/runtime/consume.dart';
export 'src/runtime/document.dart';
export 'src/runtime/freshness.dart';
export 'src/runtime/replay.dart';
export 'src/runtime/transport.dart';
export 'src/runtime/version.dart';
