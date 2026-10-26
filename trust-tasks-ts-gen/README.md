# @openvtc/trust-tasks-gen

Generate TypeScript modules for exactly the Trust Tasks an application pins, from
the registry bundles vendored beside its `trust-tasks.lock`. The modules use the
runtime in [`@openvtc/trust-tasks`](../trust-tasks-ts), imported as
`@openvtc/trust-tasks/runtime`, so the application never loads the prebuilt
modules for the whole registry.

```sh
trust-tasks add vault/get acl/grant          # the `trust-tasks` CLI: writes trust-tasks.lock + trust-tasks/
npm i @openvtc/trust-tasks
npm i -D @openvtc/trust-tasks-gen
npx trust-tasks-gen-ts --lock trust-tasks.lock --out src/trust-tasks
```

```ts
import { consumeInbound } from "@openvtc/trust-tasks/runtime";
import { VaultGet_v0_3, declareExtends } from "./trust-tasks/index.js";

declareExtends(); // once at start-up: the locked versions' `extends` declarations
// VaultGet_v0_3.Payload, .Response, .SPEC, .PAYLOAD_SCHEMA, … as in @openvtc/trust-tasks
```

`trust-tasks generate --lang ts` runs the same thing. A complete application is
[`examples/consumer-app-ts`](../examples/consumer-app-ts).

## What it does

- Reads `trust-tasks.lock` (strictly: the shape the CLI writes, nothing else) and,
  for each `[[task]]`, `trust-tasks/<slug>/<M.N>.bundle.json` — or
  `trust-tasks/@<source>/<slug>/<M.N>.bundle.json` for a task from a third-party
  source. Every bundle must hash to its locked digest before anything is generated;
  otherwise: `vendored bundle does not match trust-tasks.lock — run 'trust-tasks check'`.
- Writes `<out>/<slug>/<M.N>/payload.ts` per task (`<out>/<source>/<slug>/<M.N>/…`
  for a sourced one), `<out>/index.ts` (one namespace per task, `AclGrant_v0_2`;
  one per source, `Acme`, holding that source's tasks), and `<out>/_extends.ts`.
- Owns `<out>`: rewrites it whole, removes stale files, and marks it with
  `.trust-tasks-generated`. A non-empty directory without the marker is refused.
- Skips, with a note: withdrawn versions, and the framework's own specifications
  (`trust-task-*`, `trust-ceremony-*`), which the runtime carries.
- Output is deterministic: the same lock and bundles give byte-identical files.
  It depends on `json-schema-to-typescript` and `prettier` at exact versions for
  that reason.

## How the output compares with the prebuilt package

Generated from current bundles, every module is the one `@openvtc/trust-tasks`
ships for that version, byte for byte from its first import to its last
constant, banner aside. Shared definitions are hoisted into one
`<out>/_shared/components.ts` as the package hoists them: a bundle's
`defOrigins` names each definition that was spliced in from a shared schema,
and the one shared document it came from, so a definition reached by several
locked tasks is declared once and each module imports it and re-exports it
under its own name (`AclGrant_v0_2.AclEntry`). Three things differ, and cannot
not:

- **The identifiers inside `_shared/components.ts` depend on the lock.** A name
  with one shape among the locked tasks is bare; one with several is qualified
  with each shape's family and version (`AclEntry_AclV0_2`). The package decides
  the same question over the whole registry, and never renames a published
  identifier (`trust-tasks-ts/component-names.lock.json`) — history no bundle
  carries. So an application pinning only `acl/grant` 0.2 gets `AclEntry`
  where the package has `AclEntry_AclV0_2`, and generating every version
  differs in one name (`TrustRecord`, which shipped bare before a second shape
  appeared). The barrel does not export the components module for this reason:
  use the names each module re-exports, which never change.
- **`PAYLOAD_SCHEMA` has the bundle's member order** (sorted, RFC 8785) except
  in `properties` and the root `$defs`, which keep the order the package's
  have. It is the same JSON.
- **A bundle written before `defOrigins` existed** cannot tell a shared
  definition from the schema's own, so its module declares every definition it
  reaches itself, once each, under the name the package exports it by.
  TypeScript is structural, so those and the hoisted ones are interchangeable.
  `trust-tasks update` fetches current bundles.

`scripts/equivalence.mjs` generates every registry version and holds the result to
the prebuilt tree; its header says exactly what it compares.
