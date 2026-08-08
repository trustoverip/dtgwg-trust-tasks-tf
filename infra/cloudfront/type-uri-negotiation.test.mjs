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

describe("leaves everything else alone", () => {
  it("does not touch a request without the schema Accept", () => {
    // The human-facing site must keep working; this is the check that catches a
    // rewrite condition broad enough to break it.
    assert.equal(route("/spec/acl/grant/0.1", HTML), "/spec/acl/grant/0.1");
    assert.equal(route("/spec/acl/grant/0.1", undefined), "/spec/acl/grant/0.1");
  });

  it("does not re-enter on the asset tree", () => {
    // /specs/ is where the rewrite points. Rewriting it again would loop.
    const asset = "/specs/acl/grant/0.1/payload.schema.json";
    assert.equal(route(asset, SCHEMA), asset);
  });

  it("ignores paths that do not end in a MAJOR.MINOR version", () => {
    assert.equal(route("/spec/acl/grant", SCHEMA), "/spec/acl/grant");
    assert.equal(route("/spec/categories", SCHEMA), "/spec/categories");
    assert.equal(route("/", SCHEMA), "/");
  });
});

describe("refuses to rewrite anything that is not a plain slug", () => {
  it("rejects path traversal", () => {
    // Rewriting this would build a path escaping the specs tree.
    assert.equal(route("/spec/../../etc/0.1", SCHEMA), "/spec/../../etc/0.1");
    assert.equal(route("/spec/acl/../../../x/0.1", SCHEMA), "/spec/acl/../../../x/0.1");
  });

  it("rejects segments outside the §6.1 slug grammar", () => {
    assert.equal(route("/spec/AcL/grant/0.1", SCHEMA), "/spec/AcL/grant/0.1");
    assert.equal(route("/spec/acl_grant/0.1", SCHEMA), "/spec/acl_grant/0.1");
    assert.equal(route("/spec/-leading/0.1", SCHEMA), "/spec/-leading/0.1");
  });
});
