/**
 * SPEC.md §6.1 Type URI parsing, mirroring the `type_uri.rs` test set in
 * trust-tasks-rs — the vectors are Rust's, so the two parsers agree.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import {
  FRAMEWORK_RESERVED_SLUGS,
  TypeUriError,
  formatTypeUri,
  parseTypeUri,
  sameSpecification,
  tryParseTypeUri,
  typeUriWithVersion,
  type TypeUriErrorKind,
} from "../src/_runtime/index.js";

function kindOf(uri: string): TypeUriErrorKind | "ok" {
  try {
    parseTypeUri(uri);
    return "ok";
  } catch (e) {
    assert.ok(e instanceof TypeUriError, `${uri}: ${e}`);
    return e.kind;
  }
}

describe("parseTypeUri — the three §6.1 forms", () => {
  it("parses the public registry form", () => {
    const u = parseTypeUri("https://trusttasks.org/spec/trust-task-error/0.5");
    assert.deepEqual(u, {
      authority: "https://trusttasks.org",
      form: "path",
      slug: "trust-task-error",
      major: 0,
      minor: 5,
      variant: null,
    });
    assert.equal(parseTypeUri("https://trusttasks.org/spec/acl/grant/0.1").slug, "acl/grant");
  });

  it("parses the path form under any authority, keeping the authority verbatim", () => {
    const https = parseTypeUri("https://example.com/trust-tasks/spec/my-task/0.1");
    assert.equal(https.authority, "https://example.com/trust-tasks");
    assert.equal(https.form, "path");
    assert.equal(https.slug, "my-task");
    const did = parseTypeUri("did:web:acme.example/spec/payments/settle/1.0#response");
    assert.deepEqual(did, {
      authority: "did:web:acme.example",
      form: "path",
      slug: "payments/settle",
      major: 1,
      minor: 0,
      variant: "response",
    });
  });

  it("parses the URN form, its ':'-separated slug segments read '/'-separated", () => {
    assert.deepEqual(parseTypeUri("urn:example:acme:spec:payments:settle:1.0#request"), {
      authority: "urn:example:acme",
      form: "urn",
      slug: "payments/settle",
      major: 1,
      minor: 0,
      variant: "request",
    });
    const sub = parseTypeUri("urn:example:tasks:sub:spec:pay:3.10");
    assert.equal(sub.authority, "urn:example:tasks:sub");
    assert.equal(sub.slug, "pay");
    assert.equal(sub.minor, 10);
  });

  it("round-trips every §6.1 example with each variant", () => {
    for (const [uri, slug] of [
      ["https://trusttasks.org/spec/acl/grant/0.1", "acl/grant"],
      ["https://example.com/trust-tasks/spec/acl/grant/0.1", "acl/grant"],
      ["did:example:123456789abcdefghi/spec/acl/grant/0.1", "acl/grant"],
      ["did:web:tasks.example.com:x/spec/pay/0.1", "pay"],
      ["urn:example:tasks:spec:acl:grant:0.1", "acl/grant"],
      ["urn:example:tasks:sub:spec:pay:3.10", "pay"],
      ["tag:example.com,2026:tasks/spec/acl/grant/0.1", "acl/grant"],
    ] as const) {
      for (const fragment of ["", "#request", "#response"]) {
        const s = `${uri}${fragment}`;
        const parsed = parseTypeUri(s);
        assert.equal(parsed.slug, slug, s);
        assert.equal(parsed.variant, fragment === "" ? null : fragment.slice(1));
        assert.equal(formatTypeUri(parsed), s);
        assert.equal(formatTypeUri({ ...parsed, variant: null }), uri);
      }
    }
  });

  it("changes only the version with typeUriWithVersion, in both forms", () => {
    const urn = parseTypeUri("urn:example:tasks:spec:acl:grant:0.1#response");
    assert.equal(formatTypeUri(typeUriWithVersion(urn, 0, 2)), "urn:example:tasks:spec:acl:grant:0.2#response");
    const did = parseTypeUri("did:web:acme.example/spec/acl/grant/0.3");
    assert.equal(formatTypeUri(typeUriWithVersion(did, 0, 1)), "did:web:acme.example/spec/acl/grant/0.1");
  });

  it("treats the authority and form as part of identity", () => {
    const uris = [
      "https://trusttasks.org/spec/vault/get/0.3",
      "https://tasks.example.com/spec/vault/get/0.3",
      "urn:example:tasks:spec:vault:get:0.3",
      "did:web:tasks.example.com/spec/vault/get/0.3",
    ].map(parseTypeUri);
    for (const a of uris) {
      for (const b of uris) {
        assert.equal(a.slug, b.slug);
        assert.equal(sameSpecification(a, b), a === b);
      }
    }
    assert.ok(
      sameSpecification(
        parseTypeUri("did:web:acme.example/spec/vault/get/0.3"),
        parseTypeUri("did:web:acme.example/spec/vault/get/0.1#response"),
      ),
    );
  });
});

describe("parseTypeUri — refusals", () => {
  it("refuses http in every form, and a missing or malformed scheme", () => {
    for (const uri of [
      "http://trusttasks.org/spec/acl/grant/0.1",
      "HTTP://example.com/spec/acl/grant/0.1",
      "http://example.com/x/spec/acl/grant/0.1#response",
      "trusttasks.org/spec/acl/grant/0.1",
      "1https://example.com/spec/a/0.1",
      "",
    ]) {
      assert.equal(kindOf(uri), "scheme", uri);
    }
  });

  it("refuses a shape matching neither form", () => {
    for (const uri of [
      "https:/spec/a/0.1",
      "https:///spec/a/0.1",
      "https:example.com/spec/a/0.1",
      "did:/spec/a/0.1",
      "urn:example:spec:a:0.1", // no NSS prefix before :spec:
      "urn::tasks:spec:a:0.1", // empty NID
      "urn:example:tasks:spec:acl/grant:0.1", // '/' in the URN form
      "urn:example:tasks/spec/acl/grant/0.1", // a urn: in the path form
      "urn:example:acme:payments:settle:1.0", // no :spec:
      "urn:example:acme:spec:settle", // no version
      "did:example:123",
      "https://example.com/spec/0.1", // no slug segment at all
    ]) {
      assert.equal(kindOf(uri), "shape", uri);
    }
  });

  it("refuses a bad slug, including an empty one, in both forms", () => {
    for (const bad of ["Acl", "acl--bad", "-leading", "trailing-", "1numeric-start", "with_underscore"]) {
      assert.equal(kindOf(`https://trusttasks.org/spec/${bad}/1.0`), "slug", bad);
    }
    assert.equal(kindOf("https://example.com/spec//0.1"), "slug");
    assert.equal(kindOf("https://example.com/spec/acl//grant/0.1"), "slug");
    assert.equal(kindOf("urn:example:tasks:spec::0.1"), "slug");
    assert.equal(kindOf("urn:example:tasks:spec:Acl:0.1"), "slug");
  });

  it("refuses a missing or garbage version", () => {
    for (const bad of ["01.0", "1.01", "1", "1.0.0", "a.b", "", "4294967296.0"]) {
      assert.equal(kindOf(`https://trusttasks.org/spec/x/${bad}`), "version", bad);
    }
    assert.equal(kindOf("urn:example:tasks:spec:acl:01.0"), "version");
    assert.equal(kindOf("did:web:acme.example/spec/acl/latest"), "version");
  });

  it("refuses a fragment other than #request / #response", () => {
    assert.equal(kindOf("https://trusttasks.org/spec/acl/grant/0.1#cancel"), "fragment");
    assert.equal(kindOf("urn:example:tasks:spec:acl:0.1#"), "fragment");
  });

  it("refuses an unclaimed slug in the §6.1 reservation, under any authority", () => {
    assert.equal(kindOf("https://example.com/spec/trust-task-custom/0.1"), "reservedSlug");
    assert.equal(kindOf("urn:example:tasks:spec:trust-task-custom:0.1"), "reservedSlug");
    for (const slug of ["trust-task-evil", "trust-ceremony-evil", "trust-task/nested"]) {
      assert.equal(kindOf(`https://trusttasks.org/spec/${slug}/0.1`), "reservedSlug", slug);
    }
    for (const slug of FRAMEWORK_RESERVED_SLUGS) {
      assert.equal(kindOf(`https://trusttasks.org/spec/${slug}/0.1`), "ok", slug);
    }
    assert.equal(kindOf("https://trusttasks.org/spec/trust-tasks/0.1"), "ok", "trust-tasks is outside the reservation");
  });

  it("tryParseTypeUri returns null instead of throwing", () => {
    assert.equal(tryParseTypeUri("http://trusttasks.org/spec/acl/grant/0.1"), null);
    assert.equal(tryParseTypeUri("urn:example:acme:spec:pay:1.0")?.slug, "pay");
  });

  it("allowlists the same framework slugs as specs/spec.meta.schema.json", () => {
    let raw: string;
    try {
      raw = readFileSync(new URL("../../../specs/spec.meta.schema.json", import.meta.url), "utf8");
    } catch {
      return; // published package: the registry is not alongside it
    }
    const listed = JSON.parse(raw).properties.slug.anyOf[1].enum as string[];
    assert.deepEqual([...FRAMEWORK_RESERVED_SLUGS].sort(), [...listed].sort());
  });
});
