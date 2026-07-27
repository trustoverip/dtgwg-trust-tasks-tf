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
 *  - Copies bindings/ tree into website/bindings/ so the SPA's BindingSpecPage
 *    can fetch each binding's spec.md
 *  - Copies SPEC.md to website/SPEC.md so FrameworkSpecPage can fetch it
 *
 * Run from the repo root: `npm run build` or `npm run validate` (no website writes).
 */
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import Ajv from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS_DIR = path.join(ROOT, 'specs');
const BINDINGS_DIR = path.join(ROOT, 'bindings');
const WEBSITE_DIR = path.join(ROOT, 'website');
const META_SCHEMA_PATH = path.join(SPECS_DIR, 'spec.meta.schema.json');
const DATA_JS_PATH = path.join(WEBSITE_DIR, 'assets', 'data.js');

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

function firstAdded(dirRel) {
  // Oldest commit that touched the spec folder; treated as the spec's creation date.
  // Rename-aware tracking would require --follow per file (which doesn't apply to
  // folders) — for now the per-version folder is the unit and renames produce a
  // new "created" date, which matches how versions are independently editable.
  try {
    const out = execSync(
      `git log --reverse --format=%cI -- "${dirRel}"`,
      { cwd: ROOT, encoding: 'utf8' }
    ).trim();
    if (out) return out.split('\n')[0].slice(0, 10);
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

/* ---------- Shared schemas: discovery, slug, ref-walk ----------
 *
 * "Shared schemas" are the reusable JSON Schema documents that individual
 * payload schemas reference via $ref — the framework primitives in
 * specs/_framework/, the per-family shared $defs in specs/<family>/_shared/,
 * and the method-extension schemas under specs/<family>/_shared/.../did-method-extensions/.
 *
 * The discovery rule is simple: any `*.schema.json` file living anywhere
 * under a directory whose name starts with `_` is a shared schema. We index
 * them by absolute filesystem path so we can resolve $refs deterministically
 * (the relative path inside a $ref is what JSON Schema considers canonical).
 */
const SHARED_SCHEMA_ID_PREFIX = 'https://trusttasks.org/spec/';

function discoverSharedSchemas() {
  if (!fs.existsSync(SPECS_DIR)) return [];
  const found = [];
  walk(SPECS_DIR);
  return found;

  function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(full);
        continue;
      }
      if (!entry.isFile()) continue;
      if (!entry.name.endsWith('.schema.json')) continue;
      // skip the meta-schema and per-task payload schemas (those are owned by tasks)
      if (entry.name === 'spec.meta.schema.json') continue;
      if (entry.name === 'payload.schema.json') continue;
      // shared schemas live somewhere under an `_`-prefixed directory
      const rel = path.relative(SPECS_DIR, full);
      const segs = rel.split(path.sep);
      const underInternal = segs.slice(0, -1).some((s) => s.startsWith('_'));
      if (!underInternal) continue;
      try {
        const schema = readJson(full);
        found.push({ filePath: full, rel, schema });
      } catch (e) {
        fail(rel, `invalid JSON: ${e.message}`);
      }
    }
  }
}

/* Slug used for the website's /schema/<slug> route. Derived from the schema's
 * canonical $id (preferred — that's what cross-references resolve against) and
 * falling back to the on-disk path with `.schema.json` stripped. */
function sharedSchemaSlug({ filePath, schema }) {
  if (typeof schema.$id === 'string' && schema.$id.startsWith(SHARED_SCHEMA_ID_PREFIX)) {
    return schema.$id.slice(SHARED_SCHEMA_ID_PREFIX.length);
  }
  const rel = path.relative(SPECS_DIR, filePath).replace(/\\/g, '/');
  return rel.replace(/\.schema\.json$/, '');
}

function sharedSchemaKind(rel) {
  const segs = rel.split('/');
  if (segs[0] === '_framework') return 'framework';
  // anything sitting in or under a did-method-extensions/ folder is a method extension
  if (segs.includes('did-method-extensions')) return 'method-extension';
  return 'shared';
}

function sharedSchemaFamily(rel) {
  const segs = rel.split('/');
  if (segs[0] === '_framework') return 'framework';
  return segs[0];
}

function buildSharedRecord(entry) {
  const { filePath, rel, schema } = entry;
  const relWeb = rel.replace(/\\/g, '/');
  const slug = sharedSchemaSlug(entry);
  const defs = schema.$defs && typeof schema.$defs === 'object'
    ? Object.keys(schema.$defs)
    : [];
  return {
    slug,
    schemaId: schema.$id || null,
    title: schema.title || slug.split('/').slice(-1)[0],
    description: schema.description || null,
    kind: sharedSchemaKind(relWeb),
    family: sharedSchemaFamily(relWeb),
    sourcePath: `/specs/${relWeb}`,
    defs,
    schema
  };
}

/* Walks a payload schema and yields every external $ref it contains.
 * Internal refs (no file part, e.g. "#/$defs/Response") are skipped — those
 * are the spec's own response sub-schema and don't represent a dependency. */
function* iterRefs(node) {
  if (!node || typeof node !== 'object') return;
  if (Array.isArray(node)) {
    for (const item of node) yield* iterRefs(item);
    return;
  }
  for (const [k, v] of Object.entries(node)) {
    if (k === '$ref' && typeof v === 'string') {
      yield v;
      continue;
    }
    yield* iterRefs(v);
  }
}

/* Resolve a $ref relative to the payload schema's directory to (filePath, fragment).
 * Returns null when the ref is purely internal (no file part). */
function resolveRef(ref, baseDir) {
  const hashIdx = ref.indexOf('#');
  const filePart = hashIdx === -1 ? ref : ref.slice(0, hashIdx);
  const frag = hashIdx === -1 ? '' : ref.slice(hashIdx + 1);
  if (!filePart) return null; // internal ref — same document
  // Only resolve relative file paths; absolute URLs would need a registry lookup
  // and we don't currently use any in spec schemas.
  if (/^[a-z]+:\/\//i.test(filePart)) return { external: filePart, frag };
  const absPath = path.resolve(baseDir, filePart);
  return { filePath: absPath, frag };
}

/* Build the per-task `uses` list from a payload schema + a map from absolute
 * shared-schema filepath → shared record. Dedupes by (schemaSlug, def) so
 * `Ext` referenced from five places shows as one entry with occurrences: 5. */
function computeUses(schema, schemaPath, sharedByPath) {
  const baseDir = path.dirname(schemaPath);
  const seen = new Map(); // key → { schemaSlug, def, occurrences }
  const unresolved = []; // refs we couldn't match — surfaced for the build log
  for (const ref of iterRefs(schema)) {
    const resolved = resolveRef(ref, baseDir);
    if (!resolved) continue;                  // internal $ref
    if (resolved.external) continue;          // not a tracked dependency
    const target = sharedByPath.get(resolved.filePath);
    if (!target) {
      unresolved.push(ref);
      continue;
    }
    // Pull the def name from a `#/$defs/<Name>` fragment when present.
    const defMatch = resolved.frag.match(/^\/\$defs\/([^/]+)$/);
    const def = defMatch ? decodeURIComponent(defMatch[1]) : null;
    const key = `${target.slug}::${def || ''}`;
    const existing = seen.get(key);
    if (existing) {
      existing.occurrences += 1;
    } else {
      seen.set(key, { schemaSlug: target.slug, def, occurrences: 1, via: 'ref' });
    }
  }
  return { uses: [...seen.values()], unresolved };
}

function loadMetaValidator() {
  const ajv = new Ajv({ allErrors: true, strict: false });
  addFormats(ajv);
  return ajv.compile(readJson(META_SCHEMA_PATH));
}

/* Cross-check the category taxonomy. The category vocabulary lives in TWO
 * hand-maintained places that nothing else forces to agree:
 *   - specs/spec.meta.schema.json #/properties/category/enum (validated per spec)
 *   - website/assets/data.js window.TT_CATEGORIES (the site's id/name/color)
 * An enum value with no TT_CATEGORIES entry renders broken on the site — the
 * spec page can't resolve a name/color and the category is invisible in nav —
 * so it's a hard error. A TT_CATEGORIES entry with no enum value is dead
 * weight, so it's a warning. (This is the drift that shipped chat/message/1.0
 * broken: category added to the enum + a spec, but not to data.js.) */
function checkCategoryTaxonomy() {
  const meta = readJson(META_SCHEMA_PATH);
  const enumIds = meta?.properties?.category?.enum;
  if (!Array.isArray(enumIds)) {
    warn('spec.meta.schema.json: could not read #/properties/category/enum — skipping taxonomy cross-check');
    return;
  }
  if (!fs.existsSync(DATA_JS_PATH)) {
    warn(`${path.relative(ROOT, DATA_JS_PATH)} not found — skipping taxonomy cross-check`);
    return;
  }
  let categories;
  try {
    const sandbox = { window: {} };
    vm.createContext(sandbox);
    vm.runInContext(fs.readFileSync(DATA_JS_PATH, 'utf8'), sandbox, { filename: 'data.js' });
    categories = sandbox.window.TT_CATEGORIES;
  } catch (e) {
    fail(path.relative(ROOT, DATA_JS_PATH), `failed to evaluate window.TT_CATEGORIES: ${e.message}`);
    return;
  }
  if (!Array.isArray(categories)) {
    fail(path.relative(ROOT, DATA_JS_PATH), 'window.TT_CATEGORIES is not an array');
    return;
  }
  const dataIds = new Set(categories.map((c) => c && c.id));
  for (const id of enumIds) {
    if (!dataIds.has(id)) {
      fail(
        path.relative(ROOT, DATA_JS_PATH),
        `category '${id}' is in spec.meta.schema.json's enum but missing from window.TT_CATEGORIES — ` +
        `its specs would render without a name/color and crash the spec page. ` +
        `Add an { id: "${id}", name, color, blurb, icon } entry.`
      );
    }
  }
  for (const id of dataIds) {
    if (!enumIds.includes(id)) {
      warn(`${path.relative(ROOT, DATA_JS_PATH)}: category '${id}' has no matching enum value in spec.meta.schema.json — dead weight`);
    }
  }
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

/* Merge frontmatter `methodExtensions` declarations into the task's `uses`
 * list. These are out-of-band — they don't appear as $refs in the payload
 * schema, but they're how producers learn which method-specific shapes belong
 * inside `ext` when the payload's method discriminator matches. */
function applyMethodExtensions(meta, slug, version, uses, sharedBySlug) {
  const decls = meta.methodExtensions || [];
  const out = uses.slice();
  for (const decl of decls) {
    const target = sharedBySlug.get(decl.schema);
    if (!target) {
      fail(`${slug}/${version}/spec.md`, `methodExtensions[].schema '${decl.schema}' does not resolve to a discovered shared schema`);
      continue;
    }
    // Method extensions are tracked with `via: "methodExtension"` so the
    // website can render them in their own group rather than mixed with
    // payload-schema $ref dependencies.
    out.push({
      schemaSlug: target.slug,
      def: null,
      occurrences: 1,
      via: 'methodExtension',
      method: decl.method,
      requirement: decl.requirement || 'OPTIONAL'
    });
  }
  return out;
}

function buildTask(entry, meta, schema, uses) {
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
    created: firstAdded(path.relative(ROOT, entry.dir)),
    updated: lastModified(path.relative(ROOT, entry.dir)),
    authors: meta.authors,
    parties: meta.parties.map((p) => p.role),
    partiesDetail: meta.parties,
    proofRequirement: meta.proofRequirement,
    sideEffects: meta.sideEffects || null,
    exposure: meta.exposure || null,
    consequences: meta.consequences || [],
    subjectPath: meta.subjectPath || null,
    errorCodes: meta.errorCodes || [],
    jsonLdContext: !!meta.jsonLdContext,
    hasResponse,
    schema,
    related: meta.related || [],
    uses: uses || [],
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

function emitTasks(tasks, shared) {
  const out = path.join(WEBSITE_DIR, 'assets', 'tasks.generated.js');
  const header = [
    '/* AUTO-GENERATED by scripts/build-registry.mjs — do not edit by hand.',
    ' * Source of truth: specs/<slug>/<version>/spec.md',
    ` * Generated at: ${new Date().toISOString()}`,
    ' */',
    'window.TT_TASKS = '
  ].join('\n');
  const mid = ';\n\nwindow.TT_SHARED = ';
  const footer = ';\n\n/* derived counts */\n' + `
window.TT_STATS = (function () {
  // Count distinct Trust Tasks (one entry per slug — the latest non-retired
  // version), not how many versions exist, so coexisting 0.1/0.2 don't inflate.
  const cmpVer = (a, b) => { const pa = a.split('.').map(Number), pb = b.split('.').map(Number); return (pa[0] - pb[0]) || (pa[1] - pb[1]); };
  const bySlug = new Map();
  for (const t of window.TT_TASKS) {
    const prev = bySlug.get(t.slug);
    if (!prev) { bySlug.set(t.slug, t); continue; }
    const pr = prev.status === 'retired', tr = t.status === 'retired';
    if (pr !== tr) { if (pr) bySlug.set(t.slug, t); continue; }
    if (cmpVer(t.version, prev.version) > 0) bySlug.set(t.slug, t);
  }
  const tasks = [...bySlug.values()];
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

/* reverse index: for each shared schema, which tasks reference it.
 * Carries the same via/method metadata as the forward uses entries so
 * the schema page can show whether each usage is a structural $ref or a
 * frontmatter-declared method extension. */
window.TT_SHARED_USED_BY = (function () {
  const idx = {};
  for (const t of window.TT_TASKS) {
    for (const u of (t.uses || [])) {
      const list = idx[u.schemaSlug] = idx[u.schemaSlug] || [];
      list.push({
        slug: t.slug, version: t.version, title: t.title, status: t.status,
        def: u.def, via: u.via || 'ref', method: u.method || null, requirement: u.requirement || null
      });
    }
  }
  return idx;
})();
`;
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(
    out,
    header + JSON.stringify(tasks, null, 2)
      + mid + JSON.stringify(shared, null, 2)
      + footer
  );
  console.log(`  wrote ${path.relative(ROOT, out)}`);
}

// Plain-JSON registry for non-browser consumers (the VTA policy engine, the
// browser-extension / mobile consent surfaces). Same task metadata as
// tasks.generated.js but framed as JSON with a stable per-entry `typeUri`, so a
// consumer can key on the Type URI and read sideEffects / exposure / subjectPath
// without evaluating a `window.*` assignment. Served at
// https://trusttasks.org/registry.json.
function emitRegistryJson(tasks) {
  const out = path.join(WEBSITE_DIR, 'registry.json');
  const entries = tasks.map((t) => ({
    typeUri: `https://trusttasks.org/spec/${t.slug}/${t.version}`,
    ...t
  }));
  const doc = {
    metaSchema: 'https://trusttasks.org/internal/spec-meta/2.0',
    generatedAt: new Date().toISOString(),
    tasks: entries
  };
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, JSON.stringify(doc, null, 2));
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

function syncWebsiteBindings() {
  if (!fs.existsSync(BINDINGS_DIR)) return;
  const dst = path.join(WEBSITE_DIR, 'bindings');
  if (fs.existsSync(dst)) fs.rmSync(dst, { recursive: true, force: true });
  copyDirSync(BINDINGS_DIR, dst);
  console.log(`  synced bindings/ → ${path.relative(ROOT, dst)}/`);
}

function syncWebsiteFrameworkSpec() {
  const src = path.join(ROOT, 'SPEC.md');
  if (!fs.existsSync(src)) return;
  const dst = path.join(WEBSITE_DIR, 'SPEC.md');
  fs.copyFileSync(src, dst);
  console.log(`  synced SPEC.md → ${path.relative(ROOT, dst)}`);
}

function main() {
  console.log(`Trust Tasks build${validateOnly ? ' (validate-only)' : ''}`);
  const validate = loadMetaValidator();
  checkCategoryTaxonomy();
  const entries = discoverSpecs();
  if (entries.length === 0) {
    console.warn('No specs found under specs/<slug>/<version>/.');
  }

  // Discover shared/framework/method-extension schemas first so we can
  // resolve $refs from payload schemas against them when building tasks.
  const sharedEntries = discoverSharedSchemas();
  const sharedRecords = sharedEntries.map(buildSharedRecord);
  const sharedByPath = new Map(sharedEntries.map((e) => [e.filePath, sharedRecords[sharedEntries.indexOf(e)]]));
  const sharedBySlug = new Map(sharedRecords.map((r) => [r.slug, r]));

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
    // Friendlier targeted checks the JSON-Schema if/then/else can't phrase well:
    if (meta.supersededBy && meta.status !== 'retired') {
      fail(`${rel}/spec.md`, `supersededBy is only permitted when status is 'retired' (got status: '${meta.status}'). See SPEC §7.3 item 11.`);
      continue;
    }
    if (meta.status === 'retired' && !meta.supersededBy) {
      warn(`${rel}/spec.md: status is 'retired' but no supersededBy declared — SPEC §7.3 item 11 RECOMMENDS one`);
    }
    const idKey = `${meta.slug}@${meta.version}`;
    if (seen.has(idKey)) {
      fail(rel, `duplicate slug+version ${idKey}`);
      continue;
    }
    seen.add(idKey);
    const schema = checkPayloadSchema(slug, version, dir);
    if (!schema) continue;
    const payloadSchemaPath = path.join(dir, 'payload.schema.json');
    const { uses: refUses, unresolved } = computeUses(schema, payloadSchemaPath, sharedByPath);
    for (const u of unresolved) {
      warn(`${slug}/${version}/payload.schema.json: $ref '${u}' did not resolve to a discovered shared schema`);
    }
    const uses = applyMethodExtensions(meta, slug, version, refUses, sharedBySlug);
    tasks.push(buildTask(entry, meta, schema, uses));
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

  // Relative markdown links in spec prose must resolve on disk.
  //
  // They resolve from the VERSION directory (`specs/<slug>/<version>/`),
  // because that is where spec.md lives and where it is served from — the
  // sync below copies `specs/` verbatim to `website/specs/<slug>/<version>/`.
  // The recurring mistake is writing them as though they resolved from the
  // SLUG directory, one level shallower: from `auth/passkey/enroll/start/0.1`,
  // `../finish/0.1/spec.md` points at `.../start/finish/0.1/`, not at the
  // sibling leg. 135 links across 65 files had drifted this way before anything
  // checked, and they were broken on the live site, not merely in the repo.
  for (const { dir, slug, version, specPath } of entries) {
    const prose = fs.readFileSync(specPath, 'utf8');
    for (const m of prose.matchAll(/\]\((\.\.[^)#\s]*?)(#[^)\s]*)?\)/g)) {
      const target = path.resolve(dir, m[1]);
      if (!fs.existsSync(target)) {
        const deeper = path.resolve(dir, '../' + m[1]);
        const hint = fs.existsSync(deeper)
          ? ` — one level too shallow; '../${m[1]}' resolves`
          : '';
        fail(`${slug}/${version}/spec.md`, `relative link '${m[1]}' does not resolve${hint}`);
      }
    }
  }

  if (errors.length) {
    console.error('\nBuild failed with the following problems:');
    for (const e of errors) console.error(`  - ${e}`);
    process.exit(1);
  }

  console.log(
    `Validated ${tasks.length} spec${tasks.length === 1 ? '' : 's'}, ` +
    `indexed ${sharedRecords.length} shared schema${sharedRecords.length === 1 ? '' : 's'}.`
  );

  if (validateOnly) return;

  tasks.sort((a, b) => (a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : a.version < b.version ? 1 : -1));
  sharedRecords.sort((a, b) => (a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : 0));
  emitTasks(tasks, sharedRecords);
  emitRegistryJson(tasks);
  syncWebsiteSpecs();
  syncWebsiteBindings();
  syncWebsiteFrameworkSpec();
  console.log('Done.');
}

main();
