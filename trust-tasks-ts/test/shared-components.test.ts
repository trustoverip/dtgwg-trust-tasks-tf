/**
 * The generated tree declares each shared definition once.
 *
 * This asserts a property of the *generated output*, which nothing else here
 * does: `tsc --noEmit` is happy with 481 structurally identical `Ext`s, the
 * drift check compares the generator to itself, and `check-bindings` reads
 * only the policy constants and the response half. The duplication was
 * therefore invisible to every check in the repo, which is how it reached 341
 * modules.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, it } from "node:test";

const SRC = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "src");

function generatedModules(): string[] {
  const out: string[] = [];
  (function walk(dir: string) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (entry.name !== "_runtime") walk(full);
      } else if (entry.name.endsWith(".ts")) {
        out.push(full);
      }
    }
  })(SRC);
  return out;
}

/** Names declared at the top level of a module, in source order. */
function declarations(src: string): string[] {
  return [...src.matchAll(/^export (?:interface|type) ([A-Za-z0-9_$]+)/gm)].map((m) => m[1]);
}

/**
 * `Name` and `Name1` both declared in one module: json-schema-to-typescript's
 * signal that it emitted the same definition twice and had to number the copies.
 */
function counterSuffixed(names: string[]): string[] {
  return names.filter((n) => /\d$/.test(n) && names.includes(n.replace(/\d+$/, "")));
}

describe("generated shared definitions", () => {
  it("declares no counter-suffixed duplicate outside a spec's own $defs", () => {
    // `provision/integration` refs its *own* `$defs.DidTemplateRef` from three
    // places with a different `description` at each, and the compiler emits a
    // copy per description. That duplication is inside one schema, not across
    // files, so hoisting cross-file components does not reach it. Listed rather
    // than filtered out by pattern so that a new one has to be looked at.
    const expected = new Set([
      "provision/integration/0.1/payload.ts",
      "provision/integration/0.2/payload.ts",
      "provision/integration/0.3/payload.ts",
    ]);

    const offenders = new Map<string, string[]>();
    for (const file of generatedModules()) {
      const dupes = counterSuffixed(declarations(fs.readFileSync(file, "utf8")));
      if (dupes.length) offenders.set(path.relative(SRC, file).replace(/\\/g, "/"), dupes);
    }
    assert.deepEqual([...offenders.keys()].sort(), [...expected].sort());
  });

  it("declares Ext exactly once, in the components module", () => {
    const declaring = generatedModules().filter((file) =>
      /^export (?:interface|type) Ext\b/m.test(fs.readFileSync(file, "utf8")),
    );
    assert.deepEqual(
      declaring.map((f) => path.relative(SRC, f).replace(/\\/g, "/")),
      ["_shared/components.ts"],
    );
  });

  it("has every module import the shared definitions it re-exports", () => {
    // A module that re-exports `Ext` without importing it would be declaring a
    // copy again, which is the state this replaced.
    for (const file of generatedModules()) {
      const src = fs.readFileSync(file, "utf8");
      const reexport = /^export type \{([^}]*)\};$/m.exec(src);
      if (!reexport) continue;
      const imported = /^import type \{([^}]*)\} from "([^"]*)";$/m.exec(src);
      assert.ok(imported, `${path.relative(SRC, file)} re-exports types it does not import`);
      assert.match(imported[2], /_shared\/components\.js$/);
      const names = (clause: string) =>
        clause
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean)
          .map((s) => (s.includes(" as ") ? s.split(" as ")[1].trim() : s));
      for (const name of names(reexport[1])) {
        assert.ok(
          names(imported[1]).includes(name),
          `${path.relative(SRC, file)} re-exports ${name} without importing it`,
        );
      }
    }
  });
});
