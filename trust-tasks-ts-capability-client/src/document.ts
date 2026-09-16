import type { TrustTaskDocument } from "@openvtc/trust-tasks";
import {
  CAPABILITY_DISABLE_TYPE,
  CAPABILITY_ENABLE_TYPE,
  CAPABILITY_LIST_TYPE,
  GIT_TRUST_GRANT_TYPE,
  GIT_TRUST_REVOKE_TYPE,
} from "./constants.js";

/** A capability document: a Trust Task whose payload is an open JSON object. */
export type CapabilityDocument = TrustTaskDocument<Record<string, unknown>>;

/** A fresh document `id`. One per *attempt* — never reused across attempts;
 * see {@link newAttempt}. */
function freshId(): string {
  return `urn:uuid:${crypto.randomUUID()}`;
}

/**
 * Build a capability Trust Task addressed `issuerDid` → `recipientDid`.
 *
 * Mints a fresh `id` and stamps `issuedAt` on every call, so each built document
 * is a new attempt in the sense of SPEC §8.4. To re-send one you have already
 * built, see {@link newAttempt} — the choice between resending the identical
 * document (a §8.4 retry the consumer absorbs) and minting a new one is enforced
 * by the consumer's §7.2 item-11 record.
 */
export function buildDocument(
  issuerDid: string,
  recipientDid: string,
  typeUri: string,
  payload: Record<string, unknown>,
): CapabilityDocument {
  return {
    id: freshId(),
    type: typeUri,
    issuer: issuerDid,
    recipient: recipientDid,
    issuedAt: new Date().toISOString(),
    payload,
  };
}

/**
 * A **new attempt** at the request `previous` carried: the same addressing, type
 * and payload under a fresh `id`, a fresh `issuedAt`, and no `proof`.
 *
 * This is the counterpart of a SPEC §8.4 retry, and the two are not
 * interchangeable. A **retry** is a bit-for-bit resend of `previous` (same `id`)
 * that the consumer's §7.2 item-11 record absorbs. A **new attempt** is a
 * different document — anything that changes the bytes makes it one, including a
 * re-stamped `issuedAt` or a re-signed `proof` over identical content — and MUST
 * carry a fresh `id`, or the consumer rejects it with `idConflict`.
 *
 * `proof` is cleared because it committed to the previous `id` and `issuedAt`;
 * sign the returned document before sending it. Where `previous` opened its own
 * exchange (no `threadId`), the new attempt opens a *new* one — wait on
 * {@link correlationThread} of the returned document, not of `previous`.
 */
export function newAttempt(previous: CapabilityDocument): CapabilityDocument {
  const { proof: _proof, ...rest } = previous;
  return { ...rest, id: freshId(), issuedAt: new Date().toISOString() };
}

/** Build a `governance/capability/list` request (status `all`). */
export function buildListDocument(issuerDid: string, vtcDid: string): CapabilityDocument {
  return buildDocument(issuerDid, vtcDid, CAPABILITY_LIST_TYPE, { status: "all" });
}

/**
 * Build a `governance/capability/enable` or `/disable` request. On enable,
 * `config.authority` defaults to the community's own DID — the community is the
 * authority its capability records are issued under.
 */
export function buildToggleDocument(
  issuerDid: string,
  vtcDid: string,
  slug: string,
  version: string,
  enable: boolean,
): CapabilityDocument {
  return enable
    ? buildDocument(issuerDid, vtcDid, CAPABILITY_ENABLE_TYPE, {
        capability: slug,
        version,
        config: { authority: vtcDid },
      })
    : buildDocument(issuerDid, vtcDid, CAPABILITY_DISABLE_TYPE, { capability: slug });
}

/** Build a `git-trust/grant`: grant `subjectDid` commit-signing trust for
 * `resource` (an org or `org/repo` slug). */
export function buildGitTrustGrant(
  authorityDid: string,
  registryDid: string,
  subjectDid: string,
  resource: string,
): CapabilityDocument {
  return buildDocument(authorityDid, registryDid, GIT_TRUST_GRANT_TYPE, {
    subject: subjectDid,
    resource,
  });
}

/** Build a `git-trust/revoke`. */
export function buildGitTrustRevoke(
  authorityDid: string,
  registryDid: string,
  subjectDid: string,
  resource: string,
  reason?: string,
): CapabilityDocument {
  const payload: Record<string, unknown> = { subject: subjectDid, resource };
  if (reason !== undefined) payload.reason = reason;
  return buildDocument(authorityDid, registryDid, GIT_TRUST_REVOKE_TYPE, payload);
}

// --- correlation + envelope parsing -----------------------------------------

/** The slug and success-response flag of a Type URI string (SPEC §4.4, §6.1).
 * The version is always the final path segment, so the slug is everything after
 * `/spec/` up to the last `/`; a `#response` fragment marks the response
 * variant. */
export function typeUriParts(typeUri: string): { slug: string; isResponse: boolean } {
  const hash = typeUri.indexOf("#");
  const base = hash >= 0 ? typeUri.slice(0, hash) : typeUri;
  const isResponse = hash >= 0 && typeUri.slice(hash + 1) === "response";
  const marker = "/spec/";
  const i = base.indexOf(marker);
  const path = i >= 0 ? base.slice(i + marker.length) : base;
  const lastSlash = path.lastIndexOf("/");
  const slug = lastSlash >= 0 ? path.slice(0, lastSlash) : path;
  return { slug, isResponse };
}

/** The thread an exchange started by `doc` is correlated by: its own `threadId`,
 * or its `id` where it opens the exchange (SPEC §4.9's fallback). Hold this from
 * the moment you send a request; every reply-classifying function wants it as
 * `expectedThreadId`. */
export function correlationThread(doc: { id: string; threadId?: string }): string {
  return doc.threadId ?? doc.id;
}

/** Whether `reply` is threaded to `expectedThreadId` — SPEC §4.9 correlation,
 * and the precondition for acting on any reply. A reply with no `threadId`
 * matches nothing. */
export function repliesTo(reply: { threadId?: string }, expectedThreadId: string): boolean {
  return reply.threadId !== undefined && reply.threadId === expectedThreadId;
}

/** True when `body` has the shape of a Trust Task document. */
function isDocument(body: unknown): body is CapabilityDocument {
  return (
    typeof body === "object" &&
    body !== null &&
    typeof (body as Record<string, unknown>).id === "string" &&
    typeof (body as Record<string, unknown>).type === "string"
  );
}

/**
 * Parse a DIDComm envelope body into `[threadId, document]`, or `undefined` when
 * the body is not a threaded Trust Task document.
 *
 * The `threadId` is a **dispatch key, not a check**: it says which outstanding
 * request this document belongs to, not that it is a reply to anything you sent.
 * Correlate before acting — with {@link parseEnvelopeDocumentFor} or by finding
 * the thread in your own outstanding map.
 */
export function parseEnvelopeDocument(body: unknown): [string, CapabilityDocument] | undefined {
  if (!isDocument(body)) return undefined;
  const thid = body.threadId;
  if (typeof thid !== "string") return undefined;
  return [thid, body];
}

/** Parse a DIDComm envelope body into the document it carries, only if that
 * document is threaded to `expectedThreadId`. `undefined` covers both "not a
 * Trust Task document" and "belongs to another exchange". */
export function parseEnvelopeDocumentFor(
  body: unknown,
  expectedThreadId: string,
): CapabilityDocument | undefined {
  const parsed = parseEnvelopeDocument(body);
  if (parsed === undefined) return undefined;
  return repliesTo(parsed[1], expectedThreadId) ? parsed[1] : undefined;
}
