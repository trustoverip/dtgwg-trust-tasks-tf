import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { generate, generateFiles, MARKER_FILE, parseLock } from "../src/index.js";

// .test-build/test/generate.test.js → the package root.
const PKG = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const EXAMPLE = path.resolve(PKG, "..", "examples", "consumer-app");
const FIXTURE = path.join(PKG, "test", "fixtures", "acme", "payments", "settle", "1.0.bundle.json");

const digestOf = (bytes: Buffer | string): string => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;

interface Task {
  source?: string;
  slug: string;
  version: string;
  typeUri: string;
  bytes: Buffer | string;
}

/** An application directory: a lock and its vendored bundles. */
function app(tasks: Task[], sources: Record<string, string> = {}): { dir: string; lock: string } {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "tt-gen-test-"));
  const lines = ["# Written by `trust-tasks`. Do not edit by hand.", "version = 1", 'registry = "https://trusttasks.org"'];
  for (const [name, url] of Object.entries(sources)) lines.push("", `[sources.${name}]`, `url = "${url}"`);
  for (const t of tasks) {
    const base = t.source ? path.join(dir, "trust-tasks", `@${t.source}`) : path.join(dir, "trust-tasks");
    const file = path.join(base, ...t.slug.split("/"), `${t.version}.bundle.json`);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, t.bytes);
    lines.push("", "[[task]]");
    if (t.source) lines.push(`source = "${t.source}"`);
    lines.push(`slug = "${t.slug}"`, `version = "${t.version}"`, `type_uri = "${t.typeUri}"`, `digest = "${digestOf(t.bytes)}"`);
  }
  const lock = path.join(dir, "trust-tasks.lock");
  fs.writeFileSync(lock, lines.join("\n") + "\n");
  return { dir, lock };
}

function exampleTasks(): Task[] {
  const lock = parseLock(fs.readFileSync(path.join(EXAMPLE, "trust-tasks.lock"), "utf8"));
  return lock.tasks.map((t) => ({
    slug: t.slug,
    version: t.version,
    typeUri: t.typeUri,
    bytes: fs.readFileSync(path.join(EXAMPLE, "trust-tasks", ...t.slug.split("/"), `${t.version}.bundle.json`)),
  }));
}

function fixtureTask(over: Partial<Task> = {}): Task {
  const bytes = fs.readFileSync(FIXTURE);
  return { source: "acme", slug: "payments/settle", version: "1.0", typeUri: JSON.parse(bytes.toString()).typeUri, bytes, ...over };
}

/** A bundle with members changed (no longer canonical, which the generator does not need). */
function edited(task: Task, change: (b: Record<string, unknown>) => void): Task {
  const b = JSON.parse(task.bytes.toString());
  change(b);
  return { ...task, bytes: JSON.stringify(b) };
}

const quiet = { log: () => {} };

describe("parseLock", () => {
  it("reads the CLI's lock, with and without sources", () => {
    const lock = parseLock(fs.readFileSync(path.join(EXAMPLE, "trust-tasks.lock"), "utf8"));
    assert.equal(lock.version, 1);
    assert.deepEqual(lock.tasks.map((t) => `${t.slug}/${t.version}`), ["acl/grant/0.2", "keys/list/0.2", "vault/get/0.3"]);
    assert.ok(lock.tasks.every((t) => t.source === null));

    const sourced = parseLock(
      [
        "version = 1",
        'registry = "https://trusttasks.org"',
        "[sources.acme]",
        'path = "../acme-registry"',
        "[[task]]",
        'source = "acme"',
        'slug = "payments/settle"',
        'version = "1.0"',
        'type_uri = "https://tasks.example.com/spec/payments/settle/1.0"',
        `digest = "sha256:${"0".repeat(64)}"`,
      ].join("\n"),
    );
    assert.equal(sourced.tasks[0]!.source, "acme");
    assert.deepEqual(sourced.sources.get("acme"), { path: "../acme-registry" });
  });

  const task = ['[[task]]', 'slug = "a/b"', 'version = "0.1"', 'type_uri = "https://trusttasks.org/spec/a/b/0.1"', `digest = "sha256:${"a".repeat(64)}"`];
  const refusals: [string, string[], RegExp][] = [
    ["an unknown lock version", ["version = 2"], /version 2 is not supported/],
    ["an unknown key", ["version = 1", "colour = 1"], /unknown top-level key/],
    ["an unknown task key", ["version = 1", ...task, 'note = "x"'], /unknown \[\[task\]\] key note/],
    ["a malformed digest", ["version = 1", ...task.slice(0, 4), 'digest = "sha256:XYZ"'], /digest must be/],
    ["a task locked twice", ["version = 1", ...task, ...task], /locked more than once/],
    ["an undeclared source", ["version = 1", ...task, 'source = "acme"'], /no \[sources\.acme\] table/],
    ["a source with both url and path", ["version = 1", "[sources.acme]", 'url = "https://x"', 'path = "y"'], /exactly one of url \/ path/],
    ["a source with neither", ["version = 1", "[sources.acme]"], /exactly one of url \/ path/],
    ["a bad source name", ["version = 1", "[sources.Acme]", 'url = "https://x"'], /not a source name/],
    ["an unknown table", ["version = 1", "[tasks]"], /unexpected table/],
  ];
  for (const [what, lines, error] of refusals) {
    it(`refuses ${what}`, () => assert.throws(() => parseLock(lines.join("\n")), error));
  }
});

describe("generate", () => {
  it("writes one module per locked task, an index, _extends.ts and the marker", async () => {
    const { dir, lock } = app([...exampleTasks(), fixtureTask()], { acme: "https://tasks.example.com" });
    const out = path.join(dir, "out");
    const result = await generate({ lock, out, ...quiet });
    assert.deepEqual(result.files, [
      ".trust-tasks-generated",
      "_extends.ts",
      "acl/grant/0.2/payload.ts",
      "acme/index.ts",
      "acme/payments/settle/1.0/payload.ts",
      "index.ts",
      "keys/list/0.2/payload.ts",
      "vault/get/0.3/payload.ts",
    ]);
    const index = fs.readFileSync(path.join(out, "index.ts"), "utf8");
    assert.match(index, /^export \* as AclGrant_v0_2 from "\.\/acl\/grant\/0\.2\/payload\.js";$/m);
    assert.match(index, /^export \* as Acme from "\.\/acme\/index\.js";$/m);
    assert.match(index, /from "@openvtc\/trust-tasks\/runtime";/);
    assert.doesNotMatch(index, /\.\.\/|_runtime/, "the runtime is imported by package, never by a relative path");
    assert.match(fs.readFileSync(path.join(out, "acme", "index.ts"), "utf8"), /^export \* as PaymentsSettle_v1_0 from "\.\/payments\/settle\/1\.0\/payload\.js";$/m);

    // Identity is the bundle's Type URI, never rebuilt from the slug.
    const sourced = fs.readFileSync(path.join(out, "acme", "payments", "settle", "1.0", "payload.ts"), "utf8");
    assert.match(sourced, /^export const TYPE_URI = "https:\/\/tasks\.example\.com\/spec\/payments\/settle\/1\.0" as const;$/m);
    assert.match(sourced, /^export const RESPONSE_TYPE_URI = "https:\/\/tasks\.example\.com\/spec\/payments\/settle\/1\.0#response" as const;$/m);
    assert.doesNotMatch(sourced, /trusttasks\.org\/spec/);
    assert.match(sourced, /^export interface Amount \{$/m, "a $def referenced twice is declared once");
    assert.doesNotMatch(sourced, /\bAmount1\b/);
  });

  it("is deterministic, and removes stale files from a directory it owns", async () => {
    const { dir, lock } = app(exampleTasks());
    const out = path.join(dir, "out");
    await generate({ lock, out, ...quiet });
    const first = new Map((await generateFiles({ lock, ...quiet })).files);
    fs.writeFileSync(path.join(out, "stale.ts"), "export {};\n");
    await generate({ lock, out, ...quiet });
    assert.equal(fs.existsSync(path.join(out, "stale.ts")), false);
    for (const [rel, text] of first) assert.equal(fs.readFileSync(path.join(out, rel), "utf8"), text, rel);
  });

  it("refuses to clear a non-empty directory it did not write", async () => {
    const { dir, lock } = app(exampleTasks());
    const out = path.join(dir, "src");
    fs.mkdirSync(out);
    fs.writeFileSync(path.join(out, "main.ts"), "precious\n");
    await assert.rejects(generate({ lock, out, ...quiet }), new RegExp(`no ${MARKER_FILE.replace(".", "\\.")} marker`));
    assert.equal(fs.readFileSync(path.join(out, "main.ts"), "utf8"), "precious\n");
  });

  it("refuses a vendored bundle that does not match its digest, and generates nothing", async () => {
    const { dir, lock } = app(exampleTasks());
    fs.appendFileSync(path.join(dir, "trust-tasks", "keys", "list", "0.2.bundle.json"), " ");
    const out = path.join(dir, "out");
    await assert.rejects(generate({ lock, out, ...quiet }), /vendored bundle does not match trust-tasks\.lock — run `trust-tasks check`/);
    assert.equal(fs.existsSync(out), false);
  });

  it("refuses an unknown bundleVersion", async () => {
    const [first] = exampleTasks();
    const { lock } = app([edited(first!, (b) => (b.bundleVersion = 2))]);
    await assert.rejects(generateFiles({ lock, ...quiet }), /unknown bundleVersion 2/);
  });

  it("skips a withdrawn version and a framework specification, with a note", async () => {
    const [first] = exampleTasks();
    const notes: string[] = [];
    const framework = {
      slug: "trust-task-ok",
      version: "0.1",
      typeUri: "https://trusttasks.org/spec/trust-task-ok/0.1",
      bytes: JSON.stringify({ ...JSON.parse(first!.bytes.toString()), slug: "trust-task-ok", version: "0.1", typeUri: "https://trusttasks.org/spec/trust-task-ok/0.1" }),
    };
    const { lock } = app([edited(first!, (b) => (b.status = "withdrawn")), framework]);
    const { modules, skipped } = await generateFiles({ lock, log: (l) => notes.push(l) });
    assert.equal(modules.length, 0);
    assert.match(skipped.get("acl/grant/0.2")!, /withdrawn/);
    assert.match(skipped.get("trust-task-ok/0.1")!, /framework specification/);
    assert.equal(notes.length, 2);
  });

  describe("third-party sources", () => {
    it("refuses a source named like an official task's first segment", async () => {
      const { lock } = app([...exampleTasks(), fixtureTask({ source: "acl" })], { acl: "https://tasks.example.com" });
      await assert.rejects(generateFiles({ lock, ...quiet }), /source "acl" .* official task acl\/grant\/0\.2/);
    });

    it("refuses a sourced bundle under the official authority", async () => {
      const [first] = exampleTasks();
      const { lock } = app([{ ...first!, source: "acme" }], { acme: "https://tasks.example.com" });
      await assert.rejects(generateFiles({ lock, ...quiet }), /is under https:\/\/trusttasks\.org\/, but the lock names it as from source "acme"/);
    });

    it("refuses an official entry whose bundle is under another authority", async () => {
      const { lock } = app([fixtureTask({ source: undefined })]);
      await assert.rejects(generateFiles({ lock, ...quiet }), /is not under https:\/\/trusttasks\.org\/, but the lock names it as an official task/);
    });

    it("refuses a framework specification from a source", async () => {
      const uri = "https://tasks.example.com/spec/trust-task-ok/0.1";
      const task = edited(fixtureTask({ slug: "trust-task-ok", version: "0.1", typeUri: uri }), (b) => {
        b.slug = "trust-task-ok";
        b.version = "0.1";
        b.typeUri = uri;
      });
      const { lock } = app([task], { acme: "https://tasks.example.com" });
      await assert.rejects(generateFiles({ lock, ...quiet }), /cannot come from a third-party source/);
    });

    it("accepts a URN Type URI", async () => {
      const uri = "urn:example:acme:spec:payments:settle:1.0";
      const task = edited(fixtureTask({ typeUri: uri }), (b) => (b.typeUri = uri));
      const { lock } = app([task], { acme: "https://tasks.example.com" });
      const { files } = await generateFiles({ lock, ...quiet });
      assert.match(files.get("acme/payments/settle/1.0/payload.ts")!, /^export const TYPE_URI = "urn:example:acme:spec:payments:settle:1\.0" as const;$/m);
    });
  });
});
