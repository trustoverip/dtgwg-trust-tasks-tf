/**
 * Forward-MINOR acceptance along a declared `extends` chain (SPEC §5.2
 * "Declared extension").
 *
 * No published specification declares `extends` yet, so the generated
 * `EXTENDS_BY_TYPE_URI` is empty and these tests pass their own index — the
 * `index` option of `readForSpec` exists for this and for routing on knowledge
 * a library release does not carry. The 0.3 module is the real generated
 * `auth/authenticate/0.3`; the 0.1 one is a hand-written specimen of the
 * superseded `auth/authenticate/0.1` (`./specimens/authenticate-0.1.ts`), so
 * these tests survive that version's withdrawal. The two are genuinely
 * additive (0.2 adds `sessionKey`, 0.3 adds `principal` +
 * `delegationEvidence`), so the stub chain 0.3 → 0.2 → 0.1 is one the registry
 * build would accept.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  EXTENDS_BY_TYPE_URI,
  InMemoryReplayGuard,
  StaticTransport,
  canonicalJson,
  consequentialChecks,
  consumeInbound,
  extendsChain,
  extendsOf,
  prunePayload,
  readForSpec,
  receivedDocumentOf,
  respondWith,
  sha256Hex,
  type PayloadValidator,
  type ProofVerifier,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";
import * as V01 from "./specimens/authenticate-0.1.js";
import * as V03 from "../src/auth/authenticate/0.3/payload.js";

const ME = "did:web:auth.example";
const PEER = "did:web:subject.example";
const BASE = "https://trusttasks.org/spec/auth/authenticate";

/** 0.3 extends 0.2 extends 0.1 — what the front matter would declare. */
const INDEX = { [`${BASE}/0.3`]: "0.2", [`${BASE}/0.2`]: "0.1" };

const ISSUED = "2026-01-01T00:00:00Z";
const NOW = Date.parse(ISSUED) + 1000;

/**
 * A stand-in for a Data Integrity verifier that is sensitive to every member:
 * `proofValue` is the SHA-256 of the canonical document without its proof. A
 * proof made over the 0.3 document as sent verifies only over those bytes.
 */
const hashVerifier: ProofVerifier = {
  verify(doc) {
    const { proof, ...rest } = doc;
    return proof?.proofValue === sha256Hex(canonicalJson(rest));
  },
};

function sign<P>(doc: TrustTaskDocument<P>): TrustTaskDocument<P> {
  return {
    ...doc,
    proof: {
      type: "DataIntegrityProof",
      cryptosuite: "test-sha256",
      verificationMethod: `${PEER}#key-1`,
      created: ISSUED,
      proofPurpose: "authentication",
      proofValue: sha256Hex(canonicalJson(doc)),
    },
  };
}

/**
 * Stands in for ajv on the cases these tests need: REQUIRED members present,
 * and no member a closed root does not name — the check a 0.1 validator
 * fails an unpruned 0.3 payload on.
 */
const closedRootValidator: PayloadValidator = {
  validate(schema, payload) {
    const s = schema as { required?: string[]; properties?: object; additionalProperties?: boolean };
    const obj = payload as Record<string, unknown>;
    const errors = [
      ...(s.required ?? []).filter((k) => !(k in obj)).map((k) => `missing ${k}`),
      ...(s.additionalProperties === false
        ? Object.keys(obj).filter((k) => !(k in (s.properties ?? {}))).map((k) => `unknown ${k}`)
        : []),
    ];
    return errors.length === 0 ? true : { ok: false, errors };
  },
};

function doc03(over: Partial<TrustTaskDocument<V03.Payload>> = {}): TrustTaskDocument<V03.Payload> {
  return {
    id: "req-1",
    type: V03.TYPE_URI,
    issuer: PEER,
    recipient: ME,
    issuedAt: ISSUED,
    payload: {
      challenge: "c".repeat(32),
      sessionId: "s-1",
      principal: "did:web:principal.example",
      delegationEvidence: { kind: "vta" },
      sessionKey: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    },
    ...over,
  };
}

async function consume01(doc: TrustTaskDocument<V01.Payload>, guard = new InMemoryReplayGuard()) {
  let seen: TrustTaskDocument<V01.Payload> | undefined;
  const outcome = await consumeInbound<V01.Payload, V01.Response>({
    transport: new StaticTransport({ issuer: PEER }),
    spec: V01.SPEC,
    proofPolicy: { kind: "verify", verify: hashVerifier },
    payloadPolicy: { kind: "validate", validate: closedRootValidator },
    checks: consequentialChecks(guard),
    doc,
    myVid: ME,
    now: NOW,
    newErrorId: () => "err-1",
    handler: (accepted) => {
      seen = accepted;
      return respondWith<V01.Payload, V01.Response>(
        accepted,
        "resp-1",
        { session: {} as V01.Response["session"], tokens: {} as V01.Response["tokens"] },
        () => ISSUED,
      );
    },
  });
  return { outcome, seen };
}

describe("extends chain lookup", () => {
  it("reads direct declarations and follows them transitively", () => {
    assert.equal(extendsOf(`${BASE}/0.3`, INDEX), "0.2");
    assert.equal(extendsOf(`${BASE}/0.3#response`, INDEX), "0.2");
    assert.equal(extendsOf(`${BASE}/0.1`, INDEX), null);
    assert.deepEqual(extendsChain(`${BASE}/0.3`, INDEX), ["0.2", "0.1"]);
    assert.deepEqual(extendsChain(`${BASE}/0.3`, (uri) => INDEX[uri as keyof typeof INDEX]), ["0.2", "0.1"]);
  });

  it("stops at a declaration that is not strictly earlier within the MAJOR, or a cycle", () => {
    assert.deepEqual(extendsChain(`${BASE}/0.3`, { [`${BASE}/0.3`]: "0.3" }), []);
    assert.deepEqual(extendsChain(`${BASE}/0.3`, { [`${BASE}/0.3`]: "0.4" }), []);
    assert.deepEqual(extendsChain(`${BASE}/1.1`, { [`${BASE}/1.1`]: "0.9" }), []);
  });

  it("agrees with the generated modules' EXTENDS, and is the default index", async () => {
    // Holds whether or not the front matter declares anything, for every
    // generated module rather than one pinned version.
    const modules = Object.values(await import("../src/index.js")).filter(
      (m): m is { TYPE_URI: string; EXTENDS: string | null } =>
        typeof m === "object" && m !== null && "TYPE_URI" in m && "EXTENDS" in m,
    );
    assert.ok(modules.length > 100, "the root export lists the generated modules");
    for (const m of modules) assert.equal(EXTENDS_BY_TYPE_URI[m.TYPE_URI] ?? null, m.EXTENDS, m.TYPE_URI);
    assert.equal(EXTENDS_BY_TYPE_URI[V03.TYPE_URI] ?? null, V03.EXTENDS);
    const expected = extendsChain(V03.TYPE_URI).includes("0.1") ? "accepted" : "refused";
    assert.equal(readForSpec(doc03(), V01.SPEC).kind, expected);
    // A slug the registry does not have declares nothing.
    const x = "https://trusttasks.org/spec/example/thing";
    const read = readForSpec({ ...doc03(), type: `${x}/0.2` }, { ...V01.SPEC, typeUri: `${x}/0.1` });
    assert.equal(read.kind === "refused" && read.reason.code, "unsupportedVersion");
  });
});

describe("readForSpec", () => {
  it("passes the implemented version through untouched", () => {
    const received = { ...doc03(), type: V01.TYPE_URI, payload: { challenge: "c".repeat(32), sessionId: "s-1" } };
    const read = readForSpec<V01.Payload>(received, V01.SPEC, { index: INDEX });
    assert.equal(read.kind, "accepted");
    if (read.kind !== "accepted") return;
    assert.equal(read.extended, false);
    assert.equal(read.doc, received);
    assert.equal(receivedDocumentOf(read.doc), undefined);
  });

  it("re-reads a later version reached along extends, pruning what the target does not know", () => {
    const received = doc03();
    const before = structuredClone(received);
    const read = readForSpec<V01.Payload>(received, V01.SPEC, { index: INDEX });
    assert.equal(read.kind, "accepted");
    if (read.kind !== "accepted") return;
    assert.equal(read.extended, true);
    assert.equal(read.doc.type, V03.TYPE_URI, "the received type is kept, so the response is typed as received");
    assert.deepEqual(read.doc.payload, { challenge: "c".repeat(32), sessionId: "s-1" });
    assert.deepEqual(received, before, "the received document is not mutated");
    assert.equal(read.received, received);
    assert.equal(receivedDocumentOf(read.doc), received);
    assert.equal(receivedDocumentOf({ ...read.doc }), undefined, "a caller's copy is not recognised");
  });

  it("refuses unsupportedVersion when the chain does not reach the implemented version", () => {
    const indexes: Record<string, string>[] = [{}, { [`${BASE}/0.3`]: "0.2" }];
    for (const index of indexes) {
      const read = readForSpec(doc03(), V01.SPEC, { index });
      assert.equal(read.kind, "refused");
      if (read.kind === "refused") assert.equal(read.reason.code, "unsupportedVersion");
    }
    // Backwards is never reached: a 0.3 consumer does not read 0.1 this way.
    const old = readForSpec({ ...doc03(), type: V01.TYPE_URI }, V03.SPEC, { index: INDEX });
    assert.equal(old.kind === "refused" && old.reason.code, "unsupportedVersion");
  });

  it("refuses another specification, another variant, or a non-document", () => {
    const other = readForSpec({ ...doc03(), type: "https://trusttasks.org/spec/acl/grant/0.3" }, V01.SPEC, { index: INDEX });
    assert.equal(other.kind === "refused" && other.reason.code, "unsupportedType");
    const variant = readForSpec({ ...doc03(), type: `${V03.TYPE_URI}#response` }, V01.SPEC, { index: INDEX });
    assert.equal(variant.kind === "refused" && variant.reason.code, "unsupportedType");
    const junk = readForSpec("not a document", V01.SPEC, { index: INDEX });
    assert.equal(junk.kind === "refused" && junk.reason.code, "malformedRequest");
  });

  it("re-reads a response variant against RESPONSE_SPEC", () => {
    const received = {
      id: "resp-9",
      type: `${V03.TYPE_URI}#response`,
      issuer: ME,
      recipient: PEER,
      payload: { session: { extra03: true }, tokens: {}, futureMember: 1 },
    };
    const read = readForSpec(received, V01.RESPONSE_SPEC, { index: INDEX });
    assert.equal(read.kind, "accepted");
    if (read.kind === "accepted") assert.ok(!("futureMember" in (read.doc.payload as object)));
  });
});

describe("consumeInbound on a re-read document", () => {
  it("still handles the implemented version exactly as before", async () => {
    const received = sign({ ...doc03(), type: V01.TYPE_URI, payload: { challenge: "c".repeat(32), sessionId: "s-1" } });
    const read = readForSpec<V01.Payload>(received, V01.SPEC, { index: INDEX });
    assert.equal(read.kind, "accepted");
    if (read.kind !== "accepted") return;
    const { outcome } = await consume01(read.doc);
    assert.equal(outcome.kind, "handled");
    if (outcome.kind === "handled") assert.equal(outcome.response.type, `${V01.TYPE_URI}#response`);
  });

  it("handles a signed 0.3 document as 0.1: proof over the received bytes, response typed 0.3", async () => {
    const received = sign(doc03());
    const read = readForSpec<V01.Payload>(received, V01.SPEC, { index: INDEX });
    assert.equal(read.kind, "accepted");
    if (read.kind !== "accepted") return;

    // The proof does not hold over the pruned copy — which is why the pipeline
    // must not verify that.
    assert.equal(hashVerifier.verify(read.doc), false);
    assert.equal(hashVerifier.verify(received), true);

    const { outcome, seen } = await consume01(read.doc);
    assert.equal(outcome.kind, "handled", JSON.stringify(outcome));
    if (outcome.kind !== "handled") return;
    assert.equal(outcome.response.type, `${V03.TYPE_URI}#response`);
    assert.deepEqual(seen?.payload, { challenge: "c".repeat(32), sessionId: "s-1" }, "the handler sees only 0.1 members");
  });

  it("refuses a tampered extension member, which pruning would otherwise have hidden", async () => {
    const received = sign(doc03());
    (received.payload as { principal: string }).principal = "did:web:someone-else.example";
    const read = readForSpec<V01.Payload>(received, V01.SPEC, { index: INDEX });
    assert.equal(read.kind, "accepted");
    if (read.kind !== "accepted") return;
    const { outcome } = await consume01(read.doc);
    assert.equal(outcome.kind === "rejected" && outcome.error.payload.code, "proofInvalid");
  });

  it("without pruning, the 0.1 schema refuses the 0.3 payload", async () => {
    const { outcome } = await consume01(sign(doc03()) as unknown as TrustTaskDocument<V01.Payload>);
    assert.equal(outcome.kind === "rejected" && outcome.error.payload.code, "malformedRequest");
  });

  it("records the received document for item 11, so an extension-only difference is an idConflict", async () => {
    const guard = new InMemoryReplayGuard();
    const first = readForSpec<V01.Payload>(sign(doc03()), V01.SPEC, { index: INDEX });
    const second = readForSpec<V01.Payload>(
      sign(doc03({ payload: { ...doc03().payload, principal: "did:web:other.example" } })),
      V01.SPEC,
      { index: INDEX },
    );
    assert.ok(first.kind === "accepted" && second.kind === "accepted");
    assert.equal((await consume01(first.doc, guard)).outcome.kind, "handled");
    const again = (await consume01(second.doc, guard)).outcome;
    assert.equal(again.kind === "rejected" && again.error.payload.code, "idConflict");
  });
});

describe("prunePayload", () => {
  const schema = {
    type: "object",
    additionalProperties: false,
    properties: {
      a: { $ref: "#/$defs/Inner" },
      list: { type: "array", items: { $ref: "#/$defs/Inner" } },
      open: { type: "object", properties: { x: { type: "string" } } },
      choice: { $ref: "#/$defs/Choice" },
    },
    $defs: {
      Inner: { type: "object", additionalProperties: false, properties: { k: { type: "string" } } },
      Choice: {
        oneOf: [
          { type: "object", additionalProperties: false, required: ["kind"], properties: { kind: { const: "a" }, p: { type: "string" } } },
          { type: "object", additionalProperties: false, required: ["kind"], properties: { kind: { const: "b" }, q: { type: "string" } } },
        ],
      },
    },
  };

  it("follows $ref, items and closed objects; leaves open objects alone", () => {
    const pruned = prunePayload(
      {
        a: { k: "1", drop: 1 },
        list: [{ k: "2", drop: 2 }, { k: "3" }],
        open: { x: "y", kept: true },
        choice: { kind: "b", q: "z", drop: 3 },
        drop: 4,
      },
      schema,
    );
    assert.deepEqual(pruned, {
      a: { k: "1" },
      list: [{ k: "2" }, { k: "3" }],
      open: { x: "y", kept: true },
      choice: { kind: "b", q: "z" },
    });
  });

  it("leaves a value of a branch the schema does not have untouched", () => {
    const value = { choice: { kind: "c", r: 1 } };
    assert.deepEqual(prunePayload(value, schema), value);
  });

  it("honours unevaluatedProperties across allOf", () => {
    const s = {
      allOf: [{ properties: { a: {} } }, { properties: { b: {} } }],
      unevaluatedProperties: false,
    };
    assert.deepEqual(prunePayload({ a: 1, b: 2, c: 3 }, s), { a: 1, b: 2 });
  });
});
