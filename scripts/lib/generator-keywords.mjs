// Keywords that belong to a code generator, not to JSON Schema.
//
// json-schema-to-typescript reads `tsType` and `tsEnumNames` as TypeScript
// source and writes them into the module it emits. The TypeScript generators
// (scripts/build-ts-bindings.mjs and trust-tasks-ts-gen) add `tsType`
// themselves, after reading a schema, to name a type declared elsewhere; a
// schema in specs/ never carries either, so what is generated stays a function
// of the schema's data. trust-tasks-ts-gen refuses a registry bundle that
// carries one, and this keeps the registry from publishing one.

export const GENERATOR_KEYWORDS = Object.freeze(["tsType", "tsEnumNames"]);

// Keywords whose value maps names to subschemas.
const SCHEMA_MAPS = new Set(["properties", "patternProperties", "$defs", "definitions", "dependentSchemas"]);
// Keywords whose value is a subschema or an array of subschemas.
const SCHEMA_VALUES = new Set([
  "items",
  "prefixItems",
  "additionalItems",
  "additionalProperties",
  "unevaluatedItems",
  "unevaluatedProperties",
  "contains",
  "propertyNames",
  "not",
  "if",
  "then",
  "else",
  "allOf",
  "anyOf",
  "oneOf",
]);

const isObject = (v) => typeof v === "object" && v !== null && !Array.isArray(v);
const token = (t) => String(t).replace(/~/g, "~0").replace(/\//g, "~1");

/**
 * The JSON Pointer of every subschema of `schema` carrying a generator keyword,
 * with the keyword. A payload member that happens to be *named* `tsType` is a
 * name under `properties`, not a keyword, and is not reported.
 *
 * @returns {{ pointer: string, keyword: string }[]}
 */
export function findGeneratorKeywords(schema, pointer = "") {
  const found = [];
  const visit = (node, at) => {
    if (Array.isArray(node)) {
      node.forEach((n, i) => visit(n, `${at}/${i}`));
      return;
    }
    if (!isObject(node)) return;
    for (const keyword of GENERATOR_KEYWORDS) {
      if (Object.prototype.hasOwnProperty.call(node, keyword)) found.push({ pointer: at || "/", keyword });
    }
    for (const [key, value] of Object.entries(node)) {
      const here = `${at}/${token(key)}`;
      if (SCHEMA_MAPS.has(key) && isObject(value)) {
        for (const [name, sub] of Object.entries(value)) visit(sub, `${here}/${token(name)}`);
      } else if (SCHEMA_VALUES.has(key)) {
        visit(value, here);
      }
    }
  };
  visit(schema, pointer);
  return found;
}
