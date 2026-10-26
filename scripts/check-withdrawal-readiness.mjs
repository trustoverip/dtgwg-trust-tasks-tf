#!/usr/bin/env node
/**
 * Will the next quarterly withdrawal batch build?
 *
 * On the first day of each quarter `.github/workflows/withdraw.yml` withdraws
 * every spec version superseded or retired for 90 days (SPEC "Superseded and
 * Withdrawn Versions"), and the generators stop generating it. Hand-written code
 * that imports a generated module of one of those versions — a test specimen, a
 * smoke test, an example, a discovery responder — then stops compiling, and the
 * batch's pull request cannot merge until someone moves it. The first batch would
 * have broken 40-odd files across four languages.
 *
 * This lists every such use AHEAD of the batch. It warns while the batch is more
 * than FAIL_WITHIN_DAYS away, so a version can be superseded without the PR that
 * supersedes it having to chase every specimen at once, and fails once the batch
 * is close, so nothing is left for the batch itself.
 *
 * A Type URI string, or a JSON document naming a due version, is not a use: a
 * withdrawn version's schema stays published. Only references to the generated
 * MODULE count — Rust `specs::acl::grant::v0_1`, Go `…/specs/acl/grant/v0_1"`,
 * Dart `specs/acl/grant/v0_1/payload.dart`, TS `AclGrant_v0_1` or
 * `acl/grant/0.1/payload`.
 *
 *   node scripts/check-withdrawal-readiness.mjs                # the next batch
 *   node scripts/check-withdrawal-readiness.mjs --as-of 2027-04-01
 */
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dueVersions } from './withdraw-due.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const FAIL_WITHIN_DAYS = 30;

// The next batch: the first day of the next calendar quarter (withdraw.yml's cron).
const args = process.argv.slice(2);
const now = new Date();
let batch;
if (args.includes('--as-of')) {
  batch = new Date(`${args[args.indexOf('--as-of') + 1]}T00:00:00Z`);
} else {
  const q = Math.floor(now.getUTCMonth() / 3) + 1;
  batch = new Date(Date.UTC(now.getUTCFullYear() + (q === 4 ? 1 : 0), (q * 3) % 12, 1));
}
const batchText = batch.toISOString().slice(0, 10);
const daysLeft = Math.ceil((batch - now) / 86_400_000);

const due = dueVersions(batch);

// Hand-written source: everything git tracks that is code, minus the generated
// trees (which the batch regenerates) and the specs themselves.
const GENERATED = /^(specs\/|website\/|trust-tasks-rs\/src\/specs\/|trust-tasks-rs\/src\/(schema_index|extends_index)\.rs$|trust-tasks-rs\/tests\/conformance\/|trust-tasks-ts\/src\/(?!_runtime\/)|trust-tasks-go\/specs\/|trust-tasks-dart\/lib\/specs\/)/;
const files = execFileSync('git', ['ls-files'], { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 28 })
  .split('\n')
  .filter((f) => /\.(rs|ts|go|dart|mjs|js)$/.test(f) && !GENERATED.test(f));
const sources = files.map((f) => [f, fs.readFileSync(path.join(ROOT, f), 'utf8')]);

const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const pascal = (slug) => slug.split(/[/-]/).map((w) => w[0].toUpperCase() + w.slice(1)).join('');
const uses = [];
for (const d of due) {
  const [M, m] = d.version.split('.');
  const rx = new RegExp(
    [
      `${escape(d.slug.replace(/-/g, '_').replace(/\//g, '::'))}::v${M}_${m}\\b`, // Rust
      `${escape(`/specs/${d.slug}/v${M}_${m}`)}"`, // Go import
      escape(`specs/${d.slug.replace(/-/g, '_')}/v${M}_${m}/payload.dart`), // Dart import
      `\\b${escape(pascal(d.slug))}_v${M}_${m}\\b`, // TS root namespace
      escape(`${d.slug}/${d.version}/payload`), // TS module path
    ].join('|')
  );
  for (const [file, text] of sources) {
    const lines = text.split('\n');
    lines.forEach((line, i) => {
      if (rx.test(line)) uses.push({ file, line: i + 1, version: `${d.slug}/${d.version}`, successor: d.successor });
    });
  }
}

if (!uses.length) {
  console.log(`Withdrawal readiness: ${due.length} version(s) due on ${batchText} (${daysLeft} days); no hand-written code imports any of them.`);
  process.exit(0);
}
const failing = daysLeft <= FAIL_WITHIN_DAYS;
const level = failing ? 'error' : 'warning';
const byFile = new Map();
for (const u of uses) {
  if (!byFile.has(u.file)) byFile.set(u.file, []);
  byFile.get(u.file).push(u);
}
for (const [file, list] of byFile) {
  for (const u of list.slice(0, 3)) {
    console.log(`::${level} file=${file},line=${u.line}::imports ${u.version}, withdrawn on ${batchText}${u.successor ? `; move to ${u.successor}` : ''}`);
  }
  if (list.length > 3) console.log(`::${level} file=${file}::…and ${list.length - 3} more use(s) of due versions in this file`);
}
console.log(
  `\nWithdrawal readiness: ${uses.length} hand-written use(s) in ${byFile.size} file(s) of versions withdrawn on ${batchText} ` +
    `(${daysLeft} days). Move each to the current version (successor shown) — the batch PR cannot build until they do. ` +
    (failing ? `The batch is within ${FAIL_WITHIN_DAYS} days, so this now fails.` : `This fails from ${FAIL_WITHIN_DAYS} days before the batch.`)
);
process.exit(failing ? 1 : 0);
