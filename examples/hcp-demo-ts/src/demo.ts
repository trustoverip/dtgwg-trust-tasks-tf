// The Human Context Protocol (HCP) carried as Trust Tasks, end to end, in one
// process. A DEMONSTRATION, not a reference provider: see README.md and
// SCENARIO.md (the language-neutral script every port of this demo follows).
//
// Three parties, each with its own did:key and TSP keys:
//   - the HCP Provider (a toy, in-memory: a few entries, a grant table);
//   - a Context Client (a booking agent that wants the person's context);
//   - a Management Client (the person's own app, which can revoke grants).
//
// Every document is built with the generated types, signed with
// @openvtc/trust-tasks-proof, sealed with @openvtc/trust-tasks-tsp (a Direct
// TSP message: HPKE-Auth, signed), opened, and run through the SPEC §7.2
// pipeline (proof verified over the document as received, payload validated
// against its schema) before anything acts on it. Nothing touches the network.

import { ed25519, x25519 } from "@noble/curves/ed25519.js";
import {
  rejectWith,
  respondWith,
  type ErrorResponse,
  type SpecPolicy,
  type TrustTaskDocument,
} from "@openvtc/trust-tasks/runtime";
import { DataIntegrityProofVerifier, signerFromPrivateKey, signTrustTask, type Signer } from "@openvtc/trust-tasks-proof";
import { TspConsumer, packTrustTask, unpackTrustTask, advertisedSender } from "@openvtc/trust-tasks-tsp";

import {
  HcpEntriesQuery_v0_1 as query,
  HcpGrantsRequest_v0_1 as grantRequest,
  HcpGrantsRevoke_v0_1 as revoke,
  HcpGrantsRevoked_v0_1 as revoked,
} from "./trust-tasks/index.js";
// Framework specifications ship with the runtime package, not the generator.
import * as discovery from "@openvtc/trust-tasks/trust-task-discovery/0.4/payload";
import * as nextStep from "@openvtc/trust-tasks/trust-task-next-step/0.1/payload";
import { schemaValidator, validatePayload } from "./validator.js";

// ── Deterministic inputs ───────────────────────────────────────────────────
// A fixed clock the demo controls: it starts at START and moves one second per
// document issued. Instants are whole seconds (`…Z`), never toISOString().
export const START = "2026-11-25T10:00:00Z";

export class Clock {
  private ms = Date.parse(START);
  now = (): number => this.ms;
  iso = (): string => new Date(this.ms).toISOString().replace(".000Z", "Z");
  tick = (): string => {
    this.ms += 1000;
    return this.iso();
  };
}

/** Sequential urn:uuid identifiers: …-000000000001, …-000000000002, … */
export class Ids {
  private n = 0;
  next = (): string => `urn:uuid:00000000-0000-4000-8000-${(++this.n).toString(16).padStart(12, "0")}`;
}

// ── Parties ────────────────────────────────────────────────────────────────
// DEMO ONLY keys: each is 32 bytes of one repeated value. Never use such keys
// for anything real. The VID is the did:key of the Ed25519 signing key, and
// the same VID names the party in TSP.
export interface Party {
  name: string;
  signer: Signer;
  vid: string;
  sigPriv: Uint8Array;
  sigPub: Uint8Array;
  encPriv: Uint8Array;
  encPub: Uint8Array;
}

function party(name: string, signingSeed: number, encryptionSeed: number): Party {
  const sigPriv = new Uint8Array(32).fill(signingSeed); // DEMO ONLY
  const encPriv = new Uint8Array(32).fill(encryptionSeed); // DEMO ONLY
  const signer = signerFromPrivateKey("ed25519", sigPriv);
  return {
    name,
    signer,
    vid: signer.did,
    sigPriv,
    sigPub: ed25519.getPublicKey(sigPriv),
    encPriv,
    encPub: x25519.getPublicKey(encPriv),
  };
}

// ── The toy provider's data ────────────────────────────────────────────────
export const PURPOSE = "https://purposes.example/travel-booking";
export const MANAGEMENT_TOKEN = "hcp_mt_demo_management"; // DEMO ONLY: issued to the person's app at setup
const NEUTRAL_DENIAL = "The request is not authorized."; // one fixed text for every refusal (HCP 5.5)

/** The person's context, as the toy provider stores it (HCP 4.2, much simplified). */
const ENTRIES = [
  { ref: "hcp-ref-0001", category: "travel_preferences", value: "Prefers aisle seats and early-morning flights." },
  { ref: "hcp-ref-0002", category: "travel_preferences", value: "Avoids overnight layovers." },
  { ref: "hcp-ref-0003", category: "health_context", value: "Severe peanut allergy." },
];

interface Grant {
  grantId: string;
  token: string;
  client: string;
  categories: string[];
  purposes: string[];
  status: "active" | "revoked";
}

// ── The world: three parties, one transport, one clock ─────────────────────
export class World {
  readonly clock = new Clock();
  readonly ids = new Ids();
  readonly provider = party("Provider", 0x11, 0x12);
  readonly contextClient = party("Context Client", 0x21, 0x22);
  readonly management = party("Management Client", 0x31, 0x32);
  readonly transcript: string[] = [];
  readonly grants: Grant[] = [];
  /** Grant requests the person has not decided yet, by threadId (SPEC §8.6 hold). */
  readonly held = new Map<string, { payload: grantRequest.Payload; decided: boolean }>();
  private tokens = 0;

  private parties = [this.provider, this.contextClient, this.management];
  private verifier = new DataIntegrityProofVerifier({ now: this.clock.now });

  /** One TSP consumer per receiving party: its replay guard is that party's duplicate-execution record. */
  private consumers = new Map(
    [this.provider, this.contextClient].map((me) => [
      me.vid,
      new TspConsumer({
        recipientVid: me.vid,
        decryptionKey: me.encPriv,
        signingKey: me.sigPriv,
        resolveSender: (vid) => {
          const p = this.byVid(vid);
          return { encryptionKey: p.encPub, signingKey: p.sigPub };
        },
        proofVerifier: this.verifier,
        payloadValidator: schemaValidator,
        now: this.clock.now,
        newErrorId: this.ids.next,
      }),
    ]),
  );

  byVid(vid: string): Party {
    const p = this.parties.find((x) => x.vid === vid);
    if (p === undefined) throw new Error(`unknown VID ${vid}`);
    return p;
  }

  /** Build and sign a request; party members and issuedAt are set BEFORE signing. */
  request<P>(from: Party, to: Party, type: string, payload: P, threadId?: string): TrustTaskDocument<P> {
    const id = this.ids.next();
    const issuedAt = this.clock.tick();
    const doc: TrustTaskDocument<P> = { id, type, issuer: from.vid, recipient: to.vid, issuedAt, threadId: threadId ?? id, payload };
    return signTrustTask(doc, from.signer, { created: new Date(this.clock.now()) });
  }

  /** Sign a reply a handler built (responses, next steps, errors). */
  sign<R>(by: Party, doc: TrustTaskDocument<R>): TrustTaskDocument<R> {
    return signTrustTask(doc, by.signer, { created: new Date(this.clock.now()) });
  }

  /** Seal `doc` over TSP from → to, let `to` consume it, open the sealed reply. */
  async exchange(from: Party, to: Party, doc: TrustTaskDocument<unknown> | Record<string, unknown>): Promise<TrustTaskDocument<any>> {
    this.print(from, to, doc as TrustTaskDocument<unknown>);
    const wire = await packTrustTask(doc as Record<string, unknown>, {
      senderVid: from.vid,
      receiverVid: to.vid,
      keys: { senderSigningKey: from.sigPriv, senderEncryptionKey: from.encPriv, receiverEncryptionKey: to.encPub },
    });

    // ── receiving side ──
    const consumer = this.consumers.get(to.vid)!;
    const sender = this.byVid(advertisedSender(wire)!);
    const unpacked = await unpackTrustTask(wire, {
      recipientVid: to.vid,
      keys: { receiverDecryptionKey: to.encPriv, senderEncryptionKey: sender.encPub, senderSigningKey: sender.sigPub },
    });
    const spec = SPECS[unpacked.document.type.split("#")[0]!];
    if (spec === undefined) throw new Error(`no handler for ${unpacked.document.type}`);
    const received = await consumer.consume(unpacked, spec, (accepted) => this.handle(to, accepted));
    if (received.reply === undefined) throw new Error(`no reply: ${received.outcome.kind}`);

    // ── back at the sender: open the sealed reply and check it before use ──
    const replyWire = await consumer.packReply(received);
    const opened = await unpackTrustTask(replyWire, {
      recipientVid: from.vid,
      keys: { receiverDecryptionKey: from.encPriv, senderEncryptionKey: to.encPub, senderSigningKey: to.sigPub },
    });
    const reply = opened.document as unknown as TrustTaskDocument<any>;
    if (reply.proof !== undefined && !(await this.verifier.verify(reply))) throw new Error("reply proof does not verify");
    validatePayload(reply);
    this.print(to, from, reply);
    return reply;
  }

  // ── Handlers: what each receiving party does with an accepted document ──
  private handle(me: Party, doc: TrustTaskDocument<any>): TrustTaskDocument<unknown> | ErrorResponse {
    const reply = (payload: unknown) => this.sign(me, respondWith(doc, this.ids.next(), payload, this.clock.tick));
    const refuse = (code: string, message: string) =>
      this.sign(me, rejectWith(doc, this.ids.next(), { code, message, retryable: false }, this.clock.tick) as TrustTaskDocument<unknown>);

    switch (doc.type.split("#")[0]) {
      // Step 1 — trust-task-discovery: what the provider serves (SPEC §10).
      case discovery.TYPE_URI:
        return reply({ supportedTypes: [grantRequest.TYPE_URI, query.TYPE_URI, revoke.TYPE_URI, revoked.TYPE_URI] });

      // Step 2 — hcp/grants/request: hold with a next step until the person decides (HCP 6.1.6, SPEC §8.6).
      case grantRequest.TYPE_URI: {
        const threadId = doc.threadId!;
        const hold = this.held.get(threadId);
        if (hold === undefined) this.held.set(threadId, { payload: doc.payload, decided: false });
        if (hold === undefined || !hold.decided) {
          const step: TrustTaskDocument<nextStep.Payload> = {
            id: this.ids.next(),
            type: nextStep.TYPE_URI,
            issuer: me.vid,
            recipient: doc.issuer!,
            issuedAt: this.clock.tick(),
            threadId,
            payload: {
              expects: [{ typeUri: `${grantRequest.TYPE_URI}#request`, reason: "The request is with the person, who has not decided yet.", hint: { retryAfter: 5 } }],
              continuation: "resubmit",
              inResponseTo: { typeUri: doc.type, id: doc.id },
            },
          };
          return this.sign(me, step);
        }
        // The person approved (the toy approves everything asked, read only).
        const grant: Grant = {
          grantId: this.ids.next(),
          token: `hcp_at_demo_${String(++this.tokens).padStart(4, "0")}`, // DEMO ONLY
          client: doc.issuer!,
          categories: hold.payload.access.flatMap((a) => a.categories),
          purposes: hold.payload.access.flatMap((a) => (a.purposes ?? []).map((p) => p.code)),
          status: "active",
        };
        this.grants.push(grant);
        const grantedAt = this.clock.iso();
        const plus = (s: number) => new Date(this.clock.now() + s * 1000).toISOString().replace(".000Z", "Z");
        const response: grantRequest.Response = {
          accessToken: grant.token,
          tokenExpiresAt: plus(3600), // HCP 13.4 default
          grantId: grant.grantId,
          access: hold.payload.access.map((a) => ({ type: a.type, actions: a.actions, categories: a.categories, purposes: a.purposes! })) as grantRequest.Response["access"], // approved as asked
          grantType: "temporary",
          grantedAt,
          grantExpiresAt: plus(hold.payload.expiresIn ?? 4 * 3600), // HCP 6.1.3 default
        };
        return reply(response);
      }

      // Step 3 — hcp/entries/query: the grant is checked on every request (HCP 5.2.1, 6.2.2).
      case query.TYPE_URI: {
        const p = doc.payload as query.Payload;
        const grant = this.grants.find((g) => g.token === p.accessToken && g.client === doc.issuer);
        if (grant === undefined || grant.status !== "active") return refuse("hcp:invalidToken", "The access token is not valid.");
        const asked = p.categories ?? grant.categories;
        if (!grant.purposes.includes(p.purpose) || asked.some((c) => !grant.categories.includes(c))) {
          return refuse("permissionDenied", NEUTRAL_DENIAL);
        }
        const useUntil = new Date(this.clock.now() + 3600 * 1000).toISOString().replace(".000Z", "Z");
        const entries = ENTRIES.filter((e) => asked.includes(e.category))
          .slice(0, Math.min(p.maxEntries ?? 5, 20))
          .map((e) => ({
            ref: e.ref,
            version: 1,
            content: { profile: "https://hcp.example/spec/profiles/general-context/2.0", value: e.value, language: "en" },
            assertion: { declarationType: "person-declared" },
            classification: { categories: [e.category] },
            standing: { assurance: "authenticated", confirmed: true },
            useUntil,
          }));
        return reply({
          entries,
          retrievalId: this.ids.next(),
          minimization: { method: "lexical" },
          categoriesSearched: asked,
          respondedAt: this.clock.iso(),
        } as query.Response);
      }

      // Step 4 — hcp/grants/revoke: only the person's Management Client may (HCP 6.1.9, 6.2.1).
      case revoke.TYPE_URI: {
        const p = doc.payload as revoke.Payload;
        if (p.accessToken !== MANAGEMENT_TOKEN) return refuse("permissionDenied", NEUTRAL_DENIAL);
        const grant = this.grants.find((g) => g.grantId === p.grantId);
        if (grant === undefined) return refuse("permissionDenied", NEUTRAL_DENIAL);
        grant.status = "revoked"; // immediate (HCP 6.2.2); a second revoke is idempotent
        return reply({});
      }

      // Step 5 — hcp/grants/revoked, at the Context Client: stop using what it holds (HCP 6.2.2, 9.9).
      case revoked.TYPE_URI:
        return reply({});
    }
    throw new Error(`unhandled ${doc.type}`);
  }

  /** The person decides, in the provider's own modality (here: approves at once). */
  personApproves(threadId: string): void {
    this.held.get(threadId)!.decided = true;
    this.transcript.push(`[person] approves the grant request on thread ${threadId} in the provider's app\n`);
  }

  // ── Transcript ───────────────────────────────────────────────────────────
  step(title: string): void {
    this.transcript.push(`\n=== ${title} ===\n`);
  }

  private print(from: Party, to: Party, doc: TrustTaskDocument<unknown>): void {
    this.transcript.push(`--> ${from.name} -> ${to.name} (TSP) ${doc.type}\n${pretty(redact(doc))}\n`);
  }
}

/** The SpecPolicy each receiving party applies, by bare Type URI. */
const SPECS: Record<string, SpecPolicy> = {
  [discovery.TYPE_URI]: discovery.SPEC,
  [grantRequest.TYPE_URI]: grantRequest.SPEC,
  [query.TYPE_URI]: query.SPEC,
  [revoke.TYPE_URI]: revoke.SPEC,
  [revoked.TYPE_URI]: revoked.SPEC,
};

/** Secrets are shown as present, never in full: the first 6 characters and an ellipsis. */
export function redact(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(redact);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, k === "accessToken" && typeof v === "string" ? `${v.slice(0, 6)}…` : redact(v)]),
    );
  }
  return value;
}

/** JSON with object keys sorted (as JCS orders them) and two-space indents: the same in every language. */
export function pretty(value: unknown, indent = ""): string {
  const next = indent + "  ";
  if (Array.isArray(value)) {
    if (value.length === 0) return "[]";
    return `[\n${value.map((v) => next + pretty(v, next)).join(",\n")}\n${indent}]`;
  }
  if (value !== null && typeof value === "object") {
    const keys = Object.keys(value).sort();
    if (keys.length === 0) return "{}";
    return `{\n${keys.map((k) => `${next}${JSON.stringify(k)}: ${pretty((value as any)[k], next)}`).join(",\n")}\n${indent}}`;
  }
  return JSON.stringify(value);
}

// ── The five steps ─────────────────────────────────────────────────────────
export interface DemoResult {
  transcript: string;
  postRevokeError: TrustTaskDocument<{ code: string }>;
}

export async function runDemo(world = new World()): Promise<DemoResult> {
  const { provider: P, contextClient: C, management: M } = world;

  // 1. Discover: does the provider speak hcp/*? (trust-task-discovery 0.4, SPEC §10)
  world.step("1. Discover (trust-task-discovery 0.4)");
  const found = await world.exchange(C, P, world.request(C, P, discovery.TYPE_URI, { patterns: ["hcp/*"] }));
  if (!found.payload.supportedTypes.includes(grantRequest.TYPE_URI)) throw new Error("provider does not serve grant requests");

  // 2. Grant request: held with a next step while the person decides, then resubmitted (HCP 6.1.6, 6.1.7; SPEC §8.6)
  world.step("2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step");
  const ask: grantRequest.Payload = {
    access: [
      {
        type: "https://hcp.example/spec/authorization/context-access",
        actions: ["read"],
        categories: ["travel_preferences"],
        purposes: [{ code: PURPOSE, description: "Book trips that match how you like to travel" }],
      },
    ],
    expiresIn: 14400,
    subjectHint: "acct:alex@provider.example",
  };
  const first = world.request(C, P, `${grantRequest.TYPE_URI}#request`, ask);
  const held = await world.exchange(C, P, first);
  if (held.type !== nextStep.TYPE_URI) throw new Error("expected a next step");
  world.personApproves(first.threadId!);
  // Resubmission: a new document, fresh id, same threadId and payload.
  const granted = await world.exchange(C, P, world.request(C, P, `${grantRequest.TYPE_URI}#request`, ask, first.threadId));
  if (granted.type !== grantRequest.RESPONSE_TYPE_URI) throw new Error("expected the grant");
  const { accessToken, grantId } = granted.payload as grantRequest.Response;

  // 3. Query under the grant (hcp/entries/query 0.1, HCP 5.2.1)
  world.step("3. Query (hcp/entries/query 0.1)");
  const queryPayload: query.Payload = { query: "How does Alex like to fly?", categories: ["travel_preferences"], purpose: PURPOSE, maxEntries: 5, accessToken };
  const answer = await world.exchange(C, P, world.request(C, P, `${query.TYPE_URI}#request`, queryPayload));
  if (answer.type !== query.RESPONSE_TYPE_URI) throw new Error("expected entries");

  // 4. Revoke from the person's Management Client (hcp/grants/revoke 0.1, HCP 6.2.1)
  world.step("4. Revoke (hcp/grants/revoke 0.1) from the Management Client");
  const done = await world.exchange(M, P, world.request(M, P, `${revoke.TYPE_URI}#request`, { grantId, accessToken: MANAGEMENT_TOKEN }));
  if (done.type !== revoke.RESPONSE_TYPE_URI) throw new Error("expected the revocation");

  // 5. Revocation notice to the Context Client (hcp/grants/revoked 0.1, HCP 6.2.3, 9.9)
  world.step("5. Revocation notice (hcp/grants/revoked 0.1)");
  const ack = await world.exchange(P, C, world.request(P, C, `${revoked.TYPE_URI}#request`, { grantId, revokedAt: world.clock.iso() }));
  if (ack.type !== revoked.RESPONSE_TYPE_URI) throw new Error("expected the acknowledgement");

  // After: the same query again fails — the grant is checked on every request (HCP 6.2.2)
  world.step("After revoke: the same query fails with hcp:invalidToken");
  const refused = await world.exchange(C, P, world.request(C, P, `${query.TYPE_URI}#request`, queryPayload));

  return { transcript: world.transcript.join(""), postRevokeError: refused };
}
