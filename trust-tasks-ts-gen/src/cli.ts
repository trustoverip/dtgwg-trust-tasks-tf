#!/usr/bin/env node
/**
 * trust-tasks-gen-ts — generate TypeScript modules for the Trust Tasks an
 * application pins.
 *
 *   npx trust-tasks-gen-ts --lock trust-tasks.lock --out src/trust-tasks
 *
 * `trust-tasks generate --lang ts` runs this.
 */

import { generate } from "./generate.js";

const USAGE = `usage: trust-tasks-gen-ts [--lock <trust-tasks.lock>] --out <dir>

Reads trust-tasks.lock and the bundles vendored beside it (trust-tasks/<slug>/<M.N>.bundle.json),
verifies each against its locked digest, and writes one module per pinned version to <dir>,
with an index.ts. <dir> is owned by the generator: it is rewritten whole, and a non-empty
directory it did not write is refused.

  --lock <path>   the lockfile (default: trust-tasks.lock)
  --out <dir>     the output directory (required)
  --quiet         print nothing on success
  -h, --help      this text
`;

async function main(argv: string[]): Promise<number> {
  let lock = "trust-tasks.lock";
  let out: string | undefined;
  let quiet = false;
  let includeFramework = false;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]!;
    const value = (): string => {
      const v = argv[++i];
      if (v === undefined || v.startsWith("--")) throw new UsageError(`${arg} needs a value`);
      return v;
    };
    if (arg === "--lock") lock = value();
    else if (arg.startsWith("--lock=")) lock = arg.slice("--lock=".length);
    else if (arg === "--out") out = value();
    else if (arg.startsWith("--out=")) out = arg.slice("--out=".length);
    else if (arg === "--quiet") quiet = true;
    // Undocumented: the repository's equivalence test generates the framework
    // specifications too, to compare them with the prebuilt modules.
    else if (arg === "--include-framework") includeFramework = true;
    else if (arg === "-h" || arg === "--help") {
      process.stdout.write(USAGE);
      return 0;
    } else throw new UsageError(`unknown argument ${arg}`);
  }
  if (!out) throw new UsageError("--out is required");
  await generate({
    lock,
    out,
    includeFramework,
    log: quiet ? () => {} : (line) => process.stderr.write(`${line}\n`),
  });
  return 0;
}

class UsageError extends Error {}

main(process.argv.slice(2)).then(
  (code) => process.exit(code),
  (err: unknown) => {
    process.stderr.write(`trust-tasks-gen-ts: ${(err as Error).message}\n`);
    if (err instanceof UsageError) process.stderr.write(`\n${USAGE}`);
    process.exit(err instanceof UsageError ? 2 : 1);
  },
);
