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
import { discoverSpecs as discoverSpecsShared } from './lib/specs.mjs';
import {
  assessSecurityPrivacy,
  checkSecurityPrivacySections,
  writeAllowlist
} from './lib/security-privacy.mjs';
import { checkDisclosureFloor } from './lib/disclosure-floor.mjs';
import { createErrorCodeCasingLint, checkStandardErrorCodeCasing } from './lib/error-code-casing.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS_DIR = path.join(ROOT, 'specs');
const BINDINGS_DIR = path.join(ROOT, 'bindings');
const CEREMONIES_DIR = path.join(ROOT, 'ceremonies');
const WEBSITE_DIR = path.join(ROOT, 'website');
const META_SCHEMA_PATH = path.join(SPECS_DIR, 'spec.meta.schema.json');
const DATA_JS_PATH = path.join(WEBSITE_DIR, 'assets', 'data.js');
const BINDINGS_JS_PATH = path.join(WEBSITE_DIR, 'assets', 'bindings.js');

const validateOnly = process.argv.includes('--validate-only');
// Maintenance affordance for the Security & Privacy backlog: rewrite the
// allowlist from what is currently non-conforming and exit. Deliberately not a
// documented everyday flag — running it after regressing a spec would launder
// the regression into accepted debt, which is why the list is committed and
// reviewed rather than derived at build time.
const updateSpAllowlist = process.argv.includes('--update-security-privacy-allowlist');

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

/* Spec discovery lives in scripts/lib/specs.mjs — one rule, shared with
 * scripts/check-bindings-conformance.mjs. See that module for why: the build
 * used to find specs by `spec.md` while the code generators found them by
 * `payload.schema.json`, which agree only for as long as every version folder
 * carries both. This wiring makes the disagreement a build failure instead of
 * two tools describing two different registries in silence.
 *
 * Memoised because the module walks the tree and several callers want the list
 * (checkExampleDocuments() runs before main()'s own pass), and because the
 * structural problems below must be reported once, not once per caller. */
let specsCache = null;
function discoverSpecs() {
  if (specsCache) return specsCache;
  specsCache = discoverSpecsShared({
    specsDir: SPECS_DIR,
    onIncomplete: ({ rel, message }) => fail(rel, message),
    onNestedSlug: ({ rel, message }) => warn(`specs/${rel}: ${message}`)
  });
  return specsCache;
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

/*
 * Resolve a spec's `issuedAt` requirement for each document variant.
 *
 * §7.3 item 17 is declared with the same shape as item 8's `proofRequirement`
 * — a single `requirement` covering every variant, or a per-variant
 * `request` / `response` pair — so it is normalised the same way, including
 * the "an omitted `response` takes the request's value" rule that cannot
 * weaken a variant by omission.
 *
 * Absent means the framework baseline of §4.2 applies, which is a SHOULD.
 * That is reported as `undefined` rather than `RECOMMENDED` so a caller can
 * tell "the author said nothing" from "the author considered it and chose the
 * baseline" — the distinction the freshness floor below is entirely about.
 */
function resolveIssuedAtRequirement(meta) {
  const ir = meta.issuedAtRequirement || {};
  if (typeof ir.requirement === 'string') {
    return { request: ir.requirement, response: ir.requirement, declared: true };
  }
  if (typeof ir.request === 'string') {
    return { request: ir.request, response: ir.response ?? ir.request, declared: true };
  }
  return { request: undefined, response: undefined, declared: false };
}

/*
 * Is this a *consequential Trust Task* (SPEC §2)?
 *
 * The predicate is spelled out once in §2 and is a pure function of
 * declarations items 13 and 14 already require of every spec:
 *
 *   sideEffects.level ∈ {mutating, destructive}
 *   ∨ exposure.discloses == secret
 *   ∨ exposure.actsAsSubject == true
 *
 * §2 also fixes the fail-safe reading: an absent or unrecognized declaration
 * is consequential. That matters here, because the specs most likely to be
 * missing a declaration are the ones least likely to have thought about
 * freshness.
 */
function isConsequential(meta) {
  const level = meta.sideEffects?.level;
  if (level === undefined || !['none', 'mutating', 'destructive'].includes(level)) return true;
  if (level === 'mutating' || level === 'destructive') return true;
  const discloses = meta.exposure?.discloses;
  if (discloses === undefined || !['none', 'metadata', 'secret'].includes(discloses)) return true;
  if (discloses === 'secret') return true;
  return meta.exposure?.actsAsSubject !== false;
}

/*
 * SPEC §7.3 item 17: a specification defining a *consequential Trust Task*
 * MUST require the `issuedAt` member, raising §4.2's SHOULD to a MUST for its
 * own documents. Until `issuedAtRequirement` existed there was no way to say
 * so, so the requirement was unexpressible and no spec could comply.
 *
 * The floor is derived here rather than *substituted* for the declaration.
 * Deriving the value outright would remove the hand-edits, but it would also
 * mean that editing `sideEffects.level` from `none` to `mutating` — a purely
 * DESCRIPTIVE correction, which items 13 and 14 insist those declarations are
 * — silently changed which documents every consumer of the spec must reject.
 * §2 further makes the *handler* authoritative for consequentiality, not this
 * front matter, so a value computed from the front matter would be enforcing
 * something the front matter does not authoritatively state. So the author
 * declares it, the build derives the floor, and the two are compared:
 *
 *   declared weaker than the floor -> hard error, immediately.
 *   floor unmet because nothing is declared -> hard error, as of the commit
 *     that took the registry to 100%. It was reported as a count while the
 *     177 live consequential drafts predating `issuedAtRequirement` caught up,
 *     because failing the build for all of them at once would only have taught
 *     contributors to ignore the message. That backlog is gone, so the report
 *     is now a ratchet: a new consequential spec that says nothing about
 *     freshness fails, the same way `checkProofFloor` fails one that declares
 *     proof too weakly. There is no opt-out env var — a floor with an escape
 *     hatch is a warning wearing a different hat.
 *
 * `retired` specs are exempt from the *undeclared* case. §5.3 freezes a retired
 * specification's schema and prose at the moment of retirement, and declaring
 * REQUIRED on one would change which documents every consumer must reject for
 * a spec that is terminal and cannot take the MAJOR increment §5.2 would
 * demand for that change. They are counted and reported separately, the way the
 * error-code lints report their frozen populations: exempt, not debt. A retired
 * spec that *does* carry a declaration is still held to it — the check below
 * runs on every consequential spec — because that can only arrive by an edit.
 *
 * A non-consequential spec MAY still declare REQUIRED. The floor is a floor.
 */
function checkIssuedAtFloor(meta, rel, hasResponse) {
  const declared = resolveIssuedAtRequirement(meta);
  const consequential = isConsequential(meta);
  if (!consequential) return { consequential, unmet: false, frozen: false };

  const frozen = meta.status === 'retired';
  if (!declared.declared) return { consequential, unmet: !frozen, frozen };

  const variants = hasResponse ? ['request', 'response'] : ['request'];
  let unmet = false;
  for (const variant of variants) {
    if (declared[variant] !== 'REQUIRED') {
      unmet = true;
      fail(
        `${rel}/spec.md`,
        `issuedAtRequirement for the ${variant} variant is '${declared[variant]}' but this is a ` +
          `consequential Trust Task (sideEffects.level: ${meta.sideEffects?.level}, ` +
          `exposure.discloses: ${meta.exposure?.discloses}, ` +
          `exposure.actsAsSubject: ${meta.exposure?.actsAsSubject}). SPEC §7.3 item 17 makes ` +
          `issuedAt REQUIRED for such a specification — the duplicate-execution protection of ` +
          `§7.2 item 11 is implementable only over a window, and a document with no issuedAt ` +
          `cannot be placed in one. Declare REQUIRED, or correct the item 13/14 declarations if ` +
          `this task is not in fact consequential.`
      );
    }
  }
  return { consequential, unmet, frozen };
}

/*
 * `parties[].identifierScope: public` (framework 0.5.0) narrows the privacy
 * properties available to every producer of the task: it says the counterparty
 * must be able to recognise the same identifier it sees elsewhere, which
 * forecloses the pairwise identifiers that would otherwise stop documents of
 * this task being joined to that party's activity. The framework asks the
 * specification to justify that in prose rather than declare it and move on.
 *
 * The check is deliberately cheap and deliberately a warning: it asks only that
 * the body discuss the declaration at all, on the reasoning that a lint cannot
 * tell a justification from a sentence, but it can tell a justification from
 * silence — and silence is the failure mode the framework text is aimed at.
 */
function checkIdentifierScopeJustification(meta, body, rel) {
  const publicParties = (meta.parties || []).filter((p) => p?.identifierScope === 'public');
  if (!publicParties.length) return;
  if (/identifierScope|identifier scope/i.test(body)) return;
  warn(
    `${rel}/spec.md declares identifierScope: public on ${publicParties
      .map((p) => `'${p.role}'`)
      .join(', ')} but the prose never discusses it. Framework 0.5.0 asks a specification ` +
      `to justify a public identifier scope: say what the task needs a recognisable, ` +
      `reusable identifier for, and what a pairwise one would break. Add a paragraph — ` +
      `Security & Privacy → Correlation is the natural home.`
  );
}

/*
 * The §8.3 standard vocabulary, read off the newest published `trust-task-error`
 * payload schema rather than typed out here.
 *
 * The codes are *defined* by that schema's `code` enum — a document carrying a
 * code the declared version does not list will not validate — so the schema is
 * the only copy that cannot be wrong. A hard-coded list in this file would be a
 * fourth hand-maintained taxonomy of exactly the kind the callouts at the top of
 * CLAUDE.md exist to warn about: it would go stale the moment a
 * `trust-task-error/0.6` adds a code, and the staleness would present as a lint
 * that quietly stops catching one shadow rather than as a failure.
 *
 * "Newest" is the highest version directory under specs/trust-task-error/, by
 * numeric MAJOR then MINOR. Older versions are ignored on purpose: 0.1's frozen
 * snake_case vocabulary is a superseded spelling (see the exemption reasoning in
 * scripts/lib/error-code-casing.mjs), not part of the current set.
 */
let standardErrorCodesCache = null;
function standardErrorCodes() {
  if (standardErrorCodesCache) return standardErrorCodesCache;
  const dir = path.join(SPECS_DIR, 'trust-task-error');
  const versions = fs.existsSync(dir)
    ? fs
        .readdirSync(dir, { withFileTypes: true })
        .filter((e) => e.isDirectory() && /^\d+\.\d+$/.test(e.name))
        .map((e) => e.name)
        .sort((a, b) => {
          const [aM, am] = a.split('.').map(Number);
          const [bM, bm] = b.split('.').map(Number);
          return aM - bM || am - bm;
        })
    : [];
  const newest = versions[versions.length - 1];
  const codes = new Set();
  if (newest) {
    const schemaPath = path.join(dir, newest, 'payload.schema.json');
    if (fs.existsSync(schemaPath)) {
      try {
        const code = readJson(schemaPath)?.properties?.code;
        for (const branch of [code, ...(code?.anyOf || [])]) {
          for (const v of branch?.enum || []) if (typeof v === 'string') codes.add(v);
        }
      } catch (e) {
        fail(
          `trust-task-error/${newest}/payload.schema.json`,
          `could not be read to derive the §8.3 standard code set: ${e.message}`
        );
      }
    }
  }
  if (codes.size === 0) {
    // Never fail open. A lint that silently checks nothing is worse than one
    // that is loud about being unable to run.
    fail(
      'specs/trust-task-error',
      `no published version yields a §8.3 standard code enum from its ` +
        `payload.schema.json (looked at '${newest || 'nothing'}'). The §8.5 ` +
        `anti-shadowing check derives its vocabulary from that enum and cannot ` +
        `run without it.`
    );
  }
  standardErrorCodesCache = { codes, version: newest };
  return standardErrorCodesCache;
}

// Counters for the §8.5 anti-shadowing summary line, accumulated across specs.
const shadowLint = { declared: 0, conforming: 0, frozen: 0, frozenSpecs: new Set(), offending: 0 };

const freeTextLint = { bounded: 0, frozen: 0, frozenSpecs: new Set(), offending: 0 };

/**
 * SPEC §7.3 item 19 — a free-text member MUST declare a `maxLength`.
 *
 * A free-text member is the one place in a Trust Task document where the schema
 * constrains the shape and nothing constrains the content. It is where personal
 * data arrives in a task declaring it ingests none, where a secret arrives
 * pasted by someone asked for a reason, and where instructions addressed to a
 * downstream reader arrive in a field the specification took for a comment. It
 * is also unbounded wire cost: §10.2 puts the limit at the transport layer,
 * which is the right defence and the wrong place to pick the number, since one
 * figure there covers every task the consumer implements.
 *
 * The corpus was brought to zero violations by #296 and #301, which bounded 112
 * members. Nothing then stopped the next spec from reintroducing one, and the
 * count that gets quoted in a backlog is not a check. This is the ratchet, on
 * the same reasoning as `checkIssuedAtFloor`: a floor with no gate is a warning
 * wearing a different hat.
 *
 * Detection is by member NAME against the free-text vocabulary below, and only
 * for a string with no closed shape — a member carrying `enum`, `const`,
 * `format`, `pattern`, `$ref` or `contentEncoding` is constrained by that, not
 * free text. Naming rather than inferring is deliberate: `did`, `contextId` and
 * `cursor` are unbounded strings too, but bounding an identifier is §10.2's job
 * at the transport, and a lint that demanded `maxLength` on all 863 of them
 * would be ignored, which is how the last backlog got its size.
 *
 * `retired` specs are exempt. §5.3 freezes a retired specification's schema at
 * the moment of retirement, and adding `maxLength` narrows which documents
 * validate — a breaking change §5.2 would require a MAJOR increment for, on a
 * spec that is terminal and can take none. All 17 remaining unbounded members
 * sit in retired specs whose live successors already bound them
 * (`confirm/request/0.1` → `task-consent/request/0.1`, which bounds `note` at
 * 500 and `reason` at 1024). Counted and reported, never failed: exempt, not
 * debt.
 */
const FREE_TEXT_MEMBER =
  /^(note|notes|reason|rationale|description|message|comment|comments|summary|justification|text|body|remark|remarks|explanation|purpose|memo|label|caption|title)$/i;

function checkFreeTextBounds(meta, schema, rel) {
  const frozen = meta.status === 'retired';
  const offenders = [];

  const visit = (node, seenNodes) => {
    if (!node || typeof node !== 'object' || seenNodes.has(node)) return;
    seenNodes.add(node);
    for (const [name, prop] of Object.entries(node.properties || {})) {
      if (prop && typeof prop === 'object') {
        const type = prop.type;
        const isString = type === 'string' || (Array.isArray(type) && type.includes('string'));
        const closed = ['enum', 'const', 'format', 'pattern', '$ref', 'contentEncoding'].some(
          (k) => k in prop
        );
        if (isString && !closed && FREE_TEXT_MEMBER.test(name) && !('maxLength' in prop)) {
          offenders.push(name);
        }
        visit(prop, seenNodes);
      }
    }
    for (const key of ['items', 'additionalProperties', 'then', 'else', 'not']) {
      if (node[key] && typeof node[key] === 'object') visit(node[key], seenNodes);
    }
    for (const key of ['allOf', 'anyOf', 'oneOf', 'prefixItems']) {
      for (const sub of node[key] || []) visit(sub, seenNodes);
    }
    for (const sub of Object.values(node.$defs || node.definitions || {})) visit(sub, seenNodes);
  };
  visit(schema, new Set());

  if (!offenders.length) {
    freeTextLint.bounded++;
    return;
  }
  if (frozen) {
    freeTextLint.frozen += offenders.length;
    freeTextLint.frozenSpecs.add(rel);
    return;
  }
  freeTextLint.offending += offenders.length;
  const list = [...new Set(offenders)].map((n) => `'${n}'`).join(', ');
  fail(
    `${rel}/payload.schema.json`,
    `free-text member(s) ${list} declare no 'maxLength'. SPEC §7.3 item 19 ` +
      `requires one on any free-text member, because it is the only place in a ` +
      `document where the schema fixes the shape and nothing fixes the content ` +
      `— so it is where personal data reaches a task declaring it ingests none, ` +
      `where a pasted secret arrives, and where text addressed to a downstream ` +
      `reader arrives in a field this spec took for a comment. It is also ` +
      `unbounded wire cost; §10.2's transport limit is the right defence but ` +
      `the wrong place to pick this number. Add a 'maxLength', and prefer ` +
      `making the member OPTIONAL. 'task-consent/request/0.1' is the pattern: ` +
      `'note' bounded at 500, optional, attributed to its author on every ` +
      `surface, declared explicitly untrusted. Where the value carries meaning ` +
      `an enumeration should carry, prefer a closed 'enum' plus one bounded ` +
      `optional note.`
  );
}

function checkErrorCodeNamespaces(meta, rel) {
  const slug = meta.slug;
  if (typeof slug !== 'string') return;

  // The emitting slug plus each proper path prefix of it.
  const segments = slug.split('/');
  const permitted = new Set(
    segments.map((_, i) => segments.slice(0, i + 1).join('/'))
  );

  const { codes: standard, version: errorSpecVersion } = standardErrorCodes();

  for (const entry of meta.errorCodes || []) {
    const code = entry?.code;
    if (typeof code !== 'string') continue;
    const colon = code.lastIndexOf(':');
    if (colon < 0) continue; // grammar failure — the meta schema reports it
    const namespace = code.slice(0, colon);
    const local = code.slice(colon + 1);
    shadowLint.declared++;

    // ── SPEC §8.5, final sentence of the namespacing paragraph ──────────────
    // "Extended codes MUST NOT shadow any code listed in §8.3."
    //
    // The namespace rule above asks *where* a code may be rooted; this asks
    // whether the local part is a name the framework has already taken. A
    // declaration like `audit/verify:permissionDenied` passes the first test and
    // still breaks interoperability in both directions: a consumer switching on
    // the standard `permissionDenied` never matches it, and §8.5's fallback
    // rule maps the unrecognized extended code to `taskFailed` — a strictly
    // worse signal than the standard code it duplicates. The fix is to delete
    // the declaration and emit the standard code, which every conforming
    // consumer already recognizes (§8.3); it is never to re-case or re-spell it.
    if (standard.has(local)) {
      // Frozen by §5.3 — a retired spec's declarations are kept so already-issued
      // documents stay verifiable, and `retired` is terminal, so there is no
      // conforming version increment in which to correct one. Counted, never
      // failed, on the same reasoning as the two casing lints.
      if (meta.status === 'retired') {
        shadowLint.frozen++;
        shadowLint.frozenSpecs.add(rel);
        continue;
      }
      shadowLint.offending++;
      fail(
        `${rel}/spec.md`,
        `errorCodes['${code}'] has a local part '${local}' that shadows the ` +
          `framework standard code '${local}' (SPEC §8.3, as published by ` +
          `trust-task-error/${errorSpecVersion}). SPEC §8.5 states that extended ` +
          `codes MUST NOT shadow any code listed in §8.3. Delete the declaration ` +
          `and emit the bare '${local}' instead — every conforming consumer ` +
          `already recognizes it (§8.3), whereas the namespaced duplicate is an ` +
          `unrecognized extended code that §8.5 tells a consumer to degrade to ` +
          `'taskFailed'. Do not re-case or re-spell the local part: the problem ` +
          `is the duplicate name, not its spelling. Leaving errorCodes empty is ` +
          `fine — 'errorCodes: []' is what a spec with no task-specific codes ` +
          `declares. At draft status §5.2 requires this fix in place, ` +
          `errata-style, with no new version.`
      );
      continue;
    }
    shadowLint.conforming++;

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
  // Derived from disk, never hand-listed: a hard-coded list silently stops
  // covering the newest framework version the moment one lands, and the skip
  // below is indistinguishable from a clean pass. That is exactly how every
  // framework 0.4 example went unvalidated (#254).
  const frameworkDir = path.join(SPECS_DIR, '_framework');
  const frameworkVersions = fs.existsSync(frameworkDir)
    ? fs
        .readdirSync(frameworkDir, { withFileTypes: true })
        .filter((e) => e.isDirectory())
        .map((e) => e.name)
        .sort()
    : [];
  for (const version of frameworkVersions) {
    const p = path.join(frameworkDir, version, 'trust-task.schema.json');
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
      targetByTypePrefix.set(
        `https://trusttasks.org/spec/${slug}/${version}`,
        frameworkMinor(data.targetFrameworkVersion)
      );
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
  const unvalidatedTargets = new Map();
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
      if (!validate) {
        // Visible, not silent: a target with no compiled envelope schema means
        // those documents go unchecked, which must not read as "validated fine".
        unvalidatedTargets.set(target, (unvalidatedTargets.get(target) ?? 0) + 1);
        continue;
      }

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
  for (const [target, count] of [...unvalidatedTargets].sort()) {
    warn(
      `${count} example document(s) target framework ${target}, for which no envelope schema ` +
        `compiled — add specs/_framework/${target}/trust-task.schema.json, or correct the ` +
        `targetFrameworkVersion that names it. These documents were NOT validated`
    );
  }
  console.log(`  validated ${checked} example documents against the framework envelope schema`);
}

/*
 * `targetFrameworkVersion` accepts both `MAJOR.MINOR` (what all 349 published
 * specs declare) and `MAJOR.MINOR.PATCH` (what the canonical framework spec
 * repo, trustoverip/dtgwg-trust-tasks-spec, now normatively requires). The
 * envelope schemas on disk are keyed by `MAJOR.MINOR` — `specs/_framework/0.4/`
 * — because a patch release does not change the envelope. Truncating here is
 * what stops a spec written to the canonical text from landing in the
 * "no envelope schema compiled, NOT validated" bucket, which is a silent loss
 * of coverage dressed up as a warning nobody reads.
 */
function frameworkMinor(v) {
  const m = /^(\d+\.\d+)(?:\.\d+)?$/.exec(String(v));
  return m ? m[1] : String(v);
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
  // A closed object whose `required` names a member it does not declare can
  // never validate: the member must be present, and `additionalProperties:
  // false` forbids it. `provision/integration/0.3` shipped exactly that — a
  // rename to `digestMultibase` that left `digest` behind in `required` — and
  // nothing caught it, because every existing check looks at one keyword at a
  // time. The codegen then made it worse, emitting a required, untyped
  // `digest: serde_json::Value` from a `required` entry with no schema to read.
  for (const [label, sub] of [['', schema], ['$defs.Response: ', schema.$defs?.Response]]) {
    if (!sub || !Array.isArray(sub.required)) continue;
    // Only meaningful for an object closed by `additionalProperties: false`.
    // Under `unevaluatedProperties` an `allOf` branch may supply the member.
    if (sub.additionalProperties !== false) continue;
    const declared = new Set(Object.keys(sub.properties || {}));
    for (const name of sub.required) {
      if (!declared.has(name)) {
        fail(
          `${slug}/${version}/payload.schema.json`,
          `${label}required names \`${name}\`, which is not in \`properties\`, and ` +
            `additionalProperties is false — no document can satisfy this schema`,
        );
      }
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

/* ---------- Derived front matter: authors, keywords ----------
 *
 * Both were `required`, and both had stopped carrying information. `authors`
 * was byte-identical in 345 of 349 specs; 46% of declared keywords were
 * literally a slug segment or the spec's own category. A required field whose
 * value is the same everywhere is not metadata, it is a toll — every author
 * pays it and no reader learns anything.
 *
 * So they became OPTIONAL in the meta-schema and are derived when omitted. The
 * derivation has to be good enough that omitting is the *right* default rather
 * than the lazy one, which means it has to reach the same answer a
 * conscientious author would have typed:
 *
 *   authors  — from CODEOWNERS, which already names who reviews each slug and
 *              is kept current because GitHub enforces it. Falling back to the
 *              spec folder's git history, which is who actually wrote it.
 *   keywords — from the slug's own segments plus the category, which is exactly
 *              what 46% of hand-written keyword lists already were.
 *
 * A spec that declares either wins outright: the derivation exists to remove
 * ceremony, not to overrule an author who has something real to say.
 */

let codeownersRules = null;

/* CODEOWNERS, reduced to what this needs: (pattern, owners) in file order, last
 * match winning, as GitHub does. Only path-prefix patterns are honoured — the
 * file uses nothing else, and implementing the full gitignore grammar to serve
 * a display string would be a poor trade. */
function loadCodeowners() {
  if (codeownersRules) return codeownersRules;
  codeownersRules = [];
  for (const candidate of ['.github/CODEOWNERS', 'CODEOWNERS', 'docs/CODEOWNERS']) {
    const p = path.join(ROOT, candidate);
    if (!fs.existsSync(p)) continue;
    for (const line of fs.readFileSync(p, 'utf8').split('\n')) {
      const stripped = line.replace(/#.*$/, '').trim();
      if (!stripped) continue;
      const [pattern, ...owners] = stripped.split(/\s+/);
      const handles = owners.filter((o) => o.startsWith('@'));
      if (!handles.length) continue;
      codeownersRules.push({ pattern, owners: handles });
    }
    break;
  }
  return codeownersRules;
}

function codeownersFor(relPath) {
  let match = null;
  for (const rule of loadCodeowners()) {
    const p = rule.pattern;
    if (p === '*') { match = rule; continue; }
    const needle = p.startsWith('/') ? p.slice(1) : p;
    if (relPath === needle || relPath.startsWith(needle.endsWith('/') ? needle : `${needle}/`)) {
      match = rule;
    }
  }
  return match ? match.owners : [];
}

/* A GitHub handle rendered in the same "Name (url)" shape the declared authors
 * use, so the spec page's existing renderer links it without a special case. */
function handleToAuthor(handle) {
  const login = handle.replace(/^@/, '');
  // A team handle (@org/team) has no user profile page; link the org.
  if (login.includes('/')) {
    const [org, team] = login.split('/');
    return `${login} (https://github.com/orgs/${org}/teams/${team})`;
  }
  return `${login} (https://github.com/${login})`;
}

function gitAuthorsFor(dirRel) {
  try {
    const out = execSync(`git log --format=%an -- "${dirRel}"`, { cwd: ROOT, encoding: 'utf8' }).trim();
    if (!out) return [];
    return [...new Set(out.split('\n').map((s) => s.trim()).filter(Boolean))];
  } catch {
    return [];
  }
}

function deriveAuthors(meta, entry) {
  if (Array.isArray(meta.authors) && meta.authors.length) {
    return { authors: meta.authors, source: 'declared' };
  }
  const specRel = path.relative(ROOT, entry.specPath).split(path.sep).join('/');
  const owners = codeownersFor(specRel);
  if (owners.length) return { authors: owners.map(handleToAuthor), source: 'codeowners' };
  const fromGit = gitAuthorsFor(path.relative(ROOT, entry.dir));
  if (fromGit.length) return { authors: fromGit, source: 'git' };
  // Never empty: TT_STATS iterates authors unconditionally, and a spec with no
  // attribution at all should read as the working group's rather than as a gap.
  return { authors: ['Trust Tasks Task Force'], source: 'fallback' };
}

function deriveKeywords(meta) {
  if (Array.isArray(meta.keywords) && meta.keywords.length) {
    return { keywords: meta.keywords, source: 'declared' };
  }
  const fromSlug = String(meta.slug || '').split('/').flatMap((seg) => seg.split('-'));
  const derived = [...new Set([...fromSlug, meta.category].filter(Boolean))];
  return { keywords: derived.length ? derived : [meta.slug], source: 'slug+category' };
}

function buildTask(entry, meta, schema, uses) {
  const hasResponse = !!(schema && schema.$defs && schema.$defs.Response);
  const { authors, source: authorsSource } = deriveAuthors(meta, entry);
  const { keywords, source: keywordsSource } = deriveKeywords(meta);
  return {
    id: meta.slug,
    slug: meta.slug,
    title: meta.title,
    summary: meta.summary,
    category: meta.category,
    keywords,
    // Which way each of these was arrived at, so a reader (and a future audit
    // of this decision) can tell an author's deliberate choice from the
    // build's default without diffing the front matter.
    keywordsSource,
    status: meta.status,
    version: meta.version,
    targetFrameworkVersion: meta.targetFrameworkVersion,
    created: firstAdded(path.relative(ROOT, entry.dir)),
    updated: lastModified(path.relative(ROOT, entry.dir)),
    authors,
    authorsSource,
    parties: meta.parties.map((p) => p.role),
    partiesDetail: meta.parties,
    proofRequirement: meta.proofRequirement,
    // §7.3 item 17. Absent where the spec has not declared one, which is not
    // the same as declaring the §4.2 baseline — see resolveIssuedAtRequirement.
    issuedAtRequirement: meta.issuedAtRequirement,
    sideEffects: meta.sideEffects || null,
    // Framework 0.5.0's third descriptive dimension. Carried even when absent
    // (as null) for the same reason knownImplementations is: a reader needs to
    // tell "not declared" from "declared transient", and a field that only
    // appears when set makes those look alike to a machine consumer.
    retention: meta.retention || null,
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

  if (updateSpAllowlist) {
    const failing = discoverSpecs()
      .filter((e) => !assessSecurityPrivacy(fs.readFileSync(e.specPath, 'utf8')).conforms)
      .map((e) => e.rel);
    console.log(`  wrote scripts/lib/security-privacy-allowlist.json (${writeAllowlist(failing)} spec(s))`);
    return;
  }

  const validate = loadMetaValidator();
  checkCategoryTaxonomy();
  checkBindingRegistry();
  checkExampleDocuments();
  const entries = discoverSpecs();
  if (entries.length === 0) {
    console.warn('No specs found under specs/<slug>/<version>/.');
  }
  checkSecurityPrivacySections(entries, { warn, fail, log: console.log });
  const errorCodeCasing = createErrorCodeCasingLint({ fail, log: console.log });

  // Discover shared/framework/method-extension schemas first so we can
  // resolve $refs from payload schemas against them when building tasks.
  const sharedEntries = discoverSharedSchemas();
  const sharedRecords = sharedEntries.map(buildSharedRecord);
  const sharedByPath = new Map(sharedEntries.map((e) => [e.filePath, sharedRecords[sharedEntries.indexOf(e)]]));
  const sharedBySlug = new Map(sharedRecords.map((r) => [r.slug, r]));

  const tasks = [];
  const seen = new Set();
  let disclosureFloorOffenders = 0;
  // SPEC §7.3 item 17 — see checkIssuedAtFloor. The count is still reported as
  // one line rather than one message per spec, but it is now a line that reads
  // N/N: the undeclared case fails the build, so the only way the numerator and
  // denominator part is by adding a consequential spec that declares nothing.
  // `issuedAtFloorFrozen` holds the retired specs §5.3 exempts.
  let consequentialSpecs = 0;
  let issuedAtFloorFrozen = 0;
  const issuedAtFloorUnmet = [];

  for (const entry of entries) {
    const { slug, version, specPath, dir } = entry;
    const rel = `${slug}/${version}`;
    const src = fs.readFileSync(specPath, 'utf8');
    const { data: meta, body } = splitFrontMatter(src);
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
    errorCodeCasing.check(meta, rel);
    checkIdentifierScopeJustification(meta, body, rel);
    const idKey = `${meta.slug}@${meta.version}`;
    if (seen.has(idKey)) {
      fail(rel, `duplicate slug+version ${idKey}`);
      continue;
    }
    seen.add(idKey);
    const schema = checkPayloadSchema(slug, version, dir);
    if (!schema) continue;
    checkProofFloor(meta, rel, Boolean(schema.$defs?.Response));
    checkFreeTextBounds(meta, schema, rel);
    const freshness = checkIssuedAtFloor(meta, rel, Boolean(schema.$defs?.Response));
    if (freshness.consequential) {
      // A frozen spec is out of the ratio entirely — including the weaker-than-
      // REQUIRED case, which `checkIssuedAtFloor` has already failed on its own.
      if (freshness.frozen) {
        issuedAtFloorFrozen++;
      } else {
        consequentialSpecs++;
        if (freshness.unmet) issuedAtFloorUnmet.push(rel);
      }
    }
    const payloadSchemaPath = path.join(dir, 'payload.schema.json');
    // The machine-checkable half of the same question checkProofFloor asks:
    // checkProofFloor trusts `exposure.discloses` and derives the proof floor
    // from it; this asks whether the declaration matches the schema it describes.
    if (checkDisclosureFloor(meta, schema, payloadSchemaPath, rel, { warn, fail })) {
      disclosureFloorOffenders++;
    }
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

  console.log(
    `  Exposure floor: ${disclosureFloorOffenders} spec(s) declare exposure.discloses: none ` +
      `while returning released material` +
      `${disclosureFloorOffenders ? ' (warning — set TT_STRICT_DISCLOSURE=1 to fail the build)' : ''}`
  );
  console.log(
    `  Freshness floor (§7.3 item 17): ${consequentialSpecs - issuedAtFloorUnmet.length}/` +
      `${consequentialSpecs} live consequential spec(s) declare issuedAtRequirement: REQUIRED; ` +
      `${issuedAtFloorFrozen} frozen in retired spec(s) (§5.3 — exempt, not debt)` +
      `${
        issuedAtFloorUnmet.length
          ? ` — ${issuedAtFloorUnmet.length} undeclared, e.g. ${issuedAtFloorUnmet
              .slice(0, 3)
              .join(', ')} (error)`
          : ''
      }`
  );
  for (const rel of issuedAtFloorUnmet) {
    fail(
      `${rel}/spec.md`,
      `consequential Trust Task with no issuedAtRequirement declaration. SPEC §7.3 item 17 makes ` +
        `issuedAt REQUIRED for a specification defining a consequential Trust Task — the ` +
        `duplicate-execution protection of §7.2 item 11 is implementable only over a window, and ` +
        `a document with no issuedAt cannot be placed in one. Add an issuedAtRequirement block ` +
        `declaring REQUIRED with a rationale saying why this task needs a bounded acceptance ` +
        `window, or correct the item 13/14 declarations if this task is not in fact consequential.`
    );
  }
  errorCodeCasing.report();
  console.log(
    `  Error code shadowing: ${shadowLint.conforming}/${shadowLint.declared} extended code(s) ` +
      `avoid the §8.3 standard vocabulary (${standardErrorCodes().codes.size} code(s), derived ` +
      `from trust-task-error/${standardErrorCodes().version}); ${shadowLint.frozen} frozen in ` +
      `${shadowLint.frozenSpecs.size} retired spec(s) (§5.3 — exempt, not debt), ` +
      `${shadowLint.offending} shadowing`
  );
  console.log(
    `  Free-text bounds (§7.3 item 19): ${freeTextLint.bounded} spec(s) carry no unbounded ` +
      `free-text member; ${freeTextLint.frozen} member(s) frozen in ` +
      `${freeTextLint.frozenSpecs.size} retired spec(s) (§5.3 — exempt, not debt), ` +
      `${freeTextLint.offending} unbounded`
  );
  // The rule-4 lint above reads `errorCodes` declarations, which is where an
  // *extended* code is defined. A *standard* code (§8.3) is declared nowhere and
  // only ever referenced, so its rule-2 MUST has to be swept over the text.
  checkStandardErrorCodeCasing(entries, { fs, path, specsDir: SPECS_DIR }, { fail, log: console.log });

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
  //
  // Root-relative links (`/SPEC.md#...`) are checked too, and resolve from the
  // REPOSITORY ROOT rather than from the version directory. They exist because
  // the `../` count in a link to the framework spec is a function of slug depth
  // — which is why 731 of them had to be written four different ways, and why
  // the drift above happened at all. `/SPEC.md#anchor` is the same string from
  // every depth, and the website resolves it onto the rendered /specification
  // route. Checking them is what keeps the new form from being an unverified one.
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
    for (const m of prose.matchAll(/\]\((\/[^)#\s]+?)(#[^)\s]*)?\)/g)) {
      if (!fs.existsSync(path.join(ROOT, m[1]))) {
        fail(
          `${slug}/${version}/spec.md`,
          `root-relative link '${m[1]}' does not resolve — these resolve from the ` +
            `repository root, not from the spec folder`
        );
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
