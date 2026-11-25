/**
 * The `uncheckedRules` front-matter check.
 *
 * Some rules a specification states cannot be written in its payload schema in
 * a form every generator accepts. The Rust generator (typify) panics on
 * `if`/`then`/`else` and rejects `contains`, so a rule such as "General Context
 * content needs `value`" or "`type` includes `ContextEntry`" lives in a member's
 * `description` and in the prose, and no validator checks it. Nothing said so
 * in a form a reader or a tool could find: a consumer who validated a document
 * against the schema had no way to learn which rules were left to it.
 *
 * `uncheckedRules` declares them. Each entry names the schema location the rule
 * applies to (`member`), states the rule (`rule`) and cites where it comes from
 * (`source`). This module:
 *
 *   - checks that each `member` resolves in the spec's own payload schema, and
 *     fails the build on one that does not, so the list cannot silently drift
 *     from a schema that is restructured or renamed; and
 *   - prints one informational line per declaring spec, listing its rules. A
 *     declared rule is not a failure: it is disclosure.
 *
 * ── What `member` is ────────────────────────────────────────────────────────
 *
 * A JSON Pointer (RFC 6901) into `payload.schema.json`, written from its root:
 * `/properties/grantId`, `/$defs/Response/properties/entries/items/anyOf/1`.
 * Resolution follows `$ref` wherever the next segment is not on the node
 * itself, so a pointer can walk into a shared schema in another directory:
 * `/$defs/Response/properties/entries/items/anyOf/1/properties/type` reaches
 * through the `$ref` at `anyOf/1` into `_shared/…/context-entry.schema.json`.
 * A `$ref` fragment may be a JSON Pointer or a plain-name `$anchor`.
 *
 * The pointer names a location in the schema, not in an instance. That is
 * deliberate: it is what the build can resolve, and it is unambiguous where an
 * instance path is not (`entries/*` would not say which `anyOf` branch).
 *
 * The library validators do not yet surface these rules; that is planned
 * follow-up work (see CONTRIBUTING-SPECS.md).
 */
import fs from 'node:fs';
import path from 'node:path';

const MAX_REF_HOPS = 32;

/** Decode one RFC 6901 reference token. */
function unescapeToken(token) {
  return token.replace(/~1/g, '/').replace(/~0/g, '~');
}

/** Split an RFC 6901 pointer (`''` or `/a/b`) into decoded tokens, or null. */
export function pointerTokens(pointer) {
  if (pointer === '') return [];
  if (typeof pointer !== 'string' || !pointer.startsWith('/')) return null;
  return pointer.slice(1).split('/').map(unescapeToken);
}

function findAnchor(node, anchor) {
  if (!node || typeof node !== 'object') return undefined;
  if (node.$anchor === anchor) return node;
  for (const child of Object.values(node)) {
    const hit = findAnchor(child, anchor);
    if (hit !== undefined) return hit;
  }
  return undefined;
}

/**
 * Resolve a `$ref` relative to the file that holds it. Returns
 * `{ node, file }`, or `{ error }`.
 */
function followRef(ref, file, loadJson) {
  const hash = ref.indexOf('#');
  const target = hash === -1 ? ref : ref.slice(0, hash);
  const fragment = hash === -1 ? '' : ref.slice(hash + 1);
  if (/^[a-z][a-z0-9+.-]*:/i.test(target)) {
    return { error: `$ref '${ref}' is absolute; only relative and local $refs are followed` };
  }
  const nextFile = target ? path.resolve(path.dirname(file), target) : file;
  const doc = loadJson(nextFile);
  if (doc === undefined) return { error: `$ref '${ref}' names a file that does not exist` };
  if (fragment === '') return { node: doc, file: nextFile };
  if (fragment.startsWith('/')) {
    let node = doc;
    for (const token of pointerTokens(fragment)) {
      if (!node || typeof node !== 'object' || !(token in node)) {
        return { error: `$ref '${ref}' does not resolve` };
      }
      node = node[token];
    }
    return { node, file: nextFile };
  }
  const node = findAnchor(doc, fragment);
  return node === undefined ? { error: `$ref '${ref}' names no $anchor '${fragment}'` } : { node, file: nextFile };
}

/** A JSON loader that caches by absolute path and returns undefined for a missing file. */
export function makeJsonLoader() {
  const cache = new Map();
  return (file) => {
    if (!cache.has(file)) {
      cache.set(file, fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : undefined);
    }
    return cache.get(file);
  };
}

/**
 * Resolve `pointer` in the schema at `schemaFile`, following `$ref` where a
 * segment is not on the node itself. Returns `{ ok: true, node, file }` or
 * `{ ok: false, reason }`.
 */
export function resolveSchemaPointer(schemaFile, pointer, loadJson = makeJsonLoader()) {
  const tokens = pointerTokens(pointer);
  if (tokens === null) return { ok: false, reason: `'${pointer}' is not a JSON Pointer (it must start with '/')` };
  let file = schemaFile;
  let node = loadJson(file);
  if (node === undefined) return { ok: false, reason: `${path.basename(file)} does not exist` };
  const walked = [];
  for (const token of tokens) {
    let hops = 0;
    while (node && typeof node === 'object' && !(token in node) && typeof node.$ref === 'string') {
      if (++hops > MAX_REF_HOPS) return { ok: false, reason: `$ref chain too deep at '/${walked.join('/')}'` };
      const next = followRef(node.$ref, file, loadJson);
      if (next.error) return { ok: false, reason: `at '/${walked.join('/')}': ${next.error}` };
      ({ node, file } = next);
    }
    if (!node || typeof node !== 'object' || !(token in node)) {
      return { ok: false, reason: `no '${token}' at '/${walked.join('/')}'` };
    }
    node = node[token];
    walked.push(token);
  }
  if (node === null || (typeof node !== 'object' && typeof node !== 'boolean')) {
    return { ok: false, reason: 'it names a keyword value, not a schema' };
  }
  return { ok: true, node, file };
}

/**
 * Check one spec's `uncheckedRules`. `rel` is the spec's `<slug>/<version>`,
 * used in messages. Fails on every `member` that does not resolve; logs one
 * informational line per rule. Returns the number of rules declared.
 */
export function checkUncheckedRules(meta, payloadSchemaPath, rel, { fail, log, loadJson = makeJsonLoader() }) {
  const rules = Array.isArray(meta?.uncheckedRules) ? meta.uncheckedRules : [];
  if (rules.length === 0) return 0;
  log(`  Unchecked rules (informational): ${rel} declares ${rules.length} rule(s) its payload schema does not check:`);
  rules.forEach((entry, i) => {
    const resolved = resolveSchemaPointer(payloadSchemaPath, entry.member, loadJson);
    if (!resolved.ok) {
      fail(
        `${rel}/spec.md`,
        `uncheckedRules[${i}].member '${entry.member}' does not resolve in payload.schema.json: ` +
          `${resolved.reason}. Point it at the schema location the rule applies to, following $refs.`
      );
    }
    log(`    - ${entry.member}: ${entry.rule} (${entry.source})`);
  });
  return rules.length;
}
