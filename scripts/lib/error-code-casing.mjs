/**
 * The extended error `code` casing lint.
 *
 * SPEC §4.10 rule 4 requires string values drawn from a closed set that a
 * *Trust Task specification* itself defines — "statuses, kinds, decisions,
 * event types, extended error `code` identifiers" — to be lowerCamelCase.
 * The meta-schema's `code` pattern accepts both casings by design, so until
 * this lint nothing forced the issue and the registry drifted: 275 snake_case
 * local parts across 95 spec.md files.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * Only the *local part* — everything after the last `:`. The namespace is a
 * slug, which is lowercase-hyphenated by §6.1 and which §4.10 rule 6 puts
 * explicitly out of scope. `git-trust/grant:alreadyGranted` is conforming;
 * `git-trust/grant:already_granted` is not.
 *
 * lowerCamelCase is read strictly as /^[a-z][a-zA-Z0-9]*$/, which rejects
 * snake_case, kebab-case, SCREAMING_CASE and PascalCase in one rule. As of the
 * change that introduced this lint, snake_case is the only form that actually
 * occurred.
 *
 * ── Why this ships as an error, not a warning ───────────────────────────────
 *
 * The Security & Privacy lint (scripts/lib/security-privacy.mjs) ships as a
 * warning behind a shrinking allowlist because every spec in the registry
 * fails it. This lint is the opposite case: the corpus was brought to zero
 * offenders in the same change that added it, so a hard failure costs nothing
 * today and is the only thing that keeps the drift from returning. There is no
 * allowlist file, and deliberately so — the one exception is derivable from
 * front matter (see below) rather than hand-maintained, so it cannot rot.
 *
 * ── The one exception: retired specifications ───────────────────────────────
 *
 * A `retired` specification's schema and prose are frozen at the moment of
 * retirement (§5.3), and from `candidate` onward §5.2 applies the strict
 * classification under which a re-cased value is a breaking change requiring a
 * MAJOR increment (§4.10, final paragraph). `retired` is terminal, so there is
 * no conforming way to mint that increment. Retired specs are also kept
 * precisely "to keep already-issued documents verifiable" — re-casing a code
 * there would invalidate the documents the status exists to preserve.
 *
 * So retired specs are skipped, and the skip is *counted and reported on every
 * build* rather than hidden. That count can only shrink: `retired` is terminal
 * and frozen, so no new entry can join it, and an entry leaves only when the
 * spec is deleted from the registry. If the number ever grows, something has
 * re-cased or re-published a frozen artifact and the build log will show it.
 */

/** lowerCamelCase, strictly: a lowercase letter then letters and digits only. */
export const LOWER_CAMEL = /^[a-z][a-zA-Z0-9]*$/;

/** Suggest the conforming spelling for a non-conforming local part. */
export function toLowerCamel(local) {
  const parts = local.split(/[_-]+/).filter(Boolean);
  if (!parts.length) return local;
  return parts
    .map((p, i) => (i === 0 ? p.charAt(0).toLowerCase() + p.slice(1) : p.charAt(0).toUpperCase() + p.slice(1)))
    .join('');
}

/**
 * Assess one spec's declared error codes.
 *
 * @param {{errorCodes?: Array<{code?: string}>}} meta parsed front matter
 * @returns {Array<{code: string, local: string, suggestion: string}>} offenders
 */
export function findMiscasedErrorCodes(meta) {
  const offenders = [];
  for (const entry of meta?.errorCodes || []) {
    const code = entry?.code;
    if (typeof code !== 'string') continue;
    const colon = code.lastIndexOf(':');
    if (colon < 0) continue; // grammar failure — the meta schema reports it
    const local = code.slice(colon + 1);
    if (LOWER_CAMEL.test(local)) continue;
    offenders.push({ code, local, suggestion: `${code.slice(0, colon)}:${toLowerCamel(local)}` });
  }
  return offenders;
}

/**
 * The lint runs per-spec inside the main build loop (front matter is parsed
 * there), so it accumulates across calls and reports once at the end.
 *
 * @param {{fail: (loc: string, msg: string) => void, log: (msg: string) => void}} io
 */
export function createErrorCodeCasingLint(io) {
  let declared = 0;
  let conforming = 0;
  let frozen = 0;
  let offending = 0;
  const frozenSpecs = new Set();

  return {
    /** @param {object} meta parsed front matter @param {string} rel spec dir */
    check(meta, rel) {
      const codes = (meta?.errorCodes || []).filter((e) => typeof e?.code === 'string');
      declared += codes.length;

      const offenders = findMiscasedErrorCodes(meta);
      conforming += codes.length - offenders.length;
      if (!offenders.length) return;

      // Frozen by §5.3 — counted, never failed. See the header comment.
      if (meta?.status === 'retired') {
        frozen += offenders.length;
        frozenSpecs.add(rel);
        return;
      }

      for (const o of offenders) {
        offending++;
        io.fail(
          `${rel}/spec.md`,
          `errorCodes['${o.code}'] has a local part '${o.local}' that is not lowerCamelCase. ` +
            `SPEC §4.10 rule 4 requires lowerCamelCase for a specification's own closed-set ` +
            `string values, extended error codes included; rename it to '${o.suggestion}'. ` +
            `Only the local part is governed — the namespace is a slug and keeps its ` +
            `hyphenated spelling (§4.10 rule 6). At draft status §5.2 requires this fix in ` +
            `place, errata-style, with no new version.`
        );
      }
    },

    report() {
      io.log(
        `  Error code casing: ${conforming}/${declared} extended code(s) are lowerCamelCase; ` +
          `${frozen} frozen in ${frozenSpecs.size} retired spec(s) (§5.3 — exempt, not debt), ` +
          `${offending} non-conforming`
      );
      return { declared, conforming, frozen, frozenSpecs: frozenSpecs.size, offending };
    }
  };
}

/**
 * ════════════════════════════════════════════════════════════════════════════
 * The *standard* error `code` casing lint — SPEC §4.10 rule 2.
 * ════════════════════════════════════════════════════════════════════════════
 *
 * The lint above enforces rule 4, a **SHOULD** over values a specification
 * defines for itself. This one enforces rule 2, a **MUST** over values *the
 * framework* defines:
 *
 *   "Enumerated string values defined by this framework — notably the standard
 *    error `code` identifiers of §8.3 — MUST be expressed in lowerCamelCase."
 *
 * Writing `permission_denied` where §8.3 says `permissionDenied` is therefore a
 * stricter breach than anything the rule-4 lint reports, and it is also a
 * broken cross-reference: every such site in the corpus cited
 * `/SPEC.md#83-standard-error-codes`, a table that has not carried that
 * spelling since framework 0.2 re-cased it.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * Every `.md` and `.json` file under `specs/`, not just front matter. The
 * rule-4 lint reads `errorCodes` declarations because that is where an extended
 * code is *defined*; a standard code is never declared anywhere — it is only
 * ever *referenced*, in prose, in a schema `description`, in an example. So the
 * text is what has to be swept. (PR #280 exists precisely because the rule-4
 * lint reads declarations and could not see prose.)
 *
 * Only the ten multi-word codes can be miscased: of the fourteen in §8.3,
 * `expired`, `cancelled`, `unavailable` and `idConflict`'s siblings — four in
 * total — are single words with no snake_case spelling to get wrong.
 *
 * A match is ignored when the token is preceded by `:`. `acl/grant:permission_denied`
 * is a *namespaced extended* code, which is §8.5 and rule 4 territory, not this
 * lint's. (§8.5 does forbid an extended code from shadowing a standard one, but
 * that is checkErrorCodeNamespaces()'s question, not a casing question.)
 *
 * ── The exceptions, both derived rather than listed ─────────────────────────
 *
 * As with the rule-4 lint there is no allowlist file, because a hand-maintained
 * one rots. Two exemptions, both read off disk:
 *
 * 1. **`status: retired`** (§5.3) — identical reasoning to the lint above: a
 *    retired spec is frozen at retirement and exists to keep already-issued
 *    documents verifiable.
 *
 * 2. **The superseded spelling of the framework error vocabulary itself.** A
 *    version is exempt when *both* hold:
 *
 *      (a) its own `payload.schema.json` carries an `enum` that *is* the §8.3
 *          vocabulary — three or more standard codes in one closed set. Only
 *          `trust-task-error/*` does this: it is the specification that
 *          **defines** the codes rather than referencing them.
 *      (b) a later version of the same slug declares `wireCompatibleWith`
 *          naming it.
 *
 *    `trust-task-error/0.1` satisfies both. Its snake_case table is not a
 *    miscasing of the current vocabulary; it *is* the framework-0.1 vocabulary,
 *    published, and `trust-task-error/0.2` was minted for the sole purpose of
 *    carrying the re-cased set — the two payload schemas differ in `$id`, in
 *    those code spellings, and in nothing else. §5.2 defines
 *    `wireCompatibleWith` so that "consumers can dual-accept documents of the
 *    predecessor by mechanical normalization"; re-casing the predecessor
 *    deletes the thing being normalized, collapses 0.2's reason to exist, and
 *    contradicts SPEC Appendix B, which records that "the snake_case 0.1 codes
 *    remain valid for documents whose `type` resolves to a 0.1 specification".
 *
 *    Condition (b) is what keeps (a) narrow: a spec that merely invents a
 *    snake_case enum of its own gets no exemption — only one that a successor
 *    has formally declared it has normalized away.
 *
 * Both exemptions are counted and printed on every build rather than hidden, on
 * the same reasoning as the rule-4 lint: the number can only shrink, and if it
 * ever grows, something has re-published a frozen artifact and the log says so.
 *
 * ── Why a hard failure ──────────────────────────────────────────────────────
 *
 * The corpus was taken to zero offenders in the change that added this, so a
 * failure costs nothing today. Rule 2 is a MUST, so a warning would be the
 * wrong strength even if it did.
 */

/** The §8.3 standard codes, in the casing §4.10 rule 2 requires. */
export const STANDARD_ERROR_CODES = [
  'malformedRequest', 'unsupportedType', 'unsupportedVersion', 'expired',
  'proofRequired', 'proofInvalid', 'permissionDenied', 'wrongRecipient',
  'identityMismatch', 'idConflict', 'cancelled', 'taskFailed', 'unavailable',
  'internalError'
];

/** snake_case spelling -> conforming spelling, for the ten multi-word codes. */
export const STANDARD_SNAKE = new Map(
  STANDARD_ERROR_CODES
    .map((c) => [c.replace(/[A-Z]/g, (ch) => '_' + ch.toLowerCase()), c])
    .filter(([snake]) => snake.includes('_'))
);

/**
 * A snake_case standard code, not preceded by a namespace colon and not part of
 * a longer identifier on either side.
 */
const STANDARD_SNAKE_RE = new RegExp(
  `(?<![A-Za-z0-9_:/-])(${[...STANDARD_SNAKE.keys()].join('|')})(?![A-Za-z0-9_-])`,
  'g'
);

/**
 * Find every rule-2 offence in one file's text.
 *
 * @param {string} text
 * @returns {Array<{line: number, found: string, suggestion: string, excerpt: string}>}
 */
export function findMiscasedStandardCodes(text) {
  const out = [];
  text.split('\n').forEach((line, i) => {
    for (const m of line.matchAll(STANDARD_SNAKE_RE)) {
      out.push({
        line: i + 1,
        found: m[0],
        suggestion: STANDARD_SNAKE.get(m[0]),
        excerpt: line.trim().slice(0, 110)
      });
    }
  });
  return out;
}

/** Cheap top-level scalar read — the two fields needed are both plain strings. */
export function frontMatterField(src, key) {
  const fm = src.match(/^---\n([\s\S]*?)\n---/);
  if (!fm) return undefined;
  const m = fm[1].match(new RegExp(`^${key}:[ \\t]*["']?([^"'\\n]+?)["']?[ \\t]*$`, 'm'));
  return m ? m[1].trim() : undefined;
}

/** True when this payload schema's enums *are* the §8.3 vocabulary. */
export function definesStandardVocabulary(schemaText) {
  let schema;
  try { schema = JSON.parse(schemaText); } catch { return false; }
  let found = false;
  (function walk(node) {
    if (found || !node || typeof node !== 'object') return;
    if (Array.isArray(node.enum)) {
      const hits = node.enum.filter(
        (v) => typeof v === 'string' && (STANDARD_ERROR_CODES.includes(v) || STANDARD_SNAKE.has(v))
      );
      if (hits.length >= 3) { found = true; return; }
    }
    for (const v of Object.values(node)) if (v && typeof v === 'object') walk(v);
  })(schema);
  return found;
}

/**
 * Sweep `specs/` for SPEC §4.10 rule 2 offences.
 *
 * @param {Array<{slug: string, version: string, dir: string, specPath: string, rel: string}>} entries
 *        published spec versions, from discoverSpecs()
 * @param {{fs: object, path: object, specsDir: string}} env  injected so the lint stays testable
 * @param {{fail: (loc: string, msg: string) => void, log: (msg: string) => void}} io
 */
export function checkStandardErrorCodeCasing(entries, env, io) {
  const { fs, path, specsDir } = env;

  // ── The exempt set, built entirely from what is on disk ───────────────────
  const exempt = new Map(); // version dir -> why
  const wireCompatTargets = new Set();
  const metaSrc = new Map();
  for (const e of entries) {
    const src = fs.readFileSync(e.specPath, 'utf8');
    metaSrc.set(e.rel, src);
    const wc = frontMatterField(src, 'wireCompatibleWith');
    if (wc) wireCompatTargets.add(`${e.slug}/${wc}`);
  }
  for (const e of entries) {
    if (frontMatterField(metaSrc.get(e.rel), 'status') === 'retired') {
      exempt.set(e.dir, 'status: retired — frozen by §5.3');
      continue;
    }
    if (!wireCompatTargets.has(e.rel)) continue;
    const schemaPath = path.join(e.dir, 'payload.schema.json');
    if (!fs.existsSync(schemaPath)) continue;
    if (!definesStandardVocabulary(fs.readFileSync(schemaPath, 'utf8'))) continue;
    exempt.set(
      e.dir,
      `defines the §8.3 vocabulary and a successor declares wireCompatibleWith: "${e.version}"`
    );
  }

  // ── Sweep every spec file, not only front matter ──────────────────────────
  const files = [];
  (function walk(dir) {
    for (const d of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, d.name);
      if (d.isDirectory()) walk(p);
      else if (/\.(md|json)$/.test(d.name)) files.push(p);
    }
  })(specsDir);
  files.sort();

  let scanned = 0;
  let frozen = 0;
  let offending = 0;
  const frozenDirs = new Set();

  for (const file of files) {
    scanned++;
    const hits = findMiscasedStandardCodes(fs.readFileSync(file, 'utf8'));
    if (!hits.length) continue;

    const owner = [...exempt.keys()].find((d) => file.startsWith(d + path.sep));
    if (owner) {
      frozen += hits.length;
      frozenDirs.add(owner);
      continue;
    }

    const rel = path.relative(specsDir, file);
    for (const h of hits) {
      offending++;
      io.fail(
        `${rel}:${h.line}`,
        `'${h.found}' is a framework standard error code written in snake_case. ` +
          `SPEC §4.10 rule 2 requires framework-defined values — the §8.3 standard codes ` +
          `among them — to be lowerCamelCase, and that is a MUST, unlike the SHOULD of ` +
          `rule 4 governing a specification's own extended codes. Write '${h.suggestion}'. ` +
          `At draft status §5.2 requires the fix in place, errata-style, with no new ` +
          `version. (If the token was meant as an extended code it needs a namespace — ` +
          `but §8.5 forbids an extended code from shadowing a standard one.) ` +
          `Context: ${h.excerpt}`
      );
    }
  }

  io.log(
    `  Standard error code casing: ${scanned} spec file(s) swept for §8.3 codes; ` +
      `${frozen} snake_case occurrence(s) frozen in ${frozenDirs.size} superseded/retired ` +
      `spec(s) (§5.2/§5.3 — exempt, not debt), ${offending} non-conforming`
  );
  return { scanned, frozen, frozenDirs: frozenDirs.size, offending };
}
