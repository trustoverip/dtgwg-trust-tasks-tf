/**
 * Error codes — SPEC.md §8.3 (standard) and §8.5 (extended).
 *
 * Hand-written. Mirrors `error.rs` in trust-tasks-rs, including its acceptance
 * of both the framework 0.2 lowerCamelCase spellings and the frozen 0.1
 * snake_case ones, so a 0.2 consumer can still read an error response from a
 * 0.1 peer.
 */

/** The framework-defined standard error codes (SPEC.md §8.3). */
export const STANDARD_CODES = [
  "malformedRequest",
  "unsupportedType",
  "unsupportedVersion",
  "expired",
  "proofRequired",
  "proofInvalid",
  "permissionDenied",
  "wrongRecipient",
  "identityMismatch",
  "idConflict",
  "taskFailed",
  "unavailable",
  "internalError",
] as const;

export type StandardCode = (typeof STANDARD_CODES)[number];

/**
 * Framework 0.1 snake_case spellings, mapped to their 0.2 lowerCamelCase form.
 * `expired` and `unavailable` are single words and unchanged.
 */
const LEGACY_STANDARD: Readonly<Record<string, StandardCode>> = {
  malformed_request: "malformedRequest",
  unsupported_type: "unsupportedType",
  unsupported_version: "unsupportedVersion",
  proof_required: "proofRequired",
  proof_invalid: "proofInvalid",
  permission_denied: "permissionDenied",
  wrong_recipient: "wrongRecipient",
  identity_mismatch: "identityMismatch",
  id_conflict: "idConflict",
  task_failed: "taskFailed",
  internal_error: "internalError",
};

const STANDARD_SET: ReadonlySet<string> = new Set(STANDARD_CODES);

/**
 * Normalize a wire `code` to its canonical 0.2 spelling when it is a standard
 * code, or return it unchanged.
 *
 * A consumer comparing a received code against {@link STANDARD_CODES} must
 * normalize first, or a 0.1 peer's `proof_required` reads as an unrecognized
 * extended code and falls through to `taskFailed` (§8.5), losing the meaning.
 */
export function normalizeCode(code: string): string {
  if (STANDARD_SET.has(code)) return code;
  return LEGACY_STANDARD[code] ?? code;
}

/** Whether `code` is a standard §8.3 code, in either casing. */
export function isStandardCode(code: string): code is StandardCode {
  return STANDARD_SET.has(normalizeCode(code));
}

/**
 * The local part of an extended code: a lowercase letter, then letters of
 * either case, digits, or underscores.
 *
 * Both casings are accepted so framework 0.2 lowerCamelCase locals
 * (`documentRevoked`) and frozen 0.1 snake_case locals (`document_revoked`)
 * parse under one rule. SPEC §4.10 item 4 SHOULDs lowerCamelCase for new
 * specifications; only the first character is required to be lowercase.
 */
const LOCAL_RE = /^[a-z][A-Za-z0-9_]*$/;

/** One path segment of a slug: lowercase, hyphen-separated (§6.1). */
const SEGMENT_RE = /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/;

function validNamespace(namespace: string): boolean {
  return namespace.length > 0 && namespace.split("/").every((s) => SEGMENT_RE.test(s));
}

/**
 * Build an extended error code under a specification's own slug (SPEC §8.5).
 *
 * `typeUri` is normally a generated module's `TYPE_URI`, so the namespace
 * cannot drift from the type's identity — the same guarantee
 * `Payload::extended_code` gives in Rust. The `#response` fragment is stripped:
 * an error raised while handling a response still belongs to the bare slug.
 *
 * @throws if the derived slug or `local` is malformed.
 */
export function extendedCode(typeUri: string, local: string): string {
  const slug = slugFromTypeUri(typeUri);
  if (!LOCAL_RE.test(local)) {
    throw new Error(
      `extendedCode: local part ${JSON.stringify(local)} must match ${LOCAL_RE} ` +
        `(a lowercase first character, then letters, digits or underscores)`,
    );
  }
  return `${slug}:${local}`;
}

/**
 * Build an extended error code under a *family namespace* — a proper path
 * prefix of the specification's slug (SPEC §8.5 rule 2).
 *
 * For a condition whose meaning is defined once across a family rather than
 * per specification, such as `did-management:unknownDomain` on every
 * `did-management/*` task. Prefer {@link extendedCode} otherwise: a family
 * namespace claims the condition means the same thing across every sibling.
 *
 * `namespace` is checked against the slug derived from `typeUri` rather than
 * taken on trust, so §8.5's prefix rule holds by construction. A sibling's slug
 * is rejected — it shares a prefix but is not itself one, which is exactly the
 * confusion §8.5 forbids.
 *
 * @throws if `namespace` is neither the slug nor a path prefix of it.
 */
export function familyCode(typeUri: string, namespace: string, local: string): string {
  const slug = slugFromTypeUri(typeUri);
  const segments = slug.split("/");
  const permitted = segments.map((_, i) => segments.slice(0, i + 1).join("/"));
  if (!permitted.includes(namespace)) {
    throw new Error(
      `familyCode: namespace ${JSON.stringify(namespace)} is neither the slug ` +
        `${JSON.stringify(slug)} nor a path prefix of it (SPEC §8.5 rule 2). ` +
        `Permitted: ${permitted.map((p) => JSON.stringify(p)).join(", ")}`,
    );
  }
  if (!LOCAL_RE.test(local)) {
    throw new Error(`familyCode: local part ${JSON.stringify(local)} must match ${LOCAL_RE}`);
  }
  return `${namespace}:${local}`;
}

/** The slug of a Type URI, with any `#request` / `#response` fragment removed. */
export function slugFromTypeUri(typeUri: string): string {
  const PREFIX = "https://trusttasks.org/spec/";
  if (!typeUri.startsWith(PREFIX)) {
    throw new Error(`not a Trust Task Type URI: ${JSON.stringify(typeUri)}`);
  }
  const rest = typeUri.slice(PREFIX.length).split("#")[0]!;
  // Trailing segment is the MAJOR.MINOR version; everything before it is the slug.
  const parts = rest.split("/");
  const slug = parts.slice(0, -1).join("/");
  if (!validNamespace(slug)) {
    throw new Error(`Type URI ${JSON.stringify(typeUri)} yielded an invalid slug`);
  }
  return slug;
}
