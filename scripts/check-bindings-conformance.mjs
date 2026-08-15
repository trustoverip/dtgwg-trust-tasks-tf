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

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const SPECS_DIR = path.join(ROOT, "specs");
const TS_DIR = path.join(ROOT, "trust-tasks-ts", "src");
const RS_DIR = path.join(ROOT, "trust-tasks-rs", "src", "specs");

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

/** Every published spec version: `specs/<slug>/<version>/payload.schema.json`. */
function discoverSpecs() {
  const out = [];
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory() || entry.name.startsWith("_") || entry.name.startsWith(".")) continue;
      const full = path.join(dir, entry.name);
      if (fs.existsSync(path.join(full, "spec.md"))) {
        const rel = path.relative(SPECS_DIR, full).split(path.sep);
        out.push({
          slug: rel.slice(0, -1).join("/"),
          version: rel[rel.length - 1],
          dir: full,
        });
      } else {
        walk(full);
      }
    }
  })(SPECS_DIR);
  return out;
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

  const isBearer = meta.bearer === true;
  return {
    request: {
      isBearer,
      isProofRequired: proof.request,
      isRecipientRequired: partyRequired("recipient"),
    },
    response: {
      isBearer,
      isProofRequired: proof.response,
      isRecipientRequired: partyRequired("issuer"),
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
  return {
    isBearer: read("isBearer"),
    isProofRequired: read("isProofRequired"),
    isRecipientRequired: read("isRecipientRequired"),
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
  };
}

function comparePolicy(where, variant, expected, actual, lang) {
  if (!actual) {
    fail(where, `${lang} declares no ${variant} policy block`);
    return;
  }
  for (const key of ["isBearer", "isProofRequired", "isRecipientRequired"]) {
    if (actual[key] !== expected[key]) {
      fail(
        where,
        `${lang} ${variant} ${key} is ${actual[key]}, but spec.md front matter says ${expected[key]} ` +
          `(SPEC §7.3 items 5, 8, 12). Either the front matter changed without regenerating, or the generator is wrong.`,
      );
    }
  }
}

/* ── Main ───────────────────────────────────────────────────────────────── */

const specs = discoverSpecs();
const tsByUri = indexGenerated(TS_DIR, /^payload\.ts$/, /export const TYPE_URI = "([^"]+)"/);
const rsByUri = indexGenerated(RS_DIR, /^v\d+_\d+\.rs$/, /const TYPE_URI: &'static str =\s*"([^"]+)"/);

const seenTs = new Set();
const seenRs = new Set();

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
    `${tsByUri.size} TypeScript and ${rsByUri.size} Rust modules — all agree.`,
);
