// Import the built package the way a consumer does, from Node, as ESM.
//
// `tsc --noEmit` cannot catch what this catches. The package is ESM, and Node
// requires an explicit extension on a relative ESM specifier; TypeScript's
// `Bundler` moduleResolution accepts an extensionless one and emits it
// verbatim. So `dist/index.js` shipped specifiers Node could not resolve and
// every check stayed green — the root import of @openvtc/trust-tasks failed
// with ERR_MODULE_NOT_FOUND from 0.2.55 through 0.2.58. Nobody noticed because
// the package was types only: `import type` is erased, and bundlers tolerate
// extensionless paths.
//
// Run against `dist/`, after `npm run build`.

import assert from "node:assert/strict";

const pkg = await import("../dist/index.js");

// The runtime — the half that has to survive the round trip through `tsc`.
assert.equal(typeof pkg.consumeInbound, "function", "consumeInbound missing from root export");
assert.equal(typeof pkg.respondWith, "function", "respondWith missing from root export");
assert.equal(typeof pkg.familyCode, "function", "familyCode missing from root export");
assert.equal(typeof pkg.StaticTransport, "function", "StaticTransport missing from root export");
assert.equal(
  typeof pkg.InMemoryReplayGuard,
  "function",
  "InMemoryReplayGuard missing from root export",
);
assert.equal(
  typeof pkg.consequentialChecks,
  "function",
  "consequentialChecks missing from root export",
);

// A generated module, reached through the root barrel.
const grant = pkg.AclGrant_v0_1;
assert.ok(grant, "AclGrant_v0_1 missing from root export");
assert.equal(grant.TYPE_URI, "https://trusttasks.org/spec/acl/grant/0.1");
assert.equal(grant.SPEC.isProofRequired, true, "acl/grant declares proof REQUIRED (see #174)");

// The two halves actually working together: a spec-declared requirement
// reaching the pipeline through the built artifact.
const outcome = await pkg.consumeInbound({
  transport: new pkg.StaticTransport({ issuer: "did:web:org.example" }),
  spec: grant.SPEC,
  proofPolicy: { kind: "acceptUnverified" },
  payloadPolicy: { kind: "acceptUnvalidated" },
  // Required as of 0.12.17 (SPEC §7.2 items 4 and 11). This document is
  // refused before the record is ever consulted; `consequentialChecks` is what
  // a real acl/grant consumer passes.
  checks: pkg.notConsequentialChecks(),
  doc: {
    id: "req-1",
    type: grant.TYPE_URI,
    issuer: "did:web:org.example",
    recipient: "did:web:maintainer.example",
    payload: { entry: { subject: "did:key:zAlice", role: "admin" } },
  },
  myVid: "did:web:maintainer.example",
  now: Date.now(),
  newErrorId: () => "err-1",
  handler: () => assert.fail("handler must not run for a proofless proof-REQUIRED document"),
});
assert.equal(outcome.kind, "rejected");
assert.equal(outcome.error.payload.code, "proofRequired");

// §7.2 item 2 is reachable from the built artifact: the schema ships on SPEC,
// and a validator wired in at the call site is actually consulted. A consumer
// importing the package must be able to do this without a build step of their
// own, which is the property that was missing (#230).
assert.ok(grant.SPEC.payloadSchema, "SPEC must carry payloadSchema");
assert.ok(grant.PAYLOAD_SCHEMA, "the module must export PAYLOAD_SCHEMA");

let sawSchema = null;
const validated = await pkg.consumeInbound({
  transport: new pkg.StaticTransport({ issuer: "did:web:org.example" }),
  spec: grant.SPEC,
  proofPolicy: { kind: "acceptUnverified" },
  payloadPolicy: {
    kind: "validate",
    // Mirrors proofPolicy: the value is a PayloadValidator object, not a
    // bare function, so the call is `payloadPolicy.validate.validate(...)`.
    validate: {
      validate: (schema) => {
        sawSchema = schema;
        return { ok: false, errors: ["smoke: refusing on purpose"] };
      },
    },
  },
  checks: pkg.notConsequentialChecks(),
  doc: {
    id: "req-2",
    type: grant.TYPE_URI,
    issuer: "did:web:org.example",
    recipient: "did:web:maintainer.example",
    proof: { type: "DataIntegrityProof" },
    payload: { entry: { subject: "did:key:zAlice", role: "admin" } },
  },
  myVid: "did:web:maintainer.example",
  now: Date.now(),
  newErrorId: () => "err-2",
  handler: () => assert.fail("handler must not run when the payload policy rejects"),
});
assert.equal(validated.kind, "rejected");
assert.equal(validated.error.payload.code, "malformedRequest");
assert.ok(sawSchema, "the validator must receive the schema, not undefined");

// The 0.9.0 break is legible to a JavaScript caller, who gets no type error.
await assert.rejects(
  async () =>
    pkg.consumeInbound({
      transport: new pkg.StaticTransport({ issuer: "did:web:org.example" }),
      spec: grant.SPEC,
      proofPolicy: { kind: "acceptUnverified" },
      doc: { id: "req-3", type: grant.TYPE_URI, payload: {} },
      myVid: "did:web:maintainer.example",
      now: Date.now(),
      newErrorId: () => "err-3",
      handler: () => undefined,
    }),
  /payloadPolicy` is required as of 0.9.0/,
  "an omitted payloadPolicy must name itself, not fail on a property read",
);

console.log(`dist root import OK — ${Object.keys(pkg).length} exports, pipeline reachable`);
