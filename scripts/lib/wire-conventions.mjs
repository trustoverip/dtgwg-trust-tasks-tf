/**
 * The wire-conventions lint.
 *
 * CONTRIBUTING-SPECS.md "Wire conventions for common members" spells four
 * recurring shapes exactly one way: instants are RFC 3339 strings, pagination
 * is `cursor` + `limit` → `nextCursor` (absent on the last page), `since` is an
 * inclusive time filter, and `created` is a timestamp (`wasCreated` is the
 * response flag). Before they were written down the registry had three paging
 * shapes, seventeen Unix-seconds instants, and `created`/`since` each meaning
 * two things; #901 gave every offender a new version. This keeps it that way.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * The LATEST version of every slug whose status is not `retired` — the
 * versions a new implementation would pick. A superseded version keeps the
 * spelling it was published with (changing it is a breaking change, which is
 * why its successor exists), so it is not checked. Each payload schema is
 * walked whole, and every cross-file `$ref` is followed to the definition it
 * names, so a shared `$defs` entry is checked exactly when a current spec puts
 * it on the wire. `_framework` refs are not followed: the framework defines its
 * own envelope.
 *
 * The rules, each named by the id the allowlist uses:
 *
 *   integer-instant   an integer/number member that is an instant: its
 *                     description says Unix/epoch seconds, or its name ends in
 *                     `At` or `Deadline`, or is `timestamp`/`notBefore`/`notAfter`.
 *   timestamp-name    a member named `timestamp`: name the event (`recordedAt`).
 *   page-size         `pageSize`, `pageToken`, `nextPageToken`,
 *                     `continuationToken`: use `cursor` + `limit`.
 *   offset-paging     `offset` beside `limit` or `cursor`: use `cursor`.
 *   nullable-cursor   a `nextCursor` whose type admits `null`.
 *   truncated-flag    `truncated` beside `nextCursor`: its presence says it.
 *   response-cursor   a response carrying `cursor`, or a string `next`: the
 *                     continuation token in a response is `nextCursor`.
 *   since-not-time    a `since` that is not a `date-time` string.
 *   created-boolean   a boolean `created`: the flag is `wasCreated`.
 *   duration-unit     an integer duration whose name does not spell out its
 *                     unit: `…Secs`/`…Sec`/`…Mins`/`…Msecs`, or a bare `ttl`,
 *                     `timeout`, `duration`, `interval` or `delay`. A duration
 *                     is `…Seconds`, `…Ms`, `…Minutes`, `…Days`.
 *
 * ── The allowlist (wire-conventions-allowlist.json) ─────────────────────────
 *
 * `exceptions`, keyed by slug — or by a shared schema's family directory
 * (`auth/_shared`), for a definition many specs reach — then by
 * `<rule> <member>`, each mapped to the reason the shape is deliberate (`vault/sync` keeps `truncated` because a feed
 * that always returns its position has no cursor to omit). Keyed by slug, not
 * version, so a later version inherits the exception; an entry no longer found
 * is reported as stale. It is for shapes the conventions do not mean, never for
 * a new spec that would rather not follow them.
 */
import fs from 'node:fs';
import path from 'node:path';

const PAGE_SIZE_NAMES = new Set(['pageSize', 'pageToken', 'nextPageToken', 'continuationToken']);
const EPOCH_DESCRIPTION = /unix|epoch[- ](milli)?seconds|seconds since the (unix )?epoch|integer utc/i;
const INSTANT_NAME = /(At|Deadline)$|^(timestamp|notBefore|notAfter)$/;
const UNITLESS_DURATION = /(Secs?|Mins|Msecs?)$|^(ttl|timeout|duration|interval|delay)$/;

const typesOf = (node) => [].concat(node?.type ?? []);
const isNumeric = (node) => typesOf(node).some((t) => t === 'integer' || t === 'number');

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
      if (!entry.isDirectory()) continue;
      if (/^\d+\.\d+$/.test(entry.name)) {
        const slug = path.relative(specsDir, dir).split(path.sep).join('/');
        if (!fs.existsSync(path.join(dir, entry.name, 'spec.md'))) continue;
        if (!latest.has(slug) || compareVersions(entry.name, latest.get(slug)) > 0) latest.set(slug, entry.name);
      } else {
        walk(path.join(dir, entry.name));
      }
    }
  })(specsDir);
  return [...latest].map(([slug, version]) => ({ slug, version, dir: path.join(specsDir, slug, version) }));
}

function statusOf(dir) {
  const prose = fs.readFileSync(path.join(dir, 'spec.md'), 'utf8');
  return prose.match(/^status:\s*["']?([a-z]+)/m)?.[1] ?? null;
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
 * Every `{ rule, member, where }` the payload schema in `dir` — and every
 * shared definition it references — breaks.
 */
export function findingsFor(dir, specsDir) {
  const found = [];
  const docs = new Map();
  const load = (file) => {
    if (!docs.has(file)) docs.set(file, JSON.parse(fs.readFileSync(file, 'utf8')));
    return docs.get(file);
  };
  const followed = new Set();

  const visit = (node, file, pointer, seen, inResponse) => {
    if (!node || typeof node !== 'object' || seen.has(node)) return;
    seen.add(node);
    if (Array.isArray(node)) {
      node.forEach((item, i) => visit(item, file, `${pointer}/${i}`, seen, inResponse));
      return;
    }
    const where = `${path.relative(specsDir, file).split(path.sep).join('/')}#${pointer || '/'}`;

    if (typeof node.$ref === 'string') {
      // A local `#/$defs/…` ref is followed too: inside a shared file only the
      // referenced definition is walked, not the whole document.
      const [target, fragment = ''] = node.$ref.split('#');
      const targetFile = target ? path.resolve(path.dirname(file), target) : file;
      const key = `${targetFile}#${fragment}`;
      if (!targetFile.includes(`${path.sep}_framework${path.sep}`) && fs.existsSync(targetFile) && !followed.has(key)) {
        followed.add(key);
        const doc = load(targetFile);
        visit(atPointer(doc, fragment), targetFile, fragment, new Set(), inResponse);
      }
    }

    const props = node.properties && typeof node.properties === 'object' && !Array.isArray(node.properties)
      ? node.properties
      : null;
    if (props) {
      const has = (name) => Object.hasOwn(props, name);
      for (const [name, member] of Object.entries(props)) {
        if (!member || typeof member !== 'object') continue;
        const push = (rule) => found.push({ rule, member: name, where: `${where}/properties/${name}` });
        if (isNumeric(member) && (EPOCH_DESCRIPTION.test(member.description ?? '') || INSTANT_NAME.test(name))) {
          push('integer-instant');
        }
        if (name === 'timestamp') push('timestamp-name');
        if (PAGE_SIZE_NAMES.has(name)) push('page-size');
        if (name === 'offset' && (has('limit') || has('cursor'))) push('offset-paging');
        if (name === 'nextCursor' && typesOf(member).includes('null')) push('nullable-cursor');
        if (name === 'truncated' && has('nextCursor')) push('truncated-flag');
        if (inResponse === node && (name === 'cursor' || (name === 'next' && typesOf(member).includes('string')))) {
          push('response-cursor');
        }
        if (name === 'since' && !(typesOf(member).includes('string') && member.format === 'date-time')) {
          push('since-not-time');
        }
        if (name === 'created' && typesOf(member).includes('boolean')) push('created-boolean');
        if (isNumeric(member) && UNITLESS_DURATION.test(name)) push('duration-unit');
      }
    }

    for (const [key, child] of Object.entries(node)) {
      if (key === 'examples' || key === 'default' || key === 'enum' || key === 'const') continue;
      // The response payload is the `$defs.Response` node itself; only its own
      // members are a response's top level.
      const childIsResponse = pointer === '/$defs' && key === 'Response' ? child : inResponse;
      visit(child, file, `${pointer}/${key}`, seen, childIsResponse);
    }
  };

  for (const name of fs.readdirSync(dir).filter((n) => n.endsWith('.schema.json'))) {
    const file = path.join(dir, name);
    visit(load(file), file, '', new Set(), null);
  }
  // A shared definition reached twice is one finding, not two.
  const unique = new Map(found.map((f) => [`${f.rule} ${f.where}`, f]));
  return [...unique.values()];
}

/**
 * Run the lint over the latest version of every slug under `specsDir`.
 * Failures go to `fail`, the summary line and stale-entry notices to `log` and
 * `warn`.
 */
export function checkWireConventions({ specsDir, allowlistPath, fail, warn, log }) {
  let allowlist;
  try {
    allowlist = JSON.parse(fs.readFileSync(allowlistPath, 'utf8'));
  } catch (e) {
    // Never fail open: a lint that cannot read its allowlist checks nothing.
    fail(path.relative(path.dirname(specsDir), allowlistPath), `could not be read: ${e.message}`);
    return;
  }
  const exceptions = allowlist.exceptions ?? {};
  const used = new Set();
  const counts = { specs: 0, retired: 0, excepted: 0, offending: 0 };

  for (const { slug, version, dir } of latestSpecs(specsDir)) {
    if (statusOf(dir) === 'retired') {
      counts.retired++;
      continue;
    }
    counts.specs++;
    let findings;
    try {
      findings = findingsFor(dir, specsDir);
    } catch {
      continue; // an unparseable schema is reported by the schema checks
    }
    for (const { rule, member, where } of findings) {
      const key = `${rule} ${member}`;
      // A finding in a shared file may be excepted for that file's family
      // directory, with the version removed (`auth/_shared`).
      const sharedDir = where
        .split('#')[0]
        .split('/')
        .slice(0, -1)
        .filter((segment) => !/^\d+\.\d+$/.test(segment))
        .join('/');
      const owner = exceptions[slug]?.[key] ? slug : sharedDir.includes('_shared') && exceptions[sharedDir]?.[key] ? sharedDir : null;
      if (owner) {
        counts.excepted++;
        used.add(`${owner}\u0000${key}`);
        continue;
      }
      counts.offending++;
      fail(
        `specs/${slug}/${version}`,
        `${where}: ${MESSAGES[rule]} See CONTRIBUTING-SPECS.md "Wire conventions for common members". ` +
          `If the shape is deliberate and the convention does not mean it, add "${key}" under ` +
          `"${slug}" in scripts/lib/wire-conventions-allowlist.json with the reason.`
      );
    }
  }

  for (const [slug, entries] of Object.entries(exceptions)) {
    for (const key of Object.keys(entries)) {
      if (!used.has(`${slug}\u0000${key}`)) {
        warn(`wire-conventions-allowlist.json: "${key}" for ${slug} is no longer found — delete the entry`);
      }
    }
  }

  log(
    `  Wire conventions: ${counts.specs} current spec(s) checked, shared definitions followed; ` +
      `${counts.excepted} documented exception(s), ${counts.retired} retired skipped, ` +
      `${counts.offending} non-conforming`
  );
}

const MESSAGES = {
  'integer-instant': 'an instant is an RFC 3339 string ("format": "date-time"), never Unix seconds.',
  'timestamp-name': 'name the event the instant records (`recordedAt`), not a bare `timestamp`.',
  'page-size': 'pagination is `cursor` + `limit` in the request and `nextCursor` in the response.',
  'offset-paging': 'page with an opaque `cursor`, not an `offset`.',
  'nullable-cursor': '`nextCursor` is a plain string, absent on the last page — never `null`.',
  'truncated-flag': '`truncated` beside `nextCursor` says nothing its presence does not; remove it.',
  'response-cursor': 'a response\'s continuation token is `nextCursor`.',
  'since-not-time': '`since` is an inclusive RFC 3339 lower bound; a cursor is `cursor`, a resume token `resumeToken`, a sync position `syncToken`.',
  'created-boolean': '`created` is a timestamp; the was-it-created response flag is `wasCreated`.',
  'duration-unit': 'a duration names its unit, spelled out: `…Seconds`, `…Ms`, `…Minutes`, `…Days` — never `…Secs` or a bare `ttl`/`timeout`.',
};
