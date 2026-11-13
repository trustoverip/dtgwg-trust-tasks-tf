// The canonical framework specification is pinned: SPEC.md is generated from
// the commit scripts/framework-spec-source.json names, the pin moves only in a
// pull request (framework-spec-bump.yml), and the website deploys the committed
// copy. These checks run offline; `npm run check-framework-spec` (in
// deploy.yml, on every run) regenerates from the pinned commit and compares.

import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { readPin } from "../generate-framework-spec.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (rel) => fs.readFileSync(path.join(ROOT, rel), "utf8");

function pinFile(contents) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "tt-pin-"));
  const file = path.join(dir, "pin.json");
  fs.writeFileSync(file, JSON.stringify(contents));
  return file;
}

test("the committed pin names a full commit SHA", () => {
  const pin = readPin();
  assert.match(pin.commit, /^[0-9a-f]{40}$/);
});

test("SPEC.md's banner names the pinned commit", () => {
  const { commit } = readPin();
  const banner = read("SPEC.md").split("\n").slice(0, 12).join("\n");
  assert.ok(
    banner.includes(`/tree/${commit})`),
    "SPEC.md was not generated from the pinned commit: run `npm run generate-framework-spec` and commit the result",
  );
});

test("readPin refuses a pin that is not a full commit SHA", () => {
  for (const commit of ["main", "bdf2cdd9256f", "v1.0.0", "BDF2CDD9256F9D7B98FE8B40C776124FC27A6084", 42, undefined]) {
    assert.throws(() => readPin(pinFile({ commit, ref: "main" })), /full 40-character commit SHA/, String(commit));
  }
});

test("readPin refuses a ref that is not a plain branch or tag name", () => {
  const commit = "0".repeat(40);
  assert.throws(() => readPin(pinFile({ commit, ref: "main; rm -rf /" })), /branch or tag name/);
  assert.deepEqual(readPin(pinFile({ commit, ref: "main" })), { commit, ref: "main" });
  assert.deepEqual(readPin(pinFile({ commit })), { commit, ref: null });
});

test("the website deploy builds from the committed SPEC.md on every run", () => {
  const deploy = read(".github/workflows/deploy.yml");
  // No trigger that exists only to pull canonical changes in unreviewed.
  assert.doesNotMatch(deploy, /^\s*schedule:/m);
  assert.doesNotMatch(deploy, /^\s*repository_dispatch:/m);
  // The check runs unconditionally, and nothing regenerates from a branch.
  const step = /- name: Check the committed SPEC\.md\n((?:\s{8}.*\n)+)/.exec(deploy);
  assert.ok(step, "deploy.yml has a 'Check the committed SPEC.md' step");
  assert.doesNotMatch(step[1], /\bif:/);
  assert.match(step[1], /npm run check-framework-spec/);
  assert.doesNotMatch(deploy, /run:.*\bgenerate-framework-spec\b(?!.*--check)/);
  assert.doesNotMatch(deploy, /bump-framework-spec|--update/);
});

test("the bump workflow proposes a pull request and does not deploy", () => {
  const bump = read(".github/workflows/framework-spec-bump.yml");
  assert.match(bump, /generate-framework-spec\.mjs --update/);
  assert.match(bump, /gh pr create --base main/);
  assert.doesNotMatch(bump, /aws |s3:\/\/|cloudfront/i);
  assert.doesNotMatch(bump, /git push[^\n]*\bmain\b/);
});
