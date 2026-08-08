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

console.log(`dist root import OK — ${Object.keys(pkg).length} exports, pipeline reachable`);
