// An application that generates only the Trust Tasks it pins.
//
//   npm run generate   # trust-tasks-gen-ts --lock trust-tasks.lock --out src/trust-tasks
//   npm run build && npm start
//
// `trust-tasks.lock` pins acl/grant 0.2, keys/list 0.2 and vault/get 0.3 from
// the registry (the same three as examples/consumer-app, the Rust example),
// plus payments/settle 1.0 from a third-party source, `acme` (SPEC §6.5). The
// generated modules import the runtime from `@openvtc/trust-tasks/runtime` —
// never the package root, which would load every prebuilt module.

import assert from "node:assert/strict";

import {
  InMemoryReplayGuard,
  StaticTransport,
  consequentialChecks,
  consumeInbound,
  readForSpec,
  type TrustTaskDocument,
} from "@openvtc/trust-tasks/runtime";

import { Acme, AclGrant_v0_2, KeysList_v0_2, VaultGet_v0_3, declareExtends } from "./trust-tasks/index.js";

declareExtends();

const ME = "did:web:vault.example";
const PEER = "did:web:client.example";
const NOW = Date.parse("2026-10-26T12:00:00Z");
const ISSUED = new Date(NOW - 1000).toISOString();

const entry: VaultGet_v0_3.VaultEntry = {
  id: "entry-1",
  contextId: "ctx-1",
  targets: [{ kind: "webOrigin", origin: "https://github.com" }],
  label: "GitHub",
  secretKind: "password",
  createdAt: ISSUED,
  updatedAt: ISSUED,
  version: 1,
};

async function consume(doc: TrustTaskDocument<VaultGet_v0_3.Payload>, guard = new InMemoryReplayGuard()) {
  return consumeInbound<VaultGet_v0_3.Payload, VaultGet_v0_3.Response>({
    transport: new StaticTransport({ issuer: PEER }),
    spec: VaultGet_v0_3.SPEC,
    proofPolicy: { kind: "acceptUnverified" },
    payloadPolicy: { kind: "acceptUnvalidated" },
    checks: consequentialChecks(guard),
    doc,
    myVid: ME,
    now: NOW,
    newErrorId: () => "err-1",
    clock: () => new Date(NOW).toISOString(),
    handler: (accepted) => ({
      id: "resp-1",
      threadId: accepted.id,
      type: VaultGet_v0_3.RESPONSE_TYPE_URI,
      issuer: ME,
      recipient: accepted.issuer,
      issuedAt: new Date(NOW).toISOString(),
      payload: { entry },
    }),
  });
}

// 1. A request through the §7.2 pipeline, answered with the generated response type.
const request: TrustTaskDocument<VaultGet_v0_3.Payload> = {
  id: "req-1",
  type: VaultGet_v0_3.TYPE_URI,
  issuer: PEER,
  recipient: ME,
  issuedAt: ISSUED,
  payload: { id: "entry-1" },
};
const wire = JSON.parse(JSON.stringify(request)) as unknown;
const read = readForSpec<VaultGet_v0_3.Payload>(wire, VaultGet_v0_3.SPEC);
assert.equal(read.kind, "accepted");
const handled = await consume(read.kind === "accepted" ? read.doc : request);
assert.equal(handled.kind, "handled");
if (handled.kind === "handled") {
  const back = JSON.parse(JSON.stringify(handled.response)) as TrustTaskDocument<VaultGet_v0_3.Response>;
  assert.equal(back.type, "https://trusttasks.org/spec/vault/get/0.3#response");
  assert.equal(back.payload.entry.label, "GitHub");
}

// 2. A policy refusal: vault/get 0.3 requires a recipient.
const { recipient: _dropped, ...unaddressed } = request;
const refused = await consume({ ...unaddressed, id: "req-2" });
assert.equal(refused.kind, "rejected");

// 3. A version the application did not lock is refused.
const other = readForSpec({ ...request, type: "https://trusttasks.org/spec/vault/get/0.2" }, VaultGet_v0_3.SPEC);
assert.equal(other.kind === "refused" && other.reason.code, "unsupportedVersion");

// 4. Policy constants and identity come from the pinned bundles.
assert.equal(AclGrant_v0_2.SPEC.isProofRequired, true);
assert.equal(KeysList_v0_2.TYPE_URI, "https://trusttasks.org/spec/keys/list/0.2");

// 5. The third-party task: its identity is its own authority's Type URI.
const settle = Acme.PaymentsSettle_v1_0;
assert.equal(settle.TYPE_URI, "https://tasks.example.com/spec/payments/settle/1.0");
assert.equal(settle.SPEC.isProofRequired, true);
const payment: Acme.PaymentsSettle_v1_0.Payload = { invoiceId: "inv-1", amount: { value: 1250, currency: "EUR" } };
const sameType = readForSpec<Acme.PaymentsSettle_v1_0.Payload>(
  { id: "req-3", type: settle.TYPE_URI, issuer: PEER, recipient: ME, payload: payment },
  settle.SPEC,
);
assert.equal(sameType.kind, "accepted");

console.log("consumer-app-ts: 4 locked tasks generated; round trip, refusals and the sourced task all OK");
