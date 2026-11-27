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
 *   - checks that each `member` resolves in the spec's own payload schema to a
 *     schema (not to a keyword value, a map of schemas such as `properties` or
 *     `$defs`, or an array such as `anyOf` or `required`), and fails the build
 *     on one that does not, so the list cannot silently drift from a schema
 *     that is restructured or renamed;
 *   - checks the list against the schema's own wording (the reachability
 *     cross-check below), and fails the build on a rule the schema says it
 *     does not check but the list does not declare; and
 *   - reports the declared rules: a summary line by default, one line per rule
 *     under `--verbose`. A declared rule is not a failure: it is disclosure.
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
 * ── The reachability cross-check ────────────────────────────────────────────
 *
 * A schema says a rule is unchecked in prose, in a `description`. The check
 * recognises that by a fixed set of marker phrases (`UNCHECKED_MARKERS`):
 * "not checked by the schema" and "a validator does not check it/them". It
 * walks every schema in `payload.schema.json` and every schema reachable from
 * it through `$ref` (into `_shared/` files too, but only the parts a `$ref`
 * reaches), and fails on each schema whose `description` carries a marker but
 * that no `uncheckedRules` entry resolves at or under: an entry covers a
 * marked schema when the schema lies on the entry's resolution path (the
 * schema itself, or one it passes through, `$ref` hops included).
 *
 * What it cannot see: a rule stated without a marker phrase (write the marker
 * when a description states a rule the schema does not check), or a second
 * rule in a marked description that already has one entry: it proves that
 * every marked schema has an entry, not that every rule in it does. A
 * `_shared` schema cannot declare its own unchecked rules, so each consuming
 * spec repeats the entries; an `x-uncheckedRules` keyword on the shared
 * schema, inherited by its consumers, would remove the copies, but needs every
 * generator to strip it first (future work).
 *
 * The library validators do not yet surface these rules; that is planned
 * follow-up work (see CONTRIBUTING-SPECS.md).
 */
import fs from 'node:fs';
import path from 'node:path';

export const MAX_REF_HOPS = 32;

/**
 * The phrases a `description` uses to say the schema does not check a rule it
 * states. Matching is case-insensitive. Keep this list short and the phrases
 * specific: every match must be declared in `uncheckedRules`.
 */
export const UNCHECKED_MARKERS = [/\bnot checked by the schema\b/i, /\ba validator does not check (it|them)\b/i];

// Where a JSON Schema keyword's value is itself a schema, a map of schemas or
// an array of schemas. Any other keyword's value (`type`, `required`, `enum`,
// `const`, `description`, …) is not a schema.
const SCHEMA_KEYWORDS = new Set([
  'items',
  'additionalItems',
  'additionalProperties',
  'unevaluatedItems',
  'unevaluatedProperties',
  'propertyNames',
  'contains',
  'not',
  'if',
  'then',
  'else',
  'contentSchema',
]);
const SCHEMA_MAP_KEYWORDS = new Set(['properties', 'patternProperties', '$defs', 'definitions', 'dependentSchemas']);
const SCHEMA_ARRAY_KEYWORDS = new Set(['anyOf', 'allOf', 'oneOf', 'prefixItems']);

/** Decode one RFC 6901 reference token. */
function unescapeToken(token) {
  return token.replace(/~1/g, '/').replace(/~0/g, '~');
}

/** Encode one RFC 6901 reference token. */
function escapeToken(token) {
  return String(token).replace(/~/g, '~0').replace(/\//g, '~1');
}

/** Split an RFC 6901 pointer (`''` or `/a/b`) into decoded tokens, or null. */
export function pointerTokens(pointer) {
  if (pointer === '') return [];
  if (typeof pointer !== 'string' || !pointer.startsWith('/')) return null;
  return pointer.slice(1).split('/').map(unescapeToken);
}

/**
 * Whether a pointer's tokens, read from a schema's root, end at a schema:
 * `/properties/x` and `/anyOf/0` do; `/properties`, `/$defs`, `/anyOf`,
 * `/required` and `/properties/x/type` do not. Returns null when it does, or
 * a reason when it does not. A member named like a keyword
 * (`/properties/required`) is a schema, which a test on the last token alone
 * would get wrong.
 */
export function schemaPositionError(tokens) {
  let state = 'schema';
  let last = '';
  for (const token of tokens) {
    if (state === 'schema') {
      if (SCHEMA_KEYWORDS.has(token)) state = 'schema';
      else if (SCHEMA_MAP_KEYWORDS.has(token)) state = 'map';
      else if (SCHEMA_ARRAY_KEYWORDS.has(token)) state = 'array';
      else state = 'value';
    } else if (state === 'map') {
      state = 'schema';
    } else if (state === 'array') {
      state = /^(0|[1-9][0-9]*)$/.test(token) ? 'schema' : 'value';
    }
    last = token;
  }
  if (state === 'schema') return null;
  if (state === 'map') return `it ends at '${last}', a map of schemas: name one member of it`;
  if (state === 'array') return `it ends at '${last}', an array of schemas: name one item of it`;
  return 'it names a keyword value, not a schema';
}

/**
 * Load a file through `loadJson`, turning a parse error into a reason rather
 * than an exception. Returns `{ doc }` (undefined for a missing file) or
 * `{ error }`.
 */
function load(file, loadJson) {
  try {
    return { doc: loadJson(file) };
  } catch (err) {
    return { error: `${path.basename(file)} is not valid JSON (${err.message})` };
  }
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
  const loaded = load(nextFile, loadJson);
  if (loaded.error) return { error: `$ref '${ref}': ${loaded.error}` };
  const doc = loaded.doc;
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

/**
 * A JSON loader that caches by absolute path and returns undefined for a
 * missing file. It throws on malformed JSON; the resolver reports that as a
 * reason. Node identity is stable across calls, which the cross-check relies on.
 */
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
 * Follow `node`'s own `$ref` chain to the schema it names, appending each hop
 * to `chain`. Returns `{ node, file }` or `{ error }`.
 */
function derefAll(node, file, loadJson, chain) {
  let hops = 0;
  while (node && typeof node === 'object' && typeof node.$ref === 'string') {
    if (++hops > MAX_REF_HOPS) return { error: `$ref chain longer than ${MAX_REF_HOPS} hops (a cycle?)` };
    const next = followRef(node.$ref, file, loadJson);
    if (next.error) return { error: next.error };
    ({ node, file } = next);
    chain.push(node);
  }
  return { node, file };
}

/**
 * Resolve `pointer` in the schema at `schemaFile`, following `$ref` where a
 * segment is not on the node itself. Returns `{ ok: true, node, file, chain }`
 * or `{ ok: false, reason }`. `chain` holds every schema node the resolution
 * passed through, `$ref` hops included, and then the nodes the final schema's
 * own `$ref` chain leads to: the schemas an entry at this pointer is "at or
 * under".
 */
export function resolveSchemaPointer(schemaFile, pointer, loadJson = makeJsonLoader()) {
  const tokens = pointerTokens(pointer);
  if (tokens === null) return { ok: false, reason: `'${pointer}' is not a JSON Pointer (it must start with '/')` };
  const position = schemaPositionError(tokens);
  if (position) return { ok: false, reason: position };
  let file = schemaFile;
  const loaded = load(file, loadJson);
  if (loaded.error) return { ok: false, reason: loaded.error };
  let node = loaded.doc;
  if (node === undefined) return { ok: false, reason: `${path.basename(file)} does not exist` };
  const chain = [node];
  const walked = [];
  for (const token of tokens) {
    let hops = 0;
    while (node && typeof node === 'object' && !(token in node) && typeof node.$ref === 'string') {
      if (++hops > MAX_REF_HOPS) {
        return { ok: false, reason: `$ref chain longer than ${MAX_REF_HOPS} hops at '/${walked.join('/')}' (a cycle?)` };
      }
      const next = followRef(node.$ref, file, loadJson);
      if (next.error) return { ok: false, reason: `at '/${walked.join('/')}': ${next.error}` };
      ({ node, file } = next);
      chain.push(node);
    }
    if (!node || typeof node !== 'object' || !(token in node)) {
      return { ok: false, reason: `no '${token}' at '/${walked.join('/')}'` };
    }
    node = node[token];
    walked.push(token);
    chain.push(node);
  }
  if (node === null || Array.isArray(node) || (typeof node !== 'object' && typeof node !== 'boolean')) {
    return { ok: false, reason: 'it names a keyword value, not a schema' };
  }
  const tail = derefAll(node, file, loadJson, chain);
  if (tail.error) return { ok: false, reason: `at '${pointer}': ${tail.error}` };
  return { ok: true, node, file, chain };
}

/**
 * Every schema reachable from the payload schema whose `description` carries
 * an `UNCHECKED_MARKERS` phrase. Walks the whole payload schema, and through
 * each `$ref` the parts of other files it reaches. Returns
 * `{ marked: [{ node, pointer, file, description }], errors: [reason] }`;
 * `pointer` is the shortest path found from the payload root (written through
 * `$ref`s, as `member` is), usable as a `member`. A `$ref` that does not
 * resolve ends the walk there and is listed in `errors`; the build's other
 * checks report it.
 */
export function findMarkedSchemas(payloadSchemaPath, loadJson = makeJsonLoader()) {
  const marked = [];
  const errors = [];
  const loaded = load(payloadSchemaPath, loadJson);
  if (loaded.error) return { marked, errors: [loaded.error] };
  if (loaded.doc === undefined) return { marked, errors: [`${path.basename(payloadSchemaPath)} does not exist`] };
  const seen = new Set();
  // Breadth-first, so each marked schema is reported at its shortest pointer.
  const queue = [{ node: loaded.doc, file: payloadSchemaPath, pointer: '' }];
  while (queue.length) {
    const { node, file, pointer } = queue.shift();
    if (!node || typeof node !== 'object' || Array.isArray(node) || seen.has(node)) continue;
    seen.add(node);
    if (typeof node.description === 'string' && UNCHECKED_MARKERS.some((re) => re.test(node.description))) {
      marked.push({ node, pointer, file, description: node.description });
    }
    if (typeof node.$ref === 'string') {
      const next = followRef(node.$ref, file, loadJson);
      if (next.error) errors.push(`at '${pointer}': ${next.error}`);
      else queue.push({ node: next.node, file: next.file, pointer });
    }
    for (const [key, value] of Object.entries(node)) {
      const at = `${pointer}/${escapeToken(key)}`;
      if (SCHEMA_KEYWORDS.has(key)) {
        queue.push({ node: value, file, pointer: at });
      } else if (SCHEMA_MAP_KEYWORDS.has(key) && value && typeof value === 'object' && !Array.isArray(value)) {
        for (const [name, child] of Object.entries(value)) {
          queue.push({ node: child, file, pointer: `${at}/${escapeToken(name)}` });
        }
      } else if (SCHEMA_ARRAY_KEYWORDS.has(key) && Array.isArray(value)) {
        value.forEach((child, i) => queue.push({ node: child, file, pointer: `${at}/${i}` }));
      }
    }
  }
  return { marked, errors };
}

/** Shorten a description to a recognisable excerpt around its marker. */
function excerpt(text) {
  const at = UNCHECKED_MARKERS.map((re) => text.search(re)).find((i) => i >= 0) ?? 0;
  const start = Math.max(0, at - 100);
  const end = Math.min(text.length, at + 40);
  return `${start > 0 ? '…' : ''}${text.slice(start, end)}${end < text.length ? '…' : ''}`;
}

/**
 * Check one spec's `uncheckedRules`. `rel` is the spec's `<slug>/<version>`,
 * used in messages. Fails on every `member` that does not resolve to a schema,
 * and on every marked schema (see `findMarkedSchemas`) that no entry is at or
 * under. Logs one line per declared rule through `log`. Returns the number of
 * rules declared.
 */
export function checkUncheckedRules(meta, payloadSchemaPath, rel, { fail, log, loadJson = makeJsonLoader() }) {
  const rules = Array.isArray(meta?.uncheckedRules) ? meta.uncheckedRules : [];
  const covered = new Set();
  if (rules.length) {
    log(`  Unchecked rules: ${rel} declares ${rules.length} rule(s) its payload schema does not check:`);
  }
  rules.forEach((entry, i) => {
    const resolved = resolveSchemaPointer(payloadSchemaPath, entry?.member, loadJson);
    if (!resolved.ok) {
      fail(
        `${rel}/spec.md`,
        `uncheckedRules[${i}].member '${entry?.member}' does not resolve to a schema in payload.schema.json: ` +
          `${resolved.reason}. Point it at the schema location the rule applies to, following $refs.`
      );
    } else {
      for (const node of resolved.chain) covered.add(node);
    }
    log(`    - ${entry?.member}: ${entry?.rule} (${entry?.source})`);
  });
  for (const m of findMarkedSchemas(payloadSchemaPath, loadJson).marked) {
    if (covered.has(m.node)) continue;
    fail(
      `${rel}/spec.md`,
      `the description at '${m.pointer}' (reached from payload.schema.json) says a rule is not checked ` +
        `("${excerpt(m.description)}"), but no uncheckedRules entry is at or under it. Declare the rule with ` +
        `member '${m.pointer}' or a schema under it; if the schema now checks the rule, drop the marker phrase ` +
        `from the description.`
    );
  }
  return rules.length;
}

/**
 * The build's wiring: one reporter across every spec. `check` runs
 * `checkUncheckedRules` for one spec, printing each rule only when `verbose`;
 * `summary` returns the one line the build always prints.
 */
export function makeUncheckedRulesReporter({ fail, log, verbose = false, loadJson = makeJsonLoader() }) {
  let specs = 0;
  let count = 0;
  const detail = verbose ? log : () => {};
  return {
    check(meta, payloadSchemaPath, rel) {
      const declared = checkUncheckedRules(meta, payloadSchemaPath, rel, { fail, log: detail, loadJson });
      if (declared) {
        specs++;
        count += declared;
      }
      return declared;
    },
    summary() {
      return (
        `  Unchecked rules: ${count} rule(s) declared by ${specs} spec(s) that their payload schemas do not check ` +
        `(informational; library validators do not yet surface them${verbose ? '' : '; --verbose lists them'})`
      );
    },
  };
}
