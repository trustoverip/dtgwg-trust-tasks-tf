#!/usr/bin/env node
// Check the two lifecycle rules a change can break (#1132; the rules and why
// are in scripts/lib/spec-lifecycle.mjs):
//
//   - a spec version this change adds targets the current framework version;
//   - a status this change raises (draft → candidate → standard) is on the
//     latest live version of its slug.
//
// Compared with the merge base of VERSION_BASE_REF (default origin/main) and
// HEAD, as check-version-edits.mjs does — against the working tree, so a spec
// not yet committed is checked too.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import { lifecycleFindings } from './lib/spec-lifecycle.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS = path.join(ROOT, 'specs');
const git = (...args) => execFileSync('git', args, { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 28 });

function frontMatter(text) {
  const m = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text ?? '');
  if (!m) return null;
  try {
    const fm = YAML.parse(m[1]);
    return fm && typeof fm === 'object' ? { status: fm.status, targetFrameworkVersion: fm.targetFrameworkVersion } : null;
  } catch {
    return null; // the build reports front matter that does not parse
  }
}

// `<slug>/<M.N>` for a spec.md path relative to the repository root.
const keyOf = (rel) => {
  const m = /^specs\/(.+)\/(\d+\.\d+)\/spec\.md$/.exec(rel);
  return m && !m[1].split('/').some((s) => s.startsWith('_')) ? `${m[1]}/${m[2]}` : null;
};

const head = new Map();
(function walk(dir) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (!e.isDirectory()) continue;
    const full = path.join(dir, e.name);
    const spec = path.join(full, 'spec.md');
    const key = /^\d+\.\d+$/.test(e.name) && fs.existsSync(spec) ? keyOf(path.relative(ROOT, spec).split(path.sep).join('/')) : null;
    if (key) {
      const fm = frontMatter(fs.readFileSync(spec, 'utf8'));
      if (fm) head.set(key, fm);
    } else {
      walk(full);
    }
  }
})(SPECS);

const baseRef = process.env.VERSION_BASE_REF || 'origin/main';
const base = git('merge-base', baseRef, 'HEAD').trim();

// Only the spec.md files that differ from the merge base can break a rule;
// every other version reads the same on both sides.
const changed = new Set([
  ...git('diff', '--name-only', '--no-renames', base, '--', 'specs').split('\n'),
  ...git('ls-files', '--others', '--exclude-standard', '--', 'specs').split('\n'),
].filter((f) => f.endsWith('/spec.md')));

const baseMap = new Map(head);
for (const rel of changed) {
  const key = keyOf(rel);
  if (!key) continue;
  let text = null;
  try {
    text = execFileSync('git', ['show', `${base}:${rel}`], { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'], maxBuffer: 1 << 28 });
  } catch {
    text = null; // added by this change
  }
  const fm = text === null ? null : frontMatter(text);
  if (fm) baseMap.set(key, fm);
  else baseMap.delete(key);
}

const findings = lifecycleFindings({ base: baseMap, head });
if (findings.length) {
  console.error(`Spec lifecycle: ${findings.length} problem(s) against ${base.slice(0, 12)}:`);
  for (const { key, message } of findings) console.error(`  - specs/${key}: ${message}`);
  process.exit(1);
}
console.log(`Spec lifecycle: ${changed.size} changed spec.md file(s) against ${base.slice(0, 12)}; new versions target the current framework and no older version was promoted.`);
