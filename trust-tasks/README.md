# trust-tasks

One dependency line for the [Trust Tasks](https://trusttasks.org/) framework.

```toml
trust-tasks = { version = "0.1", features = ["https", "proof-affinidi"] }
```

The framework ships as eight independently-versioned crates, because each
transport binding drags in a different (and heavy) dependency tree and you
should only pay for the one you use. That split is right for a build; it is a
tax at the front door. This crate is the front door.

Everything here is a `pub use`. There are no wrapper types and nothing to keep
in step: `trust_tasks::TrustTask` **is** `trust_tasks_rs::TrustTask`, and
`trust_tasks::https::HttpsClient` **is** `trust_tasks_https::HttpsClient`.
Dropping the facade later is a find-and-replace, not a migration.

## Features

| feature | gives you | crate |
|---|---|---|
| *(always on)* | `TrustTask`, `consume_inbound`, `specs`, `RejectReason` at the crate root | `trust-tasks-rs` |
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
| `all-specs` *(default)* | every spec family; `default-features = false` for none | `trust-tasks-rs` |

Most deployments want **one transport plus `proof-affinidi`**: a task whose
specification declares `proof` REQUIRED cannot be produced or consumed without a
signer and a verifier.

## What this crate does not forward

`trust-tasks-rs` carries 26 per-spec-family features (`vault`, `acl`, `keys`, …).
Only the `all-specs` umbrella is forwarded. **If you are trimming spec families,
or subtracting a transport crate's own defaults, depend on those crates
directly** — that is the supported answer, not a workaround.

## Getting started

[`GETTING-STARTED.md`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/GETTING-STARTED.md)
walks a signed `acl/grant` round trip end to end, in Rust and TypeScript, and
names the traps. Its Rust is extracted from this crate's example:

```sh
cargo run -p trust-tasks --features https,proof-affinidi \
    --example acl_grant_roundtrip
```

## License

Apache-2.0. See
[LICENSE.md](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/LICENSE.md)
at the repo root.
