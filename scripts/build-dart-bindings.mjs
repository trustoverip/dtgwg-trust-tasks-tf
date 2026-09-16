// Generate Dart bindings for every Trust Tasks payload schema.
//
// Walks specs/<slug>/<version>/payload.schema.json, plus the shared schemas
// under _shared/ and _framework/, and emits one Dart library per JSON Schema
// into trust-tasks-dart/lib/specs/. The output ships in the `trust_tasks`
// package on pub.dev alongside the hand-written §7.2 runtime in
// trust-tasks-dart/lib/src/runtime/.
//
// Run from the repo root:
//   npm run build-dart-bindings
//
// Each generated request library exports, mirroring trust-tasks-rs,
// @openvtc/trust-tasks and trust-tasks-go:
//
//   typeUri                   — https://trusttasks.org/spec/<slug>/<version>
//   responseTypeUri           — the same plus "#response", emitted ONLY when the
//                               schema declares $defs.Response (SPEC §4.4.1: a
//                               specification with no success response is
//                               fire-and-forget, and its consumers MUST NOT emit
//                               a #response-variant document, so a ready-made
//                               constant for one invites a conformance violation)
//   Payload / Response        — the request and response payload classes, under
//                               those uniform names rather than the schema's
//                               title, so code can be written generic over a
//                               Trust Task
//   payloadSchemaJson         — the payload schema as JSON text (SPEC §7.2 item 2)
//   responsePayloadSchemaJson — the same for the response variant
//   spec / responseSpec       — SpecPolicy values carrying the §7.2 items
//                               5b/7/8 flags and the schema above
//
// ── Decisions specific to Dart ─────────────────────────────────────────────
//
// 1. ONE LIBRARY PER SCHEMA, imported with a prefix. Dart's `export` is flat —
//    there is no `export * as Foo`, TypeScript's mechanism for giving 500
//    modules non-colliding names in one barrel. So `lib/trust_tasks.dart`
//    exports the *runtime only*, and a consumer imports the one specification
//    it needs directly and prefixes it:
//
//      import 'package:trust_tasks/trust_tasks.dart';
//      import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
//
//    That is why each generated library may declare its own `Ext` without
//    colliding with the other 500, exactly as the TypeScript modules may.
//
// 2. CLOSED STRING SETS BECOME EXTENSION TYPES, NOT ENUMS. A Dart `enum` gives
//    exhaustive switches, and it throws on a value it does not know — so a peer
//    on a newer MINOR sending a value this version has never heard of would
//    crash the parse, in direct conflict with SPEC §5.2's forward-compatibility
//    requirement. An `extension type const Effect(String value)` is zero-cost,
//    erases to String, keeps `==` and the named constants, and represents an
//    unrecognised value without complaint. This puts Dart alongside Rust's
//    `#[non_exhaustive]` and Go's named string type; TypeScript's closed union
//    is the odd one out and is documented as such in its own codes.ts.
//
// 3. THE REF RESOLVER IS ITS OWN, AGAIN. `inlineCrossFileRefs` below is a
//    fourth independent implementation of the splice that
//    `resolve_cross_file_refs` (Rust), `build-ts-bindings.mjs` and
//    `build-go-bindings.mjs` each do. Deliberate, for the reason
//    `scripts/check-bindings-conformance.mjs` gives at its own copy of the
//    policy derivation: `npm run check-bindings` compares the emitted schema
//    text of every language against the others, and sharing one resolver would
//    reduce that comparison to asserting the resolver agrees with itself. Four
//    resolvers that agree is evidence; one resolver quoted four times is not.
//    Do not DRY this up.
//
// ── What the generated types do and do not enforce ─────────────────────────
//
// Member names, required/optional as non-nullable/nullable, and JSON
// round-tripping. They do NOT carry `oneOf` mutual exclusion, `minLength`,
// `pattern`, `minimum`, or any other constraint — exactly as the TypeScript and
// Go bindings do not. The constraint lives in the schema, and SPEC §7.2 item 2
// is where it is enforced: hand `payloadSchemaJson` to a validator.
//
// Dart does get one thing right that Go needed a workaround for: a nullable
// `List<String>?` distinguishes absent (null) from present-and-empty (`[]`)
// natively, so no pointer-to-slice trick is needed.

import fsSync, { promises as fs } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");
const SPECS_DIR = path.join(REPO_ROOT, "specs");
const PACKAGE_ROOT = path.join(REPO_ROOT, "trust-tasks-dart");
const OUT_DIR = path.join(PACKAGE_ROOT, "lib", "specs");

/** The pub.dev package name, as declared in trust-tasks-dart/pubspec.yaml. */
const PACKAGE_NAME = "trust_tasks";
/** The runtime library the generated code imports for SpecPolicy. */
const RUNTIME_IMPORT = `package:${PACKAGE_NAME}/trust_tasks.dart`;

/**
 * Schemas that describe something other than a payload shape, skipped for the
 * same reasons the TypeScript and Go generators skip them:
 *
 *   spec.meta.schema.json  — front-matter metadata for spec authors.
 *   trust-task.schema.json — the document envelope (SPEC §4.2). Generating from
 *     it would put a second document type in the package, competing with the
 *     hand-written generic `TrustTaskDocument<P>`. The generated one would be
 *     strictly worse — not generic over the payload, so unusable with
 *     consumeInbound — and having both invites picking the wrong one.
 */
const NOT_PAYLOAD_SCHEMAS = new Set(["spec.meta.schema.json", "trust-task.schema.json"]);

/* ── JSON Schema $ref resolution ──────────────────────────────────────────── */

/** Split `<path>#/$defs/<name>` into its two halves, or null if not that shape. */
function splitExternalRef(s) {
  const hash = s.indexOf("#");
  if (hash < 0) return null;
  const filePath = s.slice(0, hash);
  const defName = s.slice(hash + 1).replace(/^\/\$defs\//, "");
  if (!filePath || !defName || defName.includes("/") || defName === s.slice(hash + 1)) return null;
  return { filePath, defName };
}

/** Every `$ref` in `node` matching `predicate`, in document order. */
function collectRefs(node, predicate, out = []) {
  if (Array.isArray(node)) {
    for (const item of node) collectRefs(item, predicate, out);
  } else if (node && typeof node === "object") {
    if (typeof node.$ref === "string" && predicate(node.$ref)) out.push(node.$ref);
    for (const value of Object.values(node)) collectRefs(value, predicate, out);
  }
  return out;
}

/** Rewrite every external `$ref` to its local `#/$defs/<name>` form, in place. */
function localizeRefs(node) {
  if (Array.isArray(node)) {
    node.forEach(localizeRefs);
  } else if (node && typeof node === "object") {
    if (typeof node.$ref === "string" && !node.$ref.startsWith("#")) {
      const split = splitExternalRef(node.$ref);
      if (split) node.$ref = `#/$defs/${split.defName}`;
    }
    for (const value of Object.values(node)) localizeRefs(value);
  }
}

/**
 * Inline every cross-file `$ref` into the schema's own `$defs`, so the emitted
 * schema resolves with no filesystem access.
 *
 * See decision 3 in the file header: this is an independent fourth
 * implementation on purpose, and `npm run check-bindings` compares its output
 * against the Rust, TypeScript and Go ones.
 */
function inlineCrossFileRefs(schema, baseDir) {
  const isExternal = (r) => !r.startsWith("#");
  const frontier = collectRefs(schema, isExternal).map((ref) => ({ ref, ownerDir: baseDir }));
  const seen = new Set();

  while (frontier.length > 0) {
    const { ref, ownerDir } = frontier.pop();
    const split = splitExternalRef(ref);
    if (!split) throw new Error(`external $ref ${JSON.stringify(ref)} is not <path>#/$defs/<name>`);
    const abs = path.resolve(ownerDir, split.filePath);
    const key = `${abs}#/$defs/${split.defName}`;
    if (seen.has(key)) continue;
    seen.add(key);

    const referenced = JSON.parse(fsSync.readFileSync(abs, "utf8"));
    const fragment = referenced?.$defs?.[split.defName];
    if (fragment === undefined) {
      throw new Error(`${abs} has no $defs/${split.defName} (referenced from ${ownerDir})`);
    }

    schema.$defs ??= {};
    const existing = schema.$defs[split.defName];
    if (existing !== undefined) {
      if (JSON.stringify(existing) !== JSON.stringify(fragment)) {
        throw new Error(
          `schema already defines $defs/${split.defName} with a different shape; ` +
            `the cross-file $ref splice would overwrite it`,
        );
      }
    } else {
      schema.$defs[split.defName] = fragment;
    }

    // A spliced fragment can carry refs of its own. External ones resolve
    // against the file it came from, not the original base; internal ones
    // (`#/$defs/X`) point at siblings in that same file, so recast them as
    // external refs against it and let the same path handle both.
    const fragmentDir = path.dirname(abs);
    for (const r of collectRefs(fragment, isExternal)) {
      frontier.push({ ref: r, ownerDir: fragmentDir });
    }
    for (const r of collectRefs(fragment, (x) => x.startsWith("#"))) {
      frontier.push({ ref: `${split.filePath}${r}`, ownerDir });
    }
  }

  localizeRefs(schema);
  return schema;
}

/**
 * The response variant needs a schema of its own: the root describes the
 * *request* payload and `$defs.Response` describes the response. Wrap the latter
 * so `$defs` stays reachable and its internal `$ref`s still resolve.
 *
 * No `$id`: 2020-12 forbids a non-empty fragment in `$id`, so the natural
 * `<base>#response` will not compile under a conforming validator.
 */
function responseSchemaOf(schema) {
  if (schema?.$defs?.Response === undefined) return null;
  const doc = {};
  if (schema.$schema !== undefined) doc.$schema = schema.$schema;
  doc.$ref = "#/$defs/Response";
  doc.$defs = schema.$defs;
  return doc;
}

/* ── SPEC §7.2 policy, from the specification's front matter ─────────────── */

/**
 * Read the SPEC §7.2 policy flags out of a spec's front matter.
 *
 * A fourth independent derivation, for the reason the ref resolver is — see
 * decision 3 in the file header. A Dart consumer must reach the same verdict as
 * a Rust, TypeScript or Go one on the same document.
 *
 * Note the response-variant asymmetry (§7.3 item 5): a response document swaps
 * the parties, so the requirement governing a *response*'s `recipient` member is
 * the one declared for the *issuer* party of the request.
 */
function readSpecPolicy(schemaPath) {
  const specPath = path.join(path.dirname(schemaPath), "spec.md");
  let meta;
  try {
    const src = fsSync.readFileSync(specPath, "utf8");
    if (!src.startsWith("---")) return null;
    const end = src.indexOf("\n---", 3);
    if (end < 0) return null;
    meta = YAML.parse(src.slice(3, end).replace(/^\r?\n/, ""));
  } catch {
    return null;
  }
  if (!meta) return null;

  const partyRequirement = (member) =>
    (meta.parties || []).find((p) => p && p.member === member)?.requirement === "REQUIRED";

  // §7.3 item 8 is either a single `requirement` covering every variant, or a
  // per-variant `request` / `response` pair. Only REQUIRED obliges a consumer to
  // reject a proofless document, so each reduces to a boolean. A per-variant
  // declaration omitting `response` takes the request's value — the conservative
  // reading, and the only one that cannot weaken a variant by omission.
  const pr = meta.proofRequirement || {};
  const proofRequired =
    typeof pr.requirement === "string"
      ? { request: pr.requirement === "REQUIRED", response: pr.requirement === "REQUIRED" }
      : {
          request: pr.request === "REQUIRED",
          response: (pr.response ?? pr.request) === "REQUIRED",
        };

  // §7.3 item 17, declared with the same shape as item 8 and normalised the same
  // way. Only REQUIRED obliges a consumer to reject a document with no
  // `issuedAt`; the framework baseline of §4.2 is already a SHOULD, which
  // RECOMMENDED merely restates.
  const ir = meta.issuedAtRequirement || {};
  const issuedAtRequired =
    typeof ir.requirement === "string"
      ? { request: ir.requirement === "REQUIRED", response: ir.requirement === "REQUIRED" }
      : {
          request: ir.request === "REQUIRED",
          response: (ir.response ?? ir.request) === "REQUIRED",
        };

  return {
    isBearer: meta.bearer === true,
    isProofRequired: proofRequired.request,
    responseIsProofRequired: proofRequired.response,
    isIssuedAtRequired: issuedAtRequired.request,
    responseIsIssuedAtRequired: issuedAtRequired.response,
    // Request: the party tagged `recipient`. Response: the party tagged
    // `issuer`, because the response addresses the original producer.
    isRecipientRequired: partyRequirement("recipient"),
    responseIsRecipientRequired: partyRequirement("issuer"),
  };
}

/* ── Dart identifiers ────────────────────────────────────────────────────── */

/** Split an arbitrary schema name into lowercase word segments. */
function segments(name) {
  return String(name)
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
    .split(/[^A-Za-z0-9]+/)
    .flatMap((part) => part.split(/\s+/))
    .filter(Boolean)
    .map((s) => s.toLowerCase());
}

/** `UpperCamelCase`, for a class or extension type. */
function dartClassName(name) {
  const ident = segments(name)
    .map((s) => s[0].toUpperCase() + s.slice(1))
    .join("");
  if (ident === "") return "X";
  return /^[0-9]/.test(ident) ? `N${ident}` : ident;
}

/** `lowerCamelCase`, for a field, parameter or top-level constant. */
function dartFieldName(name) {
  const parts = segments(name);
  if (parts.length === 0) return "value";
  let ident = parts[0] + parts.slice(1).map((s) => s[0].toUpperCase() + s.slice(1)).join("");
  if (/^[0-9]/.test(ident)) ident = `n${ident}`;
  // A reserved word cannot be an identifier. Suffixing keeps the derivation
  // obvious and the wire name is carried separately anyway.
  return DART_RESERVED.has(ident) ? `${ident}Value` : ident;
}

/** `lower_snake_case`, for a file or directory name (Dart's `file_names` lint). */
function dartFileName(name) {
  return segments(name).join("_") || "x";
}

/**
 * Words that may not be used as identifiers in Dart. The built-in identifiers
 * (`show`, `hide`, `async`, …) are legal as field names and are deliberately
 * absent; only the genuinely reserved words are listed.
 */
const DART_RESERVED = new Set([
  "assert", "break", "case", "catch", "class", "const", "continue", "default",
  "do", "else", "enum", "extends", "false", "final", "finally", "for", "if",
  "in", "is", "new", "null", "rethrow", "return", "super", "switch", "this",
  "throw", "true", "try", "var", "void", "while", "with",
]);

/**
 * Quote `s` as a Dart single-quoted string literal.
 *
 * `$` must be escaped as well as the usual set: Dart interpolates `$name` and
 * `${expr}` inside every ordinary string literal, so an unescaped `$` in a
 * schema description is a compile error at best and a silent substitution at
 * worst. A raw string (`r'…'`) would avoid that but cannot itself contain a
 * quote, and these descriptions contain plenty.
 */
function dartString(s) {
  let out = "'";
  for (const ch of String(s)) {
    const cp = ch.codePointAt(0);
    if (ch === "'") out += "\\'";
    else if (ch === "\\") out += "\\\\";
    else if (ch === "$") out += "\\$";
    else if (ch === "\n") out += "\\n";
    else if (ch === "\r") out += "\\r";
    else if (ch === "\t") out += "\\t";
    else if (cp < 0x20 || cp === 0x7f) out += `\\x${cp.toString(16).padStart(2, "0")}`;
    else out += ch;
  }
  return `${out}'`;
}

/** Render `text` as a Dart doc comment, indented by `indent`. */
function docComment(text, indent = "") {
  if (typeof text !== "string" || text.trim() === "") return [];
  const words = text.replace(/\s+/g, " ").trim().split(" ");
  const lines = [];
  let line = "";
  for (const word of words) {
    if (line.length + word.length + 1 > 84 && line !== "") {
      lines.push(line.trimEnd());
      line = "";
    }
    line += `${word} `;
  }
  if (line.trim() !== "") lines.push(line.trimEnd());
  // `[` starts a doc reference in Dartdoc; a stray one in prose is a warning.
  // `[` starts a doc reference and `<` starts an HTML tag, as far as Dartdoc is
  // concerned. Schema descriptions are prose written for spec readers and
  // contain both.
  return lines.map(
    (l) =>
      `${indent}/// ${l
        .replace(/\[/g, "\\[")
        .replace(/\]/g, "\\]")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")}`,
  );
}

/* ── Schema -> Dart type emission ────────────────────────────────────────── */

/**
 * Collect the `allOf` branches of `node` into the node itself, where each branch
 * is a plain object schema. Only two schemas in the registry use `allOf` and
 * both are simple merges; a branch the merge cannot absorb leaves the node alone
 * and the type falls back to `Object?`.
 */
function mergeAllOf(node) {
  if (!Array.isArray(node.allOf)) return node;
  const merged = { ...node };
  delete merged.allOf;
  for (const branch of node.allOf) {
    if (!branch || typeof branch !== "object" || branch.$ref !== undefined) return node;
    merged.properties = { ...(merged.properties ?? {}), ...(branch.properties ?? {}) };
    merged.required = [...new Set([...(merged.required ?? []), ...(branch.required ?? [])])];
    if (branch.type !== undefined) merged.type ??= branch.type;
    if (branch.description !== undefined) merged.description ??= branch.description;
  }
  return merged;
}

/** The declared type(s) of `node`, with `null` stripped and reported separately. */
function typeOf(node) {
  const raw = node.type;
  if (raw === undefined) return { types: [], nullable: false };
  const list = Array.isArray(raw) ? raw : [raw];
  return { types: list.filter((t) => t !== "null"), nullable: list.includes("null") };
}

function newEmitter(sourceRel) {
  return { sourceRel, decls: [], names: new Map(), needsRuntime: false };
}

/** Reserve `preferred` as a type name, returning the name actually taken. */
function reserve(em, preferred) {
  let name = preferred;
  let n = 2;
  while (em.names.has(name)) name = `${preferred}${n++}`;
  em.names.set(name, true);
  return name;
}

/**
 * A type descriptor: the Dart type, plus how to read it from and write it to a
 * decoded JSON value.
 *
 * `from(expr)` converts a `dynamic` JSON value to the Dart type. `to(expr)`
 * converts a Dart value back to something `jsonEncode` accepts.
 *
 * `cast` names the target of a plain `as` cast where the conversion is exactly
 * that, so the nullable form can be built as `expr as T?`. Anything else — a
 * list, a class, an extension type — gets a null-guarded ternary instead.
 *
 * It is the cast *type* rather than a boolean because appending `?` to an
 * already-rendered expression only works if you know where the type ends:
 * `(x as Map<String, dynamic>)?` is not valid Dart, and that shipped as a
 * syntax error on the generator's first run.
 */
const descriptor = (type, from, to, cast = null) => ({ type, from, to, cast });
const identity = (expr) => expr;

/** The Dart type for `node`, emitting any named declarations it needs into `em`. */
function dartTypeFor(em, node, hint, defNames) {
  if (node === true || node === false || node === undefined) {
    return descriptor("Object?", identity, identity);
  }

  if (typeof node.$ref === "string") {
    const known = defNames.get(node.$ref);
    if (known !== undefined) return known;
    return descriptor("Object?", identity, identity);
  }

  node = mergeAllOf(node);
  const { types } = typeOf(node);

  // A union of JSON types, or a `oneOf` / `anyOf` the type system cannot
  // express. Dart has no closed union, so the honest representation is `Object?`:
  // the member is carried through unchanged and SPEC §7.2 item 2 decides whether
  // it conforms.
  if (types.length > 1 || (types.length === 0 && (node.oneOf || node.anyOf || node.not))) {
    const branches = node.oneOf ?? node.anyOf;
    // The one exception worth making: a union whose branches are all strings —
    // `anyOf: [{enum: [...]}, {pattern: "..."}]`, the SPEC §8.5 extended-code
    // idiom — is a string.
    if (
      types.length === 0 &&
      Array.isArray(branches) &&
      branches.length > 0 &&
      branches.every(
        (b) => b && typeof b === "object" && (b.type === "string" || b.enum || b.pattern),
      )
    ) {
      return descriptor("String", (e) => `${e} as String`, identity, "String");
    }
    return descriptor("Object?", identity, identity);
  }

  const type = types[0];

  if (type === "string") {
    if (
      Array.isArray(node.enum) &&
      node.enum.length > 0 &&
      node.enum.every((v) => typeof v === "string")
    ) {
      return declareExtensionType(em, node, hint);
    }
    return descriptor("String", (e) => `${e} as String`, identity, "String");
  }
  if (type === "integer") return descriptor("int", (e) => `${e} as int`, identity, "int");
  if (type === "number") {
    // `as num` then `.toDouble()`: a JSON `1` decodes to int, and `1 as double`
    // throws. Every other language's bindings accept an integral number here.
    return descriptor("double", (e) => `(${e} as num).toDouble()`, identity);
  }
  if (type === "boolean") return descriptor("bool", (e) => `${e} as bool`, identity, "bool");

  if (type === "array") {
    const item = dartTypeFor(em, node.items ?? true, `${hint}Item`, defNames);
    const itemTo = item.to("e");
    return descriptor(
      `List<${item.type}>`,
      (e) => `(${e} as List<dynamic>).map((e) => ${item.from("e")}).toList()`,
      itemTo === "e" ? identity : (e) => `${e}.map((e) => ${itemTo}).toList()`,
    );
  }

  if (type === "object") {
    if (node.properties === undefined || Object.keys(node.properties).length === 0) {
      if (node.additionalProperties && typeof node.additionalProperties === "object") {
        const value = dartTypeFor(em, node.additionalProperties, `${hint}Value`, defNames);
        const valueTo = value.to("v");
        return descriptor(
          `Map<String, ${value.type}>`,
          (e) =>
            `(${e} as Map<String, dynamic>).map((k, v) => MapEntry(k, ${value.from("v")}))`,
          valueTo === "v" ? identity : (e) => `${e}.map((k, v) => MapEntry(k, ${valueTo}))`,
        );
      }
      // `Map<String, dynamic>`, not `Map<String, Object?>`: the two are mutually
      // assignable, and naming the cast's own type keeps the nullable form a
      // plain `as Map<String, dynamic>?`.
      return descriptor(
        "Map<String, dynamic>",
        (e) => `${e} as Map<String, dynamic>`,
        identity,
        "Map<String, dynamic>",
      );
    }
    return declareClass(em, node, hint, defNames);
  }

  return descriptor("Object?", identity, identity);
}

/**
 * Declare an extension type over String for a closed set of values.
 *
 * See decision 2 in the file header for why this is not a Dart `enum`.
 */
function declareExtensionType(em, node, hint) {
  const name = reserve(em, hint);
  const lines = [];
  lines.push(
    ...docComment(
      node.description ??
        `${name} is a closed set of string values defined by this specification's schema.`,
    ),
  );
  lines.push("///");
  lines.push(
    ...docComment(
      "An extension type rather than an enum: a value from a newer MINOR of this " +
        "specification must not crash the parse (SPEC §5.2), and an enum would throw on one. " +
        "Compare against the constants below, and treat anything else as unrecognised.",
    ),
  );
  lines.push(`extension type const ${name}(String value) {`);
  const seen = new Set();
  const constNames = [];
  for (const value of node.enum) {
    let constName = dartFieldName(value);
    while (seen.has(constName)) constName += "_";
    seen.add(constName);
    constNames.push(constName);
    lines.push(`  static const ${name} ${constName} = ${name}(${dartString(value)});`);
  }
  lines.push("");
  lines.push(`  /// Every value this specification's schema permits.`);
  lines.push(`  static const List<${name}> values = <${name}>[${constNames.join(", ")}];`);
  lines.push("}");
  em.decls.push(lines.join("\n"));

  return descriptor(name, (e) => `${name}(${e} as String)`, (e) => `${e}.value`);
}

/**
 * The nullable form of `type`, unless it is already nullable.
 *
 * `Object?` is the fallback for every union and untyped member, and appending a
 * second `?` to it is a syntax error — which is exactly what the first run
 * produced (`final Object?? newValue;`).
 */
function nullable(type, isRequired) {
  if (isRequired) return type;
  return type.endsWith("?") ? type : `${type}?`;
}

/** Declare a class for an object schema, recursing into its members. */
function declareClass(em, node, hint, defNames) {
  const name = reserve(em, hint);
  const required = new Set(Array.isArray(node.required) ? node.required : []);
  const fields = [];
  const fieldNames = new Set();

  for (const [wireName, member] of Object.entries(node.properties)) {
    let fieldName = dartFieldName(wireName);
    while (fieldNames.has(fieldName)) fieldName += "_";
    fieldNames.add(fieldName);

    const memberNode = member === true || member === undefined ? {} : member;
    const { nullable } = typeOf(memberNode);
    const desc = dartTypeFor(em, member, `${name}${dartClassName(wireName)}`, defNames);
    const isRequired = required.has(wireName) && !nullable;

    fields.push({ wireName, fieldName, desc, isRequired, doc: memberNode.description });
  }

  const lines = [];
  lines.push(...docComment(node.description ?? node.title ?? `${name}, generated from its schema.`));
  lines.push(`class ${name} {`);

  // Constructor.
  lines.push(`  const ${name}({`);
  for (const f of fields) {
    lines.push(`    ${f.isRequired ? "required " : ""}this.${f.fieldName},`);
  }
  lines.push(`  });`);
  lines.push("");

  // fromJson.
  lines.push(`  /// Read this payload from a decoded JSON object.`);
  lines.push(`  factory ${name}.fromJson(Map<String, dynamic> json) => ${name}(`);
  for (const f of fields) {
    const access = `json[${dartString(f.wireName)}]`;
    let expr;
    if (f.isRequired) {
      expr = f.desc.from(access);
    } else if (f.desc.cast !== null) {
      // A plain cast widens to its nullable form directly, which also accepts an
      // explicit JSON null.
      expr = `${access} as ${f.desc.cast}?`;
    } else if (f.desc.from(access) === access) {
      // Already untyped; nothing to cast or guard.
      expr = access;
    } else {
      expr = `${access} == null ? null : ${f.desc.from(access)}`;
    }
    lines.push(`        ${f.fieldName}: ${expr},`);
  }
  lines.push(`      );`);
  lines.push("");

  // Fields.
  let first = true;
  for (const f of fields) {
    const doc = docComment(f.doc, "  ");
    if (!first && doc.length > 0) lines.push("");
    first = false;
    lines.push(...doc);
    lines.push(`  final ${nullable(f.desc.type, f.isRequired)} ${f.fieldName};`);
  }
  lines.push("");

  // toJson. A null member is omitted rather than written as JSON null, matching
  // `skip_serializing_if` in Rust and `omitempty` in Go.
  lines.push(`  /// Serialize to a JSON-encodable map, omitting absent members.`);
  lines.push(`  Map<String, dynamic> toJson() => <String, dynamic>{`);
  for (const f of fields) {
    if (f.isRequired) {
      lines.push(`        ${dartString(f.wireName)}: ${f.desc.to(f.fieldName)},`);
    } else {
      // `!` is required: a public final field is not promoted by a null check.
      const value = f.desc.to(`${f.fieldName}!`);
      lines.push(
        `        if (${f.fieldName} != null) ${dartString(f.wireName)}: ${value},`,
      );
    }
  }
  lines.push(`      };`);
  lines.push("}");

  em.decls.push(lines.join("\n"));
  return descriptor(
    name,
    (e) => `${name}.fromJson(${e} as Map<String, dynamic>)`,
    (e) => `${e}.toJson()`,
  );
}

/**
 * Declare a Dart type for every `$defs` entry, then for the root.
 *
 * `$defs` are declared first, and in two passes: the names are reserved before
 * any body is emitted, so a `$def` that refers to a sibling resolves to the
 * sibling's final name whichever order they appear in.
 */
/**
 * Every `#/$defs/<name>` that `node` reaches, directly or through its subtrees.
 */
function internalRefsOf(node, out = new Set()) {
  if (Array.isArray(node)) {
    for (const item of node) internalRefsOf(item, out);
  } else if (node && typeof node === "object") {
    if (typeof node.$ref === "string" && node.$ref.startsWith("#/$defs/")) {
      out.add(node.$ref.slice("#/$defs/".length));
    }
    for (const value of Object.values(node)) internalRefsOf(value, out);
  }
  return out;
}

/**
 * Order the `$defs` so that a definition is emitted after everything it
 * references.
 *
 * This matters because a `$def`'s *descriptor* — how to read it from and write
 * it to JSON — is only known once it has been emitted, and the answer differs by
 * kind: a class round-trips through `X.fromJson` / `x.toJson()`, an extension
 * type through `X(v as String)` / `x.value`, and an alias to `String` or
 * `Map<String, dynamic>` through neither. Emitting in declaration order meant a
 * definition that referenced a later sibling baked in the wrong one, which
 * `dart analyze` caught as "The method 'toJson' isn't defined for the type
 * 'String'" across 20-odd libraries.
 *
 * A cycle (a recursive definition) is emitted in whatever order it is reached.
 * That is safe: a cyclic definition is necessarily an object, and the class
 * shape is what the placeholder assumes anyway.
 */
function topologicalDefOrder(defs) {
  const names = Object.keys(defs);
  const known = new Set(names);
  const deps = new Map(
    names.map((name) => [
      name,
      [...internalRefsOf(defs[name])].filter((r) => known.has(r) && r !== name),
    ]),
  );

  const order = [];
  const state = new Map(names.map((n) => [n, "new"]));
  const visit = (name) => {
    const mark = state.get(name);
    if (mark === "done" || mark === "active") return; // `active` means a cycle
    state.set(name, "active");
    for (const dep of deps.get(name)) visit(dep);
    state.set(name, "done");
    order.push(name);
  };
  for (const name of names) visit(name);
  return order;
}

/**
 * Declare a Dart type for every `$defs` entry, then for the root.
 *
 * Names are reserved for every definition before any body is emitted, so a
 * definition that refers to a sibling resolves to the sibling's final name. The
 * bodies are then emitted in dependency order — see [topologicalDefOrder] for
 * why that is not merely tidiness.
 */
function emitDeclarations(em, schema, rootHint) {
  const defs = schema.$defs ?? {};
  const defNames = new Map();
  const pending = new Map();

  for (const defName of Object.keys(defs)) {
    const target = reserve(em, dartClassName(defName));
    pending.set(defName, target);
    // A class-shaped placeholder, used only for a reference inside a reference
    // cycle — every acyclic reference resolves to the real descriptor below.
    defNames.set(
      `#/$defs/${defName}`,
      descriptor(
        target,
        (e) => `${target}.fromJson(${e} as Map<String, dynamic>)`,
        (e) => `${e}.toJson()`,
      ),
    );
  }
  // The root type name is reserved last so a `$defs.Payload` keeps the bare name
  // and the root takes the suffixed one, rather than the other way round.
  const rootName = rootHint === null ? null : reserve(em, rootHint);

  for (const defName of topologicalDefOrder(defs)) {
    const defNode = defs[defName];
    const target = pending.get(defName);
    const produced = emitNamed(em, defNode, target, defNames);
    if (produced.type !== target) {
      // The node was not a class or an extension type (a bare string, a list, a
      // union), so the name needs a typedef to exist at all. The descriptor
      // recorded below is the *produced* one, so a referrer round-trips it the
      // way its underlying type requires rather than as a class.
      em.decls.push(
        [
          ...docComment(
            defNode?.description ?? `${target}, defined by this specification's schema.`,
          ),
          `typedef ${target} = ${produced.type};`,
        ].join("\n"),
      );
      defNames.set(`#/$defs/${defName}`, descriptor(target, produced.from, produced.to, produced.cast));
    } else {
      defNames.set(`#/$defs/${defName}`, produced);
    }
  }

  if (rootName !== null) {
    const produced = emitNamed(em, schema, rootName, defNames);
    if (produced.type !== rootName) {
      em.decls.push(
        [
          ...docComment(schema.description ?? `${rootName}, this specification's payload.`),
          `typedef ${rootName} = ${produced.type};`,
        ].join("\n"),
      );
    }
  }
  return { defNames, rootName };
}

/** Emit `node` under the exact name `target`, which has already been reserved. */
function emitNamed(em, node, target, defNames) {
  em.names.delete(target);
  const produced = dartTypeFor(em, node, target, defNames);
  em.names.set(target, true);
  return produced;
}

/* ── File assembly ───────────────────────────────────────────────────────── */

/** The tail every generated request library carries: URIs, schemas, policy. */
function emitTail(slugInfo, policy, schemas, hasResponse) {
  if (!slugInfo) return [];
  const { slug, version } = slugInfo;
  const uri = `https://trusttasks.org/spec/${slug}/${version}`;
  const out = [];

  out.push(
    [
      "/// The Trust Task type URI this library's [Payload] is carried under.",
      `const String typeUri = ${dartString(uri)};`,
    ].join("\n"),
  );

  if (hasResponse) {
    out.push(
      [
        "/// The success-response form of [typeUri] (SPEC §4.4.1).",
        `const String responseTypeUri = ${dartString(`${uri}#response`)};`,
      ].join("\n"),
    );
  }

  if (schemas.request) {
    out.push(
      [
        "/// This specification's payload schema, as JSON text.",
        "///",
        "/// SPEC §7.2 item 2 is performed against this. It ships with the library rather",
        "/// than only as a file under specs/ because a Dart class carries none of the",
        "/// schema's constraints — not minLength, not pattern, not the oneOf mutual",
        "/// exclusion — so without it every such rule is unenforced. Cross-file \\$refs are",
        "/// already inlined, so it needs no resolver.",
        `const String payloadSchemaJson = ${dartString(JSON.stringify(schemas.request))};`,
      ].join("\n"),
    );
    if (schemas.response) {
      out.push(
        [
          "/// As [payloadSchemaJson], for the success-response variant.",
          `const String responsePayloadSchemaJson = ${dartString(JSON.stringify(schemas.response))};`,
        ].join("\n"),
      );
    }
  }

  if (policy) {
    const block = (uriConst, proofRequired, recipientRequired, issuedAtRequired, schemaConst) =>
      [
        `  typeUri: ${uriConst},`,
        `  isBearer: ${policy.isBearer},`,
        `  isProofRequired: ${proofRequired},`,
        `  isRecipientRequired: ${recipientRequired},`,
        `  isIssuedAtRequired: ${issuedAtRequired},`,
        `  payloadSchema: ${schemaConst},`,
      ].join("\n");

    out.push(
      [
        "/// The SPEC §7.2 policy for the request variant, taken from this",
        "/// specification's front matter.",
        "///",
        "/// Pass it to [consumeInbound] — items 5b, 7 and 8 are per-specification and",
        "/// cannot be derived from the document alone, and item 2 needs the schema it",
        "/// carries.",
        "const SpecPolicy spec = SpecPolicy(",
        block(
          "typeUri",
          policy.isProofRequired,
          policy.isRecipientRequired,
          policy.isIssuedAtRequired,
          schemas.request ? "payloadSchemaJson" : "null",
        ),
        ");",
      ].join("\n"),
    );

    if (hasResponse) {
      out.push(
        [
          "/// The SPEC §7.2 policy for the success-response variant.",
          "///",
          "/// `isRecipientRequired` tracks the *issuer* party's requirement, because a",
          "/// response swaps the parties (§7.3 item 5).",
          "const SpecPolicy responseSpec = SpecPolicy(",
          block(
            "responseTypeUri",
            policy.responseIsProofRequired,
            policy.responseIsRecipientRequired,
            policy.responseIsIssuedAtRequired,
            schemas.response ? "responsePayloadSchemaJson" : "null",
          ),
          ");",
        ].join("\n"),
      );
    }
  }

  return out;
}

/** `<repo>/specs/<x>/<y>/<file>.schema.json` -> `{ slug, version }`, or null. */
function slugFromSchemaPath(schemaPath) {
  const rel = path.relative(SPECS_DIR, schemaPath);
  const parts = rel.split(path.sep);
  if (parts.some((s) => s.startsWith("_"))) return null;
  if (path.basename(schemaPath) !== "payload.schema.json") return null;
  return { slug: parts.slice(0, -2).join("/"), version: parts[parts.length - 2] };
}

/** The `MAJOR.MINOR` segment of a path under specs/, and where it sits. */
function versionSegment(parts) {
  const index = parts.findIndex((p) => /^\d+\.\d+$/.test(p));
  if (index < 0) throw new Error(`no MAJOR.MINOR version segment in ${parts.join("/")}`);
  return { index, version: parts[index] };
}

/**
 * Where a schema's library lands.
 *
 * The tree under `specs/` is mirrored, with every segment snake_cased for Dart's
 * `file_names` lint (`change-role` becomes `change_role`), `_shared` and
 * `_framework` de-underscored, and the version rendered `v0_1`. The result is
 * under `lib/` rather than `lib/src/` because these libraries are public API: a
 * consumer imports one directly and prefixes it, since Dart has no namespaced
 * re-export to hide them behind (see decision 1 in the file header).
 */
function outputFor(schemaPath) {
  const parts = path.relative(SPECS_DIR, schemaPath).split(path.sep);
  const file = parts.pop().replace(/\.schema\.json$/, "");
  const { index, version } = versionSegment(parts);
  const slugInfo = slugFromSchemaPath(schemaPath);

  const trail = parts.slice(index + 1);
  const dirs = [...parts.slice(0, index), ...trail].map((p) =>
    dartFileName(p.startsWith("_") ? p.slice(1) : p),
  );

  const dir = path.join(OUT_DIR, ...dirs, `v${version.replace(/\./g, "_")}`);
  return {
    dir,
    file: path.join(dir, `${dartFileName(file)}.dart`),
    slugInfo,
  };
}

/** Generate one Dart library from one schema. */
function generateOne(schemaPath) {
  const raw = JSON.parse(fsSync.readFileSync(schemaPath, "utf8"));
  const baseDir = path.dirname(schemaPath);
  const out = outputFor(schemaPath);

  const requestSchema = inlineCrossFileRefs(JSON.parse(JSON.stringify(raw)), baseDir);
  const responseSchema = responseSchemaOf(requestSchema);
  const hasResponse = responseSchema !== null;

  const em = newEmitter(path.relative(REPO_ROOT, schemaPath).split(path.sep).join("/"));

  // A shared schema has no payload root — it is a bag of `$defs` and nothing
  // else. Declaring a `Payload` for it would invent a type the spec does not
  // define.
  const rootHint = out.slugInfo ? "Payload" : null;
  emitDeclarations(em, requestSchema, rootHint);

  const tail = emitTail(
    out.slugInfo,
    out.slugInfo ? readSpecPolicy(schemaPath) : null,
    { request: out.slugInfo ? requestSchema : null, response: hasResponse ? responseSchema : null },
    hasResponse,
  );

  const header = [
    "// Code generated by scripts/build-dart-bindings.mjs. DO NOT EDIT.",
    `// Source: ${em.sourceRel}`,
  ];
  if (tail.length > 0) header.push("", `import '${RUNTIME_IMPORT}';`);

  const body = [...em.decls, ...tail];
  return { ...out, source: `${[...header, "", ...body.join("\n\n").split("\n")].join("\n")}\n` };
}

/* ── Driver ──────────────────────────────────────────────────────────────── */

/**
 * The `MAJOR.MINOR` SDK floor declared in the package's pubspec.
 *
 * Used as the language version the generated code is formatted at — see the
 * note at the `dart format` call.
 */
function sdkFloor() {
  const pubspec = fsSync.readFileSync(path.join(PACKAGE_ROOT, "pubspec.yaml"), "utf8");
  const m = /^\s*sdk:\s*\^?(\d+)\.(\d+)/m.exec(pubspec);
  if (!m) throw new Error("could not read the `sdk:` constraint from pubspec.yaml");
  return `${m[1]}.${m[2]}`;
}

async function walk(dir, pattern) {
  const out = [];
  for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await walk(full, pattern)));
    else if (pattern.test(entry.name)) out.push(full);
  }
  return out;
}

/**
 * Wipe the generated tree so a removed spec does not linger.
 *
 * Only `lib/specs/` is touched: `lib/src/runtime/` and `lib/trust_tasks.dart`
 * are hand-written, and the pubspec and READMEs are source.
 */
async function clean() {
  await fs.rm(OUT_DIR, { recursive: true, force: true });
  await fs.mkdir(OUT_DIR, { recursive: true });
}

async function main() {
  console.log("Trust Tasks Dart bindings generator");
  await clean();

  const schemas = (await walk(SPECS_DIR, /\.schema\.json$/)).sort();
  const filtered = schemas.filter((p) => !NOT_PAYLOAD_SCHEMAS.has(path.basename(p)));
  console.log(`Generating bindings for ${filtered.length} schemas...`);

  let ok = 0;
  const seen = new Map();
  for (const schemaPath of filtered) {
    try {
      const result = generateOne(schemaPath);
      // Snake-casing can in principle collapse two distinct on-disk names onto
      // one Dart file. Nothing in the registry does today, and a silent
      // overwrite would be very hard to spot, so check rather than trust.
      const previous = seen.get(result.file);
      if (previous !== undefined) {
        throw new Error(
          `would overwrite the library generated from ${previous} — two schema paths ` +
            `snake_case to the same Dart file name`,
        );
      }
      seen.set(result.file, path.relative(REPO_ROOT, schemaPath));

      await fs.mkdir(result.dir, { recursive: true });
      await fs.writeFile(result.file, result.source, "utf8");
      ok++;
    } catch (err) {
      console.error("  ✗ " + path.relative(REPO_ROOT, schemaPath));
      console.error("    " + err.message);
      process.exitCode = 1;
    }
  }

  // Format in-process, for the reason build-go-bindings.mjs gives at the same
  // point: a contributor without the Dart SDK would otherwise commit
  // unformatted output and discover it in CI. `dart format` exits 0 when it
  // rewrites files, so only a genuine failure is reported.
  // `--language-version` is not optional here, tempting as it looks.
  //
  // `dart format` picks its style from the code's language version, and infers
  // that from the enclosing package's *resolved* config — `.dart_tool/`, written
  // by `dart pub get`. With no resolution it falls back to the newest version it
  // knows, and since Dart 3.13 that means the new "tall" style rather than the
  // old one. So the same generator produced differently-formatted output
  // depending on whether anyone had run `dart pub get`, which made the drift
  // check fail in CI (no `.dart_tool`) while passing locally (one present).
  //
  // Reading the floor from the pubspec and passing it explicitly removes the
  // ambient dependency entirely, and keeps one source of truth for the value.
  const languageVersion = sdkFloor();
  const formatted = spawnSync(
    "dart",
    ["format", `--language-version=${languageVersion}`, OUT_DIR],
    { encoding: "utf8" },
  );
  if (formatted.error?.code === "ENOENT") {
    throw new Error(
      "dart is not on PATH. The generated tree must be committed `dart format`-clean or " +
        "the Dart drift check will fail on every PR. Install the Dart SDK " +
        "(https://dart.dev/get-dart) and re-run.",
    );
  }
  if (formatted.status !== 0) {
    throw new Error(`dart format failed:\n${formatted.stderr || formatted.stdout}`);
  }

  console.log(
    `\nGenerated ${ok} formatted Dart libraries into ${path.relative(REPO_ROOT, OUT_DIR)}`,
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
