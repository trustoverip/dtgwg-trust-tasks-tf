// Payload validation with a full JSON Schema 2020-12 engine (ajv), as
// examples/getting-started-ts/src/validator.ts recommends for a real
// application. The HCP schemas lean on `pattern` (urn:uuid identifiers),
// `format` and `uniqueItems`, which that example's structural stand-in ignores.
//
// Each generated module embeds its payload schema with cross-file `$ref`s
// already inlined, so no resolver is needed.

import { Ajv2020 } from "ajv/dist/2020.js";
import addFormatsModule from "ajv-formats";
import type { PayloadValidator, TrustTaskDocument } from "@openvtc/trust-tasks/runtime";
import * as discovery from "@openvtc/trust-tasks/trust-task-discovery/0.4/payload";
import * as error from "@openvtc/trust-tasks/trust-task-error/0.6/payload";
import * as nextStep from "@openvtc/trust-tasks/trust-task-next-step/0.1/payload";

import { HcpEntriesQuery_v0_1, HcpGrantsRequest_v0_1, HcpGrantsRevoke_v0_1, HcpGrantsRevoked_v0_1 } from "./trust-tasks/index.js";

const addFormats = addFormatsModule as unknown as (ajv: Ajv2020) => Ajv2020;
const ajv = addFormats(new Ajv2020({ strict: false, allErrors: true }));
const compiled = new WeakMap<object, ReturnType<typeof ajv.compile>>();

function check(schema: object, payload: unknown): string[] {
  let validate = compiled.get(schema);
  if (validate === undefined) {
    validate = ajv.compile(schema);
    compiled.set(schema, validate);
  }
  return validate(payload) ? [] : (validate.errors ?? []).map((e) => `payload${e.instancePath} ${e.message}`);
}

/** The validator the receiving end's SPEC §7.2 pipeline runs (item 2). */
export const schemaValidator: PayloadValidator = {
  validate(schema, payload) {
    const errors = check(schema as object, payload);
    return { ok: errors.length === 0, errors };
  },
};

/** The schema a reply's payload must satisfy, by its `type`. */
const REPLY_SCHEMAS: Record<string, object> = {
  [discovery.RESPONSE_TYPE_URI]: discovery.RESPONSE_PAYLOAD_SCHEMA,
  [nextStep.TYPE_URI]: nextStep.PAYLOAD_SCHEMA,
  [error.TYPE_URI]: error.PAYLOAD_SCHEMA,
  [HcpGrantsRequest_v0_1.RESPONSE_TYPE_URI]: HcpGrantsRequest_v0_1.RESPONSE_PAYLOAD_SCHEMA,
  [HcpEntriesQuery_v0_1.RESPONSE_TYPE_URI]: HcpEntriesQuery_v0_1.RESPONSE_PAYLOAD_SCHEMA,
  [HcpGrantsRevoke_v0_1.RESPONSE_TYPE_URI]: HcpGrantsRevoke_v0_1.RESPONSE_PAYLOAD_SCHEMA,
  [HcpGrantsRevoked_v0_1.RESPONSE_TYPE_URI]: HcpGrantsRevoked_v0_1.RESPONSE_PAYLOAD_SCHEMA,
};

/** A producer checks the reply it receives before acting on it, as a consumer checks a request. */
export function validatePayload(reply: TrustTaskDocument<unknown>): void {
  const schema = REPLY_SCHEMAS[reply.type];
  if (schema === undefined) throw new Error(`unexpected reply type ${reply.type}`);
  const errors = check(schema, reply.payload);
  if (errors.length > 0) throw new Error(`reply ${reply.type} fails its schema: ${errors.join("; ")}`);
}
