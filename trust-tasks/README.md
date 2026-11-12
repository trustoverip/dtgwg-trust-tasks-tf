# trust-tasks

One dependency line for the [Trust Tasks](https://trusttasks.org/) framework.

```sh
cargo add trust-tasks --no-default-features --features https,proof-affinidi
```

which writes

```toml
trust-tasks = { version = "…", default-features = false, features = ["https", "proof-affinidi"] }
```

with the current version. Keep `default-features = false`: the default,
`all-specs`, compiles every prebuilt specification module — deprecated, and a
build measured in minutes and gigabytes. An application generates the tasks it
uses instead (below).

The framework ships as several crates, because each transport binding drags in
a different (and heavy) dependency tree and you should only pay for the one you
use. That split is right for a build; it is a tax at the front door. This crate
is the front door: one name, and one version — the crates it re-exports release
together at the same version as this one.

Everything here is a `pub use`. There are no wrapper types and nothing to keep
in step: `trust_tasks::TrustTask` **is** `trust_tasks_rs::TrustTask`, and
`trust_tasks::https::HttpsClient` **is** `trust_tasks_https::HttpsClient`.
Dropping the facade later is a find-and-replace, not a migration.

## Features

| feature | gives you | crate |
|---|---|---|
| *(always on)* | `TrustTask`, `consume_inbound`, `RejectReason`, `issued_at_now` at the crate root | `trust-tasks-rs` |
| `https` | `trust_tasks::https` — typed client + axum server | `trust-tasks-https` |
| `didcomm` | `trust_tasks::didcomm` — DIDComm v2.1 | `trust-tasks-didcomm` |
| `didcomm-v1` | `trust_tasks::didcomm_v1` — Aries-lineage agents | `trust-tasks-didcomm-v1` |
| `tsp` | `trust_tasks::tsp` — ToIP Trust Spanning Protocol | `trust-tasks-tsp` |
| `proof-affinidi` | `trust_tasks::proof` — signing and proof verification | `trust-tasks-proof` |
| `ceremony` | `trust_tasks::ceremony` — Trust Ceremony receipts | `trust-tasks-ceremony` |
| `capability-client` | `trust_tasks::capability_client` — capability/git-trust wire helpers | `trust-tasks-capability-client` |
| `validate` | runtime JSON Schema validation of payloads | `trust-tasks-rs` |
| `all-transports` | all four transport modules | — |
| `https-jwt` | `trust_tasks::https::JwtBearerAuth` | `trust-tasks-https` |
| `all-specs` *(default)* | every prebuilt spec family under `trust_tasks::specs` — **deprecated**; turn it off with `default-features = false` | `trust-tasks-rs` |

Most deployments want **one transport plus `proof-affinidi`**: a task whose
specification declares `proof` REQUIRED cannot be produced or consumed without a
signer and a verifier.

## What this crate does not forward

`trust-tasks-rs` carries one feature per prebuilt spec family (`vault`, `acl`,
`keys`, …). Only the `all-specs` umbrella is forwarded. **If you are trimming spec families,
or subtracting a transport crate's own defaults, depend on those crates
directly** — that is the supported answer, not a workaround.

## Getting started

[`GETTING-STARTED.md`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/GETTING-STARTED.md)
walks a signed `acl/grant` round trip end to end, in Rust, TypeScript, Go and
Dart, and names the traps. Its Rust is extracted from an example application
that uses this facade with `default-features = false` and generates the one
task it speaks, `acl/grant/0.3`, in `build.rs`
([`examples/getting-started`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/examples/getting-started)):

```sh
cargo run -p trust-tasks-example-getting-started
```

The facade's default feature, `all-specs`, compiles every prebuilt
specification module; those are deprecated, and an application generates the
tasks it uses instead (see GETTING-STARTED §1). That is why the dependency line
above turns default features off.

## License

Apache-2.0 — see [`LICENSE`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks/LICENSE), included in the published crate.
