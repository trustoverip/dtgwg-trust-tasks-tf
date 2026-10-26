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

## How a module differs from the prebuilt one

The constants (`TYPE_URI`, `EXTENDS`, `DEPRECATION`, `SPEC`, `RESPONSE_SPEC`, …)
and the payload/response declarations are what `@openvtc/trust-tasks` ships for
the same version. Two things cannot be, from a bundle:

- **Shared definitions are declared in each module**, not imported from one
  registry-wide `_shared/components.ts`. A bundle's schema is self-contained and
  does not say which `$defs` entry was spliced in from which shared file, so the
  cross-module hoisting the in-repo generator does (and its
  `component-names.lock.json`) cannot be reproduced. Each definition is still
  declared once per module, under the name the prebuilt module exports it by;
  TypeScript is structural, so the same definition from two modules is
  interchangeable.
- **`PAYLOAD_SCHEMA` has the bundle's member order** (sorted, RFC 8785) except
  inside `properties`, which keeps the authored order. It is the same JSON.

`scripts/equivalence.mjs` generates every registry version and holds the result to
the prebuilt tree; its header says exactly what it compares.
