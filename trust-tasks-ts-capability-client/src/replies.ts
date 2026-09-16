import {
  GIT_TRUST_ALREADY_GRANTED_CODE,
  GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL,
  GIT_TRUST_NOT_GRANTED_CODE,
  GIT_TRUST_NOT_GRANTED_CODE_CAMEL,
} from "./constants.js";
import {
  parseEnvelopeDocumentFor,
  repliesTo,
  typeUriParts,
  type CapabilityDocument,
} from "./document.js";

// --- git-trust write replies (grant/revoke producers) -----------------------

/** The classification of a `git-trust` write reply.
 *
 * `idempotentSuccess` is load-bearing for redelivery-safe writers: an
 * `already_granted` / `not_granted` rejection means the desired end state
 * already holds, so the write is done, not failed. */
export type WriteOutcome =
  | { kind: "success" }
  | { kind: "idempotentSuccess" }
  | { kind: "rejected"; code: string; message?: string };

/**
 * How much a caller is willing to infer from a non-conforming peer. The default
 * (`{}`) infers nothing: an outcome is decided from the error `code` alone.
 */
export interface ReplyPolicy {
  /**
   * **DEPRECATED — opt-in compatibility only.** Also treat a `taskFailed` whose
   * free-text `message` contains `already_granted:` or `not_granted:` as
   * `idempotentSuccess`.
   *
   * This is how classification worked before, and it was wrong: SPEC §8.2 makes
   * `message` non-normative free text, so deciding an outcome from it hinges on
   * wording the emitting service may reword, translate or drop. Enable it only
   * while a specific peer still emits the free-text form; the correct fix is on
   * the emitting side (send the extended codes SPEC §8.5 provides).
   */
  acceptLegacyFreeTextIdempotence?: boolean;
}

function errorCodeAndMessage(doc: CapabilityDocument): { code: string; message?: string } {
  const payload = doc.payload;
  const code = typeof payload.code === "string" ? payload.code : "unknown";
  const message = typeof payload.message === "string" ? payload.message : undefined;
  return { code, message };
}

/** Whether `code` is one of the extended codes that mean "the end state you
 * asked for already holds" (SPEC §8.5). */
function isIdempotentCode(code: string): boolean {
  return (
    code === GIT_TRUST_ALREADY_GRANTED_CODE ||
    code === GIT_TRUST_NOT_GRANTED_CODE ||
    code === GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL ||
    code === GIT_TRUST_NOT_GRANTED_CODE_CAMEL
  );
}

/**
 * Classify the reply to a `git-trust/grant` or `git-trust/revoke` write.
 *
 * `expectedThreadId` is {@link correlationThread} of the document you sent. A
 * reply threaded to anything else yields `undefined` — "not an answer to this
 * request" — rather than an outcome: acting on an uncorrelated reply lets
 * whichever document arrives next decide the fate of a write it has nothing to
 * do with.
 *
 * Idempotent success is keyed on the extended error code (SPEC §8.5), never on
 * the free-text `message`. `policy.acceptLegacyFreeTextIdempotence` is the
 * deprecated compatibility path.
 */
export function classifyGitTrustReply(
  doc: CapabilityDocument,
  expectedThreadId: string,
  policy: ReplyPolicy = {},
): WriteOutcome | undefined {
  // SPEC §4.9: correlation comes first.
  if (!repliesTo(doc, expectedThreadId)) return undefined;

  const { slug, isResponse } = typeUriParts(doc.type);
  if (slug === "trust-task-error") {
    const { code, message } = errorCodeAndMessage(doc);
    if (isIdempotentCode(code)) return { kind: "idempotentSuccess" };
    // DEPRECATED: pre-extended-code peers signalled idempotence in the free-text
    // `message` under a bare `taskFailed`. SPEC §8.2 makes `message`
    // non-normative, so this is a string match on a field nobody promised to
    // keep stable — opt-in only.
    if (policy.acceptLegacyFreeTextIdempotence && code === "taskFailed") {
      const reason = message ?? "";
      if (reason.includes("already_granted:") || reason.includes("not_granted:")) {
        return { kind: "idempotentSuccess" };
      }
    }
    return { kind: "rejected", code, message };
  }
  if (isResponse && (slug === "git-trust/grant" || slug === "git-trust/revoke")) {
    return { kind: "success" };
  }
  return undefined;
}

// --- governance/capability replies (management UIs) -------------------------

/** One capability entry as rendered by a management UI. */
export interface CapabilitySummary {
  slug: string;
  title?: string;
  version: string;
  enabled: boolean;
  enabledAt?: string;
  delegate?: string;
  /** The full manifest, for a detail view. */
  manifest: Record<string, unknown>;
}

/** The classification of a `governance/capability/*` reply. */
export type CapabilityReply =
  | { kind: "listing"; entries: CapabilitySummary[] }
  | { kind: "toggled"; capability: string; enabled: boolean }
  | { kind: "rejected"; code: string; message?: string };

function asRecord(v: unknown): Record<string, unknown> | undefined {
  return typeof v === "object" && v !== null ? (v as Record<string, unknown>) : undefined;
}

function summaryOf(entry: unknown): CapabilitySummary | undefined {
  const rec = asRecord(entry);
  const manifest = rec && asRecord(rec.manifest);
  if (!rec || !manifest) return undefined;
  if (typeof manifest.capability !== "string") return undefined;
  return {
    slug: manifest.capability,
    title: typeof manifest.title === "string" ? manifest.title : undefined,
    version: typeof manifest.version === "string" ? manifest.version : "?",
    enabled: rec.enabled === true,
    enabledAt: typeof rec.enabledAt === "string" ? rec.enabledAt : undefined,
    delegate: typeof rec.delegate === "string" ? rec.delegate : undefined,
    manifest,
  };
}

/**
 * Classify a `governance/capability/*` reply document. `undefined` when it is
 * not part of this family, or is not threaded to `expectedThreadId`.
 */
export function parseCapabilityReply(
  doc: CapabilityDocument,
  expectedThreadId: string,
): CapabilityReply | undefined {
  if (!repliesTo(doc, expectedThreadId)) return undefined;

  const { slug, isResponse } = typeUriParts(doc.type);
  if (slug === "trust-task-error") {
    const { code, message } = errorCodeAndMessage(doc);
    return { kind: "rejected", code, message };
  }
  if (!isResponse) return undefined;
  switch (slug) {
    case "governance/capability/list": {
      const raw = doc.payload.capabilities;
      const entries = Array.isArray(raw)
        ? raw.map(summaryOf).filter((s): s is CapabilitySummary => s !== undefined)
        : [];
      return { kind: "listing", entries };
    }
    case "governance/capability/enable":
    case "governance/capability/disable":
      return {
        kind: "toggled",
        capability: typeof doc.payload.capability === "string" ? doc.payload.capability : "",
        enabled: doc.payload.enabled === true,
      };
    default:
      return undefined;
  }
}

/**
 * Parse an inbound envelope body directly into a reply to the request threaded
 * `expectedThreadId` — the entry point for a UI's inbound dispatch, which holds
 * only the body value. `undefined` when the body is not a
 * `governance/capability/*` reply, or belongs to a different exchange.
 */
export function parseEnvelopeReply(
  body: unknown,
  expectedThreadId: string,
): CapabilityReply | undefined {
  const doc = parseEnvelopeDocumentFor(body, expectedThreadId);
  if (doc === undefined) return undefined;
  return parseCapabilityReply(doc, expectedThreadId);
}
