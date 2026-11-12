// check-published-schemas.mjs and the front-matter half of schema-compat.mjs.
//
// #1134: the policy reader matched only `proofRequirement:` → `requirement:`,
// so for the ~200 specifications written in the per-variant form (which
// new-spec.mjs scaffolds) a `request: RECOMMENDED` → `REQUIRED` edit parsed as
// `undefined` on both sides and passed, as did any `issuedAtRequirement` or
// `parties[].requirement` change. Deleting or renaming a released file was not
// looked at at all (`--diff-filter=M`).

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { derivedPolicy, frontMatterIncompatibilities, parseFrontMatter } from "../lib/schema-compat.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../..");
const FIXTURES = path.join(HERE, "fixtures/published-schemas");
const fixture = (name) => fs.readFileSync(path.join(FIXTURES, name), "utf8");

test("the per-variant fixture: every moved policy flag is reported", () => {
  const found = frontMatterIncompatibilities(
    parseFrontMatter(fixture("per-variant.released.spec.md")),
    parseFrontMatter(fixture("per-variant.edited.spec.md")),
  );
  assert.deepEqual(found.map((f) => f.pointer).sort(), [
    "issuedAtRequirement (response)",
    "parties[member: issuer].requirement",
    "parties[member: recipient].requirement",
    "proofRequirement (request)",
  ]);
});

test("derivedPolicy: an omitted per-variant response takes the request's value", () => {
  const p = derivedPolicy(parseFrontMatter(fixture("per-variant.released.spec.md")));
  assert.deepEqual(p.request, { isBearer: false, isProofRequired: false, isRecipientRequired: false, isIssuedAtRequired: false });
  // The response's recipient is the request's issuer (the Caller, REQUIRED).
  assert.deepEqual(p.response, { isBearer: false, isProofRequired: true, isRecipientRequired: true, isIssuedAtRequired: false });
});

test("rewriting between the one-requirement and per-variant forms with the same policy passes", () => {
  const single = { proofRequirement: { requirement: "REQUIRED" }, issuedAtRequirement: { requirement: "RECOMMENDED" } };
  const perVariant = { proofRequirement: { request: "REQUIRED", response: "REQUIRED" }, issuedAtRequirement: { request: "RECOMMENDED" } };
  assert.deepEqual(frontMatterIncompatibilities(single, perVariant), []);
});

test("bearer and a removed error code are reported; an added one is not", () => {
  const found = frontMatterIncompatibilities(
    { errorCodes: [{ code: "x:a" }, { code: "x:b" }] },
    { bearer: true, errorCodes: [{ code: "x:a" }, { code: "x:c" }] },
  );
  assert.deepEqual(found.map((f) => f.pointer), ["bearer", "errorCodes"]);
  assert.match(found[1].reason, /x:b/);
});

// ── end to end, against a throwaway repository ────────────────────────────

const GIT_ENV = {
  ...process.env,
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_AUTHOR_NAME: "Fixture",
  GIT_AUTHOR_EMAIL: "fixture@example.invalid",
  GIT_COMMITTER_NAME: "Fixture",
  GIT_COMMITTER_EMAIL: "fixture@example.invalid",
};
const git = (cwd, ...args) => execFileSync("git", args, { cwd, env: GIT_ENV, encoding: "utf8" });
const write = (file, content) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
};

/** A repository whose last `trust-tasks-rs-v*` release holds the per-variant spec and a schema. */
function releasedRepo() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "published-schemas-"));
  git(dir, "init", "-q", "-b", "main");
  write(path.join(dir, "specs/fixture/per-variant/0.1/spec.md"), fixture("per-variant.released.spec.md"));
  write(
    path.join(dir, "specs/fixture/per-variant/0.1/payload.schema.json"),
    JSON.stringify({ type: "object", properties: { a: { type: "string" } }, additionalProperties: false }, null, 2),
  );
  write(path.join(dir, "specs/fixture/per-variant/0.1/payload.invalid-examples.json"), "[]\n");
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", "released");
  git(dir, "tag", "trust-tasks-rs-v0.1.0");
  git(dir, "switch", "-q", "-c", "pr");
  return dir;
}
const run = (dir) =>
  spawnSync("node", [path.join(ROOT, "scripts/check-published-schemas.mjs")], {
    cwd: dir,
    env: { ...GIT_ENV, PUBLISHED_BASE_REF: "main" },
    encoding: "utf8",
  });

test("end to end: an in-place per-variant policy edit fails", () => {
  const dir = releasedRepo();
  write(path.join(dir, "specs/fixture/per-variant/0.1/spec.md"), fixture("per-variant.edited.spec.md"));
  git(dir, "commit", "-q", "-am", "edit in place");
  const r = run(dir);
  assert.equal(r.status, 1, r.stdout + r.stderr);
  assert.match(r.stderr, /proofRequirement \(request\)/);
  assert.match(r.stderr, /issuedAtRequirement \(response\)/);
});

test("end to end: a prose-only edit passes", () => {
  const dir = releasedRepo();
  write(path.join(dir, "specs/fixture/per-variant/0.1/spec.md"), fixture("per-variant.released.spec.md") + "\nMore prose.\n");
  git(dir, "commit", "-q", "-am", "prose");
  const r = run(dir);
  assert.equal(r.status, 0, r.stdout + r.stderr);
});

test("end to end: deleting or renaming a released schema fails; deleting its invalid examples does not", () => {
  const dir = releasedRepo();
  git(dir, "mv", "specs/fixture/per-variant/0.1/payload.schema.json", "specs/fixture/per-variant/0.1/renamed.schema.json");
  git(dir, "rm", "-q", "specs/fixture/per-variant/0.1/payload.invalid-examples.json");
  git(dir, "commit", "-q", "-m", "rename");
  const r = run(dir);
  assert.equal(r.status, 1, r.stdout + r.stderr);
  assert.match(r.stderr, /payload\.schema\.json\n\s+was deleted or renamed/);
  assert.doesNotMatch(r.stderr, /invalid-examples/);
});
