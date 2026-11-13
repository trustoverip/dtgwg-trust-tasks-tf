/**
 * The member-description ratchet (#1132).
 *
 * A payload member with no `description` leaves its meaning to its name: the
 * spec page shows an empty cell, and every generated type in four libraries
 * carries no doc comment. When the lint landed, 897 members of the latest
 * versions had neither a `description` nor a `$ref` (whose target describes
 * it), in 315 slugs.
 *
 * ── What is counted ─────────────────────────────────────────────────────────
 *
 * The latest version of every slug that is not retired or withdrawn — as the
 * wire-conventions lint chooses — and in each of its own `*.schema.json`
 * files, every member of every `properties` object (examples, defaults, enums
 * and consts are not walked) that has no `description` and no `$ref`. Shared
 * schemas are not followed: they are counted where they are defined.
 *
 * ── The ratchet (member-descriptions-baseline.json) ─────────────────────────
 *
 * `baseline` maps a slug to the count its latest version was allowed when the
 * entry was last lowered. The count may only fall:
 *
 *   - more undescribed members than the slug's entry (0 for a slug with no
 *     entry — every new slug) fails the build, naming each member;
 *   - fewer warns, asking for the entry to be lowered — which
 *     `node scripts/lib/member-descriptions.mjs --lower` does for every slug.
 *     It never raises an entry.
 *
 * Keyed by slug, not version, so a new version of an old slug inherits its
 * allowance — it may describe more members, never fewer.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const BASELINE_PATH = path.join(HERE, 'member-descriptions-baseline.json');

const compareVersions = (a, b) => {
  const [a1, a2] = a.split('.').map(Number);
  const [b1, b2] = b.split('.').map(Number);
  return a1 - b1 || a2 - b2;
};

/** The latest version of every slug with a spec.md, and whether it is live. */
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
  return [...latest].map(([slug, version]) => {
    const dir = path.join(specsDir, slug, version);
    const status = fs.readFileSync(path.join(dir, 'spec.md'), 'utf8').match(/^status:\s*["']?([a-z]+)/m)?.[1];
    return { slug, version, dir, live: status !== 'retired' && status !== 'withdrawn' };
  });
}

/** Every member of a schema document with no `description` and no `$ref`, as JSON pointers. */
export function undescribedMembers(schema) {
  const out = [];
  const visit = (node, pointer) => {
    if (!node || typeof node !== 'object') return;
    if (Array.isArray(node)) {
      node.forEach((item, i) => visit(item, `${pointer}/${i}`));
      return;
    }
    const props = node.properties;
    if (props && typeof props === 'object' && !Array.isArray(props)) {
      for (const [name, member] of Object.entries(props)) {
        if (member && typeof member === 'object' && !('description' in member) && !('$ref' in member)) {
          out.push(`${pointer}/properties/${name.replace(/~/g, '~0').replace(/\//g, '~1')}`);
        }
      }
    }
    for (const [key, child] of Object.entries(node)) {
      if (key === 'examples' || key === 'default' || key === 'enum' || key === 'const') continue;
      visit(child, `${pointer}/${key.replace(/~/g, '~0').replace(/\//g, '~1')}`);
    }
  };
  visit(schema, '');
  return out;
}

/** `{ slug: [ "<file>#<pointer>", … ] }` for the latest live version of every slug. */
export function measure(specsDir) {
  const result = new Map();
  for (const { slug, version, dir, live } of latestSpecs(specsDir)) {
    if (!live) continue;
    const found = [];
    for (const name of fs.readdirSync(dir).filter((n) => n.endsWith('.schema.json')).sort()) {
      let schema;
      try {
        schema = JSON.parse(fs.readFileSync(path.join(dir, name), 'utf8'));
      } catch {
        continue; // an unparseable schema is reported by the schema checks
      }
      for (const p of undescribedMembers(schema)) found.push(`${slug}/${version}/${name}#${p}`);
    }
    result.set(slug, found);
  }
  return result;
}

/**
 * Compare the corpus with the baseline. Failures go to `fail`, entries to
 * lower to `warn`, the summary to `log`.
 */
export function checkMemberDescriptions({ specsDir, baselinePath = BASELINE_PATH, fail, warn, log }) {
  let baseline;
  try {
    baseline = JSON.parse(fs.readFileSync(baselinePath, 'utf8')).baseline ?? {};
  } catch (e) {
    // Never fail open: a ratchet that cannot read its baseline holds nothing.
    fail(path.relative(path.dirname(specsDir), baselinePath), `could not be read: ${e.message}`);
    return;
  }
  const measured = measure(specsDir);
  let total = 0;
  let lowerable = 0;
  for (const [slug, found] of measured) {
    total += found.length;
    const allowed = baseline[slug] ?? 0;
    if (found.length > allowed) {
      fail(
        `specs/${slug}`,
        `${found.length} payload member(s) with no "description" (and no "$ref"), where the ratchet allows ${allowed}. ` +
          `Describe each member — what it means, its unit or format, and what its absence means: ${found.join(', ')}`
      );
    } else if (found.length < allowed) {
      lowerable++;
    }
  }
  for (const slug of Object.keys(baseline)) {
    if (!measured.has(slug)) lowerable++;
  }
  if (lowerable) {
    warn(
      `member-descriptions-baseline.json: ${lowerable} slug(s) now describe more members than their entry allows — ` +
        `run \`node scripts/lib/member-descriptions.mjs --lower\` and commit the result to hold the gain`
    );
  }
  const allowedTotal = Object.values(baseline).reduce((a, b) => a + b, 0);
  log(`  Member descriptions: ${total} undescribed member(s) in the latest versions (ratchet allows ${allowedTotal})`);
}

/** Rewrite the baseline with every entry lowered to the measured count. Never raises one. */
export function lowerBaseline(specsDir, baselinePath = BASELINE_PATH) {
  const doc = JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
  const next = {};
  for (const [slug, found] of [...measure(specsDir)].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))) {
    const allowed = doc.baseline?.[slug] ?? 0;
    const n = Math.min(allowed, found.length);
    if (n > 0) next[slug] = n;
  }
  doc.baseline = next;
  fs.writeFileSync(baselinePath, JSON.stringify(doc, null, 2) + '\n');
  return next;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) && process.argv.includes('--lower')) {
  const specsDir = path.join(HERE, '..', '..', 'specs');
  const next = lowerBaseline(specsDir);
  console.log(`member-descriptions-baseline.json: ${Object.keys(next).length} slug(s), ${Object.values(next).reduce((a, b) => a + b, 0)} member(s) allowed`);
}
