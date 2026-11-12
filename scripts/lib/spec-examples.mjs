/**
 * The spec-examples lint (#1131).
 *
 * README.md and CONTRIBUTING-SPECS.md require every specification to carry "at
 * least one complete example", and the example is what two things lean on: an
 * implementer copies it, and trust-tasks-codegen round-trips it through the
 * generated Rust types as the spec's conformance test. A spec with no example
 * gets neither — and before this lint 136 current specs had none, which nothing
 * reported. `checkExampleDocuments` in build-registry.mjs validates every
 * example it *finds* against the framework envelope; it never asked for one,
 * and never looked inside the payload. This does both.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * The LATEST version of every slug, unless that version is `retired` or
 * `withdrawn` (a retired spec is frozen — SPEC §5.3 — and is not owed new
 * content). In its spec.md, every fenced ```json block that parses and whose
 * `type` is the spec's own Type URI is an example of it, wherever it sits —
 * not only under a `## Request` / `## Response` heading. Each is one of:
 *
 *   request   `type` is the bare Type URI or `<uri>#request`; its `payload`
 *             must validate against the payload schema.
 *   response  `type` is `<uri>#response`; its `payload` must validate against
 *             the schema's `$defs.Response`.
 *
 * The rules, each named by the id the allowlist uses:
 *
 *   missing-request    no request example.
 *   missing-response   the schema has `$defs.Response` and there is no
 *                      response example.
 *   invalid-request    a request example's payload fails the payload schema.
 *   invalid-response   a response example's payload fails `$defs.Response`
 *                      (or there is a response example and no Response).
 *
 * Payloads are validated with the schema's cross-file `$ref`s resolved on
 * disk: every `*.schema.json` under specs/ is registered under its file URL,
 * since the published `$id`s are not where the relative `$ref`s point.
 *
 * ── The baseline (spec-examples-allowlist.json) ─────────────────────────────
 *
 * `exceptions`, keyed by `<slug>/<version>` — the VERSION, not the slug, so a
 * new version of an allowlisted spec is a new spec and owes its examples —
 * then by rule id, each mapped to the reason. It is a ratchet: it may only
 * shrink. A finding not listed fails the build; a listed entry that no longer
 * applies (the spec gained its example, or was superseded) also fails, with
 * the line to delete, so the baseline cannot keep a hole open behind a fix.
 */
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import Ajv from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

export const RULES = ['missing-request', 'missing-response', 'invalid-request', 'invalid-response'];

const compareVersions = (a, b) => {
  const [a1, a2] = a.split('.').map(Number);
  const [b1, b2] = b.split('.').map(Number);
  return a1 - b1 || a2 - b2;
};

/** Every `{ slug, version, dir }` that is the latest version of its slug and has a spec.md. */
function latestSpecs(specsDir) {
  const latest = new Map();
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory() || entry.name.startsWith('_') || entry.name.startsWith('.')) continue;
      const full = path.join(dir, entry.name);
      if (/^\d+\.\d+$/.test(entry.name)) {
        if (!fs.existsSync(path.join(full, 'spec.md')) || !fs.existsSync(path.join(full, 'payload.schema.json'))) continue;
        const slug = path.relative(specsDir, dir).split(path.sep).join('/');
        if (!latest.has(slug) || compareVersions(entry.name, latest.get(slug)) > 0) latest.set(slug, entry.name);
      } else {
        walk(full);
      }
    }
  })(specsDir);
  return [...latest]
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([slug, version]) => ({ slug, version, dir: path.join(specsDir, slug, version) }));
}

function statusOf(specMd) {
  const fm = specMd.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
  return fm.match(/^status:\s*["']?([a-z]+)/m)?.[1] ?? null;
}

/**
 * Every fenced ```json block in `text` that parses, as a value. A block may be
 * indented (inside a list item) or quoted (`> ` on every line).
 */
export function jsonBlocks(text) {
  const out = [];
  const lines = text.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const open = /^([ \t>]*)```json\s*$/i.exec(lines[i]);
    if (!open) continue;
    const prefix = open[1];
    const body = [];
    let j = i + 1;
    for (; j < lines.length; j++) {
      if (/^[ \t>]*```\s*$/.test(lines[j])) break;
      let line = lines[j];
      if (prefix.includes('>')) line = line.replace(/^[ \t]*>[ \t]?/, '');
      body.push(line);
    }
    i = j;
    try {
      out.push(JSON.parse(body.join('\n')));
    } catch {
      // An illustrative fragment (`…`, a comment) — not an example of anything.
    }
  }
  return out;
}

/** One Ajv with every `*.schema.json` under `specsDir` registered under its file URL. */
function schemaRegistry(specsDir) {
  const ajv = new Ajv({ allErrors: true, strict: false, logger: false });
  addFormats(ajv);
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith('.schema.json') && dir !== specsDir) {
        let schema;
        try {
          schema = JSON.parse(fs.readFileSync(full, 'utf8'));
        } catch {
          continue; // reported by the schema checks
        }
        // The published `$id` names the registry URL, against which the
        // relative on-disk `$ref`s do not resolve; the file URL does.
        ajv.addSchema({ ...schema, $id: pathToFileURL(full).href });
      }
    }
  })(specsDir);
  return ajv;
}

const describe = (errors) =>
  (errors || [])
    .slice(0, 3)
    .map((e) => `${e.instancePath || '/'} ${e.message}`)
    .join('; ');

/**
 * Every `{ key, rule, detail }` the latest live specs under `specsDir` break,
 * plus how many specs were checked.
 */
export function exampleFindings(specsDir) {
  const ajv = schemaRegistry(specsDir);
  const findings = [];
  let checked = 0;
  for (const { slug, version, dir } of latestSpecs(specsDir)) {
    const md = fs.readFileSync(path.join(dir, 'spec.md'), 'utf8');
    const status = statusOf(md);
    if (status === 'retired' || status === 'withdrawn') continue;
    checked++;
    const key = `${slug}/${version}`;
    const schemaPath = path.join(dir, 'payload.schema.json');
    let schema;
    try {
      schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
    } catch {
      continue; // reported by the payload-schema check
    }
    const hasResponse = Boolean(schema.$defs?.Response);
    const uri = `https://trusttasks.org/spec/${slug}/${version}`;
    const docs = jsonBlocks(md).filter((d) => d && typeof d === 'object' && !Array.isArray(d));
    const requests = docs.filter((d) => d.type === uri || d.type === `${uri}#request`);
    const responses = docs.filter((d) => d.type === `${uri}#response`);

    const compile = (fragment) => {
      try {
        return ajv.getSchema(pathToFileURL(schemaPath).href + fragment) ?? null;
      } catch (e) {
        return { compileError: e.message };
      }
    };
    const check = (examples, rule, fragment) => {
      if (examples.length === 0) return;
      const validate = compile(fragment);
      if (!validate || validate.compileError) {
        findings.push({ key, rule, detail: `the schema did not compile: ${validate?.compileError ?? 'not found'}` });
        return;
      }
      examples.forEach((doc, i) => {
        if (!validate(doc.payload)) {
          findings.push({ key, rule, detail: `example ${i + 1}: ${describe(validate.errors)}` });
        }
      });
    };

    if (requests.length === 0) findings.push({ key, rule: 'missing-request', detail: `no example document with type ${uri}` });
    if (hasResponse && responses.length === 0) {
      findings.push({ key, rule: 'missing-response', detail: `no example document with type ${uri}#response` });
    }
    check(requests, 'invalid-request', '');
    if (hasResponse) check(responses, 'invalid-response', '#/$defs/Response');
    else if (responses.length) {
      findings.push({ key, rule: 'invalid-response', detail: 'a #response example, but the schema defines no $defs.Response' });
    }
  }
  return { findings, checked };
}

export function checkSpecExamples({ specsDir, allowlistPath, fail, warn, log }) {
  let allowlist;
  try {
    allowlist = JSON.parse(fs.readFileSync(allowlistPath, 'utf8')).exceptions ?? {};
  } catch (e) {
    // Never fail open: a lint that cannot read its baseline checks nothing.
    fail(path.relative(path.dirname(specsDir), allowlistPath), `could not be read: ${e.message}`);
    return;
  }
  const { findings, checked } = exampleFindings(specsDir);
  const found = new Set();
  const counts = Object.fromEntries(RULES.map((r) => [r, 0]));
  let baselined = 0;
  for (const f of findings) {
    const id = `${f.key}\u0000${f.rule}`;
    if (found.has(id)) {
      continue; // one finding per spec and rule is enough to fail or to baseline
    }
    found.add(id);
    if (allowlist[f.key]?.[f.rule]) {
      baselined++;
      continue;
    }
    counts[f.rule]++;
    const how =
      f.rule === 'missing-request' || f.rule === 'missing-response'
        ? `Add a complete example document to spec.md (a fenced \`\`\`json block whose \`type\` is the spec's own Type URI` +
          `${f.rule === 'missing-response' ? ' with #response' : ''}) — CONTRIBUTING-SPECS.md "Examples".`
        : `Correct the example so its payload validates against the spec's own schema.`;
    fail(`specs/${f.key}/spec.md`, `${f.rule}: ${f.detail}. ${how}`);
  }
  for (const [key, rules] of Object.entries(allowlist)) {
    for (const rule of Object.keys(rules)) {
      if (!found.has(`${key}\u0000${rule}`)) {
        fail(
          'scripts/lib/spec-examples-allowlist.json',
          `"${rule}" for ${key} no longer applies (the spec now passes it, or is no longer the latest live version) — ` +
            `delete the entry: the baseline only shrinks`
        );
      }
    }
  }
  const offending = Object.values(counts).reduce((a, b) => a + b, 0);
  log(
    `  Spec examples: ${checked} latest live spec(s) checked; ${baselined} baselined finding(s) ` +
      `(scripts/lib/spec-examples-allowlist.json — may only shrink), ${offending} new` +
      `${offending ? ` (${RULES.filter((r) => counts[r]).map((r) => `${counts[r]} ${r}`).join(', ')})` : ''}`
  );
  void warn;
}
