#!/usr/bin/env node
/**
 * Trust Tasks registry build.
 *
 *  - Walks specs/<slug>/<version>/spec.md
 *  - Parses YAML front matter, validates it against specs/spec.meta.schema.json
 *  - Loads sibling payload.schema.json, sanity-checks $id / $schema
 *  - Computes 'updated' from the most recent git commit touching the spec folder
 *  - Emits website/assets/tasks.generated.js (window.TT_TASKS = [...])
 *  - Copies specs/ tree into website/specs/ so the SPA can fetch prose + schema
 *
 * Run from the repo root: `npm run build` or `npm run validate` (no website writes).
 */
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import Ajv from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS_DIR = path.join(ROOT, 'specs');
const WEBSITE_DIR = path.join(ROOT, 'website');
const META_SCHEMA_PATH = path.join(SPECS_DIR, 'spec.meta.schema.json');

const validateOnly = process.argv.includes('--validate-only');

const errors = [];
const warn = (msg) => console.warn(`  warn: ${msg}`);
const fail = (loc, msg) => errors.push(`${loc}: ${msg}`);

function readJson(p) {
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function splitFrontMatter(src) {
  // Expect exactly: "---\n<yaml>\n---\n<body>"
  if (!src.startsWith('---')) return { data: null, body: src };
  const end = src.indexOf('\n---', 3);
  if (end < 0) return { data: null, body: src };
  const yamlBlock = src.slice(3, end).replace(/^\r?\n/, '');
  const body = src.slice(end + 4).replace(/^\r?\n/, '');
  return { data: YAML.parse(yamlBlock), body };
}

function lastModified(dirRel) {
  try {
    const iso = execSync(
      `git log -1 --format=%cI -- "${dirRel}"`,
      { cwd: ROOT, encoding: 'utf8' }
    ).trim();
    if (iso) return iso.slice(0, 10); // YYYY-MM-DD
  } catch {
    /* ignore — fall through */
  }
  return new Date().toISOString().slice(0, 10);
}

function discoverSpecs() {
  if (!fs.existsSync(SPECS_DIR)) return [];
  const found = [];
  walk(SPECS_DIR);
  return found;

  function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      if (entry.name.startsWith('_') || entry.name.startsWith('.')) continue;
      const full = path.join(dir, entry.name);
      const specPath = path.join(full, 'spec.md');
      if (fs.existsSync(specPath)) {
        // `full` is a version directory (it contains spec.md). Slug is the
        // relative path from SPECS_DIR to `full`'s parent, with `/` separators.
        const relVersionDir = path.relative(SPECS_DIR, full);
        const segments = relVersionDir.split(path.sep);
        const version = segments[segments.length - 1];
        const slug = segments.slice(0, -1).join('/');
        found.push({ slug, version, dir: full, specPath });
      } else {
        walk(full);
      }
    }
  }
}

function loadMetaValidator() {
  const ajv = new Ajv({ allErrors: true, strict: false });
  addFormats(ajv);
  return ajv.compile(readJson(META_SCHEMA_PATH));
}

function checkPayloadSchema(slug, version, dir) {
  const schemaPath = path.join(dir, 'payload.schema.json');
  if (!fs.existsSync(schemaPath)) {
    fail(`${slug}/${version}`, 'missing payload.schema.json');
    return null;
  }
  let schema;
  try {
    schema = readJson(schemaPath);
  } catch (e) {
    fail(`${slug}/${version}/payload.schema.json`, `invalid JSON: ${e.message}`);
    return null;
  }
  const expectedId = `https://trusttasks.org/spec/${slug}/${version}`;
  if (schema.$id !== expectedId) {
    fail(`${slug}/${version}/payload.schema.json`, `$id must be ${expectedId} (got ${schema.$id ?? 'undefined'})`);
  }
  if (schema.$schema !== 'https://json-schema.org/draft/2020-12/schema') {
    fail(`${slug}/${version}/payload.schema.json`, `$schema must be JSON Schema 2020-12`);
  }
  if (!('additionalProperties' in schema)) {
    warn(`${slug}/${version}/payload.schema.json: no additionalProperties declared (SPEC §6.3 requires explicit handling)`);
  }
  // SPEC §7.3 item 7.6: if a response sub-schema exists, it MUST live in $defs.Response
  // and declare $anchor: "response".
  const respDef = schema.$defs && schema.$defs.Response;
  if (respDef) {
    if (respDef.$anchor !== 'response') {
      fail(`${slug}/${version}/payload.schema.json`, `$defs.Response must declare $anchor: "response" (got ${JSON.stringify(respDef.$anchor)})`);
    }
    if (!('additionalProperties' in respDef)) {
      warn(`${slug}/${version}/payload.schema.json: $defs.Response has no additionalProperties declared`);
    }
  }
  // Belt-and-braces: nothing other than Response/Request/well-known $defs should declare a `response` anchor.
  for (const [k, v] of Object.entries(schema.$defs || {})) {
    if (k === 'Response') continue;
    if (v && v.$anchor === 'response') {
      fail(`${slug}/${version}/payload.schema.json`, `only $defs.Response may use $anchor: "response" (found on $defs.${k})`);
    }
  }
  return schema;
}

function buildTask(entry, meta, schema) {
  const hasResponse = !!(schema && schema.$defs && schema.$defs.Response);
  return {
    id: meta.slug,
    slug: meta.slug,
    title: meta.title,
    summary: meta.summary,
    category: meta.category,
    keywords: meta.keywords,
    status: meta.status,
    version: meta.version,
    targetFrameworkVersion: meta.targetFrameworkVersion,
    updated: lastModified(path.relative(ROOT, entry.dir)),
    authors: meta.authors,
    parties: meta.parties.map((p) => p.role),
    partiesDetail: meta.parties,
    proofRequirement: meta.proofRequirement,
    errorCodes: meta.errorCodes || [],
    jsonLdContext: !!meta.jsonLdContext,
    hasResponse,
    schema,
    related: meta.related || [],
    prosePath: `/specs/${meta.slug}/${meta.version}/spec.md`,
    schemaPath: `/specs/${meta.slug}/${meta.version}/payload.schema.json`
  };
}

function copyDirSync(src, dst) {
  fs.mkdirSync(dst, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const s = path.join(src, entry.name);
    const d = path.join(dst, entry.name);
    if (entry.isDirectory()) copyDirSync(s, d);
    else fs.copyFileSync(s, d);
  }
}

function emitTasks(tasks) {
  const out = path.join(WEBSITE_DIR, 'assets', 'tasks.generated.js');
  const header = [
    '/* AUTO-GENERATED by scripts/build-registry.mjs — do not edit by hand.',
    ' * Source of truth: specs/<slug>/<version>/spec.md',
    ` * Generated at: ${new Date().toISOString()}`,
    ' */',
    'window.TT_TASKS = '
  ].join('\n');
  const footer = ';\n\n/* derived counts */\n' + `
window.TT_STATS = (function () {
  const tasks = window.TT_TASKS;
  const byStatus = tasks.reduce((acc, t) => { acc[t.status] = (acc[t.status] || 0) + 1; return acc; }, {});
  const orgs = new Set();
  tasks.forEach(t => t.authors.forEach(a => orgs.add(a)));
  const latest = tasks.length ? tasks.reduce((a, b) => (a.updated > b.updated ? a : b)) : null;
  return {
    total: tasks.length,
    byStatus,
    categories: (window.TT_CATEGORIES || []).length,
    orgs: orgs.size,
    latest: latest ? latest.updated : null,
    latestTitle: latest ? latest.title : null
  };
})();
`;
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, header + JSON.stringify(tasks, null, 2) + footer);
  console.log(`  wrote ${path.relative(ROOT, out)}`);
}

function syncWebsiteSpecs() {
  const dst = path.join(WEBSITE_DIR, 'specs');
  if (fs.existsSync(dst)) fs.rmSync(dst, { recursive: true, force: true });
  copyDirSync(SPECS_DIR, dst);
  // strip the internal meta-schema from the published tree
  const metaInPublished = path.join(dst, 'spec.meta.schema.json');
  if (fs.existsSync(metaInPublished)) fs.unlinkSync(metaInPublished);
  console.log(`  synced specs/ → ${path.relative(ROOT, dst)}/`);
}

function main() {
  console.log(`Trust Tasks build${validateOnly ? ' (validate-only)' : ''}`);
  const validate = loadMetaValidator();
  const entries = discoverSpecs();
  if (entries.length === 0) {
    console.warn('No specs found under specs/<slug>/<version>/.');
  }

  const tasks = [];
  const seen = new Set();

  for (const entry of entries) {
    const { slug, version, specPath, dir } = entry;
    const rel = `${slug}/${version}`;
    const src = fs.readFileSync(specPath, 'utf8');
    const { data: meta } = splitFrontMatter(src);
    if (!meta) {
      fail(`${rel}/spec.md`, 'missing or malformed YAML front matter');
      continue;
    }
    if (meta.slug !== slug) {
      fail(`${rel}/spec.md`, `front matter slug '${meta.slug}' does not match folder '${slug}'`);
    }
    if (meta.version !== version) {
      fail(`${rel}/spec.md`, `front matter version '${meta.version}' does not match folder '${version}'`);
    }
    if (!validate(meta)) {
      for (const err of validate.errors || []) {
        fail(`${rel}/spec.md`, `${err.instancePath || '/'} ${err.message}`);
      }
      continue;
    }
    const idKey = `${meta.slug}@${meta.version}`;
    if (seen.has(idKey)) {
      fail(rel, `duplicate slug+version ${idKey}`);
      continue;
    }
    seen.add(idKey);
    const schema = checkPayloadSchema(slug, version, dir);
    if (!schema) continue;
    tasks.push(buildTask(entry, meta, schema));
  }

  // related[] referential integrity
  const slugSet = new Set(tasks.map((t) => t.slug));
  for (const t of tasks) {
    for (const r of t.related || []) {
      if (!slugSet.has(r)) {
        fail(`${t.slug}/${t.version}`, `related entry '${r}' does not match any known spec slug`);
      }
    }
  }

  if (errors.length) {
    console.error('\nBuild failed with the following problems:');
    for (const e of errors) console.error(`  - ${e}`);
    process.exit(1);
  }

  console.log(`Validated ${tasks.length} spec${tasks.length === 1 ? '' : 's'}.`);

  if (validateOnly) return;

  tasks.sort((a, b) => (a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : a.version < b.version ? 1 : -1));
  emitTasks(tasks);
  syncWebsiteSpecs();
  console.log('Done.');
}

main();
