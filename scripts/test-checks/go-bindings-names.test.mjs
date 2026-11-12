// build-go-bindings.mjs: a schema-derived name never redeclares an identifier
// the generated file declares itself — the tail's constants and policies, or
// an enum constant. trust-tasks-go/cmd/trust-tasks-gen's TestReservedNames
// holds the consumer-side generator to the same names.

import { test } from "node:test";
import assert from "node:assert/strict";

import { newEmitter, reserveTailIdentifiers, emitDeclarations, TAIL_IDENTIFIERS } from "../build-go-bindings.mjs";

const object = { type: "object", properties: { x: { type: "string" } } };

/** Every top-level identifier the declarations introduce, in order. */
function declared(em) {
  const names = [];
  for (const decl of em.decls) {
    for (const line of decl.split("\n")) {
      const type = /^type (\w+) /.exec(line);
      if (type) names.push(type[1]);
      const constant = /^\t(\w+) \w+ = /.exec(line);
      if (constant) names.push(constant[1]);
    }
  }
  return names;
}

function generate(schema) {
  const em = newEmitter("probe");
  reserveTailIdentifiers(em);
  const result = emitDeclarations(em, schema, "Payload");
  return { em, ...result };
}

test("a $defs name that the tail declares is suffixed", () => {
  const defs = Object.fromEntries(TAIL_IDENTIFIERS.map((n) => [n, { type: "string" }]));
  defs.Spec = object;
  defs.Extends = { type: "string", enum: ["a"] };
  const { em, defNames } = generate({ type: "object", properties: {}, $defs: defs });
  for (const name of TAIL_IDENTIFIERS) assert.equal(defNames.get(`#/$defs/${name}`), `${name}2`);
  assert.ok(declared(em).includes("Extends2A"));
});

test("an inline type whose path-derived name is the tail's is suffixed", () => {
  const { em } = generate({
    type: "object",
    properties: { schemaJson: object },
    $defs: { Response: { type: "object", properties: { spec: object } } },
  });
  const names = declared(em);
  assert.ok(names.includes("PayloadSchemaJSON2"), names.join(", "));
  assert.ok(names.includes("ResponseSpec2"), names.join(", "));
  assert.ok(names.includes("Response") && names.includes("Payload"));
});

test("an enum constant never redeclares a type or the tail", () => {
  const { em } = generate({
    type: "object",
    properties: {},
    $defs: {
      Type: { type: "string", enum: ["URI"] },
      Kind: { type: "string", enum: ["spec"] },
      KindSpec: object,
    },
  });
  const names = declared(em);
  assert.ok(names.includes("TypeURI_"), names.join(", "));
  assert.ok(names.includes("KindSpec_") && names.includes("KindSpec"), names.join(", "));
  const all = [...names, ...TAIL_IDENTIFIERS];
  assert.equal(new Set(all).size, all.length, `duplicate identifier in ${all.join(", ")}`);
});

test("names that collide with nothing are unchanged", () => {
  const { em, rootName, defNames } = generate({
    type: "object",
    properties: { kind: { type: "string", enum: ["a"] } },
    $defs: { Response: object, Thing: object },
  });
  assert.equal(rootName, "Payload");
  assert.equal(defNames.get("#/$defs/Response"), "Response");
  assert.equal(defNames.get("#/$defs/Thing"), "Thing");
  assert.ok(declared(em).includes("PayloadKindA"));
});
