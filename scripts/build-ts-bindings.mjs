// Generate TypeScript bindings for every Trust Tasks payload schema.
//
// Walks specs/<slug>/<version>/payload.schema.json, plus shared schemas under
// _shared/ and _framework/, and emits one .ts file per JSON Schema into
// trust-tasks-ts/src/. The output is hand-readable, ships in the
// @openvtc/trust-tasks npm package, and is consumed by Companions, mobile
// clients, and any other TS implementation.
//
// Each generated request file additionally exports:
//   - TYPE_URI       — the Trust Task type URI (https://trusttasks.org/spec/<slug>/<version>)
//   - RESPONSE_TYPE_URI — the response form (… + "#response"), emitted ONLY when
//     the schema declares $defs.Response. A spec with no success response is
//     fire-and-forget: SPEC.md §4.4.1 says its consumers MUST NOT emit a
//     `#response`-variant document, so handing implementers a ready-made
//     constant for one invites a conformance violation.
//   - Payload / Response — stable type aliases for the request and response
//     payload shapes. json-schema-to-typescript names the emitted interfaces
//     from each schema's `title` (ACLGrantPayload, KeysSignPayload, …), so
//     without these there is no name a consumer can rely on across specs and
//     no way to write code generic over a Trust Task. They mirror the Rust
//     bindings' uniform `Payload` / `Response` structs.
//
// Run from the repo root:
//   npm run build-ts-bindings
//
// Strategy: use json-schema-to-typescript. Cross-file $refs are resolved by
// its built-in reference walker against the on-disk schemas, and the resulting
// definitions are **inlined** into each spec's module — a spec that references
// `AclEntry` gets its own copy rather than importing one. Each generated module
// is therefore self-contained, with no relative imports at all. (An earlier
// version of this comment claimed the opposite; `declareExternallyReferenced`
// does not produce cross-module imports here.) Shared schemas are still emitted
// as their own modules and re-exported from the barrel, so a consumer wanting
// one canonical `VaultEntry` can import it directly — and TypeScript's
// structural typing means the inlined copies remain mutually assignable.
//
// Only `index.ts` carries relative specifiers, and they MUST end in `.js`; see
// the note where they are built.

import fsSync, { promises as fs } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";
import { compile, compileFromFile } from "json-schema-to-typescript";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");
const SPECS_DIR = path.join(REPO_ROOT, "specs");
const OUT_DIR = path.join(REPO_ROOT, "trust-tasks-ts", "src");
const RUNTIME_DIR = "_runtime";

// Slug → ts type renames go here when json-schema-to-typescript's
// auto-naming clashes with reserved words or produces awkward identifiers.
// Empty for now; populate as ergonomic issues come up.
const TYPE_RENAMES = {};

async function walk(dir, pattern) {
  const out = [];
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...(await walk(full, pattern)));
    } else if (pattern.test(entry.name)) {
      out.push(full);
    }
  }
  return out;
}

async function ensureDir(p) {
  await fs.mkdir(p, { recursive: true });
}

function relativeOutPath(schemaPath) {
  // Convert <repo>/specs/<x>/<y>/<file>.schema.json to <out>/<x>/<y>/<file>.ts
  // For payload.schema.json the file becomes "payload" — we rename to the
  // version-bearing form below.
  const rel = path.relative(SPECS_DIR, schemaPath);
  const tsRel = rel
    .replace(/\.schema\.json$/, ".ts")
    .replace(/[\\/]/g, "/");
  return path.join(OUT_DIR, tsRel);
}

function slugFromSchemaPath(schemaPath) {
  // <repo>/specs/<slug>/<version>/payload.schema.json -> { slug, version }
  // Returns null for shared schemas (where the path includes a "_"-prefixed segment).
  const rel = path.relative(SPECS_DIR, schemaPath);
  const segments = rel.split(path.sep);
  if (segments.some((s) => s.startsWith("_"))) return null;
  if (path.basename(schemaPath) !== "payload.schema.json") return null;
  const version = segments[segments.length - 2];
  const slug = segments.slice(0, -2).join("/");
  return { slug, version };
}

/**
 * Read the SPEC §7.2 policy flags out of a spec's front matter.
 *
 * These mirror `Payload::IS_BEARER` / `IS_PROOF_REQUIRED` /
 * `IS_RECIPIENT_REQUIRED` in trust-tasks-rs, and are derived the same way, so
 * a TypeScript consumer reaches the same verdict as a Rust one on the same
 * document. Without them a TypeScript implementation cannot apply §7.2 items
 * 5b, 7 or 8 at all — the requirement is per-specification, and nothing else
 * on the wire carries it.
 *
 * Note the response-variant asymmetry (§7.3 item 5): a response document swaps
 * the parties, so the requirement governing a *response*'s `recipient` member
 * is the one declared for the *issuer* party of the request.
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
  // declaration omitting `response` takes the request's value — the
  // conservative reading, and the only one that cannot weaken a variant by
  // omission.
  const pr = meta.proofRequirement || {};
  const proofRequired =
    typeof pr.requirement === "string"
      ? { request: pr.requirement === "REQUIRED", response: pr.requirement === "REQUIRED" }
      : {
          request: pr.request === "REQUIRED",
          response: (pr.response ?? pr.request) === "REQUIRED",
        };

  return {
    isBearer: meta.bearer === true,
    isProofRequired: proofRequired.request,
    responseIsProofRequired: proofRequired.response,
    // Request: the party tagged `recipient`. Response: the party tagged
    // `issuer`, because the response addresses the original producer.
    isRecipientRequired: partyRequirement("recipient"),
    responseIsRecipientRequired: partyRequirement("issuer"),
  };
}

// Synthetic property name used to drag `$defs.Response` into the compiler's
// reachability graph. Deliberately ugly so it cannot collide with a real
// payload member; it is stripped from the output before the file is written.
const RESPONSE_PROBE = "__ttResponseProbe";

// Annotation keywords that do not constrain the instance, so a `$defs.Response`
// carrying only these alongside `$ref` is a pure alias for its target.
const RESPONSE_ANNOTATIONS = new Set(["$ref", "$anchor", "title", "description"]);

/**
 * The `$ref` the response probe should point at.
 *
 * Normally `#/$defs/Response`. But where `$defs.Response` is a bare alias for an
 * external definition, we point the probe straight at that external target
 * instead. Going through the local alias makes the resolver splice the foreign
 * subschema into *this* document's context, after which the target's own
 * internal refs (`#/$defs/Scope` in the vta/did-templates shared schema) are
 * looked up against the wrong document and the compile dies. Referencing the
 * external target directly is the same route every other cross-file `$ref` in
 * the schema already takes, and resolves correctly.
 */
function responseProbeRef(response) {
  const isPureAlias =
    typeof response.$ref === "string" &&
    Object.keys(response).every((k) => RESPONSE_ANNOTATIONS.has(k));
  return isPureAlias ? { $ref: response.$ref } : { $ref: "#/$defs/Response" };
}

/**
 * The type name json-schema-to-typescript mints from a schema `title`:
 * split on anything that is not alphanumeric, upper-case each token's first
 * letter, and join. "Audit List — payload" becomes `AuditListPayload`;
 * "VTC Relationships Publish — payload" becomes `VTCRelationshipsPublishPayload`,
 * an already-upper token surviving intact.
 */
function titleTypeName(title) {
  if (typeof title !== "string") return null;
  const parts = title
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1));
  return parts.length ? parts.join("") : null;
}

/**
 * Name of the root interface/type the compiler emitted.
 *
 * This used to take the first `export interface|type` in the output, on the
 * assumption that the root always comes first. It does not. Where a schema
 * `$ref`s a shared definition — `DigestMultibase`, say — the compiler hoists
 * that definition to its own exported type, sometimes *ahead of* the root, and
 * the first-match rule then aliased `Payload` to the shared definition.
 * Fourteen published specifications shipped `export type Payload =
 * DigestMultibase`, a `string`, in place of their request payload interface.
 *
 * Nothing catches that downstream: the drift check compares the generator's
 * output to itself, `tsc` is satisfied because the alias is well-formed, and
 * no test asserts what `Payload` resolves to. So the rule has to be right here,
 * and it has to fail loudly when it is not — hence the invariant below rather
 * than a quieter heuristic.
 *
 * The root is identified by the name the compiler derives from the schema's own
 * `title`, and is used only when that name is actually present in the output.
 * Positional order is the fallback, for schemas carrying no usable title.
 */
function rootTypeName(ts, schemaPath, raw) {
  const names = [...ts.matchAll(/^export (?:interface|type) ([A-Za-z0-9_$]+)/gm)].map(
    (m) => m[1],
  );
  if (names.length === 0) {
    throw new Error(`${schemaPath}: compiled output declares no root type`);
  }
  const fromTitle = titleTypeName(raw?.title);
  const root = fromTitle && names.includes(fromTitle) ? fromTitle : names[0];

  // An object-rooted schema whose root compiled to a *bare* alias — `= string`,
  // `= DigestMultibase` — means we picked up a hoisted `$ref` instead of the
  // root, the exact failure this function exists to prevent. An object type
  // literal or a union is a legitimate root form and is left alone.
  const bareAlias = new RegExp(`^export type ${root} = ([A-Za-z0-9_$]+);\\s*$`, "m");
  if (raw?.type === "object" && bareAlias.test(ts)) {
    throw new Error(
      `${schemaPath}: root type '${root}' compiled to a bare alias, but the schema's ` +
        `root is an object — the alias would be emitted as this specification's ` +
        `\`Payload\`. Check that the schema's title matches the emitted interface.`,
    );
  }
  return root;
}

/**
 * Remove the synthetic response probe from the compiled output and return the
 * TypeScript type it resolved to.
 *
 * The probe exists because `$defs.Response` is referenced by nothing in the
 * schema — SPEC.md §4.4.1 addresses it out-of-band via the `response` anchor —
 * and json-schema-to-typescript emits only what it can reach from the root. A
 * plain compile therefore silently drops the response half of every
 * request/response specification.
 *
 * Injecting the probe and compiling ONCE (rather than compiling the request and
 * response halves separately) matters for correctness: the compiler
 * de-duplicates structurally distinct types that share a name by appending a
 * counter (`Ext`, `Ext1`, …). Two independent passes number those counters
 * independently, so a name minted in the response pass could quietly denote a
 * different shape than the identical name in the request pass.
 */
function stripResponseProbe(ts, schemaPath) {
  const probe = new RegExp(`^[ \\t]*${RESPONSE_PROBE}\\?: (.+);[ \\t]*\\r?\\n`, "m");
  const m = probe.exec(ts);
  if (!m) {
    throw new Error(
      `${schemaPath}: response probe did not survive compilation — ` +
        `$defs.Response could not be resolved to a TypeScript type`,
    );
  }
  const stripped = ts.replace(probe, "");
  if (probe.test(stripped)) {
    throw new Error(`${schemaPath}: response probe matched more than once`);
  }
  return { ts: stripped, responseType: m[1] };
}

function emitTail(slugInfo, ts, rootType, responseType, schemaPath, policy) {
  if (!slugInfo) return ""; // shared schemas are not Trust Tasks
  const { slug, version } = slugInfo;
  const typeUri = `https://trusttasks.org/spec/${slug}/${version}`;
  const lines = [
    "",
    `/** Trust Task type URI. */`,
    `export const TYPE_URI = ${JSON.stringify(typeUri)} as const;`,
    "",
    `/** Stable alias for this specification's request payload shape. */`,
  ];
  // If the compiler already named the root `Payload` the alias is redundant and
  // would be a self-reference; anything else named `Payload` is a genuine clash
  // we must not paper over.
  if (rootType === "Payload") {
    lines.push(`// (the root interface is already named \`Payload\`)`);
  } else if (/^export (?:interface|type) Payload\b/m.test(ts)) {
    throw new Error(`${schemaPath}: cannot alias Payload — the name is already taken`);
  } else {
    lines.push(`export type Payload = ${rootType};`);
  }
  lines.push("");

  if (responseType && responseType !== "Response" && /^export (?:interface|type) Response\b/m.test(ts)) {
    throw new Error(`${schemaPath}: cannot alias Response — the name is already taken`);
  }
  if (responseType) {
    lines.push(
      `/** Trust Task response type URI (request type URI + "#response"). */`,
      `export const RESPONSE_TYPE_URI = ${JSON.stringify(typeUri + "#response")} as const;`,
      "",
      `/** Stable alias for this specification's success-response payload shape. */`,
      ...(responseType === "Response"
        ? [`// (the response interface is already named \`Response\`)`]
        : [`export type Response = ${responseType};`]),
      "",
    );
  }

  if (policy) {
    const obj = (uri, isProofRequired, isRecipientRequired) =>
      [
        `{`,
        `  typeUri: ${uri},`,
        `  isBearer: ${policy.isBearer},`,
        `  isProofRequired: ${isProofRequired},`,
        `  isRecipientRequired: ${isRecipientRequired},`,
        `} as const;`,
      ].join("\n");

    lines.push(
      `/**`,
      ` * SPEC.md §7.2 policy for the request variant, from this specification's`,
      ` * front matter. Pass to \`consumeInbound\` — items 5b, 7 and 8 are`,
      ` * per-specification and cannot be derived from the document alone.`,
      ` */`,
      `export const SPEC = ${obj("TYPE_URI", policy.isProofRequired, policy.isRecipientRequired)}`,
      "",
    );

    if (responseType) {
      lines.push(
        `/**`,
        ` * SPEC.md §7.2 policy for the success-response variant. \`isRecipientRequired\``,
        ` * tracks the *issuer* party's requirement because a response swaps the`,
        ` * parties (§7.3 item 5).`,
        ` */`,
        `export const RESPONSE_SPEC = ${obj(
          "RESPONSE_TYPE_URI",
          policy.responseIsProofRequired,
          policy.responseIsRecipientRequired,
        )}`,
        "",
      );
    }
  }
  return lines.join("\n");
}

async function generateOne(schemaPath) {
  const outPath = relativeOutPath(schemaPath);
  await ensureDir(path.dirname(outPath));

  const opts = {
    cwd: path.dirname(schemaPath),
    bannerComment:
      "/**\n * Generated by scripts/build-ts-bindings.mjs — DO NOT EDIT BY HAND.\n * Source: " +
      path.relative(REPO_ROOT, schemaPath) +
      "\n */",
    additionalProperties: false,
    declareExternallyReferenced: true,
    enableConstEnums: false,
    strictIndexSignatures: true,
    unknownAny: true,
    style: { singleQuote: false, semi: true },
  };

  const raw = JSON.parse(await fs.readFile(schemaPath, "utf8"));
  const hasResponse = Boolean(raw?.$defs?.Response);

  let ts;
  let responseType = null;
  if (hasResponse) {
    const synthetic = {
      ...raw,
      properties: { ...raw.properties, [RESPONSE_PROBE]: responseProbeRef(raw.$defs.Response) },
    };
    ts = await compile(synthetic, path.basename(schemaPath), opts);
    ({ ts, responseType } = stripResponseProbe(ts, schemaPath));
  } else {
    ts = await compileFromFile(schemaPath, opts);
  }

  const slugInfo = slugFromSchemaPath(schemaPath);
  const policy = slugInfo ? readSpecPolicy(schemaPath) : null;
  if (slugInfo && !policy) {
    // Every task spec has front matter (the registry build enforces it). A
    // miss here would silently emit a module with no SPEC, and a consumer
    // would then have nothing to apply §7.2 items 5b/7/8 with.
    throw new Error(`${schemaPath}: could not read spec.md front matter for the §7.2 policy`);
  }
  const tail = emitTail(
    slugInfo,
    ts,
    rootTypeName(ts, schemaPath, raw),
    responseType,
    schemaPath,
    policy,
  );
  await fs.writeFile(outPath, ts + tail, "utf8");
  return { outPath, slugInfo };
}

async function emitIndex(generated) {
  // index.ts re-exports every generated module under a stable name.
  // For a payload schema at specs/vault/list/0.1/payload.schema.json the
  // export name is `VaultList_v0_1`.
  const lines = [
    "/** Generated by scripts/build-ts-bindings.mjs — DO NOT EDIT BY HAND. */",
    "",
    "// The hand-written §7.2 consumer pipeline. Re-exported flat (rather than",
    "// namespaced like the generated modules) because it is the framework API,",
    "// not one specification among many.",
    `export * from "./${RUNTIME_DIR}/index.js";`,
    "",
  ];
  for (const { outPath, slugInfo } of generated) {
    // `.js`, not extensionless. The package is ESM ("type": "module"), and Node
    // requires an explicit extension on a relative ESM specifier — it does not
    // probe for `.js` the way CommonJS resolution did. TypeScript's `Bundler`
    // moduleResolution accepts the extensionless form and emits it verbatim, so
    // `tsc` stayed quiet while `dist/index.js` shipped specifiers Node cannot
    // resolve: any `import … from "@openvtc/trust-tasks"` died with
    // ERR_MODULE_NOT_FOUND. It went unnoticed because the package was types
    // only until the runtime landed — `import type` is erased, and bundlers
    // tolerate extensionless paths. In TypeScript a `.js` specifier resolves to
    // the `.ts` source, so this is correct at both ends.
    const rel =
      "./" + path.relative(OUT_DIR, outPath).replace(/\\/g, "/").replace(/\.ts$/, ".js");
    if (slugInfo) {
      const id =
        slugInfo.slug
          .split("/")
          .map((s) =>
            s
              .split("-")
              .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
              .join(""),
          )
          .join("") + `_v${slugInfo.version.replace(/\./g, "_")}`;
      lines.push(`export * as ${id} from ${JSON.stringify(rel)};`);
    } else {
      // Shared module — flat re-export, version-qualified with the same
      // `_v<major>_<minor>` suffix the task exports above carry. Without
      // the version the alias is derived from the basename alone, so a
      // shared schema published at both 0.1 and 0.2 (device-binding,
      // policy, sync-event, the vault/* shapes, the framework schema)
      // emits the same identifier twice and breaks `tsc` with TS2300.
      const version = rel
        .replace(/^\.\//, "")
        .split("/")
        .find((s) => /^\d+\.\d+$/.test(s));
      const name = path
        .basename(outPath, ".ts")
        .split("-")
        .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
        .join("");
      const suffix = version ? `_v${version.replace(/\./g, "_")}` : "";
      lines.push(`export * as ${name}Shared${suffix} from ${JSON.stringify(rel)};`);
    }
  }

  // Fail here rather than downstream. The alias is derived from the basename
  // and version only, so it does NOT distinguish families: `config/_shared/
  // 0.1/config` and `vtc/_shared/0.1/config` both want `ConfigShared_v0_1`.
  // Emitting both is valid-looking output that breaks `tsc` with a TS2300 in
  // the generated `index.ts` — a diagnostic that points at the symptom and
  // says nothing about which two specs collided. Catch it at the source.
  const seen = new Map();
  const collisions = [];
  for (const line of lines) {
    const m = /^export \* as (\S+) from "(.+)";$/.exec(line);
    if (!m) continue;
    const [, alias, from] = m;
    if (seen.has(alias)) collisions.push(`${alias}: ${seen.get(alias)} vs ${from}`);
    else seen.set(alias, from);
  }
  if (collisions.length) {
    throw new Error(
      `duplicate TS export alias(es) — rename one of the colliding shared schemas ` +
        `(the alias comes from the file's basename + version, not its family):\n  ` +
        collisions.join("\n  "),
    );
  }

  await fs.writeFile(path.join(OUT_DIR, "index.ts"), lines.join("\n") + "\n", "utf8");
}

// Hand-written directories under src/ that the generator must not touch. The
// underscore prefix matches the `_shared` / `_framework` convention already
// used for non-task directories.
const HAND_WRITTEN = new Set([RUNTIME_DIR]);

async function clean() {
  // Wipe src/ so removed specs don't linger — but preserve the hand-written
  // runtime, which lives alongside the generated tree so that consumers get
  // types and the §7.2 consumer pipeline from one package (as trust-tasks-rs
  // does) rather than having to assemble them from two.
  await ensureDir(OUT_DIR);
  for (const entry of await fs.readdir(OUT_DIR)) {
    if (HAND_WRITTEN.has(entry)) continue;
    await fs.rm(path.join(OUT_DIR, entry), { recursive: true, force: true });
  }
}

async function main() {
  console.log("Trust Tasks TS bindings generator");
  await clean();

  const schemas = await walk(SPECS_DIR, /\.schema\.json$/);
  schemas.sort();

  // Skip two schemas that describe something other than a payload shape:
  //
  //   spec.meta.schema.json — front-matter metadata for spec authors.
  //   trust-task.schema.json — the document envelope (SPEC §4.2). Generating
  //     from it would put a second document type in the package, competing with
  //     the hand-written `TrustTaskDocument<P>` in src/_runtime. The generated
  //     one would be strictly worse — not generic over the payload, so unusable
  //     with consumeInbound — and having both invites picking the wrong one.
  const NOT_PAYLOAD_SCHEMAS = new Set(["spec.meta.schema.json", "trust-task.schema.json"]);
  const filtered = schemas.filter((p) => !NOT_PAYLOAD_SCHEMAS.has(path.basename(p)));

  console.log(`Generating bindings for ${filtered.length} schemas...`);
  const generated = [];
  for (const schemaPath of filtered) {
    try {
      const result = await generateOne(schemaPath);
      generated.push(result);
      console.log("  ✓ " + path.relative(REPO_ROOT, schemaPath));
    } catch (err) {
      console.error("  ✗ " + path.relative(REPO_ROOT, schemaPath));
      console.error("    " + err.message);
      process.exitCode = 1;
    }
  }

  await emitIndex(generated);
  console.log(`\nGenerated ${generated.length} TS modules into ${path.relative(REPO_ROOT, OUT_DIR)}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
