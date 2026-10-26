# trust-tasks-cli

The `trust-tasks` command: pin the [Trust Task](https://trusttasks.org/)
specifications an application uses, vendor each pinned version's registry
bundle, verified by digest, and check them in CI.

```sh
cargo install trust-tasks-cli     # installs the `trust-tasks` binary
```

An application records what it uses; the registry says what that resolves to;
the build reads only the vendored files — no network at build time. See
[consumer-side generation](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/docs/design/consumer-side-generation.md) for the
model.

## Files

| File | Written by | |
|---|---|---|
| `trust-tasks.toml` | you, or `add`/`remove`/`update` | **intent**: `"vault/upsert" = "0.4"` (exact) or `"acl/grant" = "latest"`; an array of versions pins several |
| `trust-tasks.lock` | this command only | **resolution**: slug, version, Type URI and `sha256:` digest of each pinned version, sorted |
| `trust-tasks/<slug>/<M.N>.bundle.json` | this command only | the bundles, byte-identical to the registry's, so `sha256(file) == digest`; a third-party source's under `trust-tasks/@<source>/` |

Commit all three. Edits to `trust-tasks.toml` keep your comments.

## Commands

| | |
|---|---|
| `trust-tasks add <slug>…` | `vault/upsert` pins the latest live version exactly; `vault/upsert/0.4` (or `@0.4`) pins that version; `vault/upsert@latest` follows the latest live version on every `update`. Refuses a withdrawn version. Re-adding what is pinned changes nothing. |
| `trust-tasks remove <slug>…` | Unpins a task (or one version, `vault/upsert/0.3`) and deletes its vendored bundles. Offline. |
| `trust-tasks update [<slug>…] [--to-latest]` | Re-resolves. A `latest` intent moves to the latest live version, saying whether the move needs code changes: `0.5 extends 0.4: no code change needed` when the new version declares an `extends` chain that reaches the pinned one, `breaking: …` otherwise. An exact pin stays put, but takes new bytes if its draft changed in place, and a newer version is reported. `--to-latest` moves exact pins too. A pin to a version that has since been withdrawn is **refused**: it is left as it is (it still builds) and the command exits non-zero. |
| `trust-tasks check` | **Offline**, for CI: every lock entry has its vendored bundle and `sha256(file)` is the locked digest; the bundle is the version it is filed as; no stray `*.bundle.json`; the manifest and the lock agree; the lock was resolved against the registry in use. Exits 1 on any problem. |
| `trust-tasks fetch` | Restores missing or altered vendored bundles by the digest the lock pins, from the registry's by-digest store. Never moves a pin. |
| `trust-tasks list [--remote]` | The pinned versions with their status and deprecation, from the vendored bundles. `--remote` adds the latest live version in the registry, and whether moving to it is an extension or a break. |

Global options: `--registry <url>` (or `TRUST_TASKS_REGISTRY`) and
`-C, --dir <dir>` (the directory holding `trust-tasks.toml`; by default the
nearest one at or above the current directory).

### The registry

`https://trusttasks.org` unless `trust-tasks.toml` sets `registry = "…"` or
`--registry` / `TRUST_TASKS_REGISTRY` overrides it. A registry is a site root
with this layout under it — so a private mirror, a `file://` URL or a plain
directory works the same (that is how the tests run offline):

```text
specs/index.json                          every version: digest, status, extends
specs/by-digest/sha256-<hex>.json         every bundle ever published, append-only
specs/<slug>/<M.N>/bundle.json            a version's current bytes
```

Bundles are fetched **by digest**. While the by-digest store is new, a digest it
does not have yet is fetched from the version's path instead — the command says
so in a `note:` — and verified against the same digest either way.

### Third-party sources

An ecosystem can publish its own specifications under its own authority (SPEC
§6.5) — `https://tasks.acme.example/spec/payments/settle/1.0`, a DID URL, or a
URN — and an application pins them beside the registry's. Declare the source in
`trust-tasks.toml` and name its tasks `<source>:<slug>`:

```toml
[sources.acme]
url = "https://tasks.acme.example"   # https only; or: path = "../acme-registry"

[tasks]
"vault/get" = "0.3"
"acme:payments/settle" = "1.0"
```

```sh
trust-tasks add acme:payments/settle/1.0
```

A source serves the same layout as the registry. `add`, `update`, `check`,
`fetch`, `list` and `remove` work for its tasks as for official ones; the lock
records the source (`[sources.acme]`, and `source = "acme"` on each of its
entries) and the bundles are vendored under `trust-tasks/@acme/`. A slug is
unique only within its authority, so `acme:vault/get` and `vault/get` are two
different specifications and can be pinned together. Refused:

- a source's bundle under `https://trusttasks.org/`, or an official bundle
  outside it;
- a framework slug (`trust-task*`, `trust-ceremony*`) from a source;
- two pins with one Type URI;
- a source named like the first segment of a pinned official slug (`vault`
  beside `vault/get`) — they would share a generated namespace;
- a source name outside `[a-z][a-z0-9]*(-[a-z0-9]+)*`, or an `http://` url;
- a bundle whose Type URI, slug and version disagree with each other or the lock.

`TRUST_TASKS_CA_FILE` names a PEM bundle trusted *instead of* the Mozilla roots,
for a source behind an internal CA.

### Publishing a source: `trust-tasks registry build`

```sh
trust-tasks registry build --specs specs --type-uri-prefix https://tasks.acme.example --out site
```

turns a `specs/<slug>/<M.N>/{spec.md,payload.schema.json}` tree into the served
layout under `site/specs/` — bundles, `index.json` and the append-only
by-digest store — with the same builder the official registry uses. A schema may
`$ref` the registry's shared components by absolute URL
(`https://trusttasks.org/specs/_framework/0.4/framework.schema.json#/$defs/Ext`);
they are inlined from `--official-specs <checkout of specs/>` or fetched once,
here. The prefix may be an https site (`https://tasks.acme.example`), a DID
(`did:web:acme.example`) or a URN namespace (`urn:acme:tasks`), never
`https://trusttasks.org`.

## Example

```console
$ trust-tasks add vault/upsert/0.3 acl/grant@latest trust-task-discovery
    Added acl/grant 0.2 (sha256:c4f49d8d3658)
    Added trust-task-discovery 0.3 (sha256:311b477416f0)
warning: vault/upsert 0.3 is deprecated: superseded by vault/upsert/0.4
    Added vault/upsert 0.3 (sha256:40e0f5c8ed6d)
$ trust-tasks list --remote
TASK                  VERSION  INTENT  STATUS  LATEST          NOTE
acl/grant             0.2      latest  draft   0.2
trust-task-discovery  0.3      exact   draft   0.3
vault/upsert          0.3      exact   draft   0.4 (breaking)  superseded by vault/upsert/0.4
$ trust-tasks add vault/upsert
    Moved vault/upsert 0.3 → 0.4 (sha256:ba7aa0bf8049) — breaking: 0.4 does not extend 0.3; code written against 0.3 must change
$ echo tampered > trust-tasks/acl/grant/0.2.bundle.json
$ trust-tasks check
problem: trust-tasks/acl/grant/0.2.bundle.json does not match trust-tasks.lock: it is sha256:92e78d0b…, the lock says sha256:c4f49d8d… — restore it with `trust-tasks fetch` (or `git checkout`)
1 problem(s)
$ trust-tasks fetch
  Fetched acl/grant 0.2 (sha256:c4f49d8d3658)
$ trust-tasks check
ok: 3 pinned task version(s) match trust-tasks.lock
```

Output is sorted by slug, then version, so two runs over the same state print
the same thing.

## Not yet

`generate --lang <rs|ts|go|dart>` — generation from the vendored bundles is
`trust-tasks-build` (Rust, from `build.rs`) for now; the CLI subcommand follows
with milestone 3.

## License

Apache-2.0.
