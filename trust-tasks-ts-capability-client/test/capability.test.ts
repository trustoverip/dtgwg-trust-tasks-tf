import { test } from "node:test";
import assert from "node:assert/strict";
import {
  buildListDocument,
  buildToggleDocument,
  buildGitTrustGrant,
  buildGitTrustRevoke,
  buildDocument,
  newAttempt,
  correlationThread,
  repliesTo,
  typeUriParts,
  parseEnvelopeDocument,
  parseEnvelopeDocumentFor,
  classifyGitTrustReply,
  parseCapabilityReply,
  parseEnvelopeReply,
  GIT_TRUST_GRANT_TYPE,
  GIT_TRUST_REVOKE_TYPE,
  GIT_TRUST_ALREADY_GRANTED_CODE,
  GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL,
  GIT_TRUST_NOT_GRANTED_CODE,
  CAPABILITY_LIST_TYPE,
  type CapabilityDocument,
} from "../src/index.js";

const ERROR_TYPE = "https://trusttasks.org/spec/trust-task-error/0.5";

function reply(type: string, threadId: string, payload: Record<string, unknown>): CapabilityDocument {
  return { id: "urn:uuid:reply", type, threadId, payload };
}

test("builders are addressed and typed", () => {
  const list = buildListDocument("did:example:me", "did:example:vtc");
  assert.equal(typeUriParts(list.type).slug, "governance/capability/list");
  assert.equal(list.issuer, "did:example:me");
  assert.equal(list.recipient, "did:example:vtc");
  assert.equal(list.payload.status, "all");

  const enable = buildToggleDocument("did:example:me", "did:example:vtc", "git-trust", "0.1", true);
  assert.equal(typeUriParts(enable.type).slug, "governance/capability/enable");
  assert.deepEqual(enable.payload.config, { authority: "did:example:vtc" });
  assert.equal(enable.payload.version, "0.1");

  const disable = buildToggleDocument("did:example:me", "did:example:vtc", "git-trust", "0.1", false);
  assert.equal(typeUriParts(disable.type).slug, "governance/capability/disable");
  assert.deepEqual(disable.payload, { capability: "git-trust" });

  const grant = buildGitTrustGrant("did:a", "did:r", "did:s", "openvtc");
  assert.equal(typeUriParts(grant.type).slug, "git-trust/grant");
  assert.equal(grant.payload.subject, "did:s");
  assert.equal(grant.payload.resource, "openvtc");

  const revoke = buildGitTrustRevoke("did:a", "did:r", "did:s", "openvtc", "ended");
  assert.equal(revoke.payload.reason, "ended");
  const revokeNoReason = buildGitTrustRevoke("did:a", "did:r", "did:s", "openvtc");
  assert.equal("reason" in revokeNoReason.payload, false);
});

test("build mints a fresh id and issuedAt on every call", () => {
  const a = buildListDocument("did:me", "did:vtc");
  const b = buildListDocument("did:me", "did:vtc");
  assert.notEqual(a.id, b.id);
  assert.match(a.id, /^urn:uuid:/);
  assert.ok(a.issuedAt);
});

test("newAttempt: fresh id and issuedAt, no proof, same content", () => {
  const previous = buildGitTrustGrant("did:a", "did:r", "did:s", "openvtc");
  previous.proof = { type: "DataIntegrityProof" } as CapabilityDocument["proof"];
  const next = newAttempt(previous);
  assert.notEqual(next.id, previous.id);
  assert.equal(next.proof, undefined);
  assert.deepEqual(next.payload, previous.payload);
  assert.equal(next.type, previous.type);
  assert.equal(next.issuer, previous.issuer);
});

test("newAttempt preserves an explicit threadId", () => {
  const previous = { ...buildGitTrustGrant("did:a", "did:r", "did:s", "x"), threadId: "urn:thread:1" };
  assert.equal(newAttempt(previous).threadId, "urn:thread:1");
});

test("typeUriParts: slug and response flag", () => {
  assert.deepEqual(typeUriParts(GIT_TRUST_GRANT_TYPE), { slug: "git-trust/grant", isResponse: false });
  assert.deepEqual(typeUriParts(`${GIT_TRUST_GRANT_TYPE}#response`), { slug: "git-trust/grant", isResponse: true });
  assert.deepEqual(typeUriParts(`${GIT_TRUST_GRANT_TYPE}#request`), { slug: "git-trust/grant", isResponse: false });
  assert.equal(typeUriParts(ERROR_TYPE).slug, "trust-task-error");
  assert.equal(typeUriParts(CAPABILITY_LIST_TYPE).slug, "governance/capability/list");
});

test("correlationThread and repliesTo", () => {
  assert.equal(correlationThread({ id: "urn:doc:1" }), "urn:doc:1");
  assert.equal(correlationThread({ id: "urn:doc:1", threadId: "urn:thread:1" }), "urn:thread:1");
  assert.equal(repliesTo({ threadId: "t" }, "t"), true);
  assert.equal(repliesTo({ threadId: "t" }, "other"), false);
  assert.equal(repliesTo({}, "t"), false);
});

test("parseEnvelopeDocument requires a threaded Trust Task document", () => {
  const doc = reply(GIT_TRUST_GRANT_TYPE, "urn:thread:1", {});
  assert.deepEqual(parseEnvelopeDocument(doc), ["urn:thread:1", doc]);
  assert.equal(parseEnvelopeDocument({ id: "x", type: "y" }), undefined); // no threadId
  assert.equal(parseEnvelopeDocument({ hello: "world" }), undefined);
  assert.equal(parseEnvelopeDocumentFor(doc, "urn:thread:1"), doc);
  assert.equal(parseEnvelopeDocumentFor(doc, "other"), undefined);
});

test("classifyGitTrustReply: success", () => {
  const doc = reply(`${GIT_TRUST_GRANT_TYPE}#response`, "urn:thread:1", {});
  assert.deepEqual(classifyGitTrustReply(doc, "urn:thread:1"), { kind: "success" });
});

test("classifyGitTrustReply: idempotent success from the extended code (both spellings)", () => {
  for (const code of [GIT_TRUST_ALREADY_GRANTED_CODE, GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL, GIT_TRUST_NOT_GRANTED_CODE]) {
    const doc = reply(ERROR_TYPE, "urn:thread:1", { code });
    assert.deepEqual(classifyGitTrustReply(doc, "urn:thread:1"), { kind: "idempotentSuccess" }, code);
  }
});

test("classifyGitTrustReply: any other error is rejected with its code and message", () => {
  const doc = reply(ERROR_TYPE, "urn:thread:1", { code: "notAuthorized", message: "nope" });
  assert.deepEqual(classifyGitTrustReply(doc, "urn:thread:1"), {
    kind: "rejected",
    code: "notAuthorized",
    message: "nope",
  });
});

test("classifyGitTrustReply: an uncorrelated reply is not an answer", () => {
  const doc = reply(`${GIT_TRUST_GRANT_TYPE}#response`, "urn:thread:OTHER", {});
  assert.equal(classifyGitTrustReply(doc, "urn:thread:1"), undefined);
});

test("classifyGitTrustReply: free-text idempotence only under the deprecated policy", () => {
  const doc = reply(ERROR_TYPE, "urn:thread:1", { code: "taskFailed", message: "already_granted: did:s on x" });
  assert.deepEqual(classifyGitTrustReply(doc, "urn:thread:1"), {
    kind: "rejected",
    code: "taskFailed",
    message: "already_granted: did:s on x",
  });
  assert.deepEqual(classifyGitTrustReply(doc, "urn:thread:1", { acceptLegacyFreeTextIdempotence: true }), {
    kind: "idempotentSuccess",
  });
});

test("parseCapabilityReply: a listing", () => {
  const doc = reply(`${CAPABILITY_LIST_TYPE}#response`, "urn:thread:1", {
    capabilities: [
      {
        enabled: true,
        enabledAt: "2026-01-01T00:00:00Z",
        delegate: "did:del",
        manifest: { capability: "git-trust", title: "Git Trust", version: "0.1" },
      },
      { enabled: false, manifest: { capability: "audit", version: "1.0" } },
      { enabled: true }, // no manifest — dropped
    ],
  });
  const r = parseCapabilityReply(doc, "urn:thread:1");
  assert.equal(r?.kind, "listing");
  assert.equal(r!.kind === "listing" && r.entries.length, 2);
  const first = r!.kind === "listing" ? r.entries[0] : undefined;
  assert.equal(first?.slug, "git-trust");
  assert.equal(first?.title, "Git Trust");
  assert.equal(first?.enabled, true);
  assert.equal(first?.enabledAt, "2026-01-01T00:00:00Z");
});

test("parseCapabilityReply: a toggle acknowledgement and an error", () => {
  const toggled = reply("https://trusttasks.org/spec/governance/capability/enable/0.1#response", "urn:thread:1", {
    capability: "git-trust",
    enabled: true,
  });
  assert.deepEqual(parseCapabilityReply(toggled, "urn:thread:1"), {
    kind: "toggled",
    capability: "git-trust",
    enabled: true,
  });

  const err = reply(ERROR_TYPE, "urn:thread:1", { code: "notAuthorized" });
  assert.deepEqual(parseCapabilityReply(err, "urn:thread:1"), {
    kind: "rejected",
    code: "notAuthorized",
    message: undefined,
  });
});

test("parseCapabilityReply: correlation and family gating", () => {
  const doc = reply(`${CAPABILITY_LIST_TYPE}#response`, "urn:thread:1", { capabilities: [] });
  assert.equal(parseCapabilityReply(doc, "other"), undefined);
  // A non-response, non-error document is not a reply.
  const request = reply(CAPABILITY_LIST_TYPE, "urn:thread:1", {});
  assert.equal(parseCapabilityReply(request, "urn:thread:1"), undefined);
});

test("parseEnvelopeReply: body → reply in one step", () => {
  const body = reply(`${GIT_TRUST_REVOKE_TYPE}#response`, "urn:thread:9", {});
  assert.equal(parseEnvelopeReply(body, "urn:thread:9"), undefined); // revoke #response is a write reply, not a capability reply
  const listing = reply(`${CAPABILITY_LIST_TYPE}#response`, "urn:thread:9", { capabilities: [] });
  assert.deepEqual(parseEnvelopeReply(listing, "urn:thread:9"), { kind: "listing", entries: [] });
  assert.equal(parseEnvelopeReply(listing, "wrong-thread"), undefined);
});

test("unused import guard", () => {
  // buildDocument is exported for callers building non-standard capability docs.
  const d = buildDocument("did:a", "did:b", "https://trusttasks.org/spec/git-trust/grant/0.1", { x: 1 });
  assert.equal(d.payload.x, 1);
});
