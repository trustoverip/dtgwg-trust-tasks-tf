#!/usr/bin/env node
// The gate that stops GETTING-STARTED.md's TypeScript, Go and Dart sections
// rotting — the counterpart of trust-tasks/tests/getting_started_snippets.rs,
// which does the same for the Rust section.
//
//   node scripts/check-getting-started-snippets.mjs examples/getting-started-ts/src/main.ts
//
// The document's code is not written by hand: each snippet is a region of an
// example application that CI generates, builds and runs (the `consumer-app`
// jobs in ts.yml, go.yml and dart.yml). This asserts the two are identical, so
// a change to the example that is not carried into the document fails the
// build rather than shipping a snippet that no longer compiles.
//
// In the document a snippet is introduced by
// `<!-- snippet: <example path>#<region> -->` immediately followed by a fence
// in the example's language. In the example a region is the lines between
// `// GETTING-STARTED:begin <region>` and `// GETTING-STARTED:end <region>`;
// blank lines at either end of a region are not part of it (Go needs one after
// the begin marker, or the marker becomes the next declaration's doc comment).
//
// Plain Node, no dependencies: the Go and Dart jobs run it on the runner's
// preinstalled Node.

import { readFileSync } from "node:fs";
import { dirname, extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const DOC_REL = "GETTING-STARTED.md";
const FENCE = { ".ts": "ts", ".go": "go", ".dart": "dart", ".rs": "rust" };

const arg = process.argv[2];
if (arg === undefined) {
  console.error("usage: check-getting-started-snippets.mjs <example source file, relative to the repo root>");
  process.exit(2);
}
const exampleRel = relative(ROOT, join(ROOT, arg)).split("\\").join("/");
const fence = FENCE[extname(exampleRel)];
if (fence === undefined) {
  console.error(`${exampleRel}: no fence language known for ${extname(exampleRel)}`);
  process.exit(2);
}

const example = readFileSync(join(ROOT, exampleRel), "utf8");
const doc = readFileSync(join(ROOT, DOC_REL), "utf8");
const failures = [];

/** Drop blank lines from both ends of a list of lines, and join them. */
function trimBlank(lines) {
  let start = 0;
  let end = lines.length;
  while (start < end && lines[start].trim() === "") start++;
  while (end > start && lines[end - 1].trim() === "") end--;
  return lines.slice(start, end).join("\n");
}

/** Every region the example declares, with its code. */
function regionsOf(source) {
  const regions = new Map();
  let open = null;
  let body = [];
  for (const line of source.split("\n")) {
    const t = line.trim();
    const begin = /^\/\/ GETTING-STARTED:begin (\S+)$/.exec(t);
    const end = /^\/\/ GETTING-STARTED:end (\S+)$/.exec(t);
    if (begin) {
      if (open !== null) failures.push(`region \`${begin[1]}\` opens inside \`${open}\` in ${exampleRel}`);
      if (regions.has(begin[1])) failures.push(`region \`${begin[1]}\` is declared twice in ${exampleRel}`);
      open = begin[1];
      body = [];
    } else if (end) {
      if (end[1] !== open) {
        failures.push(`region \`${end[1]}\` closes without being open in ${exampleRel}`);
        continue;
      }
      regions.set(open, trimBlank(body));
      open = null;
    } else if (open !== null) {
      body.push(line);
    }
  }
  if (open !== null) failures.push(`region \`${open}\` is never closed in ${exampleRel}`);
  return regions;
}

/** Every `[region, code]` the document claims to have copied from the example. */
function snippetsOf(text) {
  const out = [];
  const lines = text.split("\n");
  const prefix = `<!-- snippet: ${exampleRel}#`;
  for (let i = 0; i < lines.length; i++) {
    const t = lines[i].trim();
    if (!t.startsWith(prefix)) continue;
    const region = t.slice(prefix.length).replace(/-->$/, "").trim();
    if ((lines[i + 1] ?? "").trim() !== "```" + fence) {
      failures.push(`${DOC_REL}:${i + 1}: snippet \`${region}\` must be followed immediately by a \`\`\`${fence} fence`);
      continue;
    }
    const code = [];
    let j = i + 2;
    for (; j < lines.length && lines[j].trimEnd() !== "```"; j++) code.push(lines[j]);
    out.push([region, trimBlank(code), i + 1]);
    i = j;
  }
  return out;
}

const regions = regionsOf(example);
const snippets = snippetsOf(doc);
if (snippets.length === 0) {
  failures.push(
    `${DOC_REL} embeds no \`<!-- snippet: ${exampleRel}#… -->\` blocks — the markers were dropped, ` +
      "or the code was inlined by hand, which is what this check exists to prevent",
  );
}

for (const [region, code, line] of snippets) {
  const inExample = regions.get(region);
  if (inExample === undefined) {
    failures.push(`${DOC_REL}:${line}: snippet \`${region}\` names a region ${exampleRel} does not declare`);
  } else if (code !== inExample) {
    const a = code.split("\n");
    const b = inExample.split("\n");
    let k = 0;
    while (k < a.length && k < b.length && a[k] === b[k]) k++;
    failures.push(
      `${DOC_REL}:${line}: snippet \`${region}\` has drifted from ${exampleRel} ` +
        `(first difference at line ${k + 1} of the region:\n    doc:     ${JSON.stringify(a[k] ?? "<end>")}\n` +
        `    example: ${JSON.stringify(b[k] ?? "<end>")}).\n` +
        `  Re-copy the lines between the \`GETTING-STARTED:begin/end ${region}\` markers into the fence.`,
    );
  }
}

// The reverse direction: a region the document never embeds is a snippet
// nobody reads, and a renamed one would otherwise silently stop being checked.
const embedded = new Set(snippets.map(([r]) => r));
for (const region of regions.keys()) {
  if (!embedded.has(region)) failures.push(`region \`${region}\` of ${exampleRel} is not embedded in ${DOC_REL}`);
}

if (failures.length > 0) {
  console.error(failures.map((f) => `- ${f}`).join("\n"));
  process.exit(1);
}
console.log(`${DOC_REL}: ${snippets.length} snippet(s) match ${exampleRel}`);
