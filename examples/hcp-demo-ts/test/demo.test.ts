// The demo's own tests: hermetic (no network, no files outside this example),
// deterministic (fixed keys, ids and clock), and run with `npm test`.
// SCENARIO.md "Tests every port has" lists them; every port has the same ones.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  AFTER_APPROVAL_WINDOW,
  ASK,
  MANAGEMENT_TOKEN,
  World,
  checkReply,
  queryPayload,
  runDemo,
  stepDiscover,
  stepGrantRequest,
  stepNotice,
  stepRevoke,
} from "../src/demo.js";
import { HcpEntriesQuery_v0_1 as query, HcpGrantsRequest_v0_1 as grantRequest, HcpGrantsRevoked_v0_1 as revoked } from "../src/trust-tasks/index.js";

const GOLDEN = new URL("../../transcript.golden.txt", import.meta.url);
const ERROR = "https://trusttasks.org/spec/trust-task-error/0.6";
const TOKEN = "hcp_at_demo_0001"; // the grant's token, as the provider minted it
const REQUEST = `${grantRequest.TYPE_URI}#request`;

/** A world after the whole demo has run. */
async function afterDemo() {
  const world = new World();
  const result = await runDemo(world);
  return { world, ...result };
}

/** A world after steps 1 and 2: the grant is issued and active. */
async function afterGrant() {
  const world = new World();
  await stepDiscover(world);
  return { world, ...(await stepGrantRequest(world)) };
}

test("1. the six steps print exactly the golden transcript (shared by every language's port)", async () => {
  const { transcript } = await runDemo();
  assert.equal(transcript, readFileSync(GOLDEN, "utf8"));
});

test("2. step 5, a query after revoke, fails with hcp:invalidToken (HCP 6.2.2)", async () => {
  const { postRevokeError } = await runDemo();
  assert.equal(postRevokeError.type, ERROR);
  assert.equal(postRevokeError.payload.code, "hcp:invalidToken");
});

test("3. a document altered after signing is refused with proofInvalid", async () => {
  const { world } = await afterDemo();
  const { contextClient: C, provider: P } = world;
  const signed = world.request(C, P, `${query.TYPE_URI}#request`, queryPayload(MANAGEMENT_TOKEN));
  const tampered = { ...signed, payload: { ...signed.payload, query: "Tell me everything." } };
  const reply = await world.exchange(C, P, tampered);
  assert.equal(reply.payload.code, "proofInvalid");
});

test("4. a correctly signed document that fails its schema is refused with malformedRequest", async () => {
  const { world } = await afterDemo();
  const { contextClient: C, provider: P } = world;
  const { purpose: _dropped, ...noPurpose } = queryPayload(MANAGEMENT_TOKEN);
  const reply = await world.exchange(C, P, world.request(C, P, `${query.TYPE_URI}#request`, noPurpose as query.Payload));
  assert.equal(reply.payload.code, "malformedRequest");
});

test("5. after the notice the client holds no token and has recorded the notice; a later attempt is acknowledged, not acted on", async () => {
  const { world, grantId, notice } = await afterDemo();
  const { provider: P, contextClient: C } = world;
  assert.equal(world.wallet.has(grantId), false);
  assert.deepEqual([...world.processedNotices], [[notice.payload.noticeId, grantId]]);
  // A later delivery attempt: a new document, the same threadId and payload (noticeId included).
  const ack = await world.exchange(P, C, world.request(P, C, `${revoked.TYPE_URI}#request`, notice.payload, notice.threadId));
  assert.equal(ack.type, revoked.RESPONSE_TYPE_URI);
  assert.deepEqual(ack.payload, {});
  assert.equal(world.processedNotices.size, 1);
});

test("6. the notice alone makes the client drop the grant's token and copies", async () => {
  const { world, grantId } = await afterGrant();
  await stepRevoke(world, grantId);
  assert.equal(world.wallet.has(grantId), true);
  await stepNotice(world, grantId);
  assert.equal(world.wallet.has(grantId), false);
});

test("7. the grant's token presented from another VID is refused with permissionDenied", async () => {
  const { world } = await afterGrant();
  const { management: M, provider: P } = world;
  const reply = await world.exchange(M, P, world.request(M, P, `${query.TYPE_URI}#request`, queryPayload(TOKEN)));
  assert.equal(reply.type, ERROR);
  assert.equal(reply.payload.code, "permissionDenied");
});

test("8. a grant request from an issuer that is not a registered client is refused with permissionDenied", async () => {
  const { world } = await afterDemo();
  const { management: M, provider: P } = world;
  const reply = await world.exchange(M, P, world.request(M, P, REQUEST, ASK));
  assert.equal(reply.payload.code, "permissionDenied");
  assert.equal(world.held.size, 1);
});

test("9. a resubmission whose payload differs from the held request is refused with malformedRequest", async () => {
  const { world, threadId } = await afterGrant();
  const { contextClient: C, provider: P } = world;
  const reply = await world.exchange(C, P, world.request(C, P, REQUEST, { ...ASK, expiresIn: 3600 }, threadId));
  assert.equal(reply.payload.code, "malformedRequest");
});

test("10. within the hour after approval a resubmission gets the same grant, never a second one", async () => {
  const { world, threadId, grantId } = await afterGrant();
  const { contextClient: C, provider: P } = world;
  // The same payload with its members in another order: the provider compares JCS forms.
  const reordered = { subjectHint: ASK.subjectHint, expiresIn: ASK.expiresIn, access: ASK.access };
  const reply = await world.exchange(C, P, world.request(C, P, REQUEST, reordered, threadId));
  assert.equal(reply.type, grantRequest.RESPONSE_TYPE_URI);
  assert.equal(reply.payload.grant.grantId, grantId);
  assert.equal(world.grants.length, 1);
});

test("11. a resubmission after the hour is refused with permissionDenied", async () => {
  const { world, threadId } = await afterGrant();
  const { contextClient: C, provider: P } = world;
  world.clock.advance(AFTER_APPROVAL_WINDOW + 1);
  const reply = await world.exchange(C, P, world.request(C, P, REQUEST, ASK, threadId));
  assert.equal(reply.payload.code, "permissionDenied");
});

test("12. a resubmission once the grant is revoked is refused with permissionDenied, and no grant is issued", async () => {
  const { world, threadId } = await afterDemo();
  const { contextClient: C, provider: P } = world;
  const reply = await world.exchange(C, P, world.request(C, P, REQUEST, ASK, threadId));
  assert.equal(reply.payload.code, "permissionDenied");
  assert.equal(world.grants.length, 1);
});

test("13. a reply is used only if it comes from the party asked, on the same thread, and is signed unless the library built it", async () => {
  const world = new World();
  const { provider: P, contextClient: C } = world;
  const base = { id: "urn:uuid:00000000-0000-4000-8000-0000000000f0", type: ERROR, issuer: P.vid, recipient: C.vid, issuedAt: "2026-11-25T10:00:01Z", threadId: "t" };
  const proof = { type: "DataIntegrityProof" } as never; // presence only: the proof itself is verified separately
  checkReply({ ...base, payload: { code: "proofInvalid" } }, P, "t"); // a library-built error, unsigned
  checkReply({ ...base, payload: { code: "permissionDenied" }, proof }, P, "t");
  assert.throws(() => checkReply({ ...base, payload: { code: "permissionDenied" } }, P, "t"), /no proof/);
  assert.throws(() => checkReply({ ...base, payload: { code: "proofInvalid" } }, C, "t"), /issuer/);
  assert.throws(() => checkReply({ ...base, payload: { code: "proofInvalid" } }, P, "other"), /threadId/);
});
