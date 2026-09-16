// Generate Go bindings for every Trust Tasks payload schema.
//
// Walks specs/<slug>/<version>/payload.schema.json, plus the shared schemas
// under _shared/ and _framework/, and emits one Go package per JSON Schema
// into trust-tasks-go/specs/. The output ships in the
// github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go module alongside
// the hand-written §7.2 runtime in trust-tasks-go/trusttasks.
//
// Run from the repo root:
//   npm run build-go-bindings
//
// Each generated request package exports, mirroring trust-tasks-rs and
// @openvtc/trust-tasks:
//
//   TypeURI                   — https://trusttasks.org/spec/<slug>/<version>
//   ResponseTypeURI           — the same plus "#response", emitted ONLY when the
//                               schema declares $defs.Response. SPEC §4.4.1 says
//                               a specification with no success response is
//                               fire-and-forget and its consumers MUST NOT emit
//                               a #response-variant document, so handing
//                               implementers a ready-made constant for one
//                               invites a conformance violation.
//   Payload / Response        — the request and response payload structs, under
//                               those uniform names rather than the schema's
//                               title, so code can be written generic over a
//                               Trust Task. Mirrors the Rust bindings.
//   PayloadSchemaJSON         — the payload schema as JSON text (SPEC §7.2 item 2)
//   ResponsePayloadSchemaJSON — the same for the response variant
//   Spec / ResponseSpec       — trusttasks.SpecPolicy values carrying the §7.2
//                               items 5b/7/8 flags and the schema above
//
// ── Three decisions that differ from the TypeScript generator ───────────────
//
// 1. NO COMPONENT HOISTING. build-ts-bindings.mjs hoists every cross-file
//    `$ref` into one `_shared/components.ts`, and its header explains why that
//    is safe: TypeScript is structurally typed, so two identical `interface Ext`
//    declarations are already mutually assignable and hoisting changes names,
//    not assignability. Go is *nominally* typed, like Rust — `upsert.VaultEntry`
//    and `delete.VaultEntry` would become one type, and any consumer holding a
//    method set or an interface implementation on each would be silently
//    merged. That is the same objection that declined the hoist for Rust in
//    #283, and it carries over. So each generated package inlines what it
//    reaches, exactly as trust-tasks-codegen does, and every package is
//    self-contained: its only imports are `encoding/json` (when it needs
//    json.RawMessage) and the runtime package (for SpecPolicy).
//
// 2. THE REF RESOLVER IS ITS OWN. `inlineCrossFileRefs` below duplicates what
//    the TypeScript generator and `resolve_cross_file_refs` in
//    trust-tasks-codegen already do. That duplication is deliberate, for the
//    reason `scripts/check-bindings-conformance.mjs` gives at its own copy of
//    the policy derivation: `npm run check-bindings` compares the *emitted
//    schema text* of each language against the others, and sharing one resolver
//    would reduce that comparison to asserting the resolver agrees with itself.
//    Three independent resolvers that agree is evidence; one resolver quoted
//    three times is not. Do not DRY this up.
//
// 3. ONE PACKAGE PER SPECIFICATION VERSION, named for its leaf and version —
//    `specs/acl/grant/v0_1` declares `package aclgrantv0_1`. The package name
//    deliberately does not match the directory (which Go permits) so that an
//    import needs no alias and no two generated packages collide in one file:
//    `aclgrantv0_1.Spec` reads at the call site the way `AclGrant_v0_1.SPEC`
//    does in TypeScript. `_shared` and `_framework` become `shared` and
//    `framework` on disk because the go command ignores any directory whose
//    name begins with `_`.
//
// ── What the generated types do and do not enforce ─────────────────────────
//
// The structs carry member names, JSON tags, and required/optional as
// value/pointer. They do NOT carry `oneOf` mutual exclusion, `minLength`,
// `pattern`, `minimum`, or any other constraint — exactly as the TypeScript
// bindings do not: json-schema-to-typescript flattens `auth/revoke-session`'s
// "sessionId xor all" into a struct with both members optional, and so does
// this. The constraint lives in the schema, and SPEC §7.2 item 2 is where it is
// enforced: hand `PayloadSchemaJSON` to a validator. That is why the schema
// travels with the package rather than only sitting in `specs/`.

import fsSync, { promises as fs } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");
const SPECS_DIR = path.join(REPO_ROOT, "specs");
const MODULE_ROOT = path.join(REPO_ROOT, "trust-tasks-go");
const OUT_DIR = path.join(MODULE_ROOT, "specs");

/** Import path of the module, as declared in trust-tasks-go/go.mod. */
const MODULE_PATH = "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go";
/** Import path of the hand-written runtime the generated packages depend on. */
const RUNTIME_IMPORT = `${MODULE_PATH}/trusttasks`;

/**
 * Schemas that describe something other than a payload shape, skipped for the
 * same reasons build-ts-bindings.mjs skips them:
 *
 *   spec.meta.schema.json  — front-matter metadata for spec authors.
 *   trust-task.schema.json — the document envelope (SPEC §4.2). Generating from
 *     it would put a second document type in the module, competing with the
 *     hand-written generic `trusttasks.Document[P]`. The generated one would be
 *     strictly worse — not generic over the payload, so unusable with
 *     ConsumeInbound — and having both invites picking the wrong one.
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
 * See decision 2 in the file header: this is an independent third
 * implementation on purpose, and `npm run check-bindings` compares its output
 * against the Rust and TypeScript ones.
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
 * *request* payload and `$defs.Response` describes the response. Wrap the
 * latter so `$defs` stays reachable and its internal `$ref`s still resolve.
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
 * Mirrors `readSpecPolicy` in build-ts-bindings.mjs and
 * `read_proof_required_flag` and friends in trust-tasks-codegen, and is a third
 * independent derivation for the same reason the ref resolver is — see decision
 * 2 in the file header. A Go consumer must reach the same verdict as a Rust or
 * TypeScript one on the same document.
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

/* ── Go identifiers ──────────────────────────────────────────────────────── */

/**
 * Segments conventionally written all-caps in Go (golint's initialism list,
 * trimmed to what this registry actually uses, plus the framework's own `VID`
 * and `DID`).
 *
 * Applied to whole segments only, so `sessionId` becomes `SessionID` and
 * `identifier` is left alone. The wire name always travels in the struct tag,
 * so nothing about the mapping is load-bearing for serialization.
 */
const INITIALISMS = new Set([
  "acl", "api", "ascii", "cpu", "css", "did", "dns", "eof", "guid", "html",
  "http", "https", "id", "ip", "json", "jwe", "jwk", "jws", "jwt", "lhs",
  "mac", "ok", "otp", "qps", "ram", "rhs", "rpc", "sdk", "sha", "sla", "smtp",
  "sql", "ssh", "tcp", "tls", "ttl", "udp", "uid", "uri", "url", "usb", "utc",
  "uuid", "vc", "vid", "vm", "vp", "xml", "xsrf", "xss",
]);

/** Split an arbitrary schema name into lowercase word segments. */
function segments(name) {
  return String(name)
    // Break camelCase and PascalCase, including acronym runs (`DIDDoc` -> `DID`, `Doc`).
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
    .split(/[^A-Za-z0-9]+/)
    .flatMap((part) => part.split(/\s+/))
    .filter(Boolean)
    .map((s) => s.toLowerCase());
}

/**
 * An exported Go identifier for `name`.
 *
 * A name that starts with a digit gets an `N` prefix — Go identifiers may not,
 * and silently dropping the digit would collide `1of2` with `of2`.
 */
function goIdent(name) {
  const ident = segments(name)
    .map((s) => (INITIALISMS.has(s) ? s.toUpperCase() : s[0].toUpperCase() + s.slice(1)))
    .join("");
  if (ident === "") return "X";
  return /^[0-9]/.test(ident) ? `N${ident}` : ident;
}

/** A lowercase, underscore-free Go package name for a slug plus version. */
function goPackageName(slug, version) {
  const base = slug.split("/").flatMap(segments).join("");
  return `${base}v${version.replace(/\./g, "_")}`;
}

/** `0.1` -> `v0_1`, the on-disk directory name for a spec version. */
function versionDir(version) {
  return `v${version.replace(/\./g, "_")}`;
}

/**
 * Quote `s` as a Go interpreted string literal.
 *
 * Not `JSON.stringify`: the two escape sets overlap but are not the same. JSON
 * permits `\/` and emits `\uXXXX` surrogate pairs for astral characters, which
 * Go reads as two unpaired surrogates rather than one rune. Emitting the raw
 * UTF-8 for everything printable and escaping only what Go requires avoids the
 * question. A backtick-quoted raw literal is not an option — schema descriptions
 * are full of them (`` `effect` is `allow` ``).
 */
function goString(s) {
  let out = '"';
  for (const ch of String(s)) {
    const cp = ch.codePointAt(0);
    if (ch === '"') out += '\\"';
    else if (ch === "\\") out += "\\\\";
    else if (ch === "\n") out += "\\n";
    else if (ch === "\r") out += "\\r";
    else if (ch === "\t") out += "\\t";
    else if (cp < 0x20 || cp === 0x7f) out += `\\x${cp.toString(16).padStart(2, "0")}`;
    else out += ch;
  }
  return `${out}"`;
}

/** Render `text` as a Go doc comment on `name`, indented by `indent`. */
function docComment(name, text, indent = "") {
  if (typeof text !== "string" || text.trim() === "") return [];
  // Collapse to single lines and wrap at a comfortable width. Schema
  // descriptions are prose, frequently long, and occasionally contain newlines.
  const words = text.replace(/\s+/g, " ").trim().split(" ");
  const lines = [];
  let line = name ? `${name} ` : "";
  for (const word of words) {
    if (line.length + word.length + 1 > 88 && line.trim() !== (name ?? "")) {
      lines.push(line.trimEnd());
      line = "";
    }
    line += `${word} `;
  }
  if (line.trim() !== "") lines.push(line.trimEnd());
  return lines.map((l) => `${indent}// ${l}`);
}

/* ── Schema -> Go type emission ──────────────────────────────────────────── */

/**
 * Collect the `allOf` branches of `node` into the node itself, where each
 * branch is a plain object schema.
 *
 * Only two schemas in the registry use `allOf` and both are simple merges. A
 * branch the merge cannot absorb leaves the node alone, and the type falls back
 * to `json.RawMessage` below — the schema still carries the constraint.
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

/**
 * The declared type(s) of `node`, with `null` stripped and reported separately.
 *
 * `["string", "null"]` is a nullable string, which in Go is a `*string` — the
 * same representation an *absent* member gets, which is why nullability needs no
 * type of its own here.
 */
function typeOf(node) {
  const raw = node.type;
  if (raw === undefined) return { types: [], nullable: false };
  const list = Array.isArray(raw) ? raw : [raw];
  return { types: list.filter((t) => t !== "null"), nullable: list.includes("null") };
}

/**
 * A package under construction: the declarations emitted so far, the names
 * already taken, and whether `encoding/json` has been reached for.
 */
function newEmitter(sourceRel) {
  return { sourceRel, decls: [], names: new Map(), needsJSON: false };
}

/**
 * Reserve `preferred` as a type name, returning the name actually taken.
 *
 * Names are derived from the JSON pointer path, so they are unique by
 * construction *except* where a `$defs` entry happens to match a path-derived
 * one (a `$defs.PayloadFoo` alongside an inline object at `Payload.foo`). Those
 * get a numeric suffix rather than silently overwriting one another.
 */
function reserve(em, preferred) {
  let name = preferred;
  let n = 2;
  while (em.names.has(name)) name = `${preferred}${n++}`;
  em.names.set(name, true);
  return name;
}

/**
 * The Go type for `node`, emitting any named declarations it needs into `em`.
 *
 * `hint` is the name a newly-declared type should take — derived from the
 * position in the document, so `Payload.entries[]` yields `PayloadEntriesItem`.
 * `defNames` maps `#/$defs/<name>` to the Go type already declared for it.
 */
function goTypeFor(em, node, hint, defNames) {
  if (node === true || node === undefined) {
    em.needsJSON = true;
    return "json.RawMessage";
  }
  if (node === false) {
    em.needsJSON = true;
    return "json.RawMessage";
  }

  // A `$ref` resolves to the declaration made for that `$def`. Every ref is
  // local by the time we get here — inlineCrossFileRefs saw to that.
  if (typeof node.$ref === "string") {
    const name = defNames.get(node.$ref);
    if (name !== undefined) return name;
    em.needsJSON = true;
    return "json.RawMessage";
  }

  node = mergeAllOf(node);
  const { types } = typeOf(node);

  // A union of JSON types, or a `oneOf` / `anyOf` the type system cannot
  // express. Go has no sum type, so the honest representation is the raw
  // message: the member is carried through unchanged and SPEC §7.2 item 2
  // decides whether it conforms. TypeScript emits a union here; Go cannot, and
  // pretending otherwise would mean picking one branch and dropping the rest.
  if (types.length > 1 || (types.length === 0 && (node.oneOf || node.anyOf || node.not))) {
    // The one exception worth making: a union whose branches are all strings —
    // `anyOf: [{enum: [...]}, {pattern: "..."}]`, the SPEC §8.5 extended-code
    // idiom — is a string.
    const branches = node.oneOf ?? node.anyOf;
    if (
      types.length === 0 &&
      Array.isArray(branches) &&
      branches.length > 0 &&
      branches.every((b) => b && typeof b === "object" && (b.type === "string" || b.enum || b.pattern))
    ) {
      return "string";
    }
    em.needsJSON = true;
    return "json.RawMessage";
  }

  const type = types[0];

  if (type === "string") {
    // A closed set of string values becomes a named type with one constant per
    // value, which is what makes an invalid value a compile error at a call site
    // rather than a runtime surprise. `const` (a single permitted value) stays a
    // plain string: a one-member type buys nothing.
    if (Array.isArray(node.enum) && node.enum.length > 0 && node.enum.every((v) => typeof v === "string")) {
      return declareEnum(em, node, hint);
    }
    return "string";
  }
  if (type === "integer") return "int64";
  if (type === "number") return "float64";
  if (type === "boolean") return "bool";
  if (type === "array") {
    const item = goTypeFor(em, node.items ?? true, `${hint}Item`, defNames);
    return `[]${item}`;
  }
  if (type === "object") {
    // An object with no declared members is a free-form map. `properties` is
    // what makes a struct worth declaring.
    if (node.properties === undefined || Object.keys(node.properties).length === 0) {
      if (node.additionalProperties && typeof node.additionalProperties === "object") {
        const value = goTypeFor(em, node.additionalProperties, `${hint}Value`, defNames);
        return `map[string]${value}`;
      }
      em.needsJSON = true;
      return "map[string]json.RawMessage";
    }
    return declareStruct(em, node, hint, defNames);
  }

  // No `type` and nothing else to go on.
  em.needsJSON = true;
  return "json.RawMessage";
}

/** Declare a named string type with one constant per permitted value. */
function declareEnum(em, node, hint) {
  const name = reserve(em, hint);
  const lines = [];
  lines.push(...docComment(name, node.description ?? `${name} is a closed set of string values.`));
  lines.push(`type ${name} string`);
  lines.push("");
  lines.push(`// Values ${name} may take, per this specification's schema.`);
  lines.push("const (");
  const seen = new Set();
  for (const value of node.enum) {
    let constName = `${name}${goIdent(value)}`;
    while (seen.has(constName)) constName += "_";
    seen.add(constName);
    lines.push(`\t${constName} ${name} = ${goString(value)}`);
  }
  lines.push(")");
  em.decls.push(lines.join("\n"));
  return name;
}

/** Declare a named struct for an object schema, recursing into its members. */
function declareStruct(em, node, hint, defNames) {
  const name = reserve(em, hint);
  const required = new Set(Array.isArray(node.required) ? node.required : []);
  const fields = [];
  const fieldNames = new Set();

  for (const [wireName, member] of Object.entries(node.properties)) {
    let fieldName = goIdent(wireName);
    // A schema with both `id` and `ID`, or `did` and `DID`, would otherwise
    // produce two fields of the same name. Disambiguate rather than fail: the
    // JSON tag keeps the wire form exact either way.
    while (fieldNames.has(fieldName)) fieldName += "_";
    fieldNames.add(fieldName);

    const isRequired = required.has(wireName);
    const memberNode = member === true || member === undefined ? {} : member;
    const { nullable } = typeOf(memberNode === true ? {} : memberNode);
    let goType = goTypeFor(em, member, `${name}${fieldName}`, defNames);

    // Every optional member becomes a pointer, so that "absent" and "the zero
    // value" stay distinguishable. SPEC §7.2 turns on whether a member is
    // *present*, and the distinction is load-bearing well beyond the framework
    // checks: `acl`'s `allowedKeys` documents at length that "PRESENT-BUT-EMPTY
    // means authorized on NO keys — the opposite of absent, and deliberately
    // so", and that collapsing the two "re-creates the empty-means-unrestricted
    // class of privilege-escalation defect this family's conventions exist to
    // prevent".
    //
    // That is why optional *slices and maps* are pointers too, awkward as
    // `*[]string` is to hold. `omitempty` on a plain slice drops an empty one
    // from the wire, which is exactly the collapse that text forbids. Rust
    // spells this `Option<Vec<T>>` and TypeScript `scopes?: string[]`; a bare
    // Go slice was the only one of the three that could not say "present and
    // empty".
    //
    // json.RawMessage is the exception: it is a []byte holding the member's own
    // bytes, so an explicit `null` survives as the four bytes "null" and only a
    // genuinely absent member is zero-length.
    const nilable = goType === "json.RawMessage";
    const pointer = (!isRequired || nullable) && !nilable;
    if (pointer) goType = `*${goType}`;

    // `omitempty` exactly where the member is optional. On a required member it
    // would drop a legitimate zero value from the wire.
    const tag = isRequired && !nullable ? wireName : `${wireName},omitempty`;

    fields.push({
      name: fieldName,
      type: goType,
      tag: `\`json:${goString(tag)}\``,
      doc: memberNode.description,
      required: isRequired,
    });
  }

  const lines = [];
  lines.push(...docComment(name, node.description ?? node.title ?? `${name} is a generated payload type.`));
  lines.push(`type ${name} struct {`);
  let first = true;
  for (const field of fields) {
    const doc = docComment("", field.doc, "\t");
    if (!first && doc.length > 0) lines.push("");
    first = false;
    lines.push(...doc);
    lines.push(`\t${field.name} ${field.type} ${field.tag}`);
  }
  lines.push("}");
  em.decls.push(lines.join("\n"));
  return name;
}

/**
 * Declare a Go type for every `$defs` entry, then for the root.
 *
 * `$defs` are declared first, and in two passes: the names are reserved before
 * any body is emitted, so a `$def` that refers to a sibling resolves to the
 * sibling's final name whichever order they appear in.
 */
function emitDeclarations(em, schema, rootHint) {
  const defs = schema.$defs ?? {};
  const defNames = new Map();

  // Pass 1 — reserve. `Response` keeps its name (the uniform alias the runtime
  // and the conformance check both expect); everything else takes its `$defs`
  // key.
  for (const defName of Object.keys(defs)) {
    defNames.set(`#/$defs/${defName}`, reserve(em, goIdent(defName)));
  }
  // The root type name is reserved last so a `$defs.Payload` keeps the bare
  // name and the root takes the suffixed one, rather than the other way round.
  const rootName = rootHint === null ? null : reserve(em, rootHint);

  // Pass 2 — emit. Each `$def` body is emitted under the name reserved for it.
  for (const [defName, defNode] of Object.entries(defs)) {
    const target = defNames.get(`#/$defs/${defName}`);
    const produced = emitNamed(em, defNode, target, defNames);
    // `goTypeFor` declares the struct or enum under `target` itself when the
    // node is one. Anything else (a bare string, an array, a union) needs an
    // alias so the name exists.
    if (produced !== target) {
      em.decls.push(
        [
          ...docComment(target, defNode?.description ?? `${target} is defined by this specification's schema.`),
          `type ${target} = ${produced}`,
        ].join("\n"),
      );
    }
  }

  if (rootName !== null) {
    const produced = emitNamed(em, schema, rootName, defNames);
    if (produced !== rootName) {
      em.decls.push(
        [
          ...docComment(rootName, schema.description ?? `${rootName} is this specification's payload.`),
          `type ${rootName} = ${produced}`,
        ].join("\n"),
      );
    }
  }
  return { defNames, rootName };
}

/**
 * Emit `node` under the exact name `target`, which has already been reserved.
 *
 * `goTypeFor` reserves a fresh name for anything it declares, so the reservation
 * is temporarily released for the call and restored afterwards — that is what
 * makes a `$def` land under its own key rather than under a suffixed variant of
 * it.
 */
function emitNamed(em, node, target, defNames) {
  em.names.delete(target);
  const produced = goTypeFor(em, node, target, defNames);
  em.names.set(target, true);
  return produced;
}

/* ── File assembly ───────────────────────────────────────────────────────── */

/** The tail every generated request package carries: URIs, schemas, policy. */
function emitTail(slugInfo, policy, schemas, hasResponse) {
  if (!slugInfo) return [];
  const { slug, version } = slugInfo;
  const typeUri = `https://trusttasks.org/spec/${slug}/${version}`;
  const out = [];

  out.push(
    [
      "// TypeURI is the Trust Task type URI this package's Payload is carried under.",
      `const TypeURI = ${goString(typeUri)}`,
    ].join("\n"),
  );

  if (hasResponse) {
    out.push(
      [
        `// ResponseTypeURI is the success-response form of TypeURI (SPEC §4.4.1).`,
        `const ResponseTypeURI = ${goString(`${typeUri}#response`)}`,
      ].join("\n"),
    );
  }

  if (schemas.request) {
    out.push(
      [
        "// PayloadSchemaJSON is this specification's payload schema, as JSON text.",
        "//",
        "// SPEC §7.2 item 2 is performed against this. It ships with the package rather",
        "// than only as a file under specs/ because a Go struct carries none of the",
        "// schema's constraints — not minLength, not pattern, not the oneOf mutual",
        "// exclusion — so without it every such rule is unenforced. Cross-file $refs are",
        "// already inlined, so it needs no resolver.",
        `const PayloadSchemaJSON = ${goString(JSON.stringify(schemas.request))}`,
      ].join("\n"),
    );
    if (schemas.response) {
      out.push(
        [
          "// ResponsePayloadSchemaJSON is PayloadSchemaJSON for the success-response variant.",
          `const ResponsePayloadSchemaJSON = ${goString(JSON.stringify(schemas.response))}`,
        ].join("\n"),
      );
    }
  }

  if (policy) {
    const block = (uriConst, proofRequired, recipientRequired, issuedAtRequired, schemaConst) =>
      [
        "\tTypeURI:             " + uriConst + ",",
        "\tIsBearer:            " + policy.isBearer + ",",
        "\tIsProofRequired:     " + proofRequired + ",",
        "\tIsRecipientRequired: " + recipientRequired + ",",
        "\tIsIssuedAtRequired:  " + issuedAtRequired + ",",
        "\tPayloadSchema:       " + schemaConst + ",",
      ].join("\n");

    out.push(
      [
        "// Spec is the SPEC §7.2 policy for the request variant, taken from this",
        "// specification's front matter. Pass it to trusttasks.ConsumeInbound — items 5b,",
        "// 7 and 8 are per-specification and cannot be derived from the document alone,",
        "// and item 2 needs the schema it carries.",
        "var Spec = trusttasks.SpecPolicy{",
        block(
          "TypeURI",
          policy.isProofRequired,
          policy.isRecipientRequired,
          policy.isIssuedAtRequired,
          schemas.request ? "PayloadSchemaJSON" : '""',
        ),
        "}",
      ].join("\n"),
    );

    if (hasResponse) {
      out.push(
        [
          "// ResponseSpec is the SPEC §7.2 policy for the success-response variant.",
          "// IsRecipientRequired tracks the *issuer* party's requirement, because a response",
          "// swaps the parties (§7.3 item 5).",
          "var ResponseSpec = trusttasks.SpecPolicy{",
          block(
            "ResponseTypeURI",
            policy.responseIsProofRequired,
            policy.responseIsRecipientRequired,
            policy.responseIsIssuedAtRequired,
            schemas.response ? "ResponsePayloadSchemaJSON" : '""',
          ),
          "}",
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
 * Where a schema's package lands, and what that package is called.
 *
 * Two rules that the TypeScript generator does not need:
 *
 * `_shared` and `_framework` are renamed on the way out. The go command ignores
 * every directory whose name begins with `_`, so a package under one would be
 * invisible to `go build ./...` — generated, committed, and unreachable.
 *
 * Every schema gets a directory of its own, keyed on its file name. In
 * TypeScript one file is one module, so `auth/_shared/0.1/session.ts` and
 * `tokens.ts` are separate namespaces and may both declare `Ext`. In Go one
 * *directory* is one package, so putting them side by side is a redeclaration
 * error — and the shared directories are full of definitions that repeat
 * (`Ext`, `ClaimType`, `VettingMethod`). Giving each schema its own package
 * restores the one-schema-one-namespace property the rest of this generator
 * assumes.
 *
 * The version always lands last, so every generated package sits at
 * `<path>/vMAJOR_MINOR` whether it came from a payload schema or a shared one.
 * A shared schema nested below its version directory
 * (`did-management/_shared/0.1/vdid-method-extensions/webvh.schema.json`) keeps
 * the nesting, moved above the version.
 */
function outputFor(schemaPath) {
  const parts = path.relative(SPECS_DIR, schemaPath).split(path.sep);
  const file = parts.pop().replace(/\.schema\.json$/, "");
  const { index, version } = versionSegment(parts);

  const slugInfo = slugFromSchemaPath(schemaPath);
  // A payload schema is named for its slug alone — `payload` carries no meaning
  // worth putting in an import path. Everything else keeps its file name, and
  // anything nested below the version segment keeps that nesting too.
  const trail = slugInfo ? [] : [...parts.slice(index + 1), file];
  const dirs = [...parts.slice(0, index), ...trail].map((p) =>
    p.startsWith("_") ? p.slice(1) : p,
  );

  const dir = path.join(OUT_DIR, ...dirs, versionDir(version));
  return {
    dir,
    file: path.join(dir, `${slugInfo ? "payload" : file}.go`),
    pkg: goPackageName(dirs.join("/"), version),
    slugInfo,
  };
}

/** Generate one Go file from one schema. */
function generateOne(schemaPath) {
  const raw = JSON.parse(fsSync.readFileSync(schemaPath, "utf8"));
  const baseDir = path.dirname(schemaPath);
  const out = outputFor(schemaPath);

  // The schema shipped in the package, and the schema the types are built from,
  // are the same document: refs inlined, nothing left to resolve at runtime.
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

  const imports = [];
  if (em.needsJSON) imports.push('"encoding/json"');
  if (tail.length > 0) imports.push(goString(RUNTIME_IMPORT));

  const header = [
    "// Code generated by scripts/build-go-bindings.mjs. DO NOT EDIT.",
    `// Source: ${em.sourceRel}`,
    "",
    `package ${out.pkg}`,
  ];
  if (imports.length === 1) header.push("", `import ${imports[0]}`);
  else if (imports.length > 1) header.push("", "import (", ...imports.map((i) => `\t${i}`), ")");

  const body = [...em.decls, ...tail];
  return { ...out, source: `${[...header, "", ...body.join("\n\n").split("\n")].join("\n")}\n` };
}

/* ── Driver ──────────────────────────────────────────────────────────────── */

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
 * Only `specs/` is touched: `trusttasks/` next to it is hand-written, and
 * go.mod and the READMEs are source.
 */
async function clean() {
  await fs.rm(OUT_DIR, { recursive: true, force: true });
  await fs.mkdir(OUT_DIR, { recursive: true });
}

async function main() {
  console.log("Trust Tasks Go bindings generator");
  await clean();

  const schemas = (await walk(SPECS_DIR, /\.schema\.json$/)).sort();
  const filtered = schemas.filter((p) => !NOT_PAYLOAD_SCHEMAS.has(path.basename(p)));
  console.log(`Generating bindings for ${filtered.length} schemas...`);

  let ok = 0;
  const packages = new Set();
  for (const schemaPath of filtered) {
    try {
      const result = generateOne(schemaPath);
      await fs.mkdir(result.dir, { recursive: true });
      await fs.writeFile(result.file, result.source, "utf8");
      packages.add(result.dir);
      ok++;
    } catch (err) {
      console.error("  ✗ " + path.relative(REPO_ROOT, schemaPath));
      console.error("    " + err.message);
      process.exitCode = 1;
    }
  }

  // Format in-process rather than leaving it to a second command the way
  // `cargo run -p trust-tasks-codegen && cargo fmt --all` does.
  //
  // The Rust convention works because `cargo fmt` is unmissable — the build is
  // already a Cargo invocation. Here the generator is a Node script, so a
  // contributor without Go on their PATH would generate unformatted output,
  // commit it, and discover in CI that the drift job reformats everything. So
  // gofmt runs here, and its absence is a hard failure: silently emitting
  // output that CI will reformat is the drift this guards against.
  const formatted = spawnSync("gofmt", ["-w", OUT_DIR], { encoding: "utf8" });
  if (formatted.error?.code === "ENOENT") {
    throw new Error(
      "gofmt is not on PATH. The generated tree must be committed gofmt-clean or " +
        "the Go drift check will fail on every PR. Install Go (https://go.dev/dl/) and re-run.",
    );
  }
  if (formatted.status !== 0) {
    throw new Error(`gofmt failed:\n${formatted.stderr || formatted.stdout}`);
  }

  console.log(
    `\nGenerated ${ok} gofmt-clean Go files across ${packages.size} packages into ` +
      `${path.relative(REPO_ROOT, OUT_DIR)}`,
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
