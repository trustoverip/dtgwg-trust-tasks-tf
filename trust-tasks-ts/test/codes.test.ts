/**
 * SPEC.md §8.3 / §8.5 error-code tests, mirroring the `payload.rs` and
 * `error.rs` test sets in trust-tasks-rs.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  extendedCode,
  familyCode,
  isStandardCode,
  normalizeCode,
  slugFromTypeUri,
} from "../src/_runtime/index.js";

const GRANT = "https://trusttasks.org/spec/acl/grant/0.1";
const GRANT_RESPONSE = `${GRANT}#response`;
const DELETE = "https://trusttasks.org/spec/did-management/did/delete/0.1";
const DISCOVERY = "https://trusttasks.org/spec/trust-task-discovery/0.1";

describe("§8.3 standard codes", () => {
  it("accepts the 0.2 lowerCamelCase spellings", () => {
    assert.ok(isStandardCode("proofRequired"));
    assert.ok(isStandardCode("identityMismatch"));
    assert.ok(isStandardCode("expired"));
  });

  it("accepts the frozen 0.1 snake_case spellings and normalizes them", () => {
    // A 0.2 consumer must still read an error response from a 0.1 peer;
    // otherwise `proof_required` reads as an unrecognized extended code and
    // falls through to taskFailed (§8.5), losing the meaning.
    assert.ok(isStandardCode("proof_required"));
    assert.equal(normalizeCode("proof_required"), "proofRequired");
    assert.equal(normalizeCode("identity_mismatch"), "identityMismatch");
  });

  it("leaves an extended code alone", () => {
    assert.equal(normalizeCode("acl/grant:roleNotRecognized"), "acl/grant:roleNotRecognized");
    assert.ok(!isStandardCode("acl/grant:roleNotRecognized"));
  });
});

describe("§8.5 extended codes", () => {
  it("sources the namespace from the Type URI", () => {
    assert.equal(extendedCode(GRANT, "roleNotRecognized"), "acl/grant:roleNotRecognized");
  });

  it("works for a single-segment slug", () => {
    assert.equal(
      extendedCode(DISCOVERY, "filterUnsupported"),
      "trust-task-discovery:filterUnsupported",
    );
  });

  it("strips the #response fragment before deriving the slug", () => {
    // An error raised while handling a response still belongs to the bare slug;
    // namespacing it `acl/grant#response` would name nothing.
    assert.equal(extendedCode(GRANT_RESPONSE, "roleNotRecognized"), "acl/grant:roleNotRecognized");
  });

  it("accepts both casings of the local part", () => {
    assert.equal(extendedCode(GRANT, "documentRevoked"), "acl/grant:documentRevoked");
    assert.equal(extendedCode(GRANT, "document_revoked"), "acl/grant:document_revoked");
  });

  it("rejects a leading capital in the local part", () => {
    // Only the first character must be lowercase; the resulting code would
    // otherwise fail to round-trip.
    assert.throws(() => extendedCode(GRANT, "BadLocal"), /must match/);
  });
});

describe("§8.5 rule 2 — family namespaces", () => {
  it("accepts each path prefix of the slug", () => {
    assert.equal(
      familyCode(DELETE, "did-management", "unknown_domain"),
      "did-management:unknown_domain",
    );
    assert.equal(
      familyCode(DELETE, "did-management/did", "unknown_domain"),
      "did-management/did:unknown_domain",
    );
  });

  it("accepts the full slug, making it a superset of extendedCode", () => {
    assert.equal(
      familyCode(DELETE, "did-management/did/delete", "notOwner"),
      "did-management/did/delete:notOwner",
    );
  });

  it("rejects a sibling's slug", () => {
    // A sibling shares a prefix but is not itself one — exactly the confusion
    // §8.5 forbids ("never that of a related or referenced specification").
    assert.throws(() => familyCode(GRANT, "acl/revoke", "borrowedCode"), /path prefix/);
  });

  it("rejects an unrelated namespace", () => {
    assert.throws(() => familyCode(GRANT, "vault", "somethingElse"), /path prefix/);
  });

  it("rejects a partial segment", () => {
    // `ac` is a string prefix of `acl/grant` but names nothing.
    assert.throws(() => familyCode(GRANT, "ac", "somethingElse"), /path prefix/);
  });

  it("strips the #response fragment before checking the prefix", () => {
    assert.equal(familyCode(GRANT_RESPONSE, "acl", "permissionDenied"), "acl:permissionDenied");
  });
});

describe("slugFromTypeUri", () => {
  it("drops the version segment and any fragment", () => {
    assert.equal(slugFromTypeUri(GRANT), "acl/grant");
    assert.equal(slugFromTypeUri(GRANT_RESPONSE), "acl/grant");
    assert.equal(slugFromTypeUri(DELETE), "did-management/did/delete");
  });

  it("refuses a URI outside the Trust Tasks namespace", () => {
    assert.throws(() => slugFromTypeUri("https://example.com/spec/acl/grant/0.1"), /Type URI/);
  });
});
