# Rust: `trust-tasks-rs` and the Rust crates

Notes specific to the Rust workspace. The repository-wide guide is
[`docs/DEVELOPMENT.md`](../DEVELOPMENT.md); library versioning, including the
leading-component bump that moves every crate in the `core` group, is under
[Library versions are semver over the library's API](../DEVELOPMENT.md#library-versions-are-semver-over-the-librarys-api-not-the-frameworks).

## Generated payload types are `#[non_exhaustive]` + builders as of 0.14.0

`trust-tasks-codegen` runs typify with `with_struct_builder(true)` and marks
every generated named-field struct and every generated enum `#[non_exhaustive]`
(tuple/newtype wrappers such as `Ext` and `ExtKey` are deliberately excluded —
typify emits no builder for them, so the attribute would remove the only
construction path). The consequence worth knowing:

- **An additive schema change is no longer a source break.** A new optional
  member arrives as one more builder setter; a new `enum` value lands behind a
  wildcard arm. This is why 0.14.0 was taken as one deliberate break — don't
  undo it to make a call site shorter.
- **Cross-crate construction must use `X::builder()` or `X::default()`.**
  Struct literals still work *inside* `trust-tasks-rs` (including its unit
  tests), so a change that compiles there can still break the binding crates.
  `cargo build --workspace --all-targets` is the check that catches it — the
  integration tests under `trust-tasks-rs/tests/` are separate crates and are
  the crate's only in-repo view of what a consumer sees.
- **`RequestPayload` pairs a request payload with its response type** and is
  what lets `HttpsClient::send` take one type parameter. It is a separate trait
  from `Payload` because associated type defaults are unstable on stable Rust;
  a spec with no `$defs.Response` gets no impl. `HttpsServer::on` still takes
  both type parameters — constraining it is an open follow-up.

## Generated payload types keep absent, empty and `null` apart

A schema can give an optional member three meanings: absent, present and empty
(`[]`, `{}`), and present as `null`. `acl/update` 0.3 reads `"label": null` as
"clear the label" and an absent `label` as "leave it unchanged". typify's own
output folds these together (an empty `Vec` is skipped on the way out, and a
`null` is read as `None`), so `trust-tasks-build` rewrites the optional
members' fields after typify runs (`trust-tasks-build/src/presence.rs`):

| Optional member's schema | Field type | absent | `null` | a value |
|---|---|---|---|---|
| array or map | `Option<Vec<T>>` / `Option<Map>` | `None` | `None` | `Some(v)`, `[]` included |
| admits `null` | `Option<Option<T>>` | `None` | `Some(None)` | `Some(Some(v))` |
| `{"type": "null"}` | `Option<()>` | `None` | `Some(())` | — |
| `true`, `{}` or annotations only | `Option<Option<Value>>` | `None` | `Some(None)` | `Some(Some(v))` |
| anything else | `Option<T>`, unchanged | `None` | `None` | `Some(v)` |

**Decoding does not refuse a `null` the schema forbids; schema validation
does.** On an optional member with no null form, a `null` reads as absent:
`{"subject": "…", "purposes": null}` on `acl/update` 0.3 ("There is no null
form") decodes as "leave unchanged" and re-encodes without `purposes`. This is
the decision recorded in [`runtime-parity.md`](../runtime-parity.md) for all
four runtimes: such a `null` is refused by payload-schema validation, not by
typed decoding. In Rust that means validating the payload **as received**:

- `consume_dispatched` validates the received payload (`PayloadPolicy::Validate`)
  before decoding it, so it refuses the document `malformedRequest`;
- `ValidatedPayload::validate_value` on the received payload refuses it;
- the typed `consume_inbound::<P>` does **not**: it validates `P` re-encoded,
  from which the `null` is already gone. A typed consumer that must refuse it
  calls `P::validate_value` on the received payload before decoding.

(`trust-tasks-rs/tests/member_presence.rs` pins all three.) Refusing it at
decode instead was tried in the change that introduced these shapes and
dropped before release: a `deserialize_with` on each of the ~14,600 such
fields in the prebuilt tree cost about 2 GiB of compiler memory, and it made
Rust the only runtime refusing without a validator.

Whether a schema admits `null` is decided from `type`, `const`, `enum`,
`oneOf` / `anyOf` / `allOf`, `$ref` and a `not` of exactly `null`; a schema
those keywords don't settle (no `type`, say) is taken to admit it, so the
generator can keep a `null` the schema would refuse (validation still refuses
it) but never loses one it accepts. A `$ref` is read with JSON Schema 2020-12
semantics (its siblings apply too), which every registry schema uses. A `true`
member is treated like `{}`: any value, `null` included.

Generation fails rather than guess where a `null` the schema accepts could be
lost (read as absent), and the error names the member:

- A member that admits `null` but that typify renders as a bare collection
  instead of an `Option` (a type-less `{"additionalProperties": …}`, say). The
  error also names the owning struct. Give the member an explicit `type` that
  includes `"null"`, or one that excludes it.
- A member whose nullable tag typify dropped. Each tag carries its own id, and
  every one must be read back off a generated field. Only a tag on a member
  under a `oneOf` / `anyOf` branch can be accounted for by a required enum
  variant field of the same name, since typify writes no doc on a variant's
  fields. Move the member's schema into a `$defs` type, or report it as a
  `trust-tasks-build` bug. One case is not caught: where typify copies a
  description onto two fields, the tag read back off one copy also accounts
  for the other.

A member that admits `null` reads through one helper, `present`, which maps a
present member, `null` included, to `Some`; the other rewritten fields
(arrays and maps) need none. Every rewritten field is skipped on the way out
only when it is `None`, so a decoded document re-encodes with the same members
(except a forbidden `null`, which reads as absent). In `trust-tasks-rs` the
helper is the crate-private `specs::__presence`, shared
by every prebuilt module (the leading underscores keep it apart from the family
modules beside it: a slug segment starts with a letter); a module an
application generates carries its own private copy, so it needs nothing new
from the runtime's `__private`.
Required members are not touched. A member with a declared JSON Schema
`default` is left as typify renders it, so an absent one reads as that
default, **except** a default equal to the type's intrinsic one (`[]`, `{}`,
`null`, `false`, `0`, `""`): typify treats that as no default, so the member is
rewritten like any other and an absent one reads as `None`.

The builder setters take the field's type through `TryInto`, so:

- `.redacted_fields(vec)` still compiles and always sends the array, `[]`
  included. Leave the setter out to omit the member.
- On a nullable member, `.label(Some(x))` sends the value
  (`Option<T>` converts into `Option<Option<T>>`), and
  **`.label(None::<T>)` sends an explicit `null`**. The turbofish is needed:
  a bare `None` no longer infers. Leave the setter out (or pass
  `None::<Option<T>>`) to omit the member. A bare `T` no longer compiles; wrap
  it in `Some`.
- Reading: `p.label.flatten()` gives the value when you don't care why there
  isn't one; match `Some(None)` to see an explicit `null`.
- Any other optional member keeps its `Option<T>` setter, type and decoding.
- Decoding a `null` on an optional array or map with no null form is now
  **accepted** and read as absent. typify's bare `Vec` / map could not read a
  `null`, so `{"redactedFields": null}` used to fail to decode, and a typed
  consumer decoding `TrustTask<P>` before `consume_inbound::<P>` refused it
  there. Like every
  other forbidden `null`, it is now left to validation of the received payload
  (above).

These are public type changes, so the change that introduced them was a
breaking release of `trust-tasks-rs`, and an application generating its own
types through `trust-tasks-build` sees the same shapes.

## The per-module Rust generator lives in `trust-tasks-build`

`trust-tasks-codegen` writes the tree, the indexes and the conformance tests,
but each module is generated by `trust_tasks_build::generate_module` **from the
version's registry bundle** (`build_bundle` → `Bundle::from_slice`), the same
call an application's `build.rs` makes (`docs/design/consumer-side-generation.md`).
The schema passes, typify settings, `render_module` and the `errorCodes` reader
are there, not in `trust-tasks-codegen/src/main.rs`. Anything the generator
emits must be reachable both as `crate::…` (here) and as `::trust_tasks_rs::…`
(an application) — a new runtime item a module names may need a `#[doc(hidden)]`
door like `__private` or `DeclaredErrorCode::__from_declaration`, and
`examples/consumer-app` (the `consumer-app` job) is what catches a missing one.

## The prebuilt tree spells each shared `$defs` entry once

A shared schema (a family's `_shared/` key-roles or device-binding schema, the
framework's `Ext`, …) is inlined into every version that references it, so the
same `$defs` entry used to sit verbatim in hundreds of `PAYLOAD_SCHEMA`
literals — about 9.7 MB of the tree's 12 MB of schema text, 2.2 MB of it
distinct — and it is what took the `.crate` to within 0.1 MiB of crates.io's
10 MiB upload limit (the `package` job in `rust.yml`). After generating every
module, `trust-tasks-codegen` (`src/shared_defs.rs`) writes each entry that at
least two literals carry, and that is long enough to be worth it, once into
`src/specs/shared_defs.rs` as a `macro_rules!` expanding to its string
literal, and rewrites each literal into a `concat!` of the pieces naming it.
`concat!` expands the macros at compile time, so **every `PAYLOAD_SCHEMA` is
the same `&'static str`, byte for byte**; the rewrite reassembles each one and
compares it with the original before replacing it, and leaves alone anything
that does not reassemble. It saved 1.07 MiB of the `.crate`.

- It is the prebuilt tree only. `trust_tasks_build::generate_module` is
  unchanged — an application's `build.rs` output is one module with nothing to
  share with — so a module here is still what `generate_module` emits, except
  for the spelling of its schema constants.
- It is not one of the "change one, change all" rules: no schema changes, only
  how one Rust constant is spelled. The TypeScript, Go and Dart generators are
  not mirrored; their packages have no upload limit near their size.
- `check-bindings` reads the Rust schema constants from source, so it resolves
  the `concat!` and the `shared_defs` macros the way rustc does. A generated
  module that names a macro the file does not define fails it, and the build.
- Macro names are the definition's name and a hash of its text, so a changed
  definition gets a new macro and an unchanged one keeps its name across
  regenerations.

## `trust-tasks-rs` unit tests see only the `acl` family

`cargo test` compiles a library twice — as the library, and again with
`cfg(test)` as its unit-test harness — and the generated tree is most of the
crate (`du -sh trust-tasks-rs/src/specs`; over 90 MB at the time of writing). So the generator keeps that second build small:

- **Every gated family except those in `UNIT_TEST_FAMILIES`** (in
  `trust-tasks-codegen`, currently `["acl"]`) carries `#[cfg(not(test))]` in
  `src/specs/mod.rs` and `src/schema_index.rs`. A `#[cfg(test)]` module in the
  hand-written `src/*.rs` that names `crate::specs::vault::…` fails with an
  unresolved path: add the family to the list, or write it as an integration
  test under `tests/`, which links the full library.
- **The per-spec conformance tests are the generated integration test
  `tests/conformance/`**, not a `mod conformance` in each module. They only use
  public items. The directory is generated and rewritten whole, so never edit
  it by hand.

Measured on the `validate` build: 63s → 23s, peak memory 9.0 GB → 4.3 GB.

**A single family builds and tests on its own** —
`cargo test -p trust-tasks-rs --features vault` is about
13 s and 1 GB cold, against minutes and ~6 GB for the whole workspace — and the
`test-single-family` job in `rust.yml` keeps it that way for `acl`, `vault` and
`keys`. So a test naming a family must sit behind that family's feature: the
`acl`-specimen unit-test modules are `#[cfg(all(test, feature = "acl"))]`, and an
integration test file that uses a family opens with `#![cfg(feature = "…")]`
for each one it names.


## The Rust message bindings share one consumer — `GuardedConsumer`

`DidcommConsumer`, `TspConsumer` and `DidcommV1Consumer` are thin wrappers
that dereference to `trust_tasks_rs::GuardedConsumer` (#1096): the item-11
record, the freshness bound, and `consume_body` / `consume_body_dispatched`,
which parse the body a binding opened. A binding keeps only its unpack (an
`open` that stops short of parsing) and its envelope check (DIDComm's thread
headers), passed as a closure. Do not give a binding its own copy back.

An authenticated body that does not parse, or that the envelope check
refuses, is **answered** — `ConsumeOutcome::Rejected(malformedRequest)`,
addressed to the transport-authenticated sender, never the unchecked in-band
`issuer`, and refused before the item-11 record is written — as the Go and
Dart bindings do. Only a body with no string `id` / Trust Task `type` to name
in `inResponseTo`, or a transport with no authenticated sender, is an
`Err` (`UnanswerableBody`, mapped to the binding's own error).
