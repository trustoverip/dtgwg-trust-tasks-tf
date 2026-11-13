/**
 * Two lifecycle signals that had stopped carrying information (#1132): which
 * framework version is current, and what promoting a version out of `draft`
 * may touch.
 *
 * ── The current framework version, and the lag rule ─────────────────────────
 *
 * `CURRENT_FRAMEWORK_VERSION` is the framework release a specification is
 * written against today. It is the release `trust-tasks-rs` reports by default
 * (`DEFAULT_FRAMEWORK_VERSION` in `src/discovery.rs`, checked by the build),
 * and its envelope schema must be published under `specs/_framework/`.
 *
 * The lag rule (CONTRIBUTING-SPECS.md "The current framework version"):
 *
 *   - a NEW specification version targets the current framework or later;
 *   - an existing version keeps the target it was published with, and moves
 *     with its next revision — the next version folder of its slug.
 *
 * So no spec is asked to change in place, and the registry converges as slugs
 * are revised. `check-spec-lifecycle.mjs` enforces the first half on the
 * versions a pull request adds; the build reports how far the latest versions
 * lag.
 *
 * Moving "current" (when a framework release is brought into the registry and
 * adopted): change the constant here and `DEFAULT_FRAMEWORK_VERSION` together.
 *
 * ── The promotion rule ──────────────────────────────────────────────────────
 *
 * Only the latest live version of a slug is promoted (`draft` → `candidate` →
 * `standard`). "Superseded" is computed from maturity — a later live version
 * at an equal or higher maturity supersedes — so promoting an OLDER version
 * above a newer draft would make it current again: all four generators would
 * drop its deprecation, and its withdrawal date would move. That is library
 * churn a promotion must not cause. A newer draft published after a promotion
 * is fine: it never supersedes the promoted version, so nothing that was
 * deprecated stops being so.
 *
 * Both rules are about a change, not a state, so they are checked against the
 * pull request's merge base (scripts/check-spec-lifecycle.mjs), the way
 * check-version-edits.mjs checks package versions.
 */
import fs from 'node:fs';
import path from 'node:path';

export const CURRENT_FRAMEWORK_VERSION = '0.6.0';

export const MATURITY = { draft: 0, candidate: 1, standard: 2 };

const isLive = (status) => status !== 'retired' && status !== 'withdrawn';

/** Compare two `M.N` or `M.N.P` versions numerically (a missing part is 0). */
export function compareVersions(a, b) {
  const pa = String(a).split('.').map(Number);
  const pb = String(b).split('.').map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const d = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (d) return d;
  }
  return 0;
}

/**
 * The lifecycle findings for a change.
 *
 * `base` and `head` map `<slug>/<M.N>` to `{ status, targetFrameworkVersion }`
 * as the spec.md front matter declares them before and after. Returns
 * `[{ key, message }]`.
 */
export function lifecycleFindings({ base, head, current = CURRENT_FRAMEWORK_VERSION }) {
  const findings = [];
  const versionsOf = new Map();
  for (const key of head.keys()) {
    const i = key.lastIndexOf('/');
    const slug = key.slice(0, i);
    if (!versionsOf.has(slug)) versionsOf.set(slug, []);
    versionsOf.get(slug).push(key.slice(i + 1));
  }

  for (const [key, after] of head) {
    const before = base.get(key);
    const i = key.lastIndexOf('/');
    const slug = key.slice(0, i);
    const version = key.slice(i + 1);

    // The lag rule: a version this change adds targets the current framework.
    if (!before && isLive(after.status)) {
      const target = after.targetFrameworkVersion;
      if (target === undefined || compareVersions(target, current) < 0) {
        findings.push({
          key,
          message:
            `is a new version and targets framework ${target ?? '(none)'}; a new version targets the current framework, ` +
            `${current}, or later (CONTRIBUTING-SPECS.md "The current framework version"). Set targetFrameworkVersion: "${current}".`,
        });
      }
    }

    // The promotion rule: a raised maturity lands only on the latest live version.
    const was = before ? MATURITY[before.status] ?? 0 : 0;
    const now = MATURITY[after.status];
    if (now !== undefined && now > was) {
      const newer = versionsOf
        .get(slug)
        .filter((v) => compareVersions(v, version) > 0 && isLive(head.get(`${slug}/${v}`).status))
        .sort(compareVersions);
      if (newer.length) {
        findings.push({
          key,
          message:
            `is promoted to ${after.status}, but ${slug}/${newer[newer.length - 1]} is a later live version. Only the latest ` +
            `version of a slug is promoted: promoting an older one above a newer draft makes it current again, removing its ` +
            `deprecation in all four libraries and moving its withdrawal date (CONTRIBUTING-SPECS.md "Promoting maturity").`,
        });
      }
    }
  }
  return findings;
}

/**
 * The build's half (no merge base needed): the current framework release is
 * published and is the one `trust-tasks-rs` defaults to; and how far the
 * latest live versions and the bindings lag it — reported, not failed, since
 * the lag rule lets an existing version wait for its next revision.
 */
export function checkCurrentFramework({ root, fail, log, current = CURRENT_FRAMEWORK_VERSION }) {
  const minor = current.split('.').slice(0, 2).join('.');
  if (!fs.existsSync(path.join(root, 'specs', '_framework', minor, 'trust-task.schema.json'))) {
    fail('scripts/lib/spec-lifecycle.mjs', `CURRENT_FRAMEWORK_VERSION is ${current}, but specs/_framework/${minor}/ publishes no envelope schema`);
  }
  const discovery = path.join(root, 'trust-tasks-rs', 'src', 'discovery.rs');
  if (fs.existsSync(discovery)) {
    const declared = /const DEFAULT_FRAMEWORK_VERSION: &str = "([^"]+)"/.exec(fs.readFileSync(discovery, 'utf8'))?.[1];
    if (declared !== current) {
      fail(
        'trust-tasks-rs/src/discovery.rs',
        `DEFAULT_FRAMEWORK_VERSION is ${declared ?? '(not found)'}, but the current framework version is ${current} ` +
          `(CURRENT_FRAMEWORK_VERSION in scripts/lib/spec-lifecycle.mjs) — move them together`
      );
    }
  }

  const lagging = (dir) => {
    const latest = new Map();
    (function walk(d) {
      for (const e of fs.readdirSync(d, { withFileTypes: true })) {
        if (!e.isDirectory() || e.name.startsWith('_')) continue;
        if (/^\d+\.\d+$/.test(e.name)) {
          if (!fs.existsSync(path.join(d, e.name, 'spec.md'))) continue;
          const slug = path.relative(dir, d);
          if (!latest.has(slug) || compareVersions(e.name, latest.get(slug)) > 0) latest.set(slug, e.name);
        } else {
          walk(path.join(d, e.name));
        }
      }
    })(dir);
    let live = 0;
    let behind = 0;
    for (const [slug, version] of latest) {
      const text = fs.readFileSync(path.join(dir, slug, version, 'spec.md'), 'utf8');
      const status = /^status:\s*["']?([a-z]+)/m.exec(text)?.[1];
      if (status === 'retired' || status === 'withdrawn') continue;
      live++;
      const target = /^targetFrameworkVersion:\s*["']?([0-9.]+)/m.exec(text)?.[1];
      if (target && compareVersions(target, current) < 0) behind++;
    }
    return { live, behind };
  };
  const specs = lagging(path.join(root, 'specs'));
  const bindings = fs.existsSync(path.join(root, 'bindings')) ? lagging(path.join(root, 'bindings')) : { live: 0, behind: 0 };
  log(
    `  Framework target: current is ${current}; ${specs.behind} of ${specs.live} latest spec version(s) and ` +
      `${bindings.behind} of ${bindings.live} binding(s) target an earlier release and move with their next revision`
  );
}
