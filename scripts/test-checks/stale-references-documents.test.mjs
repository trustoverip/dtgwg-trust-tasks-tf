// The stale-references check over documents outside the registry (#1129):
// READMEs, guides, bindings and example `trust-tasks.toml` pins, in a
// throwaway tree with one slug at two versions.

import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { checkStaleReferences, documentFiles, documentStaleReferences } from "../lib/stale-references.mjs";

function tree(files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "stale-docs-"));
  const spec = (v) => `---\nslug: acl/grant\nversion: "${v}"\nstatus: draft\n---\n# acl/grant\n`;
  files = { "specs/acl/grant/0.1/spec.md": spec("0.1"), "specs/acl/grant/0.2/spec.md": spec("0.2"), ...files };
  for (const [rel, text] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
    fs.writeFileSync(path.join(dir, rel), text);
  }
  return dir;
}
const scan = (dir, allowlist) =>
  documentStaleReferences(dir, path.join(dir, "specs"), allowlist).map((f) => ({
    file: f.file,
    line: f.line,
    kind: f.kind,
    ref: `${f.target}/${f.version}`,
    allowed: f.allowed,
  }));

test("a prose mention, a link, a module path and a Type URI are each found", () => {
  const dir = tree({
    "README.md": [
      "Start with `acl/grant/0.1`.", // 1 text
      "See [the spec](specs/acl/grant/0.1/spec.md).", // 2 text, through a mirroring directory
      "```rust",
      "use trust_tasks_rs::specs::acl::grant::v0_1 as grant;", // 4 example (module path in a fence)
      "```",
      "`https://trusttasks.org/spec/acl/grant/0.1`", // 6 example (a full Type URI)
      "`acl/grant/0.2` is current.", // 7 not stale
    ].join("\n"),
  });
  assert.deepEqual(
    scan(dir)
      .map(({ line, kind }) => [line, kind])
      .sort((a, b) => a[0] - b[0]),
    [
      [1, "text"],
      [2, "text"],
      [4, "example"],
      [6, "example"],
    ]
  );
});

test("history is not a finding: a Changes-from section, `or later`, a run to the latest", () => {
  const dir = tree({
    "GUIDE.md": [
      "Use `acl/grant/0.1` or later.",
      "`acl/grant/0.1` and `0.2` differ in one member.",
      "## Changes from 0.1",
      "`acl/grant/0.1` had no ceiling.",
    ].join("\n"),
  });
  assert.deepEqual(scan(dir), []);
});

test("an example's trust-tasks.toml pin is found; other sections and third-party pins are not", () => {
  const dir = tree({
    "examples/app/trust-tasks.toml": [
      'registry = "https://trusttasks.org"',
      "[tasks]",
      '"acl/grant" = "0.1"',
      '"acme:acl/grant" = "0.1"',
      "[sources.acme]",
      'path = "acl/grant/0.1"',
    ].join("\n"),
  });
  assert.deepEqual(scan(dir), [{ file: "examples/app/trust-tasks.toml", line: 3, kind: "example", ref: "acl/grant/0.1", allowed: false }]);
});

test("generated trees, fixtures and superseded bindings are not read", () => {
  const dir = tree({
    "trust-tasks-rs/src/specs/acl/README.md": "`acl/grant/0.1`",
    "cli/tests/fixtures/README.md": "`acl/grant/0.1`",
    "bindings/https/0.1/spec.md": "`acl/grant/0.1`",
    "bindings/https/0.2/spec.md": "`acl/grant/0.1`",
  });
  assert.deepEqual(documentFiles(dir), ["bindings/https/0.2/spec.md"]);
});

test("the allowlist: a history glob and a per-file exception", () => {
  const dir = tree({
    "pkg/CHANGELOG.md": "- moved to `acl/grant/0.1`",
    "README.md": "`acl/grant/0.1` is quoted here on purpose.",
  });
  const allowlist = {
    history: { "**/CHANGELOG*.md": "history" },
    exceptions: { "README.md": { "acl/grant/0.1": "quoted" } },
  };
  assert.ok(scan(dir, allowlist).every((f) => f.allowed));
});

test("checkStaleReferences fails on a document finding and warns on an unused exception", () => {
  const dir = tree({ "README.md": "Start with `acl/grant/0.1`." });
  const allowlistPath = path.join(dir, "allowlist.json");
  fs.writeFileSync(
    allowlistPath,
    JSON.stringify({ exceptions: {}, documents: { exceptions: { "GONE.md": { "acl/grant/0.1": "gone" } } } })
  );
  const failures = [];
  const warnings = [];
  checkStaleReferences({
    specsDir: path.join(dir, "specs"),
    allowlistPath,
    fail: (at, msg) => failures.push(`${at}: ${msg}`),
    warn: (msg) => warnings.push(msg),
    log: () => {},
  });
  assert.equal(failures.length, 1);
  assert.match(failures[0], /^README\.md:1: names acl\/grant\/0\.1, superseded by acl\/grant\/0\.2/);
  assert.match(warnings.join("\n"), /"acl\/grant\/0\.1" for GONE\.md is no longer found/);
});
