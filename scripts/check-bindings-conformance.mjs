// Assert that the generated bindings say what the specs they came from say.
//
// The drift checks in rust.yml and ts.yml prove the generators were *re-run*.
// They compare each generator's output to itself, so an output that is
// consistently wrong is consistently clean. Both defects that shipped through
// them lived in exactly that gap:
//
//   * the TypeScript bindings dropped the response payload type for 265 of 273
//     request/response specs (fixed in #174), and
//   * `export type Payload` aliased a hoisted shared definition rather than the
//     schema's root type for 14 specs (fixed in #215).
//
// Neither is detectable by regenerating and diffing. Both are trivially
// detectable by reading the spec and asking whether the binding agrees with it,
// which is what this does.
//
// ── Why the checks are three-way ─────────────────────────────────────────────
//
// For the §7.2 policy constants the check compares *front matter, Rust and
// TypeScript* against each other, not just each binding against a rule
// re-implemented here. That matters: a rule re-implemented here could be
// re-implemented wrongly, and a check that shares a misreading with the thing it
// checks is decoration. The two generators share no code — different languages,
// different codebases — so a slip in either is caught by disagreement with the
// other regardless of whether the derivation below is right.
//
// ── What this adds over the drift checks, precisely ─────────────────────────
//
// Not staleness. If a spec's front matter changes and nobody regenerates, the
// drift checks already fail — regenerating produces a different file. That case
// is covered and this adds nothing to it.
//
// What it adds is the case where the *generator* is wrong: regenerating
// faithfully reproduces the wrong output, every diff is empty, and every check
// is green. That is how both defects above shipped, and it is the only failure
// mode a self-comparison can never see. The SPEC citations on each rule are what
// a reviewer checks the derivation itself against.
//
// ── Deliberately NOT factored out ────────────────────────────────────────────
//
// `expectedPolicy()` duplicates logic that also exists in
// scripts/build-ts-bindings.mjs and trust-tasks-codegen. Do not DRY it up.
// Importing the generator's helper would make this file assert that the
// generator agrees with itself, which is the property that already holds and the
// one that let both defects ship.
//
// Run from the repo root:
//   npm run check-bindings

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";
import { discoverSpecs as discoverSpecsShared } from "./lib/specs.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const SPECS_DIR = path.join(ROOT, "specs");
const TS_DIR = path.join(ROOT, "trust-tasks-ts", "src");
const RS_DIR = path.join(ROOT, "trust-tasks-rs", "src", "specs");
const GO_DIR = path.join(ROOT, "trust-tasks-go", "specs");
const DART_DIR = path.join(ROOT, "trust-tasks-dart", "lib", "specs");

const problems = [];
const fail = (where, msg) => problems.push(`${where}: ${msg}`);

/**
 * Slugs whose Rust support is hand-written in the crate's runtime rather than
 * generated under `src/specs/`.
 *
 * `trust-task-error` is the framework's own error response: the crate models its
 * payload as a first-class type in `src/error.rs` and its Type URIs in
 * `src/type_uri.rs`, because the §7.2 pipeline constructs and returns them
 * directly. A generated module would be a second, divergent representation of
 * something the runtime already owns.
 *
 * Adding a slug here is a claim that the crate implements it another way, and is
 * checked as such below — not a way to silence a missing module.
 */
const RUST_HAND_WRITTEN = new Set(["trust-task-error"]);

/* ── Discovery ──────────────────────────────────────────────────────────── */

/**
 * Every published spec version, via the shared rule in `scripts/lib/specs.mjs`.
 *
 * This one *is* safe to share, unlike `expectedPolicy()` below. Which
 * directories are specs has a single right answer and no independent-derivation
 * property to preserve; re-walking the tree here only bought a second chance to
 * disagree with the build about what the registry contains. The policy
 * derivation stays duplicated on purpose — see the header note.
 *
 * Structural problems are surfaced here too rather than swallowed: a version
 * folder missing half its pair fails this check the same way it fails the
 * build, because a spec the generators can see and the registry cannot is
 * exactly the drift this script exists to catch.
 */
function discoverSpecs() {
  return discoverSpecsShared({
    specsDir: SPECS_DIR,
    onIncomplete: ({ rel, message }) => fail(`specs/${rel}`, message),
    onNestedSlug: ({ rel, message }) => console.warn(`  warn: specs/${rel}: ${message}`),
  });
}

/**
 * Index generated modules by the Type URI they declare, rather than by deriving
 * a file path from the slug.
 *
 * Path derivation would have to reproduce each generator's own naming rules —
 * hyphen-to-underscore, version-to-module, keyword escapes — and would then be a
 * fourth place those rules live. Reading the URI each module declares asks the
 * module who it is instead, which is also what makes orphan detection fall out
 * for free.
 */
function indexGenerated(dir, filename, uriPattern) {
  const byUri = new Map();
  if (!fs.existsSync(dir)) return byUri;
  (function walk(d) {
    for (const entry of fs.readdirSync(d, { withFileTypes: true })) {
      const full = path.join(d, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (filename.test(entry.name)) {
        const src = fs.readFileSync(full, "utf8");
        const m = uriPattern.exec(src);
        if (m) byUri.set(m[1], { file: full, src });
      }
    }
  })(dir);
  return byUri;
}

/* ── Independent derivation of the §7.2 policy ──────────────────────────── */

/**
 * Derive the per-variant policy from a spec's front matter, per SPEC §7.3.
 *
 * * `bearer` — item 12. Absent is false.
 * * `proofRequirement` — item 8. Either a single `requirement` covering both
 *   variants, or a per-variant `{request, response}` pair. Where the per-variant
 *   form omits `response`, the request's value applies: the only reading that
 *   cannot weaken a variant by omission.
 * * `issuedAtRequirement` — item 17. Same two forms as `proofRequirement`,
 *   normalised the same way. Absent leaves the §4.2 SHOULD in place, which is
 *   `false` here: only `REQUIRED` obliges a consumer to reject.
 * * `parties[].requirement` — item 5, including the party swap. A *response*
 *   addresses the original producer, so the requirement governing its
 *   `recipient` member is the one declared for the request's **issuer**.
 */
function expectedPolicy(meta) {
  const partyRequired = (member) =>
    (meta.parties || []).find((p) => p && p.member === member)?.requirement === "REQUIRED";

  const pr = meta.proofRequirement || {};
  const proof =
    typeof pr.requirement === "string"
      ? { request: pr.requirement === "REQUIRED", response: pr.requirement === "REQUIRED" }
      : {
          request: pr.request === "REQUIRED",
          response: (pr.response ?? pr.request) === "REQUIRED",
        };

  const ir = meta.issuedAtRequirement || {};
  const issuedAt =
    typeof ir.requirement === "string"
      ? { request: ir.requirement === "REQUIRED", response: ir.requirement === "REQUIRED" }
      : {
          request: ir.request === "REQUIRED",
          response: (ir.response ?? ir.request) === "REQUIRED",
        };

  const isBearer = meta.bearer === true;
  return {
    request: {
      isBearer,
      isProofRequired: proof.request,
      isRecipientRequired: partyRequired("recipient"),
      isIssuedAtRequired: issuedAt.request,
    },
    response: {
      isBearer,
      isProofRequired: proof.response,
      isRecipientRequired: partyRequired("issuer"),
      isIssuedAtRequired: issuedAt.response,
    },
  };
}

/* ── Extraction from generated output ───────────────────────────────────── */

function tsPolicy(src, constName) {
  const block = new RegExp(`export const ${constName} = \\{([^}]*)\\}`, "m").exec(src);
  if (!block) return null;
  const read = (key) => {
    const m = new RegExp(`${key}:\\s*(true|false)`).exec(block[1]);
    return m ? m[1] === "true" : null;
  };
  const readDefault = (key, dflt) => {
    const v = read(key);
    return v === null ? dflt : v;
  };
  return {
    isBearer: read("isBearer"),
    isProofRequired: read("isProofRequired"),
    isRecipientRequired: read("isRecipientRequired"),
    // Absent means the module predates the field; `SpecPolicy` declares it
    // optional and `enforceSpecPolicy` reads absence as false, so the
    // comparison must too.
    isIssuedAtRequired: readDefault("isIssuedAtRequired", false),
  };
}

/** The consts inside `impl crate::Payload for <Ident> { … }`. */
function rustPolicy(src, ident) {
  const block = new RegExp(`impl crate::Payload for ${ident} \\{([\\s\\S]*?)\\n\\}`, "m").exec(src);
  if (!block) return null;
  const read = (name, dflt) => {
    const m = new RegExp(`const ${name}: bool = (true|false);`).exec(block[1]);
    return m ? m[1] === "true" : dflt;
  };
  // rustfmt wraps a long Type URI onto its own line, so the string is not
  // necessarily on the same line as the `=`.
  const uri = /const TYPE_URI: &'static str =\s*"([^"]+)"/.exec(block[1]);
  return {
    typeUri: uri ? uri[1] : null,
    // Absent means the generator relied on the trait default, which is false.
    isBearer: read("IS_BEARER", false),
    isProofRequired: read("IS_PROOF_REQUIRED", false),
    isRecipientRequired: read("IS_RECIPIENT_REQUIRED", false),
    isIssuedAtRequired: read("IS_ISSUED_AT_REQUIRED", false),
  };
}

/** The fields inside `var <name> = trusttasks.SpecPolicy{ … }`. */
function goPolicy(src, name) {
  const block = new RegExp(`var ${name} = trusttasks\\.SpecPolicy\\{([\\s\\S]*?)\\n\\}`, "m").exec(src);
  if (!block) return null;
  const read = (field, dflt) => {
    const m = new RegExp(`${field}:\\s*(true|false),`).exec(block[1]);
    return m ? m[1] === "true" : dflt;
  };
  const uri = /TypeURI:\s*(\w+),/.exec(block[1]);
  // The policy references the package's own TypeURI / ResponseTypeURI const
  // rather than repeating the literal, so resolve one level to compare it.
  let typeUri = null;
  if (uri) {
    const decl = new RegExp(`const ${uri[1]} = "([^"]+)"`).exec(src);
    typeUri = decl ? decl[1] : null;
  }
  return {
    typeUri,
    // Go's zero value for a bool is false and the generator emits every field
    // explicitly, so an absent one would be a generator bug, not a default.
    isBearer: read("IsBearer", false),
    isProofRequired: read("IsProofRequired", false),
    isRecipientRequired: read("IsRecipientRequired", false),
    isIssuedAtRequired: read("IsIssuedAtRequired", false),
  };
}

/** Parse `const <name> = "…"` — a Go interpreted string literal — out of a module. */
function goSchemaConst(src, name) {
  const m = new RegExp(`const ${name} = ("(?:[^"\\\\]|\\\\.)*")`).exec(src);
  if (!m) return null;
  try {
    // Go and JSON agree on every escape this generator emits (it escapes only
    // \", \\, \n, \r, \t and \xNN, and writes everything else as raw UTF-8),
    // except \xNN, which JSON spells \u00NN.
    const goLiteral = m[1].replace(/\\x([0-9a-fA-F]{2})/g, (_, hex) => `\\u00${hex}`);
    return JSON.parse(JSON.parse(goLiteral));
  } catch {
    return null;
  }
}

/** The named arguments inside `const SpecPolicy <name> = SpecPolicy( … );`. */
function dartPolicy(src, name) {
  const block = new RegExp(
    `const SpecPolicy ${name} = SpecPolicy\\(([\\s\\S]*?)\\n\\);`,
    "m",
  ).exec(src);
  if (!block) return null;
  const read = (field, dflt) => {
    const m = new RegExp(`${field}:\\s*(true|false),`).exec(block[1]);
    return m ? m[1] === "true" : dflt;
  };
  // The policy references the library's own `typeUri` / `responseTypeUri`
  // constant rather than repeating the literal, so resolve one level.
  const ref = /typeUri:\s*(\w+),/.exec(block[1]);
  let typeUri = null;
  if (ref) {
    // `dart format` wraps a long Type URI onto its own line, so the literal is
    // not necessarily on the same line as the `=` — the same allowance
    // rustPolicy makes for rustfmt.
    const decl = new RegExp(
      // `\\s` not `\s`: inside a template literal an unrecognised escape
      // collapses to the bare character before the RegExp sees it.
      `const String ${ref[1]}\\s*=\\s*\x27([^\x27]+)\x27;`,
    ).exec(src);
    typeUri = decl ? decl[1] : null;
  }
  return {
    typeUri,
    // The generator emits every field explicitly, so an absent one would be a
    // generator bug rather than a default.
    isBearer: read("isBearer", false),
    isProofRequired: read("isProofRequired", false),
    isRecipientRequired: read("isRecipientRequired", false),
    isIssuedAtRequired: read("isIssuedAtRequired", false),
  };
}

/**
 * Parse `const String <name> = '…';` — a Dart single-quoted literal, possibly
 * several adjacent ones after `dart format` wraps a long line.
 */
function dartSchemaConst(src, name) {
  const m = new RegExp(`const String ${name} =([\\s\\S]*?);\\n`).exec(src);
  if (!m) return null;
  const parts = m[1].match(/\x27(?:[^\x27\\]|\\.)*\x27/g);
  if (!parts || parts.length === 0) return null;
  let text = "";
  for (const part of parts) {
    const body = part.slice(1, -1);
    for (let i = 0; i < body.length; i++) {
      if (body[i] !== "\\") {
        text += body[i];
        continue;
      }
      const next = body[++i];
      if (next === "n") text += "\n";
      else if (next === "r") text += "\r";
      else if (next === "t") text += "\t";
      else if (next === "x") {
        text += String.fromCharCode(parseInt(body.slice(i + 1, i + 3), 16));
        i += 2;
      } else text += next; // \\ , \x27 and \$ all stand for themselves
    }
  }
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function comparePolicy(where, variant, expected, actual, lang) {
  if (!actual) {
    fail(where, `${lang} declares no ${variant} policy block`);
    return;
  }
  for (const key of [
    "isBearer",
    "isProofRequired",
    "isRecipientRequired",
    "isIssuedAtRequired",
  ]) {
    if (actual[key] !== expected[key]) {
      fail(
        where,
        `${lang} ${variant} ${key} is ${actual[key]}, but spec.md front matter says ${expected[key]} ` +
          `(SPEC §7.3 items 5, 8, 12, 17). Either the front matter changed without regenerating, or the generator is wrong.`,
      );
    }
  }
}

/* ── Main ───────────────────────────────────────────────────────────────── */

/** Key-sorted JSON, so two documents compare on content rather than key order. */
function stableJson(value) {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((k) => `${JSON.stringify(k)}:${stableJson(value[k])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

/** Every `$ref` in `node` that points outside the document. */
function collectExternalRefs(node, out = []) {
  if (Array.isArray(node)) {
    node.forEach((v) => collectExternalRefs(v, out));
  } else if (node && typeof node === "object") {
    if (typeof node.$ref === "string" && !node.$ref.startsWith("#")) out.push(node.$ref);
    for (const v of Object.values(node)) collectExternalRefs(v, out);
  }
  return out;
}

/**
 * Parse `export const <name> = { … } as const;` out of a generated TS module.
 *
 * Brace-counting rather than a regex: the schema is a large nested literal and
 * a lazy match stops at the first `}`. String literals are skipped so a brace
 * inside a `pattern` or a description does not throw the count off.
 */
function tsSchemaConst(src, name) {
  const start = src.indexOf(`export const ${name} = {`);
  if (start < 0) return null;
  const open = src.indexOf("{", start);
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let i = open; i < src.length; i++) {
    const ch = src[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === '"') inString = false;
      continue;
    }
    if (ch === '"') inString = true;
    else if (ch === "{") depth++;
    else if (ch === "}" && --depth === 0) {
      try {
        return JSON.parse(src.slice(open, i + 1));
      } catch {
        return null;
      }
    }
  }
  return null;
}

/**
 * Parse `const PAYLOAD_SCHEMA: Option<&'static str> = Some("…")` out of the
 * `impl crate::Payload for <ident>` block of a generated Rust module.
 *
 * rustfmt splits the string literal across lines with `\` continuations, so the
 * lines are rejoined before unescaping.
 */
function rustSchemaConst(src, ident) {
  const implStart = src.indexOf(`impl crate::Payload for ${ident} {`);
  if (implStart < 0) return null;
  const implEnd = src.indexOf("\n}", implStart);
  const block = src.slice(implStart, implEnd < 0 ? undefined : implEnd);
  const m = /const PAYLOAD_SCHEMA: Option<&'static str> = Some\(\s*"([\s\S]*?)"\s*,?\s*\)\s*;/.exec(block);
  if (!m) return null;
  // Undo rustfmt's line continuations, then the Rust string escapes.
  const joined = m[1].replace(/\\\r?\n\s*/g, "");
  try {
    return JSON.parse(JSON.parse(`"${joined}"`));
  } catch {
    return null;
  }
}

const specs = discoverSpecs();
const tsByUri = indexGenerated(TS_DIR, /^payload\.ts$/, /export const TYPE_URI = "([^"]+)"/);
const rsByUri = indexGenerated(RS_DIR, /^v\d+_\d+\.rs$/, /const TYPE_URI: &'static str =\s*"([^"]+)"/);
const goByUri = indexGenerated(GO_DIR, /^payload\.go$/, /const TypeURI = "([^"]+)"/);
// `\s*` around the `=`: `dart format` wraps a long Type URI onto its own line,
// so the literal is not necessarily on the same line as the constant's name.
const dartByUri = indexGenerated(
  DART_DIR,
  /^payload\.dart$/,
  /const String typeUri\s*=\s*'([^']+)';/,
);

const seenTs = new Set();
const seenRs = new Set();
const seenGo = new Set();
const seenDart = new Set();

/** Every hand-written Rust source, concatenated, for the RUST_HAND_WRITTEN check. */
const rustHandWrittenSrc = (function read(dir, acc = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) read(full, acc);
    else if (entry.name.endsWith(".rs")) acc.push(fs.readFileSync(full, "utf8"));
  }
  return acc;
})(path.join(ROOT, "trust-tasks-rs", "src")).join("\n");

for (const spec of specs) {
  const where = `${spec.slug}/${spec.version}`;
  const typeUri = `https://trusttasks.org/spec/${spec.slug}/${spec.version}`;

  const raw = fs.readFileSync(path.join(spec.dir, "spec.md"), "utf8");
  const end = raw.indexOf("\n---", 3);
  const meta = raw.startsWith("---") && end > 0 ? YAML.parse(raw.slice(3, end)) : null;
  if (!meta) {
    fail(where, "spec.md has no parseable front matter");
    continue;
  }
  const schema = JSON.parse(fs.readFileSync(path.join(spec.dir, "payload.schema.json"), "utf8"));
  const hasResponse = Boolean(schema?.$defs?.Response);
  const expected = expectedPolicy(meta);

  /* — TypeScript — */
  const ts = tsByUri.get(typeUri);
  if (!ts) {
    fail(where, `no generated TypeScript module declares TYPE_URI ${typeUri}`);
  } else {
    seenTs.add(typeUri);

    // SPEC §4.4.1: a specification with no success response MUST NOT emit a
    // #response document, so a RESPONSE_TYPE_URI constant for one is an
    // invitation to violate it. Checked in both directions.
    const hasResponseUri = ts.src.includes("export const RESPONSE_TYPE_URI");
    const hasResponseType = /export type Response = /.test(ts.src);
    if (hasResponse && !(hasResponseUri && hasResponseType)) {
      fail(
        where,
        `payload.schema.json declares $defs.Response, but the TypeScript module exports ` +
          `${hasResponseUri ? "" : "no RESPONSE_TYPE_URI"}${!hasResponseUri && !hasResponseType ? " and " : ""}` +
          `${hasResponseType ? "" : "no Response type"} — the response half of the specification is unreachable.`,
      );
    }
    if (!hasResponse && (hasResponseUri || hasResponseType)) {
      fail(
        where,
        `this specification declares no success response, but the TypeScript module exports a response ` +
          `constant or type. SPEC §4.4.1 says its consumers MUST NOT emit a #response document.`,
      );
    }

    // #215: an object-rooted schema whose `Payload` alias resolves to a hoisted
    // shared definition rather than the root type.
    //
    // The test is what the alias *resolves to*, not what it is called. A name
    // heuristic would be wrong in both directions: the root type is named from
    // the schema's title, so plenty of correct roots do not end in "Payload"
    // (`AuthPasskeyLoginStart`), and a hoisted definition could be named
    // anything. So resolve one level and look at the declaration: an interface
    // or an object type literal is a root, a bare scalar alias is a hoisted
    // `$ref` that displaced it.
    const alias = /^export type Payload = ([A-Za-z0-9_$]+);\s*$/m.exec(ts.src);
    if (schema?.type === "object" && alias) {
      const target = alias[1];
      const decl = new RegExp(`^export (interface|type) ${target}\\b\\s*(=\\s*)?(.)`, "m").exec(ts.src);
      const resolvesToObject = decl && (decl[1] === "interface" || decl[3] === "{");
      if (!resolvesToObject) {
        fail(
          where,
          `TypeScript exports \`Payload = ${target}\`, but this schema's root is an object and ` +
            `${target} is ${decl ? "a bare alias" : "not declared in the module"} — the alias resolves to a ` +
            `hoisted definition rather than the request payload interface.`,
        );
      }
    }

    comparePolicy(where, "request", expected.request, tsPolicy(ts.src, "SPEC"), "TypeScript");
    if (hasResponse) {
      comparePolicy(where, "response", expected.response, tsPolicy(ts.src, "RESPONSE_SPEC"), "TypeScript");
    }
  }

  /* — Go — */
  const go = goByUri.get(typeUri);
  if (!go) {
    fail(where, `no generated Go package declares TypeURI ${typeUri}`);
  } else {
    seenGo.add(typeUri);

    // SPEC §4.4.1, as for the other two: a specification with no success
    // response MUST NOT emit a #response document, so a ResponseTypeURI for one
    // is an invitation to violate it. Checked in both directions.
    const hasResponseUri = /^const ResponseTypeURI = /m.test(go.src);
    const hasResponseType = /^type Response\b/m.test(go.src);
    if (hasResponse && !(hasResponseUri && hasResponseType)) {
      fail(
        where,
        `payload.schema.json declares $defs.Response, but the Go package exports ` +
          `${hasResponseUri ? "" : "no ResponseTypeURI"}${!hasResponseUri && !hasResponseType ? " and " : ""}` +
          `${hasResponseType ? "" : "no Response type"} — the response half of the specification is unreachable.`,
      );
    }
    if (!hasResponse && (hasResponseUri || hasResponseType)) {
      fail(
        where,
        `this specification declares no success response, but the Go package exports a response ` +
          `constant or type. SPEC §4.4.1 says its consumers MUST NOT emit a #response document.`,
      );
    }

    // Every payload root in the registry is `type: object`, so `Payload` must be
    // a struct. A scalar alias there would mean the emitter fell through to its
    // json.RawMessage fallback and the specification's members are unreachable.
    if (schema?.type === "object" && !/^type Payload struct \{/m.test(go.src)) {
      fail(
        where,
        `this schema's root is an object, but the Go package does not declare \`type Payload struct\` — ` +
          `the emitter fell back to an untyped representation and the payload's members are unreachable.`,
      );
    }

    const goReq = goPolicy(go.src, "Spec");
    if (goReq && goReq.typeUri !== typeUri) {
      fail(where, `Go Spec.TypeURI is ${goReq.typeUri}, expected ${typeUri}.`);
    }
    comparePolicy(where, "request", expected.request, goReq, "Go");
    if (hasResponse) {
      const goResp = goPolicy(go.src, "ResponseSpec");
      if (goResp && goResp.typeUri !== `${typeUri}#response`) {
        fail(where, `Go ResponseSpec.TypeURI is ${goResp.typeUri}, expected ${typeUri}#response.`);
      }
      comparePolicy(where, "response", expected.response, goResp, "Go");
    }
  }

  /* — Dart — */
  const dart = dartByUri.get(typeUri);
  if (!dart) {
    fail(where, `no generated Dart library declares typeUri ${typeUri}`);
  } else {
    seenDart.add(typeUri);

    // SPEC §4.4.1, as for the other three.
    const hasResponseUri = /^const String responseTypeUri\s*=/m.test(dart.src);
    const hasResponseType = /^(class Response\b|typedef Response =|extension type const Response\()/m.test(dart.src);
    if (hasResponse && !(hasResponseUri && hasResponseType)) {
      fail(
        where,
        `payload.schema.json declares $defs.Response, but the Dart library exports ` +
          `${hasResponseUri ? "" : "no responseTypeUri"}${!hasResponseUri && !hasResponseType ? " and " : ""}` +
          `${hasResponseType ? "" : "no Response type"} — the response half of the specification is unreachable.`,
      );
    }
    if (!hasResponse && (hasResponseUri || hasResponseType)) {
      fail(
        where,
        `this specification declares no success response, but the Dart library exports a response ` +
          `constant or type. SPEC §4.4.1 says its consumers MUST NOT emit a #response document.`,
      );
    }

    // Every payload root in the registry is `type: object`, so `Payload` must be
    // a class. A typedef to a scalar there would mean the emitter fell through
    // to its `Object?` fallback and the specification's members are unreachable.
    if (schema?.type === "object" && !/^class Payload \{/m.test(dart.src)) {
      fail(
        where,
        `this schema's root is an object, but the Dart library does not declare \`class Payload\` — ` +
          `the emitter fell back to an untyped representation and the payload's members are unreachable.`,
      );
    }

    const dartReq = dartPolicy(dart.src, "spec");
    if (dartReq && dartReq.typeUri !== typeUri) {
      fail(where, `Dart spec.typeUri is ${dartReq.typeUri}, expected ${typeUri}.`);
    }
    comparePolicy(where, "request", expected.request, dartReq, "Dart");
    if (hasResponse) {
      const dartResp = dartPolicy(dart.src, "responseSpec");
      if (dartResp && dartResp.typeUri !== `${typeUri}#response`) {
        fail(where, `Dart responseSpec.typeUri is ${dartResp.typeUri}, expected ${typeUri}#response.`);
      }
      comparePolicy(where, "response", expected.response, dartResp, "Dart");
    }
  }

  /* — Rust — */
  const rs = rsByUri.get(typeUri);
  if (!rs && RUST_HAND_WRITTEN.has(spec.slug)) {
    // Not an exemption from being implemented — an exemption from being
    // *generated*. Assert the hand-written implementation still exists, so
    // deleting it fails rather than silently passing.
    //
    // The test is for the slug, not the versioned Type URI: the crate recognises
    // this slug in `type_uri.rs` and parses whatever version follows, so there is
    // no per-version literal to match and asserting one would be theatre.
    if (!rustHandWrittenSrc.includes(spec.slug)) {
      fail(
        where,
        `${spec.slug} is implemented by hand in trust-tasks-rs rather than generated, but no source ` +
          `file under trust-tasks-rs/src/ mentions the slug. Either the hand-written implementation ` +
          `was removed, or this slug should no longer be listed in RUST_HAND_WRITTEN.`,
      );
    }
  } else if (!rs) {
    fail(where, `no generated Rust module declares TYPE_URI ${typeUri}`);
  } else {
    seenRs.add(typeUri);
    const reqPolicy = rustPolicy(rs.src, "Payload");
    const respPolicy = rustPolicy(rs.src, "Response");

    if (hasResponse && !respPolicy) {
      fail(where, `payload.schema.json declares $defs.Response, but the Rust module has no \`impl crate::Payload for Response\`.`);
    }
    if (!hasResponse && respPolicy) {
      fail(where, `this specification declares no success response, but the Rust module implements one (SPEC §4.4.1).`);
    }
    if (respPolicy && respPolicy.typeUri !== `${typeUri}#response`) {
      fail(where, `Rust Response TYPE_URI is ${respPolicy.typeUri}, expected ${typeUri}#response.`);
    }

    comparePolicy(where, "request", expected.request, reqPolicy, "Rust");
    if (hasResponse && respPolicy) {
      comparePolicy(where, "response", expected.response, respPolicy, "Rust");
    }

    /* — The four shipped schemas must be the same document — */
    //
    // Each generator inlines cross-file `$ref`s itself: `resolve_cross_file_refs`
    // in Rust, `inlineCrossFileRefs` in the TS script, and a third copy of the
    // same walk in the Go script. Three independent implementations of the same
    // splice, and SPEC §7.2 item 2 is only well-defined if they agree — a
    // consumer validating in Rust, one in TypeScript and one in Go must accept
    // and reject the same payloads.
    //
    // Compared as parsed JSON, so formatting differences between
    // `serde_json::to_string_pretty`, `JSON.stringify` and the Go string literal
    // are not failures. Nothing here re-derives the expected schema: this asserts
    // the generators agree with *each other*, which is the property none can
    // establish alone. That is also why the Go generator was written with its own
    // resolver rather than importing the TypeScript one — see its file header.
    if (ts) {
      for (const [variant, tsConst, rsConst, goConst, dartConst] of [
        ["request", "PAYLOAD_SCHEMA", "Payload", "PayloadSchemaJSON", "payloadSchemaJson"],
        [
          "response",
          "RESPONSE_PAYLOAD_SCHEMA",
          "Response",
          "ResponsePayloadSchemaJSON",
          "responsePayloadSchemaJson",
        ],
      ]) {
        if (variant === "response" && !hasResponse) continue;

        const shipped = [
          ["TypeScript", tsConst, tsSchemaConst(ts.src, tsConst)],
          ["Rust", rsConst, rustSchemaConst(rs.src, rsConst)],
        ];
        if (go) shipped.push(["Go", goConst, goSchemaConst(go.src, goConst)]);
        if (dart) shipped.push(["Dart", dartConst, dartSchemaConst(dart.src, dartConst)]);

        const missing = shipped.filter(([, , doc]) => !doc);
        for (const [lang, constName] of missing) {
          fail(
            where,
            `the ${lang} binding exports no ${constName} — §7.2 item 2 has no artifact to run against.`,
          );
        }
        if (missing.length > 0) continue;

        // Compare every binding against the first, so a disagreement names both
        // sides rather than only reporting that "something" differs.
        const [refLang, , refSchema] = shipped[0];
        const refJson = stableJson(refSchema);
        for (const [lang, , doc] of shipped.slice(1)) {
          if (stableJson(doc) !== refJson) {
            fail(
              where,
              `the ${variant} schema shipped by the ${lang} binding and the one shipped by the ${refLang} ` +
                `binding are not the same document. The $ref inliners have diverged, so the two libraries ` +
                `would disagree about which payloads conform.`,
            );
          }
        }

        // An un-inlined cross-file $ref is unresolvable at runtime: the
        // consumer has no filesystem to walk and no base URI to resolve against.
        for (const [lang, , doc] of shipped) {
          const external = collectExternalRefs(doc);
          if (external.length > 0) {
            fail(
              where,
              `the ${variant} schema shipped by ${lang} still carries unresolved cross-file $ref(s) ` +
                `(${external.slice(0, 3).join(", ")}) — a runtime validator cannot follow them.`,
            );
          }
        }
      }
    }
  }
}

/* Orphans: a generated module for a spec that no longer exists is a stale
   artifact a consumer can still import and build against. */
for (const [uri, mod] of tsByUri) {
  if (!seenTs.has(uri)) {
    fail(path.relative(ROOT, mod.file), `declares TYPE_URI ${uri}, which matches no specification under specs/`);
  }
}
for (const [uri, mod] of rsByUri) {
  if (!seenRs.has(uri)) {
    fail(path.relative(ROOT, mod.file), `declares TYPE_URI ${uri}, which matches no specification under specs/`);
  }
}
for (const [uri, mod] of goByUri) {
  if (!seenGo.has(uri)) {
    fail(path.relative(ROOT, mod.file), `declares TypeURI ${uri}, which matches no specification under specs/`);
  }
}
for (const [uri, mod] of dartByUri) {
  if (!seenDart.has(uri)) {
    fail(path.relative(ROOT, mod.file), `declares typeUri ${uri}, which matches no specification under specs/`);
  }
}

/* ── The error Type URI both SDKs emit ──────────────────────────────────── */

// `trust-task-error` is in SKIP_SLUGS: the Rust side is hand-modelled in
// `error.rs` and the TypeScript side in `_runtime/document.ts`, so the version
// each SDK emits is a hand-edited constant in two languages that nothing
// compared. Adding a standard error code means publishing a new
// `trust-task-error` version and pointing BOTH at it; updating one and not the
// other leaves two libraries that disagree about the `type` of every error
// document they emit — which no drift check can see, because regenerating
// reproduces both constants exactly as written.
{
  const versions = fs
    .readdirSync(path.join(SPECS_DIR, "trust-task-error"), { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name)
    .sort((a, b) => {
      const [am, an] = a.split(".").map(Number);
      const [bm, bn] = b.split(".").map(Number);
      return am - bm || an - bn;
    });
  const newest = versions[versions.length - 1];

  const RS_DOC = path.join(ROOT, "trust-tasks-rs", "src", "document.rs");
  const TS_DOC = path.join(ROOT, "trust-tasks-ts", "src", "_runtime", "document.ts");
  const GO_DOC = path.join(ROOT, "trust-tasks-go", "trusttasks", "document.go");
  const DART_DOC = path.join(ROOT, "trust-tasks-dart", "lib", "src", "runtime", "document.dart");

  const rsSrc = fs.readFileSync(RS_DOC, "utf8");
  const tsSrc = fs.readFileSync(TS_DOC, "utf8");
  const goSrc = fs.readFileSync(GO_DOC, "utf8");
  const dartSrc = fs.readFileSync(DART_DOC, "utf8");

  const rsMatch = rsSrc.match(/TypeUri::canonical\("trust-task-error",\s*(\d+),\s*(\d+)\)/);
  const tsMatch = tsSrc.match(
    /TRUST_TASK_ERROR_TYPE_URI\s*=\s*"https:\/\/trusttasks\.org\/spec\/trust-task-error\/(\d+\.\d+)"/,
  );

  const goMatch = goSrc.match(
    /TrustTaskErrorTypeURI\s*=\s*"https:\/\/trusttasks\.org\/spec\/trust-task-error\/(\d+\.\d+)"/,
  );

  const rsVersion = rsMatch ? `${rsMatch[1]}.${rsMatch[2]}` : null;
  const tsVersion = tsMatch ? tsMatch[1] : null;
  const dartMatch = dartSrc.match(
    /trustTaskErrorTypeUri =\s*\x27https:\/\/trusttasks\.org\/spec\/trust-task-error\/(\d+\.\d+)\x27/,
  );

  const goVersion = goMatch ? goMatch[1] : null;
  const dartVersion = dartMatch ? dartMatch[1] : null;

  if (!rsVersion) {
    fail(path.relative(ROOT, RS_DOC), "no TypeUri::canonical(\"trust-task-error\", MAJOR, MINOR) found — trust_task_error_type_uri() is the crate's only statement of which error specification it emits");
  }
  if (!tsVersion) {
    fail(path.relative(ROOT, TS_DOC), "no TRUST_TASK_ERROR_TYPE_URI found — it is the package's only statement of which error specification it emits");
  }

  if (!goVersion) {
    fail(path.relative(ROOT, GO_DOC), "no TrustTaskErrorTypeURI found — it is the module's only statement of which error specification it emits");
  }
  if (!dartVersion) {
    fail(path.relative(ROOT, DART_DOC), "no trustTaskErrorTypeUri found — it is the package's only statement of which error specification it emits");
  }

  const emitted = [
    ["trust-tasks-rs", rsVersion, "document.rs, trust_task_error_type_uri"],
    ["@openvtc/trust-tasks", tsVersion, "_runtime/document.ts, TRUST_TASK_ERROR_TYPE_URI"],
    ["trust-tasks-go", goVersion, "trusttasks/document.go, TrustTaskErrorTypeURI"],
    ["trust_tasks", dartVersion, "lib/src/runtime/document.dart, trustTaskErrorTypeUri"],
  ].filter(([, version]) => version !== null);

  const distinct = new Set(emitted.map(([, version]) => version));
  if (distinct.size > 1) {
    fail(
      "trust-tasks-rs / trust-tasks-ts / trust-tasks-go / trust_tasks",
      `the SDKs emit different error documents: ` +
        emitted.map(([name, version, where]) => `${name} sends trust-task-error/${version} (${where})`).join("; ") +
        `. A new error specification version must be adopted in all of them or none.`,
    );
  }

  const agreed = distinct.size === 1 ? [...distinct][0] : null;

  if (agreed && !fs.existsSync(path.join(SPECS_DIR, "trust-task-error", agreed))) {
    fail(
      "trust-tasks-rs / trust-tasks-ts / trust-tasks-go / trust_tasks",
      `every SDK emits trust-task-error/${agreed}, which does not exist under specs/trust-task-error/`,
    );
  } else if (agreed && agreed !== newest) {
    // Lagging is a decision, not necessarily a defect — a published version may
    // be deliberately unadopted — so this is a note, not a failure.
    console.log(
      `  note: every SDK emits trust-task-error/${agreed}; specs/trust-task-error/${newest} is published. ` +
        `Adopt it in trust_task_error_type_uri(), TRUST_TASK_ERROR_TYPE_URI, TrustTaskErrorTypeURI ` +
        `and trustTaskErrorTypeUri together, or leave all four.`,
    );
  }
}

/* ── The Dart package's two version declarations ────────────────────────── */

// `version:` in pubspec.yaml is what pub.dev publishes under; `packageVersion`
// in lib/src/runtime/version.dart is what a consumer can log. Nothing at
// runtime reads both, so a release that moved one and not the other would ship
// a package whose own constant misreports its version — and the first attempt at
// the rewrite in scripts/release-dart-pr.sh did exactly that, because its regex
// could not match a version containing dots.
{
  const PUBSPEC = path.join(ROOT, "trust-tasks-dart", "pubspec.yaml");
  const VERSION_DART = path.join(ROOT, "trust-tasks-dart", "lib", "src", "runtime", "version.dart");

  const pubspecVersion = /^version:\s*(\S+)\s*$/m.exec(fs.readFileSync(PUBSPEC, "utf8"));
  const constVersion = /^const String packageVersion = \x27([^\x27]+)\x27;$/m.exec(
    fs.readFileSync(VERSION_DART, "utf8"),
  );

  if (!pubspecVersion) {
    fail(path.relative(ROOT, PUBSPEC), "no top-level `version:` found");
  }
  if (!constVersion) {
    fail(path.relative(ROOT, VERSION_DART), "no `const String packageVersion` found");
  }
  if (pubspecVersion && constVersion && pubspecVersion[1] !== constVersion[1]) {
    fail(
      "trust-tasks-dart",
      `pubspec.yaml declares version ${pubspecVersion[1]} but ` +
        `lib/src/runtime/version.dart declares ${constVersion[1]}. The Release PR moves both; ` +
        `one moving without the other means the published package misreports its own version.`,
    );
  }
}

/* ── Report ─────────────────────────────────────────────────────────────── */


if (problems.length > 0) {
  console.error(`\nBindings do not agree with their specifications:\n`);
  for (const p of problems) console.error(`  - ${p}`);
  console.error(
    `\n${problems.length} problem(s). These are not drift: regenerating will not fix them ` +
      `unless the generator itself is corrected.\n`,
  );
  process.exit(1);
}

console.log(
  `Bindings conformance: ${specs.length} specifications checked against ` +
    `${tsByUri.size} TypeScript, ${rsByUri.size} Rust, ${goByUri.size} Go and ` +
    `${dartByUri.size} Dart modules — all agree.`,
);
