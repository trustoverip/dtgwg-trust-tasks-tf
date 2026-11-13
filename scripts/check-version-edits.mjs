#!/usr/bin/env node
// Refuse a pull request that changes a package's version declaration, unless
// it is the Release PR.
//
// Every publish job keys on "this manifest's version is not on the registry /
// not tagged yet" (publish.yml), so a version moved by an ordinary feature PR
// publishes the moment that PR merges — bypassing the Release PR, and for Go and
// crates.io irreversibly. docs/DEVELOPMENT.md and RELEASING.md say never to do it; this
// makes it a check rather than a convention (#1137). The workflow runs it only
// when the head branch is not this repository's `release` branch.
//
// The declarations watched, compared as parsed values (so reformatting a file
// is not a version change):
//
//   <workspace member>/Cargo.toml         [package] version
//   trust-tasks-ts*/package.json           "version"
//   trust-tasks-go/**/version.go           const Version
//   trust-tasks-dart*/pubspec.yaml         version:
//   trust-tasks-dart*/lib/src/runtime/version.dart   packageVersion
//
// A file added or deleted by the PR is not a version change: a new package
// declares its first version (its first publish is manual — RELEASING.md).
//
//   VERSION_BASE_REF  the ref to compare with (default origin/main); the diff is
//                     against its merge base with HEAD.

import { execFileSync } from "node:child_process";
import YAML from "yaml";

const git = (...args) => execFileSync("git", args, { encoding: "utf8", maxBuffer: 1 << 28 });
const show = (rev, file) => {
  try {
    return execFileSync("git", ["show", `${rev}:${file}`], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], maxBuffer: 1 << 28 });
  } catch {
    return null;
  }
};

const baseRef = process.env.VERSION_BASE_REF || "origin/main";
const base = git("merge-base", baseRef, "HEAD").trim();

// Workspace members, read from the root manifest on both sides so a member
// added by this PR is covered too.
const members = new Set();
for (const rev of [base, "HEAD"]) {
  const root = show(rev, "Cargo.toml") ?? "";
  const block = root.match(/^members\s*=\s*\[([\s\S]*?)\]/m)?.[1] ?? "";
  for (const m of block.matchAll(/"([^"]+)"/g)) members.add(`${m[1]}/Cargo.toml`);
}

// The [package] table only, up to the next table header: a dependency's
// `version = "…"` requirement is not the package's version.
const cargoVersion = (text) =>
  (text.match(/^\[package\]\r?\n([\s\S]*?)(?=^\[|(?![\s\S]))/m)?.[1] ?? "").match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const READERS = [
  { test: (f) => members.has(f), read: cargoVersion, what: "[package] version" },
  { test: (f) => /^trust-tasks-ts(-[a-z0-9-]+)?\/package\.json$/.test(f), read: (t) => JSON.parse(t).version, what: '"version"' },
  { test: (f) => /^trust-tasks-go\/(.+\/)?version\.go$/.test(f), read: (t) => t.match(/^const Version = "([^"]*)"/m)?.[1], what: "const Version" },
  { test: (f) => /^trust-tasks-dart(-[a-z0-9-]+)?\/pubspec\.yaml$/.test(f), read: (t) => YAML.parse(t)?.version, what: "version:" },
  {
    test: (f) => /^trust-tasks-dart(-[a-z0-9-]+)?\/lib\/src\/runtime\/version\.dart$/.test(f),
    read: (t) => t.match(/packageVersion\s*=\s*'([^']*)'|packageVersion\s*=\s*"([^"]*)"/)?.slice(1).find(Boolean),
    what: "packageVersion",
  },
];

const changed = git("diff", "--name-only", "--no-renames", "--diff-filter=M", base, "HEAD").split("\n").filter(Boolean);
const moved = [];
for (const file of changed) {
  const reader = READERS.find((r) => r.test(file));
  if (!reader) continue;
  const parse = (text) => {
    try {
      return text === null ? undefined : reader.read(text);
    } catch {
      return undefined; // unparseable: another check reports it
    }
  };
  const before = parse(show(base, file));
  const after = parse(show("HEAD", file));
  if (before !== after) moved.push({ file, what: reader.what, before, after });
}

if (!moved.length) {
  console.log(`No version declaration changed (${changed.length} file(s) modified since ${base.slice(0, 12)}).`);
  process.exit(0);
}
for (const m of moved) {
  console.error(`::error file=${m.file}::${m.file}: ${m.what} changed from ${m.before ?? "(none)"} to ${m.after ?? "(none)"}`);
}
console.error(`
Only the Release PR (the \`release\` branch, kept by scripts/release-pr.sh) moves
a version. Every publish job releases a version that is not on its registry yet,
so a version changed here would publish the moment this PR merges — bypassing
the Release PR, and for Go and crates.io irreversibly.

Revert the version change. The Release PR assigns the next version from this
PR's conventional-commit title (\`feat\`, \`fix\`, a \`!\` scoped to the package
for a break). See RELEASING.md.`);
process.exit(1);
