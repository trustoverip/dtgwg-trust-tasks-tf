/**
 * The one spec-discovery rule.
 *
 * Four places used to walk `specs/` looking for published spec versions, and
 * they did not agree on what a spec version *is*:
 *
 *   * `scripts/build-registry.mjs` and `scripts/check-bindings-conformance.mjs`
 *     recursed until they found a directory containing `spec.md`;
 *   * `trust-tasks-codegen/src/main.rs` and `scripts/build-ts-bindings.mjs`
 *     recursed until they found a directory containing `payload.schema.json`.
 *
 * While every version folder carries both files the two rules pick the same
 * set, so the disagreement is invisible — right up until a folder carries one
 * and not the other. Then the registry and the generators quietly describe
 * different registries: a folder with only `payload.schema.json` is a spec to
 * the codegen and does not exist to the build, and a folder with only `spec.md`
 * is the reverse. Neither side reports anything, because from where each one
 * stands nothing is wrong. A review of this tree once concluded a shipped spec
 * was missing its `spec.md` on exactly this ambiguity; it was not, but the
 * ambiguity that made the reading plausible was real.
 *
 * So this module owns the rule, and states it as a rule rather than as two
 * coincidences:
 *
 *   A **spec version directory** is a directory under `specs/` that contains
 *   BOTH `spec.md` and `payload.schema.json`. A directory with exactly one of
 *   them is a build error, not a spec and not a non-spec.
 *
 * Two structural conditions are reported rather than silently absorbed:
 *
 *   * `onIncomplete` — a version folder holding one of the pair but not the
 *     other. Callers wire this to a hard failure: it is precisely the state in
 *     which the old two rules disagreed.
 *   * `onNestedSlug` — a non-version directory sitting among another slug's
 *     version siblings (`vtc/auth/recognise/{0.1,0.2,challenge}`). Legal, and
 *     the registry handles it, but it is the layout that makes a version folder
 *     and a slug segment look alike to a reader and to a naive walker, so it is
 *     surfaced as a warning.
 *
 * `trust-tasks-codegen/src/main.rs` and `scripts/build-ts-bindings.mjs` should
 * adopt this rule too — the Rust one by porting it, since it cannot import
 * JavaScript. Both are owned elsewhere and are deliberately left untouched here.
 *
 * Note this is *not* the `expectedPolicy()` situation that CLAUDE.md warns
 * against DRY-ing up. That duplication is load-bearing: it makes
 * `check-bindings-conformance.mjs` an independent derivation of the §7.2 policy
 * rather than a restatement of the generator's. Discovery carries no such
 * property — "which directories are specs" has exactly one right answer, and
 * two implementations of it can only ever agree or be a bug.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
export const SPECS_DIR = path.join(REPO_ROOT, 'specs');

export const SPEC_PROSE_FILE = 'spec.md';
export const PAYLOAD_SCHEMA_FILE = 'payload.schema.json';

/** A directory is skipped entirely when its name starts with `_` or `.`. */
function isInternal(name) {
  return name.startsWith('_') || name.startsWith('.');
}

/**
 * Walk `specs/` and return every spec version directory, sorted by slug then
 * version so callers get a stable order without sorting again.
 *
 * @param {object} [opts]
 * @param {string} [opts.specsDir]     root to walk (defaults to `<repo>/specs`)
 * @param {(problem: {rel: string, message: string}) => void} [opts.onIncomplete]
 *        called for a version directory holding exactly one of the required pair
 * @param {(problem: {rel: string, message: string}) => void} [opts.onNestedSlug]
 *        called for a slug directory nested among another slug's version siblings
 * @returns {Array<{slug: string, version: string, dir: string, specPath: string, schemaPath: string, rel: string}>}
 */
export function discoverSpecs(opts = {}) {
  const specsDir = opts.specsDir ?? SPECS_DIR;
  const found = [];
  if (!fs.existsSync(specsDir)) return found;

  walk(specsDir);
  found.sort((a, b) => (a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : a.version < b.version ? -1 : a.version > b.version ? 1 : 0));
  return found;

  function walk(dir) {
    const versionDirs = [];
    const plainDirs = [];

    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory() || isInternal(entry.name)) continue;
      const full = path.join(dir, entry.name);
      const hasProse = fs.existsSync(path.join(full, SPEC_PROSE_FILE));
      const hasSchema = fs.existsSync(path.join(full, PAYLOAD_SCHEMA_FILE));

      if (hasProse && hasSchema) {
        versionDirs.push(entry.name);
        found.push(record(full, specsDir));
        continue;
      }

      if (hasProse || hasSchema) {
        // The disagreement case. Report it and treat the folder as a version
        // directory regardless — recursing into it would hide the defect by
        // turning a broken spec into an empty namespace.
        versionDirs.push(entry.name);
        const present = hasProse ? SPEC_PROSE_FILE : PAYLOAD_SCHEMA_FILE;
        const missing = hasProse ? PAYLOAD_SCHEMA_FILE : SPEC_PROSE_FILE;
        opts.onIncomplete?.({
          rel: path.relative(specsDir, full).split(path.sep).join('/'),
          message:
            `has ${present} but no ${missing}. A spec version directory MUST carry both: ` +
            `the registry build discovers specs by ${SPEC_PROSE_FILE} and the code generators ` +
            `discover them by ${PAYLOAD_SCHEMA_FILE}, so a folder with one of them is published ` +
            `by half the toolchain and invisible to the other half.`
        });
        continue;
      }

      plainDirs.push(entry.name);
    }

    // A slug segment sharing a parent with version folders reads as a version
    // to anything scanning names rather than contents.
    if (versionDirs.length && plainDirs.length) {
      const relParent = path.relative(specsDir, dir).split(path.sep).join('/') || '.';
      opts.onNestedSlug?.({
        rel: relParent,
        message:
          `holds version folder(s) [${versionDirs.join(', ')}] alongside nested slug ` +
          `director${plainDirs.length === 1 ? 'y' : 'ies'} [${plainDirs.join(', ')}]. ` +
          `This is legal and the registry handles it, but the layout makes a slug segment ` +
          `and a version segment indistinguishable by name — check that the nested slug is ` +
          `deliberate and not a version folder that lost its files.`
      });
    }

    for (const name of plainDirs) walk(path.join(dir, name));
  }
}

function record(full, specsDir) {
  const segments = path.relative(specsDir, full).split(path.sep);
  const version = segments[segments.length - 1];
  const slug = segments.slice(0, -1).join('/');
  return {
    slug,
    version,
    dir: full,
    rel: `${slug}/${version}`,
    specPath: path.join(full, SPEC_PROSE_FILE),
    schemaPath: path.join(full, PAYLOAD_SCHEMA_FILE)
  };
}
