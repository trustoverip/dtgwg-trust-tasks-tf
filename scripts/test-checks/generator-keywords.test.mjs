// scripts/lib/generator-keywords.mjs: a schema in specs/ carries no keyword a
// code generator reads as source.

import { test } from "node:test";
import assert from "node:assert/strict";

import { findGeneratorKeywords } from "../lib/generator-keywords.mjs";

test("finds tsType and tsEnumNames at any subschema", () => {
  const schema = {
    type: "object",
    properties: { a: { type: "string", tsType: "X" } },
    $defs: { D: { oneOf: [{ enum: ["x"], tsEnumNames: ["X"] }] } },
    items: { tsType: "Y" },
  };
  assert.deepEqual(findGeneratorKeywords(schema), [
    { pointer: "/properties/a", keyword: "tsType" },
    { pointer: "/$defs/D/oneOf/0", keyword: "tsEnumNames" },
    { pointer: "/items", keyword: "tsType" },
  ]);
});

test("a member named tsType, or the word in data, is not a keyword", () => {
  const schema = {
    type: "object",
    properties: { tsType: { type: "string", enum: ["tsType"], examples: [{ tsType: "x" }] } },
    const: { tsType: "x" },
  };
  assert.deepEqual(findGeneratorKeywords(schema), []);
});
