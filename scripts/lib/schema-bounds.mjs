/**
 * The schema-bounds lint.
 *
 * CONTRIBUTING-SPECS.md "Bounds and URL members" asks two things of a payload
 * schema that nothing checked before this lint:
 *
 *   - Every request-side collection and paging member has an upper bound, so a
 *     consumer can size its work from the schema: an array declares
 *     `maxItems`, a `limit` declares `maximum`, a `cursor` declares
 *     `maxLength`. (`checkFreeTextBounds` in build-registry.mjs bounds
 *     free-text names; the wire-conventions lint fixes the paging member
 *     *names*; neither looks at these bounds.)
 *   - Every string member named as a URL says which schemes it admits: a
 *     `pattern` anchored on a scheme (`^https://`), an `enum` / `const`, or a
 *     URI `format` whose description names the schemes it allows.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * The LATEST version of every slug unless that version is `retired` or
 * `withdrawn`. Each payload schema is walked from its root (the request) and
 * from `$defs.Response` (the response); every `$ref`, local or into a shared
 * schema, is followed to the definition it names, with the side of the member
 * that reached it. A `$defs` entry nothing references is not walked. `_framework`
 * refs are not followed: the framework defines its own envelope.
 *
 * The rules, each named by the id the allowlist uses:
 *
 *   array-max-items    a request-side `array` with no `maxItems` (an array
 *                      whose `items` is `false` is bounded by `prefixItems`).
 *   limit-maximum      a request-side `limit` with no `maximum`.
 *   cursor-max-length  a request-side `cursor` with no `maxLength`.
 *   url-scheme         a string member (or an array of them) whose name ends in
 *                      `url`, `uri`, `endpoint`, `href`, `link`, `callback` or
 *                      `webhook` (any case, singular or plural), on either side,
 *                      that admits any scheme: it has no `enum`/`const`, no
 *                      `pattern` anchored on a scheme (`^https://`,
 *                      `^(https|wss)://`, `^did:`), and is not a URI `format`
 *                      whose description names the schemes it allows. A
 *                      `typeUri` (a Type URI, an identifier in any scheme) and
 *                      a `…Base64Url` (an encoding) are not URL members.
 *
 * ── The baseline (schema-bounds-allowlist.json) ─────────────────────────────
 *
 * `exceptions`, keyed by `<slug>/<version>` for a finding in a spec's own
 * schema, or by the shared schema file (`vtc/_shared/0.2/schema-registry.schema.json`)
 * for one in a shared definition — then by `<rule> <pointer>`, mapped to the
 * reason. Keyed by VERSION, so a new version of a listed spec is a new spec and
 * owes the bounds; adding a bound to a released version narrows it, which is a
 * breaking change, so the fix for a listed entry is the next version. It is a
 * ratchet: it may only shrink. A finding not listed fails the build, and a
 * listed entry that no longer applies fails it too, with the line to delete.
 *
 * `node scripts/lib/schema-bounds.mjs --baseline` prints the allowlist the
 * current tree would need. It is how the baseline was written; do not use it
 * to excuse a new spec.
 */
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const RULES = ['array-max-items', 'limit-maximum', 'cursor-max-length', 'url-scheme'];

const URL_NAME = /(urls?|uris?|endpoints?|hrefs?|links?|callbacks?|webhooks?)$/i;
// `typeUri` names a Trust Task Type URI — an identifier compared as a string,
// which SPEC §6.5 lets a private specification mint in any scheme — and
// `…Base64Url` names an encoding, not a URL.
const NOT_A_URL = /(typeUris?|base64Url|b64Url)$/i;
const URI_FORMATS = new Set(['uri', 'uri-reference', 'iri', 'iri-reference']);
// `^https://`, `^(https|wss)://`, `^https:`, `^did:` — a pattern that fixes the
// scheme before anything else may vary.
const SCHEME_PATTERN = /^\^(\(\?:)?\(?[a-z][a-z0-9+.-]*(\|[a-z][a-z0-9+.-]*)*\)?(\\?:|\?:)/i;
const SCHEME_PROSE = /\bschemes?\b|`https`|\bhttps:\/\/|\bHTTPS\b/;

const typesOf = (node) => [].concat(node?.type ?? []);

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

function statusOf(dir) {
  const md = fs.readFileSync(path.join(dir, 'spec.md'), 'utf8');
  const fm = md.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
  return fm.match(/^status:\s*["']?([a-z]+)/m)?.[1] ?? null;
}

/** Resolve a JSON pointer fragment (`/$defs/Foo`) inside `doc`. */
function atPointer(doc, fragment) {
  if (!fragment || fragment === '/') return doc;
  let node = doc;
  for (const raw of fragment.replace(/^\//, '').split('/')) {
    const key = raw.replace(/~1/g, '/').replace(/~0/g, '~');
    if (node == null || typeof node !== 'object') return undefined;
    node = node[key];
  }
  return node;
}

/**
 * Every `{ rule, owner, pointer }` the payload schema in `dir` — and every
 * shared definition it references — breaks. `owner` is the allowlist key:
 * `<slug>/<version>` for the spec's own file, the shared file's path under
 * specs/ otherwise.
 */
export function findingsFor(dir, specsDir, key) {
  const found = new Map();
  const docs = new Map();
  const load = (file) => {
    if (!docs.has(file)) docs.set(file, JSON.parse(fs.readFileSync(file, 'utf8')));
    return docs.get(file);
  };
  const ownFile = path.join(dir, 'payload.schema.json');
  const ownerOf = (file) => (file === ownFile ? key : path.relative(specsDir, file).split(path.sep).join('/'));
  const followed = new Set();

  // One `$ref` hop, for a member written as `{ "$ref": … }` whose bound lives
  // in the definition (`"cursor": { "$ref": "#/$defs/Cursor" }`).
  const resolved = (node, file) => {
    let current = node;
    let currentFile = file;
    for (let hops = 0; hops < 8 && current && typeof current.$ref === 'string'; hops++) {
      const [target, fragment = ''] = current.$ref.split('#');
      const targetFile = target ? path.resolve(path.dirname(currentFile), target) : currentFile;
      if (!fs.existsSync(targetFile)) break;
      const next = atPointer(load(targetFile), fragment);
      if (!next || typeof next !== 'object') break;
      // A sibling keyword beside the `$ref` (a local `maxLength`) still counts.
      current = { ...next, ...Object.fromEntries(Object.entries(current).filter(([k]) => k !== '$ref')) };
      currentFile = targetFile;
    }
    return current ?? {};
  };

  const urlConstrained = (member) => {
    if (member.enum !== undefined || member.const !== undefined) return true;
    if (typeof member.pattern === 'string' && SCHEME_PATTERN.test(member.pattern)) return true;
    return URI_FORMATS.has(member.format) && SCHEME_PROSE.test(member.description ?? '');
  };

  const visit = (node, file, pointer, side, seen) => {
    if (!node || typeof node !== 'object' || seen.has(node)) return;
    seen.add(node);
    if (Array.isArray(node)) {
      node.forEach((item, i) => visit(item, file, `${pointer}/${i}`, side, seen));
      return;
    }
    const push = (rule, at) => {
      const owner = ownerOf(file);
      found.set(`${owner}\u0000${rule} ${at}`, { rule, owner, pointer: at, side });
    };

    if (typeof node.$ref === 'string') {
      const [target, fragment = ''] = node.$ref.split('#');
      const targetFile = target ? path.resolve(path.dirname(file), target) : file;
      const followKey = `${targetFile}#${fragment}#${side}`;
      if (!targetFile.includes(`${path.sep}_framework${path.sep}`) && fs.existsSync(targetFile) && !followed.has(followKey)) {
        followed.add(followKey);
        visit(atPointer(load(targetFile), fragment), targetFile, fragment, side, new Set());
      }
    }

    if (side === 'request' && typesOf(node).includes('array') && node.maxItems === undefined && node.items !== false) {
      push('array-max-items', pointer || '/');
    }

    const props =
      node.properties && typeof node.properties === 'object' && !Array.isArray(node.properties) ? node.properties : null;
    if (props) {
      for (const [name, raw] of Object.entries(props)) {
        if (!raw || typeof raw !== 'object') continue;
        const member = resolved(raw, file);
        const at = `${pointer}/properties/${name}`;
        if (side === 'request' && name === 'limit' && member.maximum === undefined && member.exclusiveMaximum === undefined) {
          push('limit-maximum', at);
        }
        if (side === 'request' && name === 'cursor' && member.maxLength === undefined) push('cursor-max-length', at);
        if (URL_NAME.test(name) && !NOT_A_URL.test(name)) {
          const strings = typesOf(member).includes('string')
            ? member
            : typesOf(member).includes('array')
              ? resolved(member.items, file)
              : null;
          if (strings && typesOf(strings).includes('string') && !urlConstrained(strings)) push('url-scheme', at);
        }
      }
    }

    for (const [k, child] of Object.entries(node)) {
      // `$defs` entries are reached through the `$ref`s that use them, with the
      // side of the member that does.
      if (k === '$defs' || k === 'definitions' || k === 'examples' || k === 'default' || k === 'enum' || k === 'const') continue;
      visit(child, file, `${pointer}/${k}`, side, seen);
    }
  };

  const root = load(ownFile);
  visit(root, ownFile, '', 'request', new Set());
  if (root.$defs?.Response) visit(root.$defs.Response, ownFile, '/$defs/Response', 'response', new Set());
  return [...found.values()];
}

/** Every finding over the latest live specs under `specsDir`, deduplicated, plus how many were checked. */
export function boundsFindings(specsDir) {
  const all = new Map();
  let checked = 0;
  for (const { slug, version, dir } of latestSpecs(specsDir)) {
    const status = statusOf(dir);
    if (status === 'retired' || status === 'withdrawn') continue;
    checked++;
    let findings;
    try {
      findings = findingsFor(dir, specsDir, `${slug}/${version}`);
    } catch {
      continue; // an unparseable schema is reported by the schema checks
    }
    for (const f of findings) {
      const id = `${f.owner}\u0000${f.rule} ${f.pointer}`;
      if (!all.has(id)) all.set(id, { ...f, reachedFrom: `${slug}/${version}` });
    }
  }
  return { findings: [...all.values()], checked };
}

export function checkSchemaBounds({ specsDir, allowlistPath, fail, warn, log }) {
  let allowlist;
  try {
    allowlist = JSON.parse(fs.readFileSync(allowlistPath, 'utf8')).exceptions ?? {};
  } catch (e) {
    // Never fail open: a lint that cannot read its baseline checks nothing.
    fail(path.relative(path.dirname(specsDir), allowlistPath), `could not be read: ${e.message}`);
    return;
  }
  const { findings, checked } = boundsFindings(specsDir);
  const found = new Set();
  const counts = Object.fromEntries(RULES.map((r) => [r, 0]));
  let baselined = 0;
  for (const f of findings) {
    const entry = `${f.rule} ${f.pointer}`;
    found.add(`${f.owner}\u0000${entry}`);
    if (allowlist[f.owner]?.[entry]) {
      baselined++;
      continue;
    }
    counts[f.rule]++;
    const where = f.owner.endsWith('.json') ? `specs/${f.owner}` : `specs/${f.owner}/payload.schema.json`;
    fail(
      `${where}#${f.pointer}`,
      `${f.rule}: ${MESSAGES[f.rule]}` +
        `${f.owner.endsWith('.json') ? ` (reached from ${f.reachedFrom})` : ''} ` +
        `See CONTRIBUTING-SPECS.md "Bounds and URL members".`
    );
  }
  for (const [owner, entries] of Object.entries(allowlist)) {
    for (const entry of Object.keys(entries)) {
      if (!found.has(`${owner}\u0000${entry}`)) {
        fail(
          'scripts/lib/schema-bounds-allowlist.json',
          `"${entry}" for ${owner} no longer applies (the schema now declares it, or is no longer the latest live version) — ` +
            `delete the entry: the baseline only shrinks`
        );
      }
    }
  }
  const offending = Object.values(counts).reduce((a, b) => a + b, 0);
  log(
    `  Schema bounds: ${checked} latest live spec(s) checked, shared definitions followed; ${baselined} baselined ` +
      `finding(s) (scripts/lib/schema-bounds-allowlist.json — may only shrink), ${offending} new` +
      `${offending ? ` (${RULES.filter((r) => counts[r]).map((r) => `${counts[r]} ${r}`).join(', ')})` : ''}`
  );
  void warn;
}

const MESSAGES = {
  'array-max-items': 'a request-side array declares `maxItems`.',
  'limit-maximum': '`limit` declares a `maximum` (and a `minimum` of 1).',
  'cursor-max-length': '`cursor` declares a `maxLength`.',
  'url-scheme':
    'a URL member states the schemes it admits — a `pattern` anchored on the scheme, such as `^https://`, ' +
    'or `"format": "uri"` with a description naming them — or is an `enum`/`const`.',
};

const REASONS = {
  'array-max-items': 'Baseline: no maxItems when the lint landed; the next version declares one.',
  'limit-maximum': 'Baseline: no maximum when the lint landed; the next version declares one.',
  'cursor-max-length': 'Baseline: no maxLength when the lint landed; the next version declares one.',
  'url-scheme': 'Baseline: no scheme constraint when the lint landed; the next version declares one.',
};

/** The allowlist the tree under `specsDir` needs today. */
export function baselineFor(specsDir) {
  const exceptions = {};
  const { findings } = boundsFindings(specsDir);
  for (const f of [...findings].sort((a, b) =>
    a.owner < b.owner ? -1 : a.owner > b.owner ? 1 : `${a.rule} ${a.pointer}` < `${b.rule} ${b.pointer}` ? -1 : 1
  )) {
    (exceptions[f.owner] ??= {})[`${f.rule} ${f.pointer}`] = REASONS[f.rule];
  }
  return exceptions;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href && process.argv.includes('--baseline')) {
  const specsDir = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..', 'specs');
  const out = {
    $comment:
      'The schema-bounds baseline, read by scripts/lib/schema-bounds.mjs: request-side arrays without maxItems, `limit` ' +
      'without maximum, `cursor` without maxLength, and URL members without a URI format and scheme constraint, in the ' +
      'latest live spec versions when the lint landed. Keyed by <slug>/<version> (or a shared schema file), then by ' +
      '"<rule> <pointer>", with the reason. It may only SHRINK: a finding not listed fails the build, and an entry that ' +
      'no longer applies fails the build until it is deleted. Do not add an entry for a new spec or a new version — ' +
      'declare the bound.',
    exceptions: baselineFor(specsDir),
  };
  process.stdout.write(JSON.stringify(out, null, 2) + '\n');
}
