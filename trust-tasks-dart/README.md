# trust_tasks

Dart bindings for the [Trust Tasks](https://trusttasks.org) framework: a
generated payload type for every specification in the registry, and the
hand-written SPEC.md §7.2 consumer pipeline that makes those types mean
something.

The fourth of four reference implementations, alongside
[`trust-tasks-rs`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-rs),
[`@openvtc/trust-tasks`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-ts)
and
[`trust-tasks-go`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-go).
All four must reach the same verdict on the same document; `npm run
check-bindings` is what holds them to it.

```console
dart pub add trust_tasks
```

Requires Dart 3.3 or later. **No dependencies** — the cryptosuite and the JSON
Schema engine are yours to choose, so this package declares interfaces for them
and implements neither.

## Importing

The package's own library exports the **runtime only**. Dart's `export` is flat
— there is no namespaced re-export — so exporting 500 generated libraries from
one barrel would collide on the first shared definition anyone calls `Ext`.
Import the specification you need directly, with a prefix:

```dart
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
```

Each generated library exports a uniform surface:

| Identifier | |
|---|---|
| `Payload`, `Response` | the request and response payload classes |
| `typeUri`, `responseTypeUri` | the Trust Task type URIs (`responseTypeUri` only where the specification defines a success response — SPEC §4.4.1) |
| `payloadSchemaJson`, `responsePayloadSchemaJson` | the payload schema as JSON text, cross-file `$ref`s already inlined |
| `spec`, `responseSpec` | `SpecPolicy` values carrying the §7.2 policy flags and the schema above |

Paths are snake_cased for Dart's `file_names` lint, so the slug `acl/change-role`
becomes `specs/acl/change_role/v0_1/payload.dart`.

## Consuming a document

`consumeInbound` runs SPEC §7.2 items 2 and 4–8, plus the two stateful checks —
the freshness bound of item 4 and the duplicate-execution record of item 11 —
and then either calls your handler or builds the correctly-routed error response
per §8.1.

```dart
// One guard per consumer, not one per document: it IS the record.
final guard = InMemoryReplayGuard();

final outcome = await consumeInbound<acl_grant.Payload, acl_grant.Response>(
  transport: transport,
  spec: acl_grant.spec,
  proofPolicy: ProofPolicy.verify(myVerifier),
  payloadPolicy: PayloadPolicy.validate(mySchemaEngine),
  // acl/grant is consequential: a replayed envelope must not grant twice.
  checks: consequentialChecks(guard),
  doc: doc,
  myVid: 'did:web:maintainer.example',
  now: DateTime.now(),
  newErrorId: () => uuid.v4(),
  payloadToJson: (p) => p.toJson(),
  handler: (accepted, parties) async => respondWith(
    accepted,
    uuid.v4(),
    acl_grant.Response(entry: await applyGrant(accepted.payload.entry, parties)),
  ),
);

switch (outcome) {
  case Handled(:final response):
    emit(response);
  case Rejected(:final error):
    emit(error); // already addressed per §8.1
  case Suppressed(:final reason):
    log(reason); // §8.1 — no response may be emitted
  case Accepted():
    break; // fire-and-forget (§4.4.1): nothing to emit
  case DuplicateOutcome(:final priorResponse):
    // §7.2 item 11 — already executed. NOT an error.
    if (priorResponse != null) emit(priorResponse);
}
```

The outcome is a sealed hierarchy, so that `switch` is exhaustive and the
compiler tells you if a case is missing.

`payloadPolicy` and `checks` are **required**. Both have an obvious "skip it"
setting, and defaulting to it would silently reproduce the defects they exist to
remove: a payload nobody validated, and a consequential task executed twice by an
ordinary mediator retry.

## Three things worth knowing

**The generated types carry no schema constraints.** No `minLength`, no
`pattern`, no `minimum`, and no `oneOf` mutual exclusion —
`auth/revoke-session`'s "`sessionId` xor `all`" is a class with both members
optional. This matches the TypeScript and Go bindings exactly, and it is why
`payloadSchemaJson` ships with every library: SPEC §7.2 item 2 is where those
rules are enforced, and you enforce them by handing that schema to a JSON Schema
engine through `PayloadValidator`.

**Closed value sets are extension types, not enums.** A Dart `enum` throws on a
value it does not know, so a peer on a newer MINOR would crash your parse — in
direct conflict with SPEC §5.2's forward-compatibility requirement. An
`extension type const Effect(String value)` is zero-cost, erases to `String`,
keeps `==` and the named constants, and carries an unrecognised value through
without complaint. Compare against the constants and treat anything else as
unrecognised.

**Absent is not empty.** Optional members are nullable, so `List<String>?`
distinguishes absent (`null`) from present-and-empty (`[]`) natively. Several
specifications make that distinction load-bearing — `acl`'s `allowedKeys`
documents at length that "PRESENT-BUT-EMPTY means authorized on NO keys — the
opposite of absent". Dart gets this right for free where Go needed a
pointer-to-slice.

Unrecognized **top-level** members survive a round trip in
`TrustTaskDocument.extra`, per the §7.1 / §7.2 guidance to preserve them.

## Regenerating

From the repository root:

```console
npm run build-dart-bindings
```

The generator runs `dart format` on its own output, so the Dart SDK must be on
your PATH. The `bindings-drift` job in `.github/workflows/dart.yml` fails if the
committed tree differs from what the generator produces.

A change under `specs/` must regenerate **all four** libraries in the same PR —
see the repository `CLAUDE.md`.

## Licence

Apache-2.0.
