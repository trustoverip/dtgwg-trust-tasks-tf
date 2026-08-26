/**
 * The `exposure.discloses` floor lint.
 *
 * SPEC §7.3 item 14 defines `discloses` as "the sensitivity of data the task
 * returns to the caller", with `secret` reserved for confidential material the
 * caller retains — "released credential material" being the named example. That
 * declaration is not decoration: §2 folds `exposure.discloses === 'secret'` into
 * the definition of a *consequential Trust Task*, `checkProofFloor` derives the
 * response's proof floor from it, and a delegated-execution consumer gates human
 * approval on it without reading a line of the prose. A task that hands back a
 * signed credential while declaring `discloses: none` therefore under-gates a
 * real disclosure everywhere at once, and nothing catches it, because the
 * declaration and the schema are written in different files by different halves
 * of the same brain.
 *
 * Five specs shipped that way — `vta/credentials/issue` 0.1 and 0.2 and
 * `vtc/endorsements/issue/0.1` each returning a REQUIRED `credential`,
 * `vtc/relationships/request` 0.1 and 0.2 each returning a REQUIRED `vrc` — while
 * the sibling `vtc/invitations/issue/0.1`, whose response has the same shape,
 * declared `secret`. The inconsistency between siblings is what makes this
 * machine-checkable rather than a matter of judgement.
 *
 * ── The rule ────────────────────────────────────────────────────────────────
 *
 * A response schema that reaches a member named `credential`, `vc`, `vp`, `vrc`
 * or `value` — through its own `properties`, or through any `$ref`, including
 * one into a shared schema in another directory — cannot declare
 * `discloses: none`. It may be `metadata` or `secret`; which of the two is a
 * judgement the lint does not attempt, because `value` in particular covers both
 * a free-text memory item and a config echo. What it can tell is that "an
 * acknowledgement or a determination only" is not a description of a response
 * carrying any of those members, and `none` means exactly that.
 *
 * The name list is deliberately short and deliberately literal. Every entry is a
 * member name the corpus actually uses for released material; a wider net
 * (`token`, `key`, `secret`) would catch the request half of tasks that ingest
 * such material, which `exposure.ingests` describes and this lint does not.
 *
 * ── Why this ships as a warning, and why the allowlist is empty ─────────────
 *
 * Shipped in the same shape as the Security & Privacy lint (see
 * `security-privacy.mjs`): a warning with an enumerated allowlist, so pre-existing
 * debt is countable rather than blocking. The difference is that the corpus is
 * **clean** — the five offenders above were corrected in the same change that
 * added this file, and no other published spec trips it. `ALLOWLIST` is therefore
 * empty, and a maintainer who wants CI to hold the line can either set
 * `TT_STRICT_DISCLOSURE=1` or change the one `warn` below to `fail`; neither
 * requires rewriting a single spec first.
 */
import fs from 'node:fs';
import path from 'node:path';

export const STRICT_ENV_VAR = 'TT_STRICT_DISCLOSURE';

/**
 * Member names whose presence in a response is incompatible with
 * `discloses: none`. Compared case-insensitively.
 */
export const RELEASED_MEMBER_NAMES = ['credential', 'vc', 'vp', 'vrc', 'value'];

/**
 * Specs permitted to declare `discloses: none` despite matching. Empty: every
 * spec in the corpus conforms. An entry here is DEBT, not permission, and is
 * expected to name the spec and carry a reason in a comment.
 */
export const ALLOWLIST = new Set();

const NAMES = new Set(RELEASED_MEMBER_NAMES.map((n) => n.toLowerCase()));
const MAX_DEPTH = 24;

const fileCache = new Map();
function loadJson(p) {
  if (!fileCache.has(p)) {
    try {
      fileCache.set(p, JSON.parse(fs.readFileSync(p, 'utf8')));
    } catch {
      fileCache.set(p, null);
    }
  }
  return fileCache.get(p);
}

/** Resolve an RFC 6901 JSON Pointer fragment (`/$defs/Foo`) against a document. */
function resolvePointer(doc, fragment) {
  const parts = fragment.replace(/^#/, '').split('/').filter(Boolean);
  let node = doc;
  for (const raw of parts) {
    const key = raw.replace(/~1/g, '/').replace(/~0/g, '~');
    if (node && typeof node === 'object' && key in node) node = node[key];
    else return null;
  }
  return node;
}

/** Resolve a plain-name fragment (`#issuedCredentialBase`) by `$anchor`. */
function resolveAnchor(doc, anchor) {
  let found = null;
  const walk = (node) => {
    if (found || !node || typeof node !== 'object') return;
    if (node.$anchor === anchor) {
      found = node;
      return;
    }
    for (const v of Object.values(node)) walk(v);
  };
  walk(doc);
  return found;
}

/**
 * Collect the released-material member names a schema node reaches, following
 * `$ref` across files. `seen` keys on resolved `file#fragment` so a cyclic or
 * diamond reference graph terminates.
 */
function collect(node, baseFile, seen, hits, depth) {
  if (!node || typeof node !== 'object' || depth > MAX_DEPTH) return;

  if (Array.isArray(node)) {
    for (const item of node) collect(item, baseFile, seen, hits, depth + 1);
    return;
  }

  if (typeof node.$ref === 'string') {
    const [file, fragment] = node.$ref.split('#');
    const target = file ? path.resolve(path.dirname(baseFile), file) : baseFile;
    const id = `${target}#${fragment || ''}`;
    if (!seen.has(id)) {
      seen.add(id);
      const doc = loadJson(target);
      if (doc) {
        const sub = !fragment
          ? doc
          : fragment.startsWith('/')
            ? resolvePointer(doc, fragment)
            : resolveAnchor(doc, fragment);
        if (sub) collect(sub, target, seen, hits, depth + 1);
      }
    }
  }

  if (node.properties && typeof node.properties === 'object') {
    for (const name of Object.keys(node.properties)) {
      if (NAMES.has(name.toLowerCase())) hits.add(name);
    }
  }

  for (const [key, value] of Object.entries(node)) {
    if (key === '$ref') continue;
    collect(value, baseFile, seen, hits, depth + 1);
  }
}

/**
 * @param {object} responseSchema the `$defs.Response` sub-schema, or null
 * @param {string} payloadSchemaPath absolute path to the owning payload.schema.json
 * @returns {string[]} the matching member names, sorted; empty when none
 */
export function releasedMembersInResponse(responseSchema, payloadSchemaPath) {
  if (!responseSchema) return [];
  const hits = new Set();
  collect(responseSchema, payloadSchemaPath, new Set(), hits, 0);
  return [...hits].sort();
}

/**
 * Warn when a spec declaring `discloses: none` returns released material.
 *
 * @param {object} meta parsed front matter
 * @param {object} schema the parsed payload.schema.json
 * @param {string} payloadSchemaPath absolute path to that file
 * @param {string} rel `<slug>/<version>`
 * @param {{warn: (msg: string) => void, fail: (loc: string, msg: string) => void}} io
 * @returns {boolean} true when the spec was reported
 */
export function checkDisclosureFloor(meta, schema, payloadSchemaPath, rel, io) {
  if (meta?.exposure?.discloses !== 'none') return false;

  const hits = releasedMembersInResponse(schema?.$defs?.Response, payloadSchemaPath);
  if (hits.length === 0) return false;
  if (ALLOWLIST.has(rel)) return false;

  const which = hits.map((h) => `\`${h}\``).join(', ');
  const msg =
    `${rel}/spec.md declares exposure.discloses: none, but its response schema reaches ` +
    `${which}. SPEC §7.3 item 14 reserves \`none\` for "an acknowledgement or a determination ` +
    `only"; a response carrying released material is \`secret\` (confidential material the ` +
    `caller retains) or at least \`metadata\`. The value feeds the *consequential Trust Task* ` +
    `definition (SPEC §2) that consumers gate execution on, so \`none\` here under-gates a ` +
    `real disclosure. Correct the declaration and add the \`rationale\` item 14 then requires, ` +
    `or — only where the member genuinely is not released material — add '${rel}' to ` +
    `ALLOWLIST in scripts/lib/disclosure-floor.mjs with a reason.`;

  if (process.env[STRICT_ENV_VAR] === '1') io.fail(`${rel}/spec.md`, msg);
  else io.warn(msg);
  return true;
}
