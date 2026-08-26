/**
 * The `## Security & Privacy` section lint.
 *
 * SPEC §10 and §7.3 make security and privacy a first-class obligation on a
 * specification, and `CONTRIBUTING-SPECS.md` used to list the section under
 * "plus anything else useful". Nothing checked it, and the corpus shows what
 * that bought: of 349 published specs, 14 carry no such section at all, 206 of
 * the 335 that do are three non-blank lines or fewer, and corpus-wide exactly
 * two mention data minimisation. The prose that does exist is mostly a
 * restatement of the proof requirement, which is the one property already
 * declared in front matter, machine-checked, and rendered on the page.
 *
 * The four sub-headings are chosen to be the questions a reader of a *task*
 * registry actually needs answered, and the ones free prose reliably skips:
 *
 *   ### Data carried      — what personal or sensitive data the request and
 *                           response actually move, and what a producer should
 *                           not put in them. This is where data minimisation
 *                           gets written down.
 *   ### Correlation       — what an observer, an intermediary, or the recipient
 *                           can join across documents: identifiers, threadIds,
 *                           subject DIDs, timing.
 *   ### Retention         — how long a recipient should keep what it receives,
 *                           and what the document's evidentiary value implies
 *                           about deletion.
 *   ### Consent/purpose   — the purpose the data is collected for and the limits
 *                           on reusing it. Descriptive only: per SPEC §7.3
 *                           item 13 a specification MUST NOT declare that a
 *                           consent or approval step is required.
 *
 * ── Why this ships as a warning ─────────────────────────────────────────────
 *
 * Every spec in the registry fails it today. A hard failure would mean either
 * a red build until 349 specs are rewritten, or a lint nobody turns on. The
 * allowlist is the third option: the debt is enumerated in one file, it is
 * countable (`length` of the array), it can only shrink — an allowlisted spec
 * that starts conforming is reported as a stale entry to delete — and a spec
 * that is not on it warns loudly the moment it lands. New specs therefore get
 * the rule immediately; old ones get it as they are touched.
 *
 * ── Flipping it to an error ─────────────────────────────────────────────────
 *
 * When the allowlist is empty (or when you want CI to hold a line on a branch),
 * set the environment variable:
 *
 *     TT_STRICT_SECURITY_PRIVACY=1 npm run build
 *
 * Every non-conforming spec then fails the build, allowlist or not. Once the
 * allowlist reaches zero, delete the file, make `STRICT` default to true here,
 * and drop the environment variable — that is the intended end state, and this
 * paragraph is the switch's documentation.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const ALLOWLIST_PATH = path.join(HERE, 'security-privacy-allowlist.json');
export const STRICT_ENV_VAR = 'TT_STRICT_SECURITY_PRIVACY';

/** The four sub-headings, in the order they should appear. */
export const REQUIRED_SUBHEADINGS = ['Data carried', 'Correlation', 'Retention', 'Consent/purpose'];

/** Normalise a heading for comparison: lowercase, alphanumerics only. */
function normalise(text) {
  return text.toLowerCase().replace(/[^a-z0-9]+/g, '');
}

const SUBHEADING_KEYS = REQUIRED_SUBHEADINGS.map(normalise);

/**
 * Locate the Security & Privacy section in a spec body and return its content.
 * Accepts the canonical `## Security & Privacy` and the three legacy specs
 * titled `## Security and privacy considerations` — the point of the lint is the
 * content, and rejecting a heading synonym would only produce a second flavour
 * of noise.
 *
 * @returns {{heading: string, body: string} | null}
 */
export function findSecurityPrivacySection(src) {
  const re = /^##[ \t]+(security[ \t]*(?:&|&amp;|and)[ \t]*privacy[^\n]*)$/gim;
  const m = re.exec(src);
  if (!m) return null;
  const after = src.slice(m.index + m[0].length);
  const next = after.search(/^##[ \t]+/m);
  return { heading: m[1].trim(), body: next < 0 ? after : after.slice(0, next) };
}

/**
 * Assess one spec's Security & Privacy section.
 *
 * @returns {{conforms: boolean, missingSection: boolean, missingSubheadings: string[], lines: number}}
 */
export function assessSecurityPrivacy(src) {
  const section = findSecurityPrivacySection(src);
  if (!section) {
    return { conforms: false, missingSection: true, missingSubheadings: [...REQUIRED_SUBHEADINGS], lines: 0 };
  }
  const present = new Set();
  for (const h of section.body.matchAll(/^###[ \t]+([^\n]+)$/gm)) {
    const key = normalise(h[1]);
    const idx = SUBHEADING_KEYS.findIndex((k) => key === k || key.startsWith(k));
    if (idx >= 0) present.add(REQUIRED_SUBHEADINGS[idx]);
  }
  const missing = REQUIRED_SUBHEADINGS.filter((h) => !present.has(h));
  const lines = section.body.split('\n').filter((l) => l.trim()).length;
  return { conforms: missing.length === 0, missingSection: false, missingSubheadings: missing, lines };
}

export function readAllowlist() {
  if (!fs.existsSync(ALLOWLIST_PATH)) return [];
  try {
    const doc = JSON.parse(fs.readFileSync(ALLOWLIST_PATH, 'utf8'));
    return Array.isArray(doc.specs) ? doc.specs : [];
  } catch {
    return [];
  }
}

export function writeAllowlist(ids) {
  const doc = {
    $comment: [
      'Specs published before the Security & Privacy lint landed that do not yet carry the',
      `section and its four sub-headings (${REQUIRED_SUBHEADINGS.join(', ')}).`,
      'This list is DEBT, not permission: it may only shrink. A spec here that starts',
      'conforming is reported as a stale entry to delete, and any spec NOT here that fails',
      'the lint warns on every build. See scripts/lib/security-privacy.mjs for the rule and',
      `for the ${STRICT_ENV_VAR}=1 switch that turns the warning into a build failure.`,
      'Regenerate with: node scripts/build-registry.mjs --update-security-privacy-allowlist'
    ].join(' '),
    strictEnvVar: STRICT_ENV_VAR,
    requiredSubheadings: REQUIRED_SUBHEADINGS,
    specs: [...ids].sort()
  };
  fs.writeFileSync(ALLOWLIST_PATH, JSON.stringify(doc, null, 2) + '\n');
  return doc.specs.length;
}

/**
 * Run the lint over every discovered spec.
 *
 * @param {Array<{rel: string, specPath: string}>} entries
 * @param {{warn: (msg: string) => void, fail: (loc: string, msg: string) => void, log: (msg: string) => void}} io
 */
export function checkSecurityPrivacySections(entries, io) {
  const strict = process.env[STRICT_ENV_VAR] === '1';
  const allowed = new Set(readAllowlist());

  let conforming = 0;
  const offenders = [];
  const stale = [];

  for (const entry of entries) {
    const src = fs.readFileSync(entry.specPath, 'utf8');
    const result = assessSecurityPrivacy(src);
    if (result.conforms) {
      conforming++;
      if (allowed.has(entry.rel)) stale.push(entry.rel);
      continue;
    }
    offenders.push({ rel: entry.rel, ...result });
  }

  const notAllowlisted = offenders.filter((o) => !allowed.has(o.rel));
  const carried = offenders.length - notAllowlisted.length;

  for (const o of notAllowlisted) {
    const why = o.missingSection
      ? 'has no `## Security & Privacy` section'
      : `is missing the sub-heading${o.missingSubheadings.length === 1 ? '' : 's'} ` +
        o.missingSubheadings.map((h) => `\`### ${h}\``).join(', ');
    const msg =
      `${o.rel}/spec.md ${why}. SPEC §10 and §7.3 make this a specification obligation; ` +
      `CONTRIBUTING-SPECS.md carries the template. Fix the spec, or — only for content ` +
      `predating this lint — add '${o.rel}' to ${path.basename(ALLOWLIST_PATH)}.`;
    if (strict) io.fail(`${o.rel}/spec.md`, why);
    else io.warn(msg);
  }

  for (const rel of stale) {
    io.warn(
      `${rel} now carries a conforming Security & Privacy section but is still on ` +
      `${path.basename(ALLOWLIST_PATH)} — remove the entry so the backlog count stays honest`
    );
  }

  const orphans = [...allowed].filter((rel) => !entries.some((e) => e.rel === rel));
  for (const rel of orphans) {
    io.warn(`${path.basename(ALLOWLIST_PATH)} lists '${rel}', which is not a published spec — remove it`);
  }

  io.log(
    `  Security & Privacy: ${conforming}/${entries.length} spec(s) carry the section and all four ` +
    `sub-headings; ${carried} allowlisted as pre-existing debt, ${notAllowlisted.length} not allowlisted` +
    `${strict ? ' (STRICT: failing the build)' : ` (warning — set ${STRICT_ENV_VAR}=1 to fail the build)`}`
  );

  return { conforming, allowlisted: carried, offending: notAllowlisted.length, stale: stale.length };
}
