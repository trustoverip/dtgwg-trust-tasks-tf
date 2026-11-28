#!/usr/bin/env node
// trust-tasks-rs/prebuilt-modules.txt only shrinks.
//
// The prebuilt Rust modules are frozen (milestone 5 of
// docs/design/consumer-side-generation.md): the list names every specification
// version that keeps a module in trust-tasks-rs/src/specs, and a version it does
// not name is consumer-side only — an application generates it with
// trust-tasks-build. The crate is the whole prebuilt tree, and it sits just under
// crates.io's 10 MiB upload limit, so a line added here grows a crate that cannot
// grow. Removing lines is how the prebuilt modules are retired.
//
// Compared with the merge base of VERSION_BASE_REF (default origin/main) and the
// working tree, as check-spec-lifecycle.mjs does. A base with no list (the change
// that introduces it) passes.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const LIST = 'trust-tasks-rs/prebuilt-modules.txt';
const git = (...args) => execFileSync('git', args, { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 28 });

const entries = (text) =>
  new Set(
    text
      .split('\n')
      .map((l) => l.trim())
      .filter((l) => l && !l.startsWith('#')),
  );

const head = entries(fs.readFileSync(path.join(ROOT, LIST), 'utf8'));

const baseRef = process.env.VERSION_BASE_REF || 'origin/main';
const base = git('merge-base', baseRef, 'HEAD').trim();
let baseText;
try {
  baseText = git('show', `${base}:${LIST}`);
} catch {
  console.log(`${LIST} is new in this change (no list at ${base.slice(0, 12)}): nothing to compare.`);
  process.exit(0);
}
const before = entries(baseText);

const added = [...head].filter((e) => !before.has(e)).sort();
const removed = [...before].filter((e) => !head.has(e)).length;
if (added.length > 0) {
  console.error(
    `${LIST} only shrinks, but this change adds ${added.length} line(s):\n` +
      added.map((e) => `  + ${e}`).join('\n') +
      `\n\nNew specification versions are consumer-side only in Rust: leave them off the list and ` +
      `regenerate (cargo run -p trust-tasks-codegen); an application generates them with ` +
      `trust-tasks-build. See docs/DEVELOPMENT.md, "Every prebuilt specification module is deprecated".`,
  );
  process.exit(1);
}
console.log(
  `${LIST}: ${head.size} prebuilt version(s), ${removed} retired by this change, none added.`,
);
