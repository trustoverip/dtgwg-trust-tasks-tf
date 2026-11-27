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
// TSP 1.0 message: HPKE-Base sealed and signed, the sender authenticated by the
// signature and the ESSR sender field), opened, and run through the SPEC §7.2
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
import { canonicalize, DataIntegrityProofVerifier, signerFromPrivateKey, signTrustTask, type Signer } from "@openvtc/trust-tasks-proof";
import { TspConsumer, packTrustTask, unpackTrustTask, advertisedSender } from "@openvtc/trust-tasks-tsp";

import {
  HcpEntriesQuery_v0_1 as query,
  HcpGrantsRequest_v0_1 as grantRequest,
  HcpGrantsRevoke_v0_1 as revoke,
  HcpGrantsRevoked_v0_1 as revoked,
} from "./trust-tasks/index.js";
// Framework specifications ship with the runtime package, not the generator.
import * as discovery from "@openvtc/trust-tasks/trust-task-discovery/0.4/payload";
import * as error from "@openvtc/trust-tasks/trust-task-error/0.6/payload";
import * as nextStep from "@openvtc/trust-tasks/trust-task-next-step/0.1/payload";
import { schemaValidator, validNextStepHint, validatePayload } from "./validator.js";

// ── Deterministic inputs ───────────────────────────────────────────────────
// A fixed clock the demo controls: it starts at START and moves one second per
// document issued. It never reads the system time.
export const START = "2026-11-25T10:00:00Z";

/**
 * An instant as the documents write it: whole seconds, `YYYY-MM-DDTHH:MM:SSZ`.
 * The repository's rule is "never write toISOString() into a document": it
 * carries milliseconds, and it invites stamping the system time. Every instant
 * the demo writes comes from its injected Clock, which holds whole seconds
 * only, and goes through this one function.
 */
export function iso(ms: number): string {
  if (ms % 1000 !== 0) throw new Error("the demo's instants are whole seconds");
  const d = new Date(ms);
  const two = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${two(d.getUTCMonth() + 1)}-${two(d.getUTCDate())}T${two(d.getUTCHours())}:${two(d.getUTCMinutes())}:${two(d.getUTCSeconds())}Z`;
}

export class Clock {
  private ms = Date.parse(START);
  now = (): number => this.ms;
  iso = (): string => iso(this.ms);
  /** `seconds` after now, as an instant. */
  plus = (seconds: number): string => iso(this.ms + seconds * 1000);
  tick = (): string => {
    this.ms += 1000;
    return this.iso();
  };
  /** Let time pass without issuing a document (a client waiting out `retryAfter`, or a test). */
  advance = (seconds: number): void => {
    this.ms += seconds * 1000;
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
/** The person, by the pseudonym the provider uses with this client (HCP 3.4). */
const SUBJECT = "hcp-sub-demo-alex";
/** How long after the response a resubmission still gets the same grant (hcp/grants/request "After approval"). */
export const AFTER_APPROVAL_WINDOW = 3600;
/**
 * The hold's lifetime, provider policy: how long a request the person has not
 * decided stays held, counted from the held request's `issuedAt`. When it
 * passes, the policy denies (hcp/grants/request "Holding" rule 6).
 */
export const HOLD_LIFETIME = 300;

/** The person's context, as the toy provider stores it (HCP 4.2, much simplified). */
const ENTRIES = [
  { ref: "hcp-ref-0001", category: "travel_preferences", value: "Prefers aisle seats and early-morning flights." },
  { ref: "hcp-ref-0002", category: "travel_preferences", value: "Avoids overnight layovers." },
  { ref: "hcp-ref-0003", category: "health_context", value: "Severe peanut allergy." },
];

/** A grant in the provider's table. */
export interface Grant {
  grantId: string;
  token: string;
  tokenExpiresAt: string;
  client: string; // the client's VID
  access: grantRequest.Response["grant"]["access"];
  categories: string[];
  purposes: string[];
  issuedAt: string;
  expiresAt: string;
  status: "active" | "revoked";
  revokedAt?: string;
}

/**
 * A grant request on one thread (hcp/grants/request "Holding"): held, then
 * approved by the person and granted, or ended: declined by the person, or
 * expired when its lifetime passed undecided. An ended thread is never reopened.
 */
interface Hold {
  issuer: string;
  payload: grantRequest.Payload;
  canonical: string; // JCS of the payload, to compare resubmissions
  heldAt: number; // the held request's issuedAt, in ms: the lifetime counts from it
  state: "pending" | "approved" | "granted" | "declined" | "expired";
  grantId?: string;
  respondedAt?: number; // when the response was first sent, in ms
}

/** What the Context Client keeps for a grant it holds: the token, and the refs of the copies disclosed under it. */
export interface HeldGrant {
  token: string;
  copies: string[];
}

// ── The world: three parties, one transport, one clock ─────────────────────
export class World {
  readonly clock = new Clock();
  readonly ids = new Ids();
  readonly provider = party("Provider", 0x11, 0x12);
  readonly contextClient = party("Context Client", 0x21, 0x22);
  readonly management = party("Management Client", 0x31, 0x32);
  readonly transcript: string[] = [];

  // ── The provider's state ──
  /** Its one registered client, by VID, with the registration the person is shown (HCP 6.1.5, 6.1.6). */
  readonly registeredClients = new Map([
    [this.contextClient.vid, { name: "Wayfare booking agent", operator: { id: "booking.example", name: "Wayfare Travel Ltd" } }],
  ]);
  readonly grants: Grant[] = [];
  /** Grant requests by threadId (the SPEC §8.6 hold). */
  readonly held = new Map<string, Hold>();
  private tokens = 0;

  // ── The Context Client's state ──
  /** The grants it holds, by grantId. */
  readonly wallet = new Map<string, HeldGrant>();
  /** Revocation notices it has processed: noticeId → grantId (hcp/grants/revoked "Deduplication"). */
  readonly processedNotices = new Map<string, string>();

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
      keys: { senderSigningKey: from.sigPriv, receiverEncryptionKey: to.encPub },
    });

    // ── receiving side ──
    const consumer = this.consumers.get(to.vid)!;
    const sender = this.byVid(advertisedSender(wire)!);
    const unpacked = await unpackTrustTask(wire, {
      recipientVid: to.vid,
      keys: { receiverDecryptionKey: to.encPriv, senderSigningKey: sender.sigPub },
    });
    const spec = SPECS[unpacked.document.type.split("#")[0]!];
    if (spec === undefined) throw new Error(`no handler for ${unpacked.document.type}`);
    const received = await consumer.consume(unpacked, spec, (accepted) => this.handle(to, accepted));
    if (received.reply === undefined) throw new Error(`no reply: ${received.outcome.kind}`);

    // ── back at the sender: open the sealed reply and check it before use ──
    const replyWire = await consumer.packReply(received);
    const opened = await unpackTrustTask(replyWire, {
      recipientVid: from.vid,
      keys: { receiverDecryptionKey: from.encPriv, senderSigningKey: to.sigPub },
    });
    const reply = opened.document as unknown as TrustTaskDocument<any>;
    const sent = doc as TrustTaskDocument<unknown>;
    checkReply(reply, to, sent.threadId ?? sent.id);
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
      case grantRequest.TYPE_URI:
        return this.grantRequest(me, doc as TrustTaskDocument<grantRequest.Payload>, reply, refuse);

      // Steps 3 and 5 — hcp/entries/query: the grant is checked on every request (HCP 5.2.1, 6.2.2).
      case query.TYPE_URI: {
        const p = doc.payload as query.Payload;
        const grant = this.grants.find((g) => g.token === p.accessToken);
        if (grant === undefined) return refuse("hcp:invalidToken", "The access token is not valid.");
        // A token issued to another client is no use from this VID (query "The access token").
        if (grant.client !== doc.issuer) return refuse("permissionDenied", NEUTRAL_DENIAL);
        if (grant.status !== "active") return refuse("hcp:invalidToken", "The access token is not valid.");
        const asked = p.categories ?? grant.categories;
        if (!grant.purposes.includes(p.purpose) || asked.some((c) => !grant.categories.includes(c))) {
          return refuse("permissionDenied", NEUTRAL_DENIAL);
        }
        const useUntil = this.clock.plus(3600);
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
          directives: [], // a Context Client always gets the list; the person set none (HCP 6.4)
          respondedAt: this.clock.iso(),
        } as unknown as query.Response);
      }

      // Step 4 — hcp/grants/revoke: only the person's Management Client may (HCP 6.1.9, 6.2.1).
      case revoke.TYPE_URI: {
        const p = doc.payload as revoke.Payload;
        if (p.accessToken !== MANAGEMENT_TOKEN) return refuse("permissionDenied", NEUTRAL_DENIAL);
        const grant = this.grants.find((g) => g.grantId === p.grantId);
        if (grant === undefined) return refuse("permissionDenied", NEUTRAL_DENIAL);
        if (grant.status === "active") {
          grant.status = "revoked"; // immediate (HCP 6.2.2); a second revoke is idempotent
          grant.revokedAt = this.clock.iso();
        }
        return reply({});
      }

      // Step 6 — hcp/grants/revoked, at the Context Client (HCP 6.2.2, 9.9; hcp/grants/revoked "Delivery rules").
      case revoked.TYPE_URI: {
        const p = doc.payload as revoked.Payload;
        // Deduplicate on noticeId: a later attempt of a processed notice is acknowledged, not acted on again.
        // (A real client keeps the record at least 25 hours; the demo keeps it for the run.)
        if (!this.processedNotices.has(p.noticeId)) {
          this.processedNotices.set(p.noticeId, p.grantId);
          this.forgetGrant(p.grantId);
        }
        return reply({});
      }
    }
    throw new Error(`unhandled ${doc.type}`);
  }

  /** The provider's side of hcp/grants/request: the hold, the grant, and "After approval". */
  private grantRequest(
    me: Party,
    doc: TrustTaskDocument<grantRequest.Payload>,
    reply: (payload: unknown) => TrustTaskDocument<unknown>,
    refuse: (code: string, message: string) => TrustTaskDocument<unknown>,
  ): TrustTaskDocument<unknown> {
    const issuer = doc.issuer!;
    const threadId = doc.threadId!;
    // Only a registered client may ask (hcp/grants/request "Authorization").
    if (!this.registeredClients.has(issuer)) return refuse("permissionDenied", NEUTRAL_DENIAL);

    const canonical = canonicalize(doc.payload);
    const hold = this.held.get(threadId);
    if (hold === undefined) {
      this.held.set(threadId, { issuer, payload: doc.payload, canonical, heldAt: this.clock.now(), state: "pending" });
      return this.nextStep(me, doc);
    }
    // A resubmission is matched by (threadId, issuer); a refusal leaves the held request as it is.
    if (hold.issuer !== issuer) return refuse("permissionDenied", NEUTRAL_DENIAL);
    if (hold.canonical !== canonical) return refuse("malformedRequest", "The resubmission differs from the held request.");

    this.expireIfDue(hold);
    switch (hold.state) {
      case "pending":
        return this.nextStep(me, doc);
      case "declined":
      case "expired":
        // The thread has ended and is never reopened: the same refusal a decline and an unknown person get.
        return refuse("permissionDenied", NEUTRAL_DENIAL);
      case "approved": {
        // The person approved (the toy approves everything asked, read only): issue the one grant.
        const grantedAt = this.clock.iso();
        const expiresAt = this.clock.plus(hold.payload.expiresIn ?? 4 * 3600); // HCP 6.1.3 default
        const grant: Grant = {
          grantId: this.ids.next(),
          token: `hcp_at_demo_${String(++this.tokens).padStart(4, "0")}`, // DEMO ONLY
          // No refresh in 0.1, so the token lasts as long as the grant (hcp/grants/request "Response").
          tokenExpiresAt: expiresAt,
          client: issuer,
          access: hold.payload.access.map((a) => ({
            type: a.type,
            actions: a.actions,
            categories: a.categories,
            purposes: (a.purposes ?? []).map((p) => ({ code: p.code, description: p.description })),
          })) as unknown as Grant["access"], // approved as asked
          categories: hold.payload.access.flatMap((a) => a.categories),
          purposes: hold.payload.access.flatMap((a) => (a.purposes ?? []).map((p) => p.code)),
          issuedAt: grantedAt,
          expiresAt,
          status: "active",
        };
        this.grants.push(grant);
        hold.state = "granted";
        hold.grantId = grant.grantId;
        hold.respondedAt = this.clock.now();
        return reply(this.grantResponse(grant));
      }
      case "granted": {
        // After approval: the same grant within the hour, never a second one; after it, permissionDenied.
        const grant = this.grants.find((g) => g.grantId === hold.grantId)!;
        const late = this.clock.now() - hold.respondedAt! > AFTER_APPROVAL_WINDOW * 1000;
        if (late || grant.status !== "active") return refuse("permissionDenied", NEUTRAL_DENIAL);
        return reply(this.grantResponse(grant));
      }
    }
  }

  /** The hold: a next step naming the request Type URI, continuation resubmit (SPEC §8.6). */
  private nextStep(me: Party, doc: TrustTaskDocument<grantRequest.Payload>): TrustTaskDocument<unknown> {
    const step: TrustTaskDocument<nextStep.Payload> = {
      id: this.ids.next(),
      type: nextStep.TYPE_URI,
      issuer: me.vid,
      recipient: doc.issuer!,
      issuedAt: this.clock.tick(),
      threadId: doc.threadId!,
      payload: {
        expects: [{ typeUri: `${grantRequest.TYPE_URI}#request`, reason: "The request is with the person, who has not decided yet.", hint: { retryAfter: 1 } }],
        continuation: "resubmit",
        inResponseTo: { typeUri: doc.type, id: doc.id },
      },
    };
    return this.sign(me, step);
  }

  /** The grant response: the token and HCP's grant object, as hcp/grants/read-own would return it. */
  private grantResponse(grant: Grant): grantRequest.Response {
    const registration = this.registeredClients.get(grant.client)!;
    return {
      accessToken: grant.token,
      tokenExpiresAt: grant.tokenExpiresAt,
      grant: {
        grantId: grant.grantId,
        subject: { id: SUBJECT },
        authorizingPrincipal: { id: SUBJECT },
        client: { id: grant.client, ...registration },
        access: grant.access,
        grantType: "temporary",
        issuedAt: grant.issuedAt,
        expiresAt: grant.expiresAt,
        status: grant.status,
      },
    };
  }

  /** A pending hold whose lifetime has passed ends: the policy denies (hcp/grants/request "Holding" rule 6). */
  private expireIfDue(hold: Hold): void {
    if (hold.state === "pending" && this.clock.now() - hold.heldAt > HOLD_LIFETIME * 1000) hold.state = "expired";
  }

  /** The person decides, in the provider's own modality. Only a pending hold changes; an ended thread stays ended. */
  private personDecides(threadId: string, verb: "approves" | "declines", state: "approved" | "declined"): void {
    const hold = this.held.get(threadId)!;
    this.expireIfDue(hold);
    const line = `[person] ${verb} the grant request on thread ${threadId} in the provider's app`;
    if (hold.state === "pending") {
      hold.state = state;
      this.transcript.push(`${line}\n`);
    } else {
      this.transcript.push(`${line}, but the hold has ended: nothing changes\n`);
    }
  }

  personApproves(threadId: string): void {
    this.personDecides(threadId, "approves", "approved");
  }

  personDeclines(threadId: string): void {
    this.personDecides(threadId, "declines", "declined");
  }

  /** The person never decides: the hold's lifetime passes with no document sent. */
  personDoesNotDecide(threadId: string): void {
    this.transcript.push(`[person] does not decide on thread ${threadId}; the hold's lifetime (${HOLD_LIFETIME} s) passes\n`);
    this.clock.advance(HOLD_LIFETIME);
  }

  /** The Context Client stops using a grant: drops its token and every copy disclosed under it (HCP 6.2.2, 7.6). */
  forgetGrant(grantId: string): void {
    this.wallet.delete(grantId);
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

/**
 * The codes the receiving end's SPEC §7.2 pipeline raises before any handler
 * runs. The library builds those error responses itself, unsigned; every other
 * reply (each response, next step and handler refusal) is signed.
 */
export const PIPELINE_CODES = new Set([
  "malformedRequest",
  "unsupportedType",
  "unsupportedVersion",
  "unsupportedExtension",
  "expired",
  "proofRequired",
  "proofInvalid",
  "wrongRecipient",
  "identityMismatch",
  "idConflict",
]);

/** The sender's checks on a reply before it trusts it: who it is from, which thread, and that it is signed. */
export function checkReply(reply: TrustTaskDocument<any>, from: Party, threadId: string): void {
  if (reply.issuer !== from.vid) throw new Error(`reply issuer ${reply.issuer} is not ${from.vid}`);
  if (reply.threadId !== threadId) throw new Error(`reply threadId ${reply.threadId} is not ${threadId}`);
  const libraryError = reply.type === error.TYPE_URI && PIPELINE_CODES.has(reply.payload?.code);
  if (reply.proof === undefined && !libraryError) throw new Error(`reply ${reply.type} carries no proof`);
}

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

/** How long the client waits before it resubmits when the next step's hint is absent or fails NextStepHint. */
export const DEFAULT_RETRY_AFTER = 5;

/**
 * The seconds to wait before resubmitting, read from a next step's `hint`.
 * The client validates the hint against `$defs.NextStepHint` in
 * hcp/grants/request's schema (an object whose only member, `retryAfter`, is
 * an integer from 1 to 3600) and ignores a hint that fails, as if it were
 * absent (hcp/grants/request "Holding" rule 5, trust-task-next-step).
 */
export function retryAfterOf(hint: unknown): number {
  if (hint === undefined || !validNextStepHint(hint)) return DEFAULT_RETRY_AFTER;
  return (hint as { retryAfter: number }).retryAfter;
}

// ── The eight steps, one function each ───────────────────────────────────────

/** What the Context Client asks for in step 2. */
export const ASK: grantRequest.Payload = {
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

/** The query the Context Client sends in steps 3 and 5, under the grant's token. */
export function queryPayload(accessToken: string): query.Payload {
  return { query: "How does Alex like to fly?", categories: ["travel_preferences"], purpose: PURPOSE, maxEntries: 5, accessToken };
}

/** 1. Discover: does the provider speak hcp/*? (trust-task-discovery 0.4, SPEC §10) */
export async function stepDiscover(world: World): Promise<void> {
  const { provider: P, contextClient: C } = world;
  world.step("1. Discover (trust-task-discovery 0.4)");
  const found = await world.exchange(C, P, world.request(C, P, discovery.TYPE_URI, { patterns: ["hcp/*"] }));
  if (!found.payload.supportedTypes.includes(grantRequest.TYPE_URI)) throw new Error("provider does not serve grant requests");
}

/**
 * 2. Grant request: held with a next step while the person decides, then
 * resubmitted on the same thread (HCP 6.1.6, 6.1.7; SPEC §8.6). The client
 * keeps the token in its wallet. Returns the thread and the grant.
 */
export async function stepGrantRequest(world: World): Promise<{ threadId: string; grantId: string }> {
  const { provider: P, contextClient: C } = world;
  world.step("2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step");
  const first = world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK);
  const held = await world.exchange(C, P, first);
  if (held.type !== nextStep.TYPE_URI) throw new Error("expected a next step");
  world.personApproves(first.threadId!);
  // The client SHOULD NOT resubmit sooner than the next step's retryAfter (hcp/grants/request "Holding").
  world.clock.advance(retryAfterOf((held.payload as nextStep.Payload).expects[0]?.hint));
  // Resubmission: a new document, fresh id, same threadId and payload.
  const granted = await world.exchange(C, P, world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK, first.threadId));
  if (granted.type !== grantRequest.RESPONSE_TYPE_URI) throw new Error("expected the grant");
  const { accessToken, grant } = granted.payload as grantRequest.Response;
  world.wallet.set(grant.grantId, { token: accessToken, copies: [] });
  return { threadId: first.threadId!, grantId: grant.grantId };
}

/** 3. Query under the grant (hcp/entries/query 0.1, HCP 5.2.1). The client keeps the refs of the copies. */
export async function stepQuery(world: World, grantId: string): Promise<void> {
  const { provider: P, contextClient: C } = world;
  world.step("3. Query (hcp/entries/query 0.1)");
  const held = world.wallet.get(grantId)!;
  const answer = await world.exchange(C, P, world.request(C, P, `${query.TYPE_URI}#request`, queryPayload(held.token)));
  if (answer.type !== query.RESPONSE_TYPE_URI) throw new Error("expected entries");
  held.copies.push(...(answer.payload as query.Response).entries.map((e) => (e as { ref: string }).ref));
}

/** 4. Revoke from the person's Management Client (hcp/grants/revoke 0.1, HCP 6.2.1). */
export async function stepRevoke(world: World, grantId: string): Promise<void> {
  const { provider: P, management: M } = world;
  world.step("4. Revoke (hcp/grants/revoke 0.1) from the Management Client");
  const done = await world.exchange(M, P, world.request(M, P, `${revoke.TYPE_URI}#request`, { grantId, accessToken: MANAGEMENT_TOKEN }));
  if (done.type !== revoke.RESPONSE_TYPE_URI) throw new Error("expected the revocation");
}

/**
 * 5. The same query, sent before any notice arrives, fails with
 * hcp:invalidToken: the grant is checked on every request, so revocation takes
 * effect at once (HCP 6.2.2). The client acts on that at once too: it drops
 * the token and the copies (hcp/grants/revoked "Independence").
 */
export async function stepQueryAfterRevoke(world: World, grantId: string): Promise<TrustTaskDocument<{ code: string }>> {
  const { provider: P, contextClient: C } = world;
  world.step("5. Query after revoke (hcp/entries/query 0.1) fails with hcp:invalidToken");
  const held = world.wallet.get(grantId)!;
  const refused = await world.exchange(C, P, world.request(C, P, `${query.TYPE_URI}#request`, queryPayload(held.token)));
  if (refused.type === error.TYPE_URI && refused.payload.code === "hcp:invalidToken") world.forgetGrant(grantId);
  return refused;
}

/**
 * 6. Revocation notice to the grant's client (hcp/grants/revoked 0.1, HCP
 * 6.2.3, 9.9). The client records the noticeId and the grant and stops using
 * the grant; it never sends the old token again. Returns the notice sent.
 */
export async function stepNotice(world: World, grantId: string): Promise<TrustTaskDocument<revoked.Payload>> {
  const P = world.provider;
  world.step("6. Revocation notice (hcp/grants/revoked 0.1)");
  const grant = world.grants.find((g) => g.grantId === grantId)!;
  const client = world.byVid(grant.client);
  const notice = world.request(P, client, `${revoked.TYPE_URI}#request`, { grantId, noticeId: world.ids.next(), revokedAt: grant.revokedAt! });
  const ack = await world.exchange(P, client, notice);
  if (ack.type !== revoked.RESPONSE_TYPE_URI) throw new Error("expected the acknowledgement");
  return notice;
}

/** Throws unless `reply` is the fixed permissionDenied every refusal of a grant request carries. */
function expectDenied(reply: TrustTaskDocument<any>): void {
  if (reply.type !== error.TYPE_URI || reply.payload.code !== "permissionDenied") throw new Error("expected permissionDenied");
}

/**
 * 7. The person declines (HCP 6.1.7): the client's next resubmission on the
 * thread is refused with the fixed permissionDenied, and no grant is issued
 * (hcp/grants/request "Errors"). Returns the thread and the refusal.
 */
export async function stepDecline(world: World): Promise<{ threadId: string; refused: TrustTaskDocument<any> }> {
  const { provider: P, contextClient: C } = world;
  world.step("7. Grant request declined (hcp/grants/request 0.1): the person says no");
  const first = world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK);
  const held = await world.exchange(C, P, first);
  if (held.type !== nextStep.TYPE_URI) throw new Error("expected a next step");
  world.personDeclines(first.threadId!);
  world.clock.advance(retryAfterOf((held.payload as nextStep.Payload).expects[0]?.hint));
  const refused = await world.exchange(C, P, world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK, first.threadId));
  expectDenied(refused);
  return { threadId: first.threadId!, refused };
}

/**
 * 8. The hold's lifetime passes with no decision: the policy denies. The next
 * resubmission gets the same permissionDenied; the thread is never reopened,
 * so a late approval changes nothing and a further resubmission is refused
 * too; a new request on a new thread is put to the person again
 * (hcp/grants/request "Holding" rule 6). Returns both threads and the refusals.
 */
export async function stepExpiredHold(
  world: World,
): Promise<{ threadId: string; newThreadId: string; refused: TrustTaskDocument<any>[] }> {
  const { provider: P, contextClient: C } = world;
  world.step("8. Grant request expired (hcp/grants/request 0.1): the hold's lifetime passes");
  const first = world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK);
  if ((await world.exchange(C, P, first)).type !== nextStep.TYPE_URI) throw new Error("expected a next step");
  world.personDoesNotDecide(first.threadId!);
  const refused = [await world.exchange(C, P, world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK, first.threadId))];
  world.personApproves(first.threadId!); // too late: the thread has ended
  refused.push(await world.exchange(C, P, world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK, first.threadId)));
  refused.forEach(expectDenied);
  // A client that still needs a grant asks again, on a new thread.
  const again = world.request(C, P, `${grantRequest.TYPE_URI}#request`, ASK);
  if ((await world.exchange(C, P, again)).type !== nextStep.TYPE_URI) throw new Error("expected a next step");
  return { threadId: first.threadId!, newThreadId: again.threadId!, refused };
}

export interface DemoResult {
  transcript: string;
  threadId: string;
  grantId: string;
  postRevokeError: TrustTaskDocument<{ code: string }>;
  notice: TrustTaskDocument<revoked.Payload>;
  declined: { threadId: string; refused: TrustTaskDocument<any> };
  expiredHold: { threadId: string; newThreadId: string; refused: TrustTaskDocument<any>[] };
}

/** The eight steps in order. */
export async function runDemo(world = new World()): Promise<DemoResult> {
  await stepDiscover(world);
  const { threadId, grantId } = await stepGrantRequest(world);
  await stepQuery(world, grantId);
  await stepRevoke(world, grantId);
  const postRevokeError = await stepQueryAfterRevoke(world, grantId);
  const notice = await stepNotice(world, grantId);
  const declined = await stepDecline(world);
  const expiredHold = await stepExpiredHold(world);
  return { transcript: world.transcript.join(""), threadId, grantId, postRevokeError, notice, declined, expiredHold };
}
