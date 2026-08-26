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
