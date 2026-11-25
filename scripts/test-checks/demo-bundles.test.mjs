// scripts/check-demo-bundles.mjs: the HCP demos' vendored bundles and locks
// against a registry build. These run on a temporary tree; deploy.yml runs the
// script itself on the real one after `npm run build-bundles`.

import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { checkDemoBundles, parseLock } from "../check-demo-bundles.mjs";

const BUNDLE = '{"slug":"hcp/x","version":"0.1"}';
const sha = (s) => `sha256:${createHash("sha256").update(s).digest("hex")}`;

function lockText({ registry = "https://trusttasks.org", digest = sha(BUNDLE) } = {}) {
  return `# Written by \`trust-tasks\`. Do not edit by hand.\nversion = 1\nregistry = "${registry}"\n\n[[task]]\nslug = "hcp/x"\nversion = "0.1"\ntype_uri = "https://trusttasks.org/spec/hcp/x/0.1"\ndigest = "${digest}"\n`;
}

/** A tree with a registry build holding hcp/x 0.1 and one demo vendoring it. */
function tree({ lock = lockText(), vendored = BUNDLE, extra } = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "tt-demo-bundles-"));
  const bundlesDir = path.join(root, "specs");
  fs.mkdirSync(path.join(bundlesDir, "hcp/x/0.1"), { recursive: true });
  fs.writeFileSync(path.join(bundlesDir, "hcp/x/0.1/bundle.json"), BUNDLE);
  const demo = path.join(root, "examples/hcp-demo-xx");
  fs.mkdirSync(path.join(demo, "trust-tasks/hcp/x"), { recursive: true });
  fs.writeFileSync(path.join(demo, "trust-tasks.lock"), lock);
  if (vendored !== null) fs.writeFileSync(path.join(demo, "trust-tasks/hcp/x/0.1.bundle.json"), vendored);
  if (extra !== undefined) fs.writeFileSync(path.join(demo, "trust-tasks/hcp/x/0.2.bundle.json"), extra);
  return { examplesDir: path.join(root, "examples"), bundlesDir };
}

test("parseLock reads the registry line and every [[task]]", () => {
  const lock = parseLock(lockText());
  assert.equal(lock.registry, "https://trusttasks.org");
  assert.deepEqual(lock.tasks, [{ slug: "hcp/x", version: "0.1", type_uri: "https://trusttasks.org/spec/hcp/x/0.1", digest: sha(BUNDLE) }]);
});

test("a demo whose lock and bundles match the registry build passes", () => {
  assert.deepEqual(checkDemoBundles(tree()), { demos: ["hcp-demo-xx"], problems: [] });
});

test("a demo that vendors fewer bundles than it pins passes", () => {
  assert.deepEqual(checkDemoBundles(tree({ vendored: null })).problems, []);
});

test("a stale vendored bundle fails", () => {
  const { problems } = checkDemoBundles(tree({ vendored: '{"slug":"hcp/x","version":"0.1","old":true}' }));
  assert.equal(problems.length, 1);
  assert.match(problems[0], /trust-tasks\/hcp\/x\/0\.1\.bundle\.json differs/);
});

test("a stale lock digest fails", () => {
  const { problems } = checkDemoBundles(tree({ lock: lockText({ digest: sha("old") }) }));
  assert.equal(problems.length, 1);
  assert.match(problems[0], /the lock pins hcp\/x\/0\.1 at sha256:/);
});

test("a lock still naming a local registry fails", () => {
  const { problems } = checkDemoBundles(tree({ lock: lockText({ registry: "/tmp/website" }) }));
  assert.equal(problems.length, 1);
  assert.match(problems[0], /records registry \/tmp\/website/);
});

test("a vendored bundle the lock does not pin fails", () => {
  const { problems } = checkDemoBundles(tree({ extra: BUNDLE }));
  assert.equal(problems.length, 1);
  assert.match(problems[0], /0\.2\.bundle\.json is not pinned/);
});

test("a pin the registry build has no bundle for fails", () => {
  const lock = lockText().replace('version = "0.1"', 'version = "0.9"');
  const { problems } = checkDemoBundles(tree({ lock, vendored: null }));
  assert.equal(problems.length, 1);
  assert.match(problems[0], /no bundle for/);
});
