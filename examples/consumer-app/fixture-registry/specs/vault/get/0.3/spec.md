---
slug: vault/get
version: "0.3"
title: Vault — Get (acme)
summary: Read one value from acme's configuration vault by path. A third-party specification under https://tasks.example.com (SPEC §6.5) — the registry's slug and version, and a different specification.
status: draft
parties:
  - role: Reader
    requirement: REQUIRED
    member: issuer
  - role: Vault
    requirement: REQUIRED
    member: recipient
---

# acme Vault — Get 0.3

The fixture registry of `examples/consumer-app`: a specification a third
party, "acme", publishes under its own authority. Its slug and version are the
registry's `vault/get` 0.3, and it is a different specification — a slug is
unique only within its authority (SPEC §6.1) — with Type URI
`https://tasks.example.com/spec/vault/get/0.3`.

Rebuild the served layout under `site/` with:

```sh
cargo run -p trust-tasks-cli -- registry build \
  --specs examples/consumer-app/fixture-registry/specs \
  --type-uri-prefix https://tasks.example.com \
  --out examples/consumer-app/fixture-registry/site \
  --official-specs specs
```
