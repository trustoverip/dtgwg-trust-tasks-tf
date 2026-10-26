/**
 * Parsed Trust Task *Type URI* — every form SPEC.md §6.1 defines: the public
 * registry's canonical form, and the two shapes a third-party specification
 * (SPEC.md §6.5) may use under any other authority:
 *
 * ```text
 * https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>          public registry
 * <prefix>/spec/<slug>/<MAJOR.MINOR>                        path form (https, DID URL, …)
 * urn:<NID>:<prefix>:spec:<slug-segments>:<MAJOR.MINOR>     URN form
 * ```
 *
 * each optionally followed by `#request` or `#response` (§4.4.1). A Type URI
 * whose scheme is `urn` must use the URN form, and every other scheme the path
 * form; `http` is refused in every form. {@link formatTypeUri} reproduces the
 * parsed string exactly — a Type URI is a name, compared by its whole string
 * (§6.1), so the authority is kept verbatim and is part of identity: the same
 * slug under two authorities is two specifications.
 *
 * Hand-written. Mirrors `type_uri.rs` in trust-tasks-rs, whose unit tests are
 * the source of most of the vectors in `test/type-uri.test.ts`.
 */

/** The `#request` / `#response` fragment of a Type URI (SPEC §4.4.1). */
export type TypeUriVariant = "request" | "response";

/** A parsed Type URI. Plain data; build a string back with {@link formatTypeUri}. */
export interface TypeUri {
  /**
   * Everything before `/spec/` (path form) or `:spec:` (URN form), verbatim —
   * e.g. `https://trusttasks.org`, `did:web:acme.example`, `urn:example:tasks`.
   */
  readonly authority: string;
  /** `"path"` — `<authority>/spec/<slug>/<M.N>`; `"urn"` — `<authority>:spec:<seg>:…:<M.N>`. */
  readonly form: "path" | "urn";
  /** The slug, `/`-separated in both forms (`payments:settle` in a URN is `payments/settle`). */
  readonly slug: string;
  readonly major: number;
  readonly minor: number;
  /** The fragment, or `null` when absent (semantically equivalent to `"request"`, §4.4.1). */
  readonly variant: TypeUriVariant | null;
}

/** Why a string is not a Type URI — the cases of Rust's `ParseTypeUriError`. */
export type TypeUriErrorKind =
  /** No scheme, a malformed one, or `http` (refused in every form, §6.1). */
  | "scheme"
  /** Neither the path form nor (for `urn:`) the URN form. */
  | "shape"
  /** The slug fails the §6.1 grammar. */
  | "slug"
  /** The slug is in the §6.1 reserved namespace and is not a framework slug. */
  | "reservedSlug"
  /** The version is not `MAJOR.MINOR` (§5.1). */
  | "version"
  /** A fragment other than `#request` / `#response`. */
  | "fragment";

/** Thrown by {@link parseTypeUri}. */
export class TypeUriError extends Error {
  readonly kind: TypeUriErrorKind;
  /** The string that failed to parse (for `slug` / `version` / `fragment`, the offending part). */
  readonly input: string;

  constructor(kind: TypeUriErrorKind, input: string, message: string) {
    super(message);
    this.name = "TypeUriError";
    this.kind = kind;
    this.input = input;
  }
}

/** The public registry's authority. */
export const REGISTRY_TYPE_URI_AUTHORITY = "https://trusttasks.org";

const SCHEME_RE = /^[A-Za-z][A-Za-z0-9+.-]*$/;
const SEGMENT_RE = /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/;
const VERSION_RE = /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/;
const U32_MAX = 0xffff_ffff;

/**
 * The framework slugs permitted inside the §6.1 reservation. A third
 * hand-maintained copy of `#/properties/slug/anyOf[1]/enum` in
 * `specs/spec.meta.schema.json` (Rust's `is_allowed_framework_slug` is the
 * second); `test/type-uri.test.ts` reads that file and fails if they disagree.
 */
export const FRAMEWORK_RESERVED_SLUGS: readonly string[] = Object.freeze([
  "trust-task",
  "trust-task-error",
  "trust-task-ok",
  "trust-task-next-step",
  "trust-task-discovery",
  "trust-task-control",
  "trust-ceremony-receipt",
]);

/** Whether `slug` is in the §6.1 reserved namespace `^trust-(task|ceremony)($|-|/)`. */
function isReservedNamespace(slug: string): boolean {
  const first = slug.split("/")[0]!;
  return ["trust-task", "trust-ceremony"].some((stem) => first === stem || first.startsWith(`${stem}-`));
}

/** Throws unless `slug` satisfies the §6.1 grammar and reservation. */
function checkSlug(slug: string): void {
  if (slug.length === 0 || !slug.split("/").every((s) => SEGMENT_RE.test(s))) {
    throw new TypeUriError("slug", slug, `slug ${JSON.stringify(slug)} does not satisfy SPEC §6.1 grammar`);
  }
  if (isReservedNamespace(slug) && !FRAMEWORK_RESERVED_SLUGS.includes(slug)) {
    throw new TypeUriError("reservedSlug", slug, `slug ${JSON.stringify(slug)} is reserved per SPEC §6.1`);
  }
}

function parseVersion(s: string): [number, number] {
  const m = VERSION_RE.exec(s);
  const major = m ? Number(m[1]) : NaN;
  const minor = m ? Number(m[2]) : NaN;
  if (!(major <= U32_MAX && minor <= U32_MAX)) {
    throw new TypeUriError("version", s, `version ${JSON.stringify(s)} must be MAJOR.MINOR per SPEC §5.1`);
  }
  return [major, minor];
}

/**
 * Parse a Type URI in any SPEC §6.1 form.
 *
 * @throws {TypeUriError} if `typeUri` is not one — its `kind` says why.
 */
export function parseTypeUri(typeUri: string): TypeUri {
  if (typeof typeUri !== "string") {
    throw new TypeUriError("scheme", String(typeUri), "a Type URI must be a string");
  }
  const hash = typeUri.indexOf("#");
  const bare = hash < 0 ? typeUri : typeUri.slice(0, hash);
  const fragment = hash < 0 ? null : typeUri.slice(hash + 1);
  let variant: TypeUriVariant | null = null;
  if (fragment !== null) {
    if (fragment !== "request" && fragment !== "response") {
      throw new TypeUriError(
        "fragment",
        fragment,
        `only #request and #response fragments are permitted in a Type URI (got #${fragment})`,
      );
    }
    variant = fragment;
  }

  // SPEC §6.1: any absolute URI, but never `http` — an `http` Type URI would
  // normalise a transport-downgrade path for a consumer that dereferences it.
  const colon = bare.indexOf(":");
  const scheme = colon < 0 ? "" : bare.slice(0, colon);
  if (!SCHEME_RE.test(scheme) || scheme.toLowerCase() === "http") {
    throw new TypeUriError(
      "scheme",
      typeUri,
      `Type URI must have a scheme other than http (SPEC §6.1): ${JSON.stringify(typeUri)}`,
    );
  }
  const shape = (): TypeUriError =>
    new TypeUriError(
      "shape",
      typeUri,
      `Type URI must be <prefix>/spec/<slug>/<major.minor>, or ` +
        `urn:<nid>:<prefix>:spec:<slug-segments>:<major.minor>: ${JSON.stringify(typeUri)}`,
    );

  let authority: string;
  let slug: string;
  let version: string;
  let form: TypeUri["form"];
  if (scheme.toLowerCase() === "urn") {
    // `urn:<NID>:<prefix>:spec:<seg>:…:<MAJOR.MINOR>` — the slug is the
    // segments between the first `:spec:` and the version.
    const at = bare.indexOf(":spec:");
    if (at < 0) throw shape();
    authority = bare.slice(0, at);
    const rest = bare.slice(at + ":spec:".length);
    const parts = authority.split(":");
    const nid = parts[1];
    const ns = parts.slice(2).join(":");
    if (!nid || !ns || rest.includes("/")) throw shape();
    const cut = rest.lastIndexOf(":");
    if (cut < 0) throw shape();
    slug = rest.slice(0, cut).replace(/:/g, "/");
    version = rest.slice(cut + 1);
    form = "urn";
  } else {
    // `<prefix>/spec/<slug>/<MAJOR.MINOR>` — an https authority (with an
    // optional path prefix), a DID URL, or any other scheme's prefix.
    const at = bare.indexOf("/spec/");
    if (at < 0) throw shape();
    authority = bare.slice(0, at);
    const afterScheme = authority.slice(scheme.length + 1);
    if (
      afterScheme.replace(/^\/+/, "").length === 0 ||
      (scheme.toLowerCase() === "https" && !afterScheme.startsWith("//"))
    ) {
      throw shape();
    }
    const rest = bare.slice(at + "/spec/".length);
    // The version is the final '/' segment; everything before it is the slug.
    const cut = rest.lastIndexOf("/");
    if (cut < 0) throw shape();
    slug = rest.slice(0, cut);
    version = rest.slice(cut + 1);
    form = "path";
  }

  checkSlug(slug);
  const [major, minor] = parseVersion(version);
  return { authority, form, slug, major, minor, variant };
}

/** {@link parseTypeUri}, returning `null` instead of throwing. */
export function tryParseTypeUri(typeUri: string): TypeUri | null {
  try {
    return parseTypeUri(typeUri);
  } catch (e) {
    if (e instanceof TypeUriError) return null;
    throw e;
  }
}

/**
 * The string form of `uri`. For a value {@link parseTypeUri} returned, exactly
 * the string it parsed.
 */
export function formatTypeUri(uri: TypeUri): string {
  const v = `${uri.major}.${uri.minor}`;
  const base =
    uri.form === "urn"
      ? `${uri.authority}:spec:${uri.slug.replace(/\//g, ":")}:${v}`
      : `${uri.authority}/spec/${uri.slug}/${v}`;
  return uri.variant === null ? base : `${base}#${uri.variant}`;
}

/**
 * A sibling of `uri` at `major.minor`: the same authority, form, slug and
 * variant, only the version changed. A declared-extension chain (SPEC §5.2)
 * names only versions, so this is how it is walked — and why a chain never
 * crosses authorities. (Rust's `TypeUri::with_version`.)
 */
export function typeUriWithVersion(uri: TypeUri, major: number, minor: number): TypeUri {
  return { ...uri, major, minor };
}

/**
 * Whether `a` and `b` name versions of the same specification: the same
 * authority, form and slug. Two Type URIs with the same slug under different
 * authorities are different specifications (SPEC §6.1).
 */
export function sameSpecification(a: TypeUri, b: TypeUri): boolean {
  return a.authority === b.authority && a.form === b.form && a.slug === b.slug;
}
