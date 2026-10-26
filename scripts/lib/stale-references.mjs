/**
 * The stale-references check.
 *
 * A specification names other specifications by version: a Type URI in a
 * schema `const`, a link in its prose (`../../acl/list/0.2/spec.md`), a
 * mention in a description (`use vault/release/0.1 to obtain secret bytes`),
 * a sample document in an example. When the named specification gains a new
 * version, every such reference silently goes stale — #913 found one wire pin
 * two versions behind (a relying party serving the current approve-request
 * could not answer auth/step-up/start at all) and 560 prose references, none
 * of which any check could see. This keeps them current.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * The LATEST version of every slug whose status is not `retired`, as the
 * wire-conventions lint does: a superseded version keeps the references it
 * was published with. Every reference in its spec.md, payload schema and
 * invalid examples to another specification at a version older than that
 * specification's latest non-retired version is a finding. Four kinds:
 *
 *   wire      a `const`, `enum`, `pattern` or `$ref` in the payload schema.
 *             Changing it changes what validates, so it takes a NEW VERSION
 *             of the referencing spec (SPEC §5) — the fixer never touches it.
 *   normative a full Type URI in prose, or a code-span version in a sentence
 *             with an RFC 2119 keyword: a value a conforming implementation
 *             emits or accepts (`expects` naming vtc/admin/actions/show/0.2).
 *             Moving it changes conformance, so it is decided by hand —
 *             usually as a new version, as a wire pin is.
 *   example   a Type URI inside a sample document (a fenced JSON block in
 *             spec.md, or an invalid example). The sample's payload was
 *             written for the old version and may not fit the new one, so it
 *             is updated by hand.
 *   text      a prose link, a prose mention, or a schema description. The
 *             fixer rewrites these (`npm run fix-stale-references`).
 *
 * Not findings, because they name an old version on purpose:
 *
 *   - anything under a history heading — `Changes from …`, `What 0.4 changed
 *     from 0.3`, `Version history` — at any level, which is about the past;
 *   - a version run that ends at the latest (`offers/list/0.1`, `0.2`, `0.3`
 *     and `0.4`); for a text reference also a line that names the same spec
 *     at another version — a table row, a version pairing — or the latest
 *     version shortly after (`git-ns/bridge/job (0.4 to 0.6)`). A normative
 *     value is not excused by a neighbour, so a half-fixed line still fails;
 *   - a version followed by `or later`, `and later` or `onward`, or preceded
 *     by `since`, `added in`, `introduced in`, `until` or `before`;
 *   - the front matter.
 *
 * Anything else deliberate goes in `stale-references-allowlist.json`, keyed by
 * the referencing slug, then by the `<slug>/<version>` it names, with the
 * reason. An entry no longer found is reported as stale.
 */
import fs from 'node:fs';
import path from 'node:path';

const compareVersions = (a, b) => {
  const [x, y] = a.split('.').map(Number);
  const [u, v] = b.split('.').map(Number);
  return x - u || y - v;
};

/** `specs`: slug → Map(version → status); `latest`/`latestLive`: slug → its latest (non-retired) version. */
function registry(specsDir) {
  const specs = new Map();
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory() || entry.name.startsWith('_')) continue;
      const full = path.join(dir, entry.name);
      if (/^\d+\.\d+$/.test(entry.name) && fs.existsSync(path.join(full, 'spec.md'))) {
        const slug = path.relative(specsDir, dir).split(path.sep).join('/');
        const status = fs.readFileSync(path.join(full, 'spec.md'), 'utf8').match(/^status:\s*["']?([a-z]+)/m)?.[1] ?? null;
        if (!specs.has(slug)) specs.set(slug, new Map());
        specs.get(slug).set(entry.name, status);
      } else {
        walk(full);
      }
    }
  })(specsDir);
  const latest = new Map(); // slug → latest version (any status)
  const latestLive = new Map(); // slug → latest non-retired version
  for (const [slug, versions] of specs) {
    const sorted = [...versions.keys()].sort(compareVersions);
    latest.set(slug, sorted.at(-1));
    const live = sorted.filter((v) => versions.get(v) !== 'retired');
    if (live.length) latestLive.set(slug, live.at(-1));
  }
  return { specs, latest, latestLive };
}

// A Type URI, or a bare `<slug>/<MAJOR.MINOR>` (a mention, or a relative link
// whose `../` prefix is resolved against the referencing directory).
const TYPE_URI = /trusttasks\.org\/spec\/([a-z0-9][a-z0-9/-]*?)\/(\d+\.\d+)(?![\d.])/g;
const BARE = /(?<![a-z0-9/.-])((?:\.\.\/)*)([a-z0-9][a-z0-9-]*(?:\/[a-z0-9-]+)*)\/(\d+\.\d+)(?![\d.])/g;
const HISTORY_HEADING = /^(Changes from\b|What .*\bchanged\b|Version history\b|History\b)/i;
const LATER = /^[^\n]{0,3}\b(or later|and later|onwards?)\b/;

/**
 * Every stale reference in the latest version of `slug`, as
 * `{ kind, file, line, index, target, version, latest, text }`. `index` is the
 * offset of the version number itself, so the fixer rewrites only that.
 */
function referencesIn(slug, dir, specsDir, reg) {
  const found = [];
  for (const file of ['spec.md', 'payload.schema.json', 'payload.invalid-examples.json']) {
    const full = path.join(dir, file);
    if (!fs.existsSync(full)) continue;
    const text = fs.readFileSync(full, 'utf8');
    const lines = text.split('\n');
    const lineStarts = [0];
    for (let i = 0; i < text.length; i++) if (text[i] === '\n') lineStarts.push(i + 1);
    const lineOf = (index) => {
      let lo = 0;
      let hi = lineStarts.length - 1;
      while (lo < hi) {
        const mid = (lo + hi + 1) >> 1;
        if (lineStarts[mid] <= index) lo = mid;
        else hi = mid - 1;
      }
      return lo;
    };
    // Per-line context for spec.md: front matter, a fenced block, a Changes section, or prose.
    const context = [];
    if (file === 'spec.md') {
      let front = lines[0] === '---';
      let fence = false;
      let changes = 0; // the heading level of the history section we are in, or 0
      lines.forEach((line, i) => {
        if (front) {
          context.push('front');
          if (i > 0 && line === '---') front = false;
          return;
        }
        if (line.startsWith('```')) {
          fence = !fence;
          context.push('fence');
          return;
        }
        const heading = !fence && line.match(/^(#{2,6})\s+(.*)$/);
        if (heading) {
          const level = heading[1].length;
          if (changes && level <= changes) changes = 0;
          if (!changes && HISTORY_HEADING.test(heading[2])) changes = level;
        }
        context.push(fence ? 'fence' : changes ? 'changes' : 'prose');
      });
    }
    const seen = new Set();
    const consider = (target, version, versionAt) => {
      if (target === slug || !reg.latestLive.has(target) || seen.has(versionAt)) return;
      if (target === 'trust-task-error') return; // SPEC §8.1 routes to the current version; checkBindingErrorSpecPin owns it
      const latest = reg.latestLive.get(target);
      if (compareVersions(version, latest) >= 0) return;
      seen.add(versionAt);
      const ln = lineOf(versionAt);
      const where = context[ln] ?? 'json';
      if (where === 'front' || where === 'changes') return;
      const after = text.slice(versionAt + version.length, lineStarts[ln + 1] ?? text.length);
      if (LATER.test(after)) return;
      const line = lines[ln];
      let kind;
      if (file === 'spec.md') {
        // The sentence around the reference, which may run across lines (a paragraph ends at a blank line).
        const from = Math.max(text.lastIndexOf('. ', versionAt), text.lastIndexOf('\n\n', versionAt), 0);
        const ends = [text.indexOf('. ', versionAt), text.indexOf('\n\n', versionAt)].filter((i) => i >= 0);
        const sentence = text.slice(from, ends.length ? Math.min(...ends) : text.length);
        const typeUri = /trusttasks\.org\/spec\/$/.test(text.slice(versionAt - target.length - 30, versionAt - target.length - 1));
        const codeSpan = text[versionAt + version.length] === '`' && !/\]\(/.test(text.slice(versionAt + version.length, versionAt + version.length + 3));
        if (where === 'fence') kind = 'example';
        else if (typeUri || (codeSpan && /\b(MUST|SHOULD|SHALL|REQUIRED|MAY)\b/.test(sentence))) kind = 'normative';
        else kind = 'text';
      } else if (file === 'payload.invalid-examples.json') kind = /trusttasks\.org\/spec\/$/.test(text.slice(versionAt - target.length - 30, versionAt - target.length - 1)) ? 'example' : 'text';
      else kind = /"(const|enum|pattern|\$ref)"\s*:/.test(line) || /^\s*"https:\/\/trusttasks\.org\/spec\//.test(line) ? 'wire' : 'text';
      // The same specification at another version on the line: a table row or a
      // version list (`rooms/keys/act`'s type table, a session ↔ submit pairing).
      const others = [...line.matchAll(new RegExp(`${target.split('/').at(-1).replace(/[-]/g, '\\-')}(?:\\/|\`\\s+)(\\d+\\.\\d+)(?![\\d.])`, 'g'))].map((m) => m[1]);
      // A strict version run ending at the latest (`/0.1`, `0.2`, `0.3` and `0.4`; `/0.2` or `/0.3`):
      // nothing but versions, commas and and/or/to, so a half-rewritten line cannot pass for one.
      const latestRe = latest.replace('.', '\\.');
      if (new RegExp(`^\`?(?:\\s*(?:,|and|or|to)\\s*\`?\\/?\\d+\\.\\d+\`?)*\\s*(?:,|and|or|to)\\s*\`?\\/?${latestRe}(?![\\d.])`).test(after)) return;
      const enumerated = (kind !== 'normative' && kind !== 'wire');
      if (enumerated && others.some((v) => v !== version)) return;
      // An enumeration that runs on to the latest version (`(0.4 to 0.6)`, `0.1, 0.2 and 0.3`).
      if (enumerated && new RegExp(`^[^\\n]{0,80}?(?<![\\d.])${latestRe}(?![\\d.])`).test(after)) return;
      // History: `since X`, `added in X`, `introduced in X`, `until X`.
      const before = text.slice(Math.max(lineStarts[ln], versionAt - target.length - 60), versionAt - target.length);
      if (/\b(since|added in|introduced in|introduced by|until|before)\b[^.;:]{0,25}$/i.test(before)) return;
      found.push({ kind, file, line: ln + 1, index: versionAt, target, version, latest, text: line.trim() });
    };
    for (const m of text.matchAll(TYPE_URI)) {
      consider(m[1], m[2], m.index + m[0].length - m[2].length);
    }
    for (const m of text.matchAll(BARE)) {
      const [, ups, rel, version] = m;
      let target = rel;
      if (ups || !reg.latestLive.has(rel)) {
        // A relative path: resolve it against the referencing directory.
        const resolved = path.relative(specsDir, path.resolve(dir, `${ups}${rel}`)).split(path.sep).join('/');
        if (resolved.startsWith('..') || !reg.latestLive.has(resolved)) continue;
        target = resolved;
      }
      consider(target, version, m.index + m[0].length - version.length);
    }
  }
  return found;
}

/** Every finding across the registry, with allowlisted ones marked. */
export function staleReferences(specsDir, allowlist = {}) {
  const reg = registry(specsDir);
  const out = [];
  for (const [slug, versions] of reg.specs) {
    const version = reg.latest.get(slug);
    if (versions.get(version) === 'retired') continue;
    const dir = path.join(specsDir, slug, version);
    for (const f of referencesIn(slug, dir, specsDir, reg)) {
      const key = `${f.target}/${f.version}`;
      out.push({ ...f, slug, specVersion: version, dir, allowed: Boolean(allowlist[slug]?.[key]) });
    }
  }
  return out;
}

export function checkStaleReferences({ specsDir, allowlistPath, fail, warn, log }) {
  let allowlist;
  try {
    allowlist = JSON.parse(fs.readFileSync(allowlistPath, 'utf8')).exceptions ?? {};
  } catch (e) {
    // Never fail open: a check that cannot read its allowlist checks nothing.
    fail(path.relative(path.dirname(specsDir), allowlistPath), `could not be read: ${e.message}`);
    return;
  }
  const findings = staleReferences(specsDir, allowlist);
  const used = new Set();
  const counts = { wire: 0, normative: 0, example: 0, text: 0, allowed: 0 };
  for (const f of findings) {
    const key = `${f.target}/${f.version}`;
    if (f.allowed) {
      counts.allowed++;
      used.add(`${f.slug}\u0000${key}`);
      continue;
    }
    counts[f.kind]++;
    const at = `specs/${f.slug}/${f.specVersion}/${f.file}:${f.line}`;
    const fix =
      f.kind === 'wire'
        ? `It is on the wire, so it takes a new version of ${f.slug} (SPEC §5).`
        : f.kind === 'example'
          ? `Update the sample document by hand — its payload was written for ${key} and must fit ${f.target}/${f.latest}.`
          : f.kind === 'normative'
            ? `It says what a conforming implementation emits or accepts, so moving it changes conformance — decide by hand.`
            : `Run \`npm run fix-stale-references\`.`;
    fail(
      at,
      `names ${key}, superseded by ${f.target}/${f.latest}. ${fix} If the old version is meant, add "${key}" under ` +
        `"${f.slug}" in scripts/lib/stale-references-allowlist.json with the reason.`
    );
  }
  for (const [slug, entries] of Object.entries(allowlist)) {
    for (const key of Object.keys(entries)) {
      if (!used.has(`${slug}\u0000${key}`)) {
        warn(`stale-references-allowlist.json: "${key}" for ${slug} is no longer found — delete the entry`);
      }
    }
  }
  log(
    `  Stale references: ${counts.wire} wire pin(s), ${counts.normative} normative value(s), ${counts.example} example(s), ` +
      `${counts.text} text reference(s) ` +
      `name a superseded version; ${counts.allowed} documented exception(s)`
  );
}
