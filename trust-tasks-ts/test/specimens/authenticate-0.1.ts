/**
 * A hand-written stand-in for the generated `auth/authenticate/0.1` module,
 * reduced to the parts `extends.test.ts` reads.
 *
 * The extends tests used to import the generated 0.1 module, which is
 * superseded and leaves the library in a withdrawal batch (CLAUDE.md
 * "Superseded, retired and withdrawn versions"). Any real older version is
 * superseded by definition, so a generated specimen would have to move every
 * quarter; this one does not. It keeps the real Type URI and the real root
 * shape — REQUIRED `challenge` + `sessionId`, closed root, closed response —
 * so the chain 0.3 → 0.2 → 0.1 is still one the registry build would accept.
 */

import type { SpecPolicy } from "../../src/_runtime/index.js";

export const TYPE_URI = "https://trusttasks.org/spec/auth/authenticate/0.1" as const;
export const RESPONSE_TYPE_URI = `${TYPE_URI}#response` as const;

export interface Payload {
  challenge: string;
  sessionId: string;
  scope?: string[];
  ext?: Record<string, unknown>;
}

export interface Response {
  session: Record<string, unknown>;
  tokens: Record<string, unknown>;
  ext?: Record<string, unknown>;
}

const OPEN_OBJECT = { type: "object" } as const;
const EXT = { type: "object" } as const;

export const PAYLOAD_SCHEMA = {
  type: "object",
  additionalProperties: false,
  required: ["challenge", "sessionId"],
  properties: {
    challenge: { type: "string", minLength: 16 },
    sessionId: { type: "string", minLength: 1 },
    scope: { type: "array", items: { type: "string", minLength: 1 } },
    ext: EXT,
  },
} as const;

export const RESPONSE_PAYLOAD_SCHEMA = {
  type: "object",
  additionalProperties: false,
  required: ["session", "tokens"],
  properties: { session: OPEN_OBJECT, tokens: OPEN_OBJECT, ext: EXT },
} as const;

export const SPEC: SpecPolicy = {
  typeUri: TYPE_URI,
  isBearer: false,
  isProofRequired: true,
  isRecipientRequired: true,
  isIssuedAtRequired: true,
  payloadSchema: PAYLOAD_SCHEMA,
};

export const RESPONSE_SPEC: SpecPolicy = {
  typeUri: RESPONSE_TYPE_URI,
  isBearer: false,
  isProofRequired: true,
  isRecipientRequired: true,
  isIssuedAtRequired: true,
  payloadSchema: RESPONSE_PAYLOAD_SCHEMA,
};
