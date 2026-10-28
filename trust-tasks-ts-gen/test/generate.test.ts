import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { generate, generateFiles, MARKER_FILE, parseLock, taskLabel } from "../src/index.js";

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
  return lock.tasks.filter((t) => t.source === null).map((t) => ({
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
    // One slug, two authorities: a slug is unique only within its source (SPEC §6.5).
    assert.deepEqual(lock.tasks.map(taskLabel), ["acl/grant/0.2", "keys/list/0.2", "vault/get/0.3", "@acme/vault/get/0.3"]);
    assert.deepEqual(lock.sources.get("acme"), { path: "fixture-registry/site" });

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

  describe("shared definitions (defOrigins)", () => {
    const FRAMEWORK_EXT = "https://trusttasks.org/spec/trust-task/0.4/framework#/$defs/Ext";
    const money = (version: string) => `https://tasks.example.com/spec/payments/_shared/${version}/money#/$defs/Amount`;

    /** acl/grant 0.2 as a current bundle carries it: every definition but Response is shared. */
    function aclWithOrigins(): Task {
      const acl = exampleTasks().find((t) => t.slug === "acl/grant")!;
      return edited(acl, (b) => {
        const defs = (b.schema as { $defs: Record<string, unknown> }).$defs;
        b.defOrigins = Object.fromEntries(
          Object.keys(defs)
            .filter((k) => k !== "Response")
            .map((k) => [k, k === "Ext" ? FRAMEWORK_EXT : `https://trusttasks.org/spec/acl/_shared/0.2/acl-entry#/$defs/${k}`]),
        );
      });
    }

    /** The sourced fixture with the framework's Ext (as acl/grant carries it) and a shared Amount. */
    function settleWithOrigins(over: Partial<Task> = {}, amountVersion = "1.0", amountDescription?: string): Task {
      const aclExt = (JSON.parse(aclWithOrigins().bytes.toString()).schema.$defs as Record<string, unknown>).Ext;
      return edited(fixtureTask(over), (b) => {
        for (const root of [b.schema, b.responseSchema] as { $defs: Record<string, Record<string, unknown>> }[]) {
          root.$defs.Ext = structuredClone(aclExt) as Record<string, unknown>;
          if (amountDescription) root.$defs.Amount!.description = amountDescription;
        }
        b.defOrigins = { Amount: money(amountVersion), Ext: FRAMEWORK_EXT };
        b.defsOrder = { "/schema": ["Response", "Amount", "Ext"], "/responseSchema": ["Response", "Amount", "Ext"] };
        if (over.slug) {
          b.slug = over.slug;
          b.typeUri = over.typeUri;
        }
      });
    }

    it("declares each shared definition once in _shared/components.ts, and modules import it", async () => {
      const { lock } = app([aclWithOrigins(), settleWithOrigins()], { acme: "https://tasks.example.com" });
      const { files } = await generateFiles({ lock, ...quiet });
      const components = files.get("_shared/components.ts")!;
      assert.ok(components, "the components module is written");
      assert.equal(components.match(/^export interface Ext \{$/gm)?.length, 1, "Ext, reached by both tasks, is declared once");
      assert.match(components, /^export interface AclEntry \{$/m);
      assert.match(components, /^export interface Amount \{$/m);

      const acl = files.get("acl/grant/0.2/payload.ts")!;
      assert.match(acl, /^import type \{ [^}]*\bAclEntry\b[^}]* \} from "\.\.\/\.\.\/\.\.\/_shared\/components\.js";$/m);
      assert.match(acl, /^export type \{ [^}]*\bAclEntry\b[^}]* \};$/m, "re-exported under its own name");
      assert.doesNotMatch(acl, /^export (interface|type) (AclEntry|Ext)\b/m);

      const settle = files.get("acme/payments/settle/1.0/payload.ts")!;
      assert.match(settle, /^import type \{ Amount, Ext \} from "\.\.\/\.\.\/\.\.\/\.\.\/_shared\/components\.js";$/m);
      assert.doesNotMatch(settle, /^export interface Amount\b/m);
      // The schema constant still carries every definition, in defsOrder's order.
      assert.match(settle, /"\$defs": \{\n    "Response": \{[\s\S]*\n    "Amount": \{[\s\S]*\n    "Ext": \{/);
    });

    it("qualifies every shape of a name the locked tasks reach in more than one shape", async () => {
      const refundUri = "https://tasks.example.com/spec/payments/refund/1.0";
      const { lock } = app(
        [settleWithOrigins(), settleWithOrigins({ slug: "payments/refund", typeUri: refundUri }, "1.1", "A refund, in minor units.")],
        { acme: "https://tasks.example.com" },
      );
      const { files } = await generateFiles({ lock, ...quiet });
      const components = files.get("_shared/components.ts")!;
      assert.match(components, /^export interface Amount_AcmePaymentsV1_0 \{$/m);
      assert.match(components, /^export interface Amount_AcmePaymentsV1_1 \{$/m);
      assert.doesNotMatch(components, /^export interface Amount \{$/m);
      assert.match(files.get("acme/payments/settle/1.0/payload.ts")!, /import type \{ Amount_AcmePaymentsV1_0 as Amount, Ext \}/);
      assert.match(files.get("acme/payments/refund/1.0/payload.ts")!, /import type \{ Amount_AcmePaymentsV1_1 as Amount, Ext \}/);
    });

    it("falls back to declaring definitions in the module for a bundle without defOrigins", async () => {
      const keys = exampleTasks().find((t) => t.slug === "keys/list")!;
      assert.equal("defOrigins" in JSON.parse(keys.bytes.toString()), false, "the example's vendored bundles predate defOrigins");
      const { lock } = app([aclWithOrigins(), keys]);
      const { files } = await generateFiles({ lock, ...quiet });
      assert.match(files.get("acl/grant/0.2/payload.ts")!, /from "\.\.\/\.\.\/\.\.\/_shared\/components\.js";$/m);
      const list = files.get("keys/list/0.2/payload.ts")!;
      assert.doesNotMatch(list, /_shared\/components/);
      assert.match(list, /^export interface Ext \{$/m, "an older bundle's module declares its definitions itself");

      const { files: old } = await generateFiles({ lock: app(exampleTasks()).lock, ...quiet });
      assert.equal(old.has("_shared/components.ts"), false, "only older bundles: no components module");
    });

    it("refuses a defsOrder that does not list exactly the $defs", async () => {
      const { lock } = app([edited(aclWithOrigins(), (b) => (b.defsOrder = { "/schema": ["Response"] }))]);
      await assert.rejects(generateFiles({ lock, ...quiet }), /defsOrder: \/schema lists \[Response\]/);
    });

    it("refuses a defOrigins entry for a definition the schema does not have", async () => {
      const { lock } = app([edited(aclWithOrigins(), (b) => ((b.defOrigins as Record<string, string>).Nope = FRAMEWORK_EXT))]);
      await assert.rejects(generateFiles({ lock, ...quiet }), /defOrigins names \$defs\/Nope/);
    });
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

    it("generates one slug from two authorities side by side (the Rust example's lock)", async () => {
      const { files } = await generateFiles({ lock: path.join(EXAMPLE, "trust-tasks.lock"), ...quiet });
      assert.match(files.get("vault/get/0.3/payload.ts")!, /^export const TYPE_URI = "https:\/\/trusttasks\.org\/spec\/vault\/get\/0\.3" as const;$/m);
      assert.match(files.get("acme/vault/get/0.3/payload.ts")!, /^export const TYPE_URI = "https:\/\/tasks\.example\.com\/spec\/vault\/get\/0\.3" as const;$/m);
      assert.match(files.get("index.ts")!, /^export \* as VaultGet_v0_3 from "\.\/vault\/get\/0\.3\/payload\.js";$/m);
      assert.match(files.get("acme/index.ts")!, /^export \* as VaultGet_v0_3 from "\.\/vault\/get\/0\.3\/payload\.js";$/m);
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

describe("a bare component's nested branch types", () => {
  /** A registry bundle vendored under test/fixtures/registry. */
  const registryTask = (slug: string, version: string): Task => {
    const bytes = fs.readFileSync(path.join(PKG, "test", "fixtures", "registry", ...slug.split("/"), `${version}.bundle.json`));
    return { slug, version, typeUri: JSON.parse(bytes.toString()).typeUri, bytes };
  };

  // device/list 0.3 reaches one shape of `ConsumerKind`, so it is bare, and its
  // `oneOf` branches are titled `Companion` and `Service`. auth/oob/claim 0.2
  // shares a definition named `Service` too. Before this was handled the
  // compiler numbered the second `Service1` and generation refused the lock.
  it("are qualified when a title collides, and modules still bind the plain name", async () => {
    const { lock } = app([registryTask("device/list", "0.3"), registryTask("auth/oob/claim", "0.2")]);
    const { files } = await generateFiles({ lock, ...quiet });
    const text = (p: string) => files.get(p)!;

    const components = text("_shared/components.ts");
    assert.match(components, /^export interface Service \{$/m, "auth/oob's Service stays bare");
    assert.match(components, /^export interface Service_DeviceV0_2 \{$/m, "the package's name for device's branch");
    assert.match(components, /^export type ConsumerKind = Companion_DeviceV0_2 \| Service_DeviceV0_2;$/m);
    assert.doesNotMatch(components, /\bService1\b/);

    assert.match(text("device/list/0.3/payload.ts"), /\bService_DeviceV0_2 as Service\b/);
    assert.match(text("auth/oob/claim/0.2/payload.ts"), /import type \{[^}]*\bService\b(?! as)/);
  });

  it("keep their own titles when nothing collides", async () => {
    const { lock } = app([registryTask("device/list", "0.3")]);
    const { files } = await generateFiles({ lock, ...quiet });
    const components = files.get("_shared/components.ts")!;
    assert.match(components, /^export type ConsumerKind = Companion \| Service;$/m);
  });
});
