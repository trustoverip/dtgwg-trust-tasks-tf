#!/usr/bin/env node
// API compatibility check for @openvtc/trust-tasks against its last npm release.
//
//   PR_TITLE='feat(ts)!: …' node scripts/api-compat.mjs [--baseline <version>]
//                                                        [--baseline-dir <dir>]
//
// Rust has cargo-semver-checks; this is the TypeScript side of the same guard,
// run on every pull request (the `ts API compatibility` job in ts.yml) so an
// API break is caught when it is made rather than shipped as a patch.
//
// HOW
//
// 1. The PR's declarations are emitted into a FRESH directory (`tsc
//    --emitDeclarationOnly --outDir <tmp>`), never the working tree's `dist/`,
//    which tsc does not prune: a module deleted from `src/` would otherwise
//    still be there and the removal would go unseen. The baseline is the
//    published tarball (`npm pack @openvtc/trust-tasks@<latest>`), i.e. exactly
//    what consumers have.
//
// 2. The package's `exports` map has a `./*` entry, so EVERY declaration file
//    is a public entry point, not just `index.d.ts`. api-extractor analyses one
//    entry point, so for each side we write a synthetic one that does
//    `export * as M_<path> from "./<path>.js"` for every .d.ts in the tree
//    (index.d.ts included — its namespace names are API too). Removing a file
//    therefore removes a namespace, which is reported.
//
// 3. api-extractor (pinned in devDependencies) turns each side into its doc
//    model (.api.json): every exported declaration and member, keyed by a
//    stable canonical reference, with its signature text. Doc comments are
//    dropped. The two models are then diffed item by item:
//
//    BREAKING  - an item in the release is missing from the PR (removed or
//                renamed export, namespace, member, overload);
//              - an item's signature text changed (type, optionality,
//                readonly, parameters, heritage…), except as below;
//              - a REQUIRED property or method was added to an existing
//                interface (a consumer's object literal no longer satisfies
//                it).
//    COMPATIBLE - anything new (a new module/spec version, a new export, a
//                new optional member);
//              - a string-literal union that only GAINED members
//                (`"a" | "b"` → `"a" | "b" | "c"`) or was reordered. This is
//                deliberate policy, the TypeScript counterpart of Rust's
//                `#[non_exhaustive]`: a spec enum gaining a value is an
//                additive spec change (check-published-schemas allows it in
//                place) and stays a patch. A union LOSING or renaming a member
//                is still breaking. Strictly, a widened union can break an
//                exhaustive `switch` with a `never` default; consumers are
//                expected to keep a default arm, as Rust's are made to.
//                The same holds for a readonly tuple of string literals that
//                was only APPENDED to (`STANDARD_CODES`, from which
//                `StandardCode` is derived); inserting, reordering or dropping
//                an element is breaking. Implemented textually: maximal runs
//                of string literals joined by `|` (or a whole `[…]` tuple of
//                them) are lifted out of both signatures, the remaining
//                skeletons must be identical, and each run is compared as a
//                set (union: old ⊆ new) or a sequence (tuple: old is a prefix).
//              - the literal TYPE of an embedded JSON Schema document (the
//                generated `PAYLOAD_SCHEMA`, `RESPONSE_PAYLOAD_SCHEMA` and
//                `SPEC.payloadSchema` constants are `as const`, so every
//                description edit changes their type). Those are schema data,
//                and their compatibility is what `check-published-schemas`
//                judges, with JSON-Schema-aware rules. The constants'
//                existence, and SPEC's policy flags (`isProofRequired`, …),
//                are still compared.
//
// 4. Verdict: no break → pass. Breaks and a PR title whose `!` scope list
//    includes `ts` → pass, listing them. Breaks without it → fail.
//
// The title is read from the PR_TITLE environment variable only.

import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PKG_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PKG_NAME = "@openvtc/trust-tasks";
const SCOPE = "ts";
const ENTRY = "__api_compat_entry__.d.ts";

// ── extraction (run in a child process per side: ~5 GB peak each) ─────────

function listDts(dir) {
  const out = [];
  (function walk(d) {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith(".d.ts") && e.name !== ENTRY) out.push(path.relative(dir, p).split(path.sep).join("/"));
    }
  })(dir);
  return out.sort();
}

function extract(pkgDir, outFile) {
  // api-extractor is CommonJS; createRequire keeps extract() synchronous.
  const { Extractor, ExtractorConfig } = createRequire(path.join(PKG_DIR, "package.json"))("@microsoft/api-extractor");
  const dist = path.join(pkgDir, "dist");
  const seen = new Map();
  const lines = listDts(dist).map((f) => {
    const ns = "M_" + f.replace(/\.d\.ts$/, "").replace(/[^A-Za-z0-9]/g, "_");
    if (seen.has(ns)) throw new Error(`namespace collision: ${f} and ${seen.get(ns)} both map to ${ns}`);
    seen.set(ns, f);
    return `export * as ${ns} from "./${f.replace(/\.d\.ts$/, ".js")}";`;
  });
  const entry = path.join(dist, ENTRY);
  fs.writeFileSync(entry, lines.join("\n") + "\n");
  const apiJson = outFile + ".raw.json";
  const config = ExtractorConfig.prepare({
    configObjectFullPath: path.join(pkgDir, "api-extractor.json"), // virtual; never read
    packageJsonFullPath: path.join(pkgDir, "package.json"),
    configObject: {
      projectFolder: pkgDir,
      mainEntryPointFilePath: entry,
      compiler: {
        overrideTsconfig: {
          compilerOptions: {
            target: "ES2022", module: "ESNext", moduleResolution: "Bundler",
            lib: ["ES2022"], strict: true, skipLibCheck: true, types: [],
          },
          include: [entry],
        },
      },
      apiReport: { enabled: false },
      docModel: { enabled: true, apiJsonFilePath: apiJson },
      dtsRollup: { enabled: false },
      tsdocMetadata: { enabled: false },
      messages: {
        compilerMessageReporting: { default: { logLevel: "warning" } },
        extractorMessageReporting: { default: { logLevel: "none" } },
        tsdocMessageReporting: { default: { logLevel: "none" } },
      },
    },
  });
  let forgotten = 0;
  const result = Extractor.invoke(config, {
    localBuild: true,
    messageCallback(m) {
      // A referenced-but-unexported declaration is invisible to the diff
      // (its changes would show nowhere), so count them and say so.
      if (m.messageId === "ae-forgotten-export") forgotten++;
      if (m.category !== "Compiler") m.handled = true;
      if (m.messageId === "console-compiler-version-notice") m.handled = true;
    },
  });
  if (!result.succeeded) throw new Error(`api-extractor failed on ${pkgDir}`);

  // Flatten to { canonicalReference: {kind, text, optional, parent} }, docs dropped.
  const model = JSON.parse(fs.readFileSync(apiJson, "utf8"));
  fs.rmSync(apiJson);
  const items = {};
  (function walk(node, parent) {
    for (const m of node.members ?? []) {
      const text = (m.excerptTokens ?? []).map((t) => t.text).join("").replace(/\s+/g, " ").trim();
      items[m.canonicalReference] = { kind: m.kind, text, optional: !!m.isOptional, parent };
      walk(m, m.canonicalReference);
    }
  })(model.members[0], null);
  fs.writeFileSync(outFile, JSON.stringify({ forgotten, items }));
}

// ── comparison ───────────────────────────────────────────────────────────

const STR = String.raw`"(?:[^"\\]|\\.)*"`;
const LITERAL_UNION = new RegExp(`${STR}(?:\\s*\\|\\s*${STR})*`, "g");
const LITERAL_TUPLE = new RegExp(`\\[\\s*${STR}(?:\\s*,\\s*${STR})*\\s*\\]`, "g");

/** Replace every embedded JSON Schema document (`{ readonly $schema: … }`) with a marker. */
export function maskSchemas(text) {
  let out = text;
  for (;;) {
    const at = out.search(/\{\s*(readonly )?\$schema:/);
    if (at < 0) return out;
    let depth = 0, i = at, inStr = false;
    for (; i < out.length; i++) {
      const c = out[i];
      if (inStr) { if (c === "\\") i++; else if (c === '"') inStr = false; continue; }
      if (c === '"') inStr = true;
      else if (c === "{") depth++;
      else if (c === "}" && --depth === 0) break;
    }
    out = out.slice(0, at) + "<json-schema>" + out.slice(i + 1);
  }
}

/**
 * Compare two signature texts. Returns "same", "widened" (only string-literal
 * unions gained members — compatible by policy) or "changed".
 */
export function compareText(a, b) {
  if (a === b) return "same";
  // Tuples of string literals first (`readonly ["a", "b"]`, e.g. the
  // STANDARD_CODES that StandardCode is derived from): compatible when the old
  // elements are a PREFIX of the new — appended, not inserted or reordered.
  // Then unions of string literals: compatible when the old members are a
  // SUBSET of the new (order is not significant in a union).
  const split = (t) => {
    const runs = [];
    const lits = (run) => run.match(new RegExp(STR, "g"));
    const skeleton = t
      .replace(LITERAL_TUPLE, (run) => (runs.push({ tuple: lits(run) }), "\u0000"))
      .replace(LITERAL_UNION, (run) => (runs.push({ union: new Set(lits(run)) }), "\u0001"));
    return { skeleton, runs };
  };
  const x = split(a), y = split(b);
  if (x.skeleton !== y.skeleton || x.runs.length !== y.runs.length) return "changed";
  let widened = false;
  for (let i = 0; i < x.runs.length; i++) {
    const o = x.runs[i], n = y.runs[i];
    if (o.tuple) {
      if (!n.tuple || o.tuple.some((lit, j) => n.tuple[j] !== lit)) return "changed";
      if (n.tuple.length > o.tuple.length) widened = true;
    } else {
      if (!n.union) return "changed";
      for (const lit of o.union) if (!n.union.has(lit)) return "changed";
      if (n.union.size > o.union.size) widened = true;
    }
  }
  return widened ? "widened" : "same";
}

export function diff(oldItems, newItems) {
  const breaks = [], widened = [];
  let added = 0;
  for (const [ref, o] of Object.entries(oldItems)) {
    const n = newItems[ref];
    if (!n) {
      // Report only the outermost removal, not every member under it.
      if (o.parent === null || newItems[o.parent]) breaks.push(`removed: ${ref}`);
      continue;
    }
    const verdict = n.kind !== o.kind ? "changed" : compareText(maskSchemas(o.text), maskSchemas(n.text));
    if (verdict === "changed") breaks.push(`changed: ${ref}\n    was: ${o.text}\n    now: ${n.text}`);
    else if (verdict === "widened") widened.push(ref);
  }
  for (const [ref, n] of Object.entries(newItems)) {
    if (oldItems[ref]) continue;
    added++;
    const parent = n.parent && oldItems[n.parent];
    if (parent && parent.kind === "Interface" && !n.optional &&
        ["PropertySignature", "MethodSignature", "IndexSignature", "CallSignature", "ConstructSignature"].includes(n.kind)) {
      breaks.push(`required member added to an existing interface: ${ref}\n    ${n.text}`);
    }
  }
  return { breaks, widened, added };
}

/** Does a conventional-commit title declare a break for `scope`? */
export function declaresBreak(title, scope) {
  const m = /^[a-z]+\(([^)]*)\)!:/.exec(title ?? "");
  return !!m && m[1].split(",").map((s) => s.trim()).includes(scope);
}

// ── driver ───────────────────────────────────────────────────────────────

function runExtract(pkgDir, outFile) {
  const r = spawnSync(process.execPath, ["--max-old-space-size=8192", fileURLToPath(import.meta.url), "--extract", pkgDir, outFile], { stdio: "inherit" });
  if (r.status !== 0) throw new Error(`extraction failed for ${pkgDir}`);
  return JSON.parse(fs.readFileSync(outFile, "utf8"));
}

function main(argv) {
  const arg = (name) => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : undefined; };
  const work = fs.mkdtempSync(path.join(os.tmpdir(), "tt-api-compat-"));
  const t0 = Date.now();
  try {
    // Baseline: the published package.
    let baseDir = arg("--baseline-dir");
    let version = arg("--baseline");
    if (!baseDir) {
      version ??= execFileSync("npm", ["view", PKG_NAME, "version"], { encoding: "utf8" }).trim();
      const tgz = execFileSync("npm", ["pack", `${PKG_NAME}@${version}`, "--silent", "--pack-destination", work], { encoding: "utf8" }).trim().split("\n").pop();
      execFileSync("tar", ["xzf", path.join(work, tgz), "-C", work]);
      baseDir = path.join(work, "package");
    }
    // PR: declarations only, into a fresh tree (see header, point 1).
    const prDir = path.join(work, "pr");
    fs.mkdirSync(prDir);
    fs.copyFileSync(path.join(PKG_DIR, "package.json"), path.join(prDir, "package.json"));
    execFileSync(process.execPath, [path.join(PKG_DIR, "node_modules/typescript/bin/tsc"), "-p", path.join(PKG_DIR, "tsconfig.json"),
      "--emitDeclarationOnly", "--declarationMap", "false", "--sourceMap", "false", "--outDir", path.join(prDir, "dist")], { stdio: "inherit" });

    const base = runExtract(baseDir, path.join(work, "base.json"));
    const pr = runExtract(prDir, path.join(work, "pr.json"));
    const { breaks, widened, added } = diff(base.items, pr.items);
    const secs = ((Date.now() - t0) / 1000).toFixed(0);
    const label = `${PKG_NAME}@${version ?? JSON.parse(fs.readFileSync(path.join(baseDir, "package.json"), "utf8")).version}`;
    for (const [side, m] of [["release", base], ["PR", pr]])
      if (m.forgotten) console.log(`note: ${m.forgotten} declaration(s) in the ${side} are referenced but not exported; changes to them are not visible to this check`);
    if (widened.length) console.log(`${widened.length} string-literal union(s)/tuple(s) gained members (compatible by policy):\n  ${widened.join("\n  ")}`);

    if (breaks.length === 0) {
      console.log(`ok: API compatible with ${label} — ${added} item(s) added, ${widened.length} literal union(s)/tuple(s) widened, none removed or changed (${secs}s)`);
      return 0;
    }
    const list = breaks.map((b) => `  - ${b}`).join("\n");
    if (declaresBreak(process.env.PR_TITLE, SCOPE)) {
      console.log(`ok: ${breaks.length} incompatible change(s) against ${label}, declared by the PR title's \`${SCOPE}\` scope with \`!\`:\n${list}`);
      return 0;
    }
    console.log(`::error::${breaks.length} incompatible API change(s) against ${label}, and the PR title does not declare a breaking change for \`${SCOPE}\``);
    console.log(`${list}\n
Either make the change compatible, or retitle the PR with \`feat(${SCOPE})!: …\`
(a \`!\` whose scope list includes \`${SCOPE}\`) so the release is sized as breaking.
See RELEASING.md.`);
    return 1;
  } finally {
    fs.rmSync(work, { recursive: true, force: true });
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === "--extract") extract(process.argv[3], process.argv[4]);
  else process.exitCode = main(process.argv.slice(2));
}
