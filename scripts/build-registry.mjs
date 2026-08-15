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
const CEREMONIES_DIR = path.join(ROOT, 'ceremonies');
const WEBSITE_DIR = path.join(ROOT, 'website');
const META_SCHEMA_PATH = path.join(SPECS_DIR, 'spec.meta.schema.json');
const DATA_JS_PATH = path.join(WEBSITE_DIR, 'assets', 'data.js');
const BINDINGS_JS_PATH = path.join(WEBSITE_DIR, 'assets', 'bindings.js');

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

/* Resolve `$ref`s inside an `errorCodes[].detailsSchema` front-matter fragment.
 *
 * A detailsSchema is a JSON Schema fragment describing an error's `details`
 * object, but it lives in YAML front matter rather than in a schema file, so
 * nothing resolved `$ref`s there — a ref written in one would dangle silently.
 * That left families with no way to share a shape between an error and the
 * rest of the registry, which is how the step-up challenge came to be restated
 * verbatim in three vault specs.
 *
 * Refs are resolved and INLINED into the emitted registry rather than passed
 * through. `detailsSchema` is consumed only by machine readers via
 * registry.json / tasks.generated.js, and handing them a relative `$ref` with
 * no base URI would be strictly worse than the duplication it replaces. So the
 * source gets one home for the shape and consumers still get the whole thing.
 */
function resolveDetailsSchemas(meta, dir, rel, sharedByPath) {
  const codes = meta.errorCodes || [];
  if (!codes.some((c) => c && c.detailsSchema)) return { errorCodes: codes, uses: [] };

  const uses = [];
  const out = codes.map((code) => {
    if (!code || !code.detailsSchema) return code;
    return { ...code, detailsSchema: resolve(code.detailsSchema, dir, code.code, 0) };
  });
  return { errorCodes: out, uses };

  function resolve(node, baseDir, codeName, depth) {
    if (Array.isArray(node)) return node.map((n) => resolve(n, baseDir, codeName, depth));
    if (!node || typeof node !== 'object') return node;

    if (typeof node.$ref === 'string') {
      if (depth > 8) {
        fail(rel, `errorCodes['${codeName}'].detailsSchema: $ref nesting too deep (cycle?) at '${node.$ref}'`);
        return node;
      }
      const resolved = resolveRef(node.$ref, baseDir);
      if (!resolved || resolved.external) {
        fail(rel, `errorCodes['${codeName}'].detailsSchema: $ref '${node.$ref}' must point at a shared schema file (a bare local '#/...' ref has nothing to resolve against here)`);
        return node;
      }
      const target = sharedByPath.get(resolved.filePath);
      if (!target) {
        fail(rel, `errorCodes['${codeName}'].detailsSchema: $ref '${node.$ref}' did not resolve to a discovered shared schema`);
        return node;
      }
      const frag = pointerInto(target.schema, resolved.frag);
      if (frag === undefined) {
        fail(rel, `errorCodes['${codeName}'].detailsSchema: $ref '${node.$ref}' resolved to a file but its fragment '${resolved.frag}' does not exist`);
        return node;
      }
      const defMatch = resolved.frag.match(/^\/\$defs\/([^/]+)$/);
      uses.push({
        schemaSlug: target.slug,
        def: defMatch ? decodeURIComponent(defMatch[1]) : null,
        occurrences: 1,
        via: 'ref'
      });
      // Recurse into the resolved fragment so a shared def that itself refs
      // another one still inlines whole. Its refs resolve against ITS file.
      const { $ref, ...siblings } = node;
      return { ...resolve(frag, path.dirname(resolved.filePath), codeName, depth + 1), ...siblings };
    }

    const copy = {};
    for (const [k, v] of Object.entries(node)) copy[k] = resolve(v, baseDir, codeName, depth);
    return copy;
  }
}

/* Walk an RFC 6901 JSON Pointer into a parsed schema. Returns undefined when
 * any segment is missing, which the caller reports as a dangling fragment. */
function pointerInto(root, pointer) {
  if (!pointer) return root;
  let node = root;
  for (const raw of pointer.split('/').slice(1)) {
    const seg = decodeURIComponent(raw).replace(/~1/g, '/').replace(/~0/g, '~');
    if (node === null || typeof node !== 'object' || !(seg in node)) return undefined;
    node = node[seg];
  }
  return node;
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

/*
 * Resolve a spec's proof requirement for each document variant.
 *
 * §7.3 item 8 accepts a single `requirement` covering every variant, or a
 * per-variant `request` / `response` pair. The single form is normalised onto
 * both so callers never branch.
 */
function resolveProofRequirement(meta) {
  const pr = meta.proofRequirement || {};
  if (typeof pr.requirement === 'string') {
    return { request: pr.requirement, response: pr.requirement, perVariant: false };
  }
  return {
    request: pr.request,
    // A spec that declares no `response` level takes the request's — the
    // conservative reading, and the only one that cannot silently weaken a
    // variant by omission.
    response: pr.response ?? pr.request,
    perVariant: true,
  };
}

/*
 * SPEC §7.3 item 8 requires a specification's declared `proof` requirement to be
 * "no weaker than the default applicable under §4.7.1". That constraint cannot
 * be checked as written: §4.7.1's default is a function of the *transport*
 * (MAY omit over an authenticated channel, SHOULD otherwise, MUST where the
 * document will be relied on by third parties), and a specification does not
 * know its transport at authoring time. So nothing ever enforced it, and specs
 * that exercise a subject's authority or release secrets shipped with the proof
 * member merely RECOMMENDED.
 *
 * This derives the floor from the declarations the spec *does* make — the
 * side-effect class (item 13) and exposure class (item 14) — and applies each
 * to the variant it actually describes:
 *
 *   sideEffects.level == destructive   the REQUEST causes an irreversible
 *   exposure.actsAsSubject == true     effect, or exercises the subject's
 *                                      authority -> the request must be proven
 *
 *   exposure.discloses == secret       the RESPONSE carries the confidential
 *                                      material -> the response must be proven
 *
 * Splitting them is the point. A task that destroys state but returns only an
 * acknowledgement needs its request attributable and has nothing to protect on
 * the way back; a task that returns a secret in answer to a harmless read is the
 * reverse. Before the per-variant form both were forced to REQUIRED on
 * everything, overstating whichever half did not need it.
 *
 * Note this does not conflict with items 13/14 being "descriptive, not
 * prescriptive": that rule forbids deriving a *consent or approval* requirement
 * from the class. An integrity floor governs how a document is authenticated,
 * not whether a human must approve it.
 */
function checkProofFloor(meta, rel, hasResponse) {
  const declared = resolveProofRequirement(meta);

  const requestTriggers = [];
  if (meta.sideEffects?.level === 'destructive') {
    requestTriggers.push('sideEffects.level: destructive');
  }
  if (meta.exposure?.actsAsSubject === true) {
    requestTriggers.push('exposure.actsAsSubject: true');
  }
  const responseTriggers = [];
  if (meta.exposure?.discloses === 'secret') {
    responseTriggers.push('exposure.discloses: secret');
  }

  const complain = (variant, level, triggers, why) => {
    fail(
      `${rel}/spec.md`,
      `proofRequirement for the ${variant} variant is '${level}' but the spec declares ` +
        `${triggers.join(' and ')}. ${why} MUST declare proof REQUIRED — ` +
        `§7.2 item 7 only rejects proofless documents for specs that declare it. ` +
        `See SPEC §7.3 item 8 and §4.7.1.`
    );
  };

  if (requestTriggers.length && declared.request !== 'REQUIRED') {
    complain(
      'request',
      declared.request,
      requestTriggers,
      'A request that is irreversible or acts with the subject\'s authority'
    );
  }
  // A fire-and-forget spec has no response document to constrain (§4.4.1).
  if (hasResponse && responseTriggers.length && declared.response !== 'REQUIRED') {
    complain(
      'response',
      declared.response,
      responseTriggers,
      'A response that carries secret material back to the caller'
    );
  }
}

function checkErrorCodeNamespaces(meta, rel) {
  const slug = meta.slug;
  if (typeof slug !== 'string') return;

  // The emitting slug plus each proper path prefix of it.
  const segments = slug.split('/');
  const permitted = new Set(
    segments.map((_, i) => segments.slice(0, i + 1).join('/'))
  );

  for (const entry of meta.errorCodes || []) {
    const code = entry?.code;
    if (typeof code !== 'string') continue;
    const colon = code.lastIndexOf(':');
    if (colon < 0) continue; // grammar failure — the meta schema reports it
    const namespace = code.slice(0, colon);
    if (permitted.has(namespace)) continue;

    fail(
      `${rel}/spec.md`,
      `errorCodes['${code}'] is namespaced '${namespace}', which is neither this ` +
        `specification's slug nor a path prefix of it. SPEC §8.5 permits only ` +
        `'${slug}' or a family namespace (${[...permitted]
          .filter((p) => p !== slug)
          .map((p) => `'${p}'`)
          .join(', ') || 'none available for a single-segment slug'}). ` +
        `A code namespaced under an unrelated slug is unreachable from ` +
        `Payload::extended_code, which derives the namespace from TYPE_URI, so ` +
        `the registry and the generated libraries would disagree silently.`
    );
  }
}

/*
 * Validate every example Trust Task document in the repo against the framework
 * envelope schema for its declared target framework version.
 *
 * Two things make this worth doing. The envelope schemas were authored after
 * 285 specs had already been published against framework 0.1 and 0.2, so they
 * describe versions that were previously defined only in §4.2 prose — this is
 * what demonstrates they are faithful to what shipped rather than a retroactive
 * tightening. And an example document is the thing implementers copy: a wrong
 * one is a bug that propagates, and nothing checked them before.
 *
 * Only blocks that parse as JSON *and* look like an envelope (a `type` plus a
 * `payload`) are considered. Specs legitimately include illustrative fragments
 * that are not valid JSON — `bindings/tsp/0.1` shows envelope shape using a
 * `/* … *​/` comment — so unparseable blocks are counted and warned about
 * rather than failed, which would punish a deliberate ellipsis.
 */
function checkExampleDocuments() {
  const ajv = new Ajv({ allErrors: true, strict: false });
  addFormats(ajv);
  const validators = new Map();
  for (const version of ['0.1', '0.2', '0.3']) {
    const p = path.join(SPECS_DIR, '_framework', version, 'trust-task.schema.json');
    if (!fs.existsSync(p)) continue;
    try {
      validators.set(version, ajv.compile(readJson(p)));
    } catch (e) {
      fail(`_framework/${version}/trust-task.schema.json`, `invalid envelope schema: ${e.message}`);
    }
  }
  if (validators.size === 0) return;

  // A document's envelope version is the *target framework version* of the spec
  // whose slug the document's `type` names (§7.2 item 1) — not the version in
  // the type URI, which identifies the task specification.
  const targetByTypePrefix = new Map();
  for (const { slug, version, specPath } of discoverSpecs()) {
    const { data } = splitFrontMatter(fs.readFileSync(specPath, 'utf8'));
    if (data?.targetFrameworkVersion) {
      targetByTypePrefix.set(`https://trusttasks.org/spec/${slug}/${version}`, data.targetFrameworkVersion);
    }
  }

  const sources = [path.join(ROOT, 'SPEC.md')];
  for (const base of [SPECS_DIR, path.join(ROOT, 'bindings')]) {
    if (!fs.existsSync(base)) continue;
    const walkMd = (dir) => {
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) walkMd(full);
        else if (entry.name === 'spec.md') sources.push(full);
      }
    };
    walkMd(base);
  }

  let checked = 0;
  let unparseable = 0;
  for (const file of sources) {
    const rel = path.relative(ROOT, file);
    const src = fs.readFileSync(file, 'utf8');
    for (const match of src.matchAll(/```json\n([\s\S]*?)```/g)) {
      // Examples inside a blockquote carry a "> " prefix on every line.
      const raw = match[1].split('\n').map((l) => l.replace(/^>\s?/, '')).join('\n');
      let doc;
      try {
        doc = JSON.parse(raw);
      } catch {
        unparseable++;
        continue;
      }
      if (!doc || typeof doc !== 'object' || Array.isArray(doc)) continue;
      if (typeof doc.type !== 'string' || !doc.type.startsWith('https://trusttasks.org/spec/')) continue;
      if (!('payload' in doc)) continue;

      const bare = doc.type.split('#')[0];
      const target = targetByTypePrefix.get(bare) ?? '0.2';
      const validate = validators.get(target);
      if (!validate) continue;

      checked++;
      if (!validate(doc)) {
        const why = (validate.errors || [])
          .map((e) => `${e.instancePath || '/'} ${e.message}`)
          .join('; ');
        fail(rel, `example document (type ${doc.type}) fails the framework ${target} envelope schema: ${why}`);
      }
    }
  }
  if (unparseable > 0) {
    warn(`${unparseable} fenced JSON block(s) did not parse and were skipped — expected for illustrative fragments, but check none is a malformed example`);
  }
  console.log(`  validated ${checked} example documents against the framework envelope schema`);
}

/** Whether a schema states how unrecognized members are treated (§7.3 item 7.4). */
function declaresClosure(schema) {
  return 'additionalProperties' in schema || 'unevaluatedProperties' in schema;
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
  // §7.3 item 7.4 asks a schema to say how unrecognized members are treated.
  // `unevaluatedProperties` satisfies that as squarely as `additionalProperties`
  // and is the only one that composes: a schema built by `allOf` over a shared
  // definition cannot use `additionalProperties`, because it is evaluated
  // per-subschema against the whole instance and rejects the members the outer
  // schema declares.
  if (!declaresClosure(schema)) {
    warn(`${slug}/${version}/payload.schema.json: no additionalProperties or unevaluatedProperties declared (SPEC §7.3 item 7.4 requires explicit handling)`);
  }
  // SPEC §7.3 item 7.6: if a response sub-schema exists, it MUST live in $defs.Response
  // and declare $anchor: "response".
  const respDef = schema.$defs && schema.$defs.Response;
  if (respDef) {
    if (respDef.$anchor !== 'response') {
      fail(`${slug}/${version}/payload.schema.json`, `$defs.Response must declare $anchor: "response" (got ${JSON.stringify(respDef.$anchor)})`);
    }
    if (!declaresClosure(respDef)) {
      warn(`${slug}/${version}/payload.schema.json: $defs.Response declares neither additionalProperties nor unevaluatedProperties`);
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
    // Adoption / lifecycle signals. These live in front matter and were, until
    // now, invisible to every registry reader — which defeated the point of
    // declaring them: `knownImplementations` exists to separate adoption from
    // aspiration, and `wireCompatibleWith` exists to tell an implementer it can
    // dual-accept a predecessor version. Both have to reach the page to work.
    knownImplementations: meta.knownImplementations || [],
    wireCompatibleWith: meta.wireCompatibleWith || null,
    supersededBy: meta.supersededBy || null,
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

function syncWebsiteCeremonies() {
  if (!fs.existsSync(CEREMONIES_DIR)) return;
  const dst = path.join(WEBSITE_DIR, 'ceremonies');
  if (fs.existsSync(dst)) fs.rmSync(dst, { recursive: true, force: true });
  copyDirSync(CEREMONIES_DIR, dst);
  // The definition meta-schema and its fixtures are authoring tools, not
  // published artifacts — SPEC §6.7 leaves the *content* of a ceremony
  // definition out of scope for framework 0.4, so publishing the format it must
  // satisfy would advertise something not yet normative.
  for (const internal of ['ceremony.meta.schema.json', 'ceremony.invalid-examples.json']) {
    const p = path.join(dst, internal);
    if (fs.existsSync(p)) fs.unlinkSync(p);
  }
  console.log(`  synced ceremonies/ → ${path.relative(ROOT, dst)}/`);
}

function syncWebsiteFrameworkSpec() {
  const src = path.join(ROOT, 'SPEC.md');
  if (!fs.existsSync(src)) return;
  const dst = path.join(WEBSITE_DIR, 'SPEC.md');
  fs.copyFileSync(src, dst);
  console.log(`  synced SPEC.md → ${path.relative(ROOT, dst)}`);
}

/**
 * Cross-check `bindings/<slug>/<version>/spec.md` against `window.TT_BINDINGS`.
 *
 * `bindings.js` is hand-edited — the build only *copies* the bindings tree to
 * the website, it never enumerates it — so a binding can ship complete and
 * still be invisible on the registry site. Both `didcomm/0.2` and
 * `didcomm-v1/0.1` did exactly that: merged, published, and absent from the
 * list for as long as nobody looked.
 *
 * This is the same hand-maintained-list failure the category taxonomy has, and
 * it gets the same treatment: an on-disk binding with no entry fails the build,
 * an entry with no binding on disk warns (a stale row renders a dead page, but
 * does not hide anything).
 */
function checkBindingRegistry() {
  if (!fs.existsSync(BINDINGS_DIR)) return;
  if (!fs.existsSync(BINDINGS_JS_PATH)) {
    warn(`${path.relative(ROOT, BINDINGS_JS_PATH)} not found — skipping binding registry cross-check`);
    return;
  }

  const onDisk = new Set();
  for (const slug of fs.readdirSync(BINDINGS_DIR, { withFileTypes: true })) {
    if (!slug.isDirectory() || slug.name.startsWith('_') || slug.name.startsWith('.')) continue;
    const slugDir = path.join(BINDINGS_DIR, slug.name);
    for (const version of fs.readdirSync(slugDir, { withFileTypes: true })) {
      if (!version.isDirectory()) continue;
      if (fs.existsSync(path.join(slugDir, version.name, 'spec.md'))) {
        onDisk.add(`${slug.name}/${version.name}`);
      }
    }
  }

  let bindings;
  try {
    const sandbox = { window: {} };
    vm.createContext(sandbox);
    vm.runInContext(fs.readFileSync(BINDINGS_JS_PATH, 'utf8'), sandbox, { filename: 'bindings.js' });
    bindings = sandbox.window.TT_BINDINGS;
  } catch (e) {
    fail(path.relative(ROOT, BINDINGS_JS_PATH), `failed to evaluate window.TT_BINDINGS: ${e.message}`);
    return;
  }
  if (!Array.isArray(bindings)) {
    fail(path.relative(ROOT, BINDINGS_JS_PATH), 'window.TT_BINDINGS is not an array');
    return;
  }

  const listed = new Map(bindings.filter(Boolean).map((b) => [b.id, b]));
  for (const id of onDisk) {
    if (!listed.has(id)) {
      fail(
        path.relative(ROOT, BINDINGS_JS_PATH),
        `binding '${id}' exists at bindings/${id}/spec.md but has no window.TT_BINDINGS entry — ` +
        `it would be published and unreachable from the registry site. ` +
        `Add an { id: "${id}", slug, version, title, summary, bindingURI, envelopeType, status, accent, prosePath, implementations } entry.`
      );
      continue;
    }
    // A wrong prosePath renders an empty page rather than an error, so check it.
    const expected = `/bindings/${id}/spec.md`;
    if (listed.get(id).prosePath !== expected) {
      fail(
        path.relative(ROOT, BINDINGS_JS_PATH),
        `binding '${id}' declares prosePath '${listed.get(id).prosePath}', expected '${expected}'`
      );
    }
  }
  for (const id of listed.keys()) {
    if (!onDisk.has(id)) {
      warn(`${path.relative(ROOT, BINDINGS_JS_PATH)}: binding '${id}' is listed but has no bindings/${id}/spec.md — its detail page will render empty`);
    }
  }
}

function main() {
  console.log(`Trust Tasks build${validateOnly ? ' (validate-only)' : ''}`);
  const validate = loadMetaValidator();
  checkCategoryTaxonomy();
  checkBindingRegistry();
  checkExampleDocuments();
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
    checkErrorCodeNamespaces(meta, rel);
    const idKey = `${meta.slug}@${meta.version}`;
    if (seen.has(idKey)) {
      fail(rel, `duplicate slug+version ${idKey}`);
      continue;
    }
    seen.add(idKey);
    const schema = checkPayloadSchema(slug, version, dir);
    if (!schema) continue;
    checkProofFloor(meta, rel, Boolean(schema.$defs?.Response));
    const payloadSchemaPath = path.join(dir, 'payload.schema.json');
    const { uses: refUses, unresolved } = computeUses(schema, payloadSchemaPath, sharedByPath);
    for (const u of unresolved) {
      warn(`${slug}/${version}/payload.schema.json: $ref '${u}' did not resolve to a discovered shared schema`);
    }
    const uses = applyMethodExtensions(meta, slug, version, refUses, sharedBySlug);
    // A shared shape referenced from an error's detailsSchema is as real a
    // dependency as one referenced from the payload schema — fold it into the
    // same list so the shared schema's "used by" index stays honest.
    const details = resolveDetailsSchemas(meta, dir, `${rel}/spec.md`, sharedByPath);
    for (const u of details.uses) {
      const existing = uses.find((e) => e.schemaSlug === u.schemaSlug && e.def === u.def && (e.via || 'ref') === 'ref');
      if (existing) existing.occurrences += u.occurrences;
      else uses.push(u);
    }
    tasks.push(buildTask(entry, { ...meta, errorCodes: details.errorCodes }, schema, uses));
  }

  // wireCompatibleWith referential integrity: the named predecessor must be a
  // real, strictly-earlier version of the SAME slug. The field's whole value is
  // that an implementer can act on it without checking — "dual-accept 0.1 by
  // re-casing and retyping" is worthless advice if 0.1 was never published.
  const versionsBySlug = new Map();
  for (const t of tasks) {
    if (!versionsBySlug.has(t.slug)) versionsBySlug.set(t.slug, new Set());
    versionsBySlug.get(t.slug).add(t.version);
  }
  const cmpVersion = (a, b) => {
    const pa = a.split('.').map(Number), pb = b.split('.').map(Number);
    return (pa[0] - pb[0]) || (pa[1] - pb[1]);
  };
  for (const t of tasks) {
    if (!t.wireCompatibleWith) continue;
    const rel = `${t.slug}/${t.version}`;
    if (!versionsBySlug.get(t.slug).has(t.wireCompatibleWith)) {
      fail(rel, `wireCompatibleWith names version '${t.wireCompatibleWith}', which does not exist for slug '${t.slug}'`);
    } else if (cmpVersion(t.wireCompatibleWith, t.version) >= 0) {
      fail(rel, `wireCompatibleWith must name a strictly earlier version of the same slug (got '${t.wireCompatibleWith}' for version '${t.version}')`);
    }
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
  syncWebsiteCeremonies();
  syncWebsiteFrameworkSpec();
  console.log('Done.');
}

main();
