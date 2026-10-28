// Policy tests for scripts/api-compat.mjs — the verdict logic, without the
// minute-long api-extractor run. `node --test scripts/api-compat.test.mjs`.
import assert from "node:assert/strict";
import { test } from "node:test";
import { compareText, declaresBreak, diff, maskSchemas } from "./api-compat.mjs";

test("a string-literal union may gain members, in any order, but not lose or rename one", () => {
  assert.equal(compareText('d?: "a" | "b";', 'd?: "a" | "b" | "c";'), "widened");
  assert.equal(compareText('d?: "a" | "b";', 'd?: "b" | "a";'), "same");
  assert.equal(compareText('d: "a" | null;', 'd: "a" | "b" | null;'), "widened");
  assert.equal(compareText('d?: "a" | "b";', 'd?: "a";'), "changed");
  assert.equal(compareText('d?: "a" | "b";', 'd?: "a" | "x";'), "changed");
  assert.equal(compareText('d: "a" | null;', 'd: "a" | "b";'), "changed");
  assert.equal(compareText('d: "a|b";', 'd: "a" | "b";'), "changed");
});

test("a string-literal tuple (STANDARD_CODES) may be appended to, not reordered or shortened", () => {
  assert.equal(compareText('readonly ["a", "b"]', 'readonly ["a", "b", "c"]'), "widened");
  assert.equal(compareText('readonly ["a", "b"]', 'readonly ["c", "a", "b"]'), "changed");
  assert.equal(compareText('readonly ["a", "b"]', 'readonly ["a"]'), "changed");
});

test("any other signature change is a change", () => {
  assert.equal(compareText("x: string;", "x: number;"), "changed");
  assert.equal(compareText("x?: string;", "x: string;"), "changed");
});

test("embedded JSON Schema literals are masked; the flags beside them are not", () => {
  const spec = (desc, bearer) =>
    `{ readonly isBearer: ${bearer}; readonly payloadSchema: { readonly $schema: "s"; readonly d: "${desc}"; readonly n: { a: "}" }; }; }`;
  assert.equal(compareText(maskSchemas(spec("one", false)), maskSchemas(spec("two", false))), "same");
  assert.equal(compareText(maskSchemas(spec("one", false)), maskSchemas(spec("one", true))), "changed");
});

test("diff: removals and required additions break; optional additions and new modules do not", () => {
  const item = (kind, text, parent, optional = false) => ({ kind, text, parent, optional });
  const old = {
    "p!A:namespace": item("Namespace", "", null),
    "p!A.I:interface": item("Interface", "export interface I", "p!A:namespace"),
    "p!A.I#x:member": item("PropertySignature", "x: string;", "p!A.I:interface"),
    "p!B:namespace": item("Namespace", "", null),
    "p!B.T:type": item("TypeAlias", "export type T = string;", "p!B:namespace"),
  };
  const ok = diff(old, {
    ...old,
    "p!A.I#y:member": item("PropertySignature", "y?: string;", "p!A.I:interface", true),
    "p!C:namespace": item("Namespace", "", null),
    "p!C.J:interface": item("Interface", "export interface J", "p!C:namespace"),
    "p!C.J#z:member": item("PropertySignature", "z: string;", "p!C.J:interface"),
  });
  assert.deepEqual(ok.breaks, []);
  assert.equal(ok.added, 4);

  const rest = { ...old };
  delete rest["p!B:namespace"];
  delete rest["p!B.T:type"];
  const bad = diff(old, { ...rest, "p!A.I#w:member": item("PropertySignature", "w: string;", "p!A.I:interface") });
  assert.equal(bad.breaks.length, 2); // B removed (once, not per member) + required w
  assert.match(bad.breaks[0], /^removed: p!B:namespace/);
  assert.match(bad.breaks[1], /^required member added/);
});

test("only a `!` whose scope list names ts declares a break", () => {
  assert.equal(declaresBreak("feat(ts)!: x", "ts"), true);
  assert.equal(declaresBreak("feat(rs, ts, go, dart)!: x", "ts"), true);
  assert.equal(declaresBreak("feat(rs)!: x", "ts"), false);
  assert.equal(declaresBreak("feat(ts-proof)!: x", "ts"), false);
  assert.equal(declaresBreak("feat(ts): x", "ts"), false);
  assert.equal(declaresBreak("feat!: x", "ts"), false);
  assert.equal(declaresBreak(undefined, "ts"), false);
});
