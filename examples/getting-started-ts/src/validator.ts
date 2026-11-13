// A stand-in for a JSON Schema engine, so this example runs with no dependency
// beyond the Trust Tasks packages.
//
// It understands the subset of JSON Schema 2020-12 the generated payload
// schemas use for their structure — local `$ref`s into `$defs`, `type`,
// `required`, `properties`, `additionalProperties: false`, `const`, `enum`,
// `oneOf`, `items`, `minItems`, `minLength` and `maxLength` — and ignores the
// rest (`pattern`, `format`, `uniqueItems`, …). That is enough to refuse a
// payload with a missing, misspelt or mistyped member, which is the failure an
// unvalidated TypeScript consumer otherwise never sees.
//
// A real application passes a full Draft 2020-12 engine instead — ajv
// (`new Ajv2020().compile(schema)`) is the usual choice. GETTING-STARTED.md
// §1, "What you bring yourself", says why the runtime bundles none.

import type { PayloadValidator } from "@openvtc/trust-tasks/runtime";

type Schema = { readonly [key: string]: unknown };

const isObject = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

function typeMatches(type: unknown, value: unknown): boolean {
  switch (type) {
    case "object":
      return isObject(value);
    case "array":
      return Array.isArray(value);
    case "string":
      return typeof value === "string";
    case "boolean":
      return typeof value === "boolean";
    case "integer":
      return Number.isInteger(value);
    case "number":
      return typeof value === "number";
    default:
      return true;
  }
}

function check(schema: Schema, value: unknown, at: string, root: Schema, errors: string[]): void {
  const ref = schema.$ref;
  if (typeof ref === "string") {
    const name = ref.startsWith("#/$defs/") ? ref.slice("#/$defs/".length) : undefined;
    const target = name === undefined ? undefined : (root.$defs as Record<string, Schema> | undefined)?.[name];
    if (target === undefined) {
      errors.push(`${at}: unsupported $ref ${ref}`);
      return;
    }
    check(target, value, at, root, errors);
  }
  if (schema.type !== undefined && !typeMatches(schema.type, value)) {
    errors.push(`${at}: expected ${String(schema.type)}`);
    return;
  }
  if ("const" in schema && JSON.stringify(schema.const) !== JSON.stringify(value)) {
    errors.push(`${at}: expected ${JSON.stringify(schema.const)}`);
  }
  if (Array.isArray(schema.enum) && !schema.enum.includes(value)) {
    errors.push(`${at}: not one of ${JSON.stringify(schema.enum)}`);
  }
  if (Array.isArray(schema.oneOf)) {
    const matching = (schema.oneOf as Schema[]).filter((branch) => {
      const branchErrors: string[] = [];
      check(branch, value, at, root, branchErrors);
      return branchErrors.length === 0;
    }).length;
    if (matching !== 1) errors.push(`${at}: matches ${matching} of the oneOf branches, not exactly one`);
  }
  if (typeof value === "string") {
    const length = [...value].length; // code points, as JSON Schema counts them
    if (typeof schema.minLength === "number" && length < schema.minLength) errors.push(`${at}: too short`);
    if (typeof schema.maxLength === "number" && length > schema.maxLength) errors.push(`${at}: too long`);
  }
  if (Array.isArray(value)) {
    if (typeof schema.minItems === "number" && value.length < schema.minItems) errors.push(`${at}: too few items`);
    if (isObject(schema.items)) value.forEach((item, i) => check(schema.items as Schema, item, `${at}[${i}]`, root, errors));
  }
  if (isObject(value)) {
    const properties = (schema.properties ?? {}) as Record<string, Schema>;
    for (const member of (schema.required ?? []) as string[]) {
      if (!(member in value)) errors.push(`${at}: missing required member ${member}`);
    }
    for (const [member, memberValue] of Object.entries(value)) {
      if (member in properties) check(properties[member]!, memberValue, `${at}.${member}`, root, errors);
      else if (schema.additionalProperties === false) errors.push(`${at}: unknown member ${member}`);
    }
  }
}

/** Checks a payload against the generated `PAYLOAD_SCHEMA` (the subset above). */
export const structuralValidator: PayloadValidator = {
  validate(schema, payload) {
    const errors: string[] = [];
    check(schema as Schema, payload, "payload", schema as Schema, errors);
    return { ok: errors.length === 0, errors };
  },
};
