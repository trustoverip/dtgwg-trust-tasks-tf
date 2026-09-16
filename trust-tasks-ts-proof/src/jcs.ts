/**
 * JSON Canonicalization Scheme (RFC 8785) — the byte form eddsa-jcs-2022 and
 * ecdsa-jcs-2019 hash.
 *
 * Object keys are sorted by UTF-16 code unit (what `Array.prototype.sort` does
 * on strings, which is what RFC 8785 §3.2.3 specifies), strings are serialised
 * with JSON's own escaping, and numbers use the shortest round-tripping form —
 * which for the integers and JSON doubles a Trust Task document carries is
 * exactly `JSON.stringify(n)`. Trust Task documents carry no number outside the
 * safe-integer / plain-double range, so no bespoke number formatter is needed.
 */
export function canonicalize(value: unknown): string {
  if (value === null || typeof value === "boolean" || typeof value === "number") {
    if (typeof value === "number" && !Number.isFinite(value)) {
      throw new Error("JCS: non-finite number");
    }
    return JSON.stringify(value);
  }
  if (typeof value === "string") return JSON.stringify(value);
  if (Array.isArray(value)) return "[" + value.map(canonicalize).join(",") + "]";
  if (typeof value === "object") {
    const obj = value as Record<string, unknown>;
    const keys = Object.keys(obj)
      .filter((k) => obj[k] !== undefined)
      .sort();
    return "{" + keys.map((k) => JSON.stringify(k) + ":" + canonicalize(obj[k])).join(",") + "}";
  }
  throw new Error(`JCS: cannot canonicalize ${typeof value}`);
}

const encoder = new TextEncoder();

/** The UTF-8 bytes of the canonical form of `value`. */
export function canonicalBytes(value: unknown): Uint8Array {
  return encoder.encode(canonicalize(value));
}
