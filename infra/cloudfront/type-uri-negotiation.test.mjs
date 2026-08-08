// Rewrite-logic tests for the CloudFront viewer-request function.
//
// The function itself cannot be exercised in CI — it runs on CloudFront's own
// constrained engine, against a distribution this repo does not manage. What is
// testable, and worth testing, is the routing decision: which requests get
// rewritten, to what, and which are left alone. A mistake there either breaks
// the human-facing site or silently stops serving schemas at the URIs SPEC §6.2
// promises, and neither shows up until someone hits the live site.
//
// The function is loaded as source and evaluated rather than imported, because
// CloudFront Functions declare a bare `function handler(event)` with no module
// system — the same reason it cannot use ESM idioms.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(join(here, "type-uri-negotiation.js"), "utf8");
const handler = new Function(`${source}; return handler;`)();

const SCHEMA = "application/schema+json";
const HTML = "text/html";

/** Run a request through the function and return the resulting URI. */
function route(uri, accept) {
  const headers = accept === undefined ? {} : { accept: { value: accept } };
  return handler({ request: { uri, headers } }).uri;
}

describe("rewrites a Type URI to its schema", () => {
  it("maps the framework envelope out of the task tree", () => {
    // `trust-task` is reserved for the framework (§6.1) and its envelope schema
    // lives under _framework/, not alongside a task.
    assert.equal(
      route("/spec/trust-task/0.2", SCHEMA),
      "/specs/_framework/0.2/trust-task.schema.json",
    );
  });

  it("maps a single-segment slug", () => {
    assert.equal(
      route("/spec/acl/grant/0.1", SCHEMA),
      "/specs/acl/grant/0.1/payload.schema.json",
    );
  });

  it("maps a multi-segment slug", () => {
    assert.equal(
      route("/spec/did-management/did/delete/0.1", SCHEMA),
      "/specs/did-management/did/delete/0.1/payload.schema.json",
    );
  });

  it("tolerates a trailing slash", () => {
    assert.equal(
      route("/spec/acl/grant/0.1/", SCHEMA),
      "/specs/acl/grant/0.1/payload.schema.json",
    );
  });

  it("handles a 1.0-style version as readily as 0.x", () => {
    assert.equal(
      route("/spec/vta/did-templates/create/2.0", SCHEMA),
      "/specs/vta/did-templates/create/2.0/payload.schema.json",
    );
  });
});

describe("passes real files through untouched", () => {
  it("does not re-enter on the asset trees", () => {
    // /specs/ is where the schema rewrite points; rewriting again would loop.
    // These must also stay untouched so a missing object 404s honestly rather
    // than returning the SPA shell under a 200 — the bug this function exists
    // to fix.
    for (const uri of [
      "/specs/acl/grant/0.1/payload.schema.json",
      "/specs/_framework/0.2/trust-task.schema.json",
      "/bindings/didcomm/0.1/spec.md",
      "/assets/data.js",
    ]) {
      assert.equal(route(uri, SCHEMA), uri);
      assert.equal(route(uri, HTML), uri);
    }
  });

  it("passes root files through", () => {
    for (const uri of ["/index.html", "/registry.json", "/SPEC.md"]) {
      assert.equal(route(uri, HTML), uri);
    }
  });

  it("404s a missing asset rather than swallowing it", () => {
    // The point of the inverted rule: anything under an asset prefix is left
    // for the origin to answer, so a typo produces a real 404.
    const missing = "/specs/does/not/exist.schema.json";
    assert.equal(route(missing, SCHEMA), missing);
  });
});

describe("falls back to the SPA for client-side routes", () => {
  it("serves the shell for app routes", () => {
    for (const uri of ["/", "/categories", "/about", "/registry", "/specification"]) {
      assert.equal(route(uri, HTML), "/index.html");
    }
  });

  it("serves the shell for a Type URI viewed in a browser", () => {
    // No schema Accept, so this is a human reading prose — the SPA renders it.
    assert.equal(route("/spec/acl/grant/0.1", HTML), "/index.html");
    assert.equal(route("/spec/acl/grant/0.1", undefined), "/index.html");
  });

  it("serves the shell for an unknown Type URI in a browser", () => {
    // The SPA shows its own not-found UI. Only the schema request 404s.
    assert.equal(route("/spec/does-not-exist/0.1", HTML), "/index.html");
  });
});

describe("refuses to rewrite anything that is not a plain slug", () => {
  // These fall through to the SPA rather than building a path into the specs
  // tree. That is the safe direction: a bogus URL renders the site, it does not
  // reach for a file.
  it("rejects path traversal", () => {
    assert.equal(route("/spec/../../etc/0.1", SCHEMA), "/index.html");
    assert.equal(route("/spec/acl/../../../x/0.1", SCHEMA), "/index.html");
  });

  it("rejects segments outside the §6.1 slug grammar", () => {
    assert.equal(route("/spec/AcL/grant/0.1", SCHEMA), "/index.html");
    assert.equal(route("/spec/acl_grant/0.1", SCHEMA), "/index.html");
    assert.equal(route("/spec/-leading/0.1", SCHEMA), "/index.html");
  });

  it("does not rewrite a path with no version segment", () => {
    assert.equal(route("/spec/acl/grant", SCHEMA), "/index.html");
  });
});
