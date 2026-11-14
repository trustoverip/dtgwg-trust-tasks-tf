// Build the CloudFront viewer-request function for deployment.
//
// CloudFront Functions cap the function code at 10,240 bytes. The source,
// type-uri-negotiation.js, keeps the comments that explain each routing rule
// and passed that cap when #1217 added the JSON-404 branch — `update-function`
// then fails with FunctionSizeLimitExceeded. So what is deployed is this build
// of it: comments and whitespace removed, nothing else. `--minify-whitespace`
// only, never identifier or syntax minification, so the deployed code is the
// source's code; and an ES5 target, so esbuild prints every string with plain
// quotes instead of choosing template literals the source does not use.
//
//   node infra/cloudfront/build-function.mjs   # writes infra/cloudfront/build/type-uri-negotiation.min.js
//
// `npm run test:infra` runs the routing tests against this build as well as
// the source, and fails when the build nears the cap.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { transformSync } from "esbuild";

const here = dirname(fileURLToPath(import.meta.url));

/** CloudFront Functions' code size limit, in bytes. */
export const FUNCTION_SIZE_LIMIT = 10240;

/** The deployable function code, built from the source. */
export function buildFunction() {
  const source = readFileSync(join(here, "type-uri-negotiation.js"), "utf8");
  return transformSync(source, {
    loader: "js",
    minifyWhitespace: true,
    legalComments: "none",
    target: "es5",
  }).code;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const code = buildFunction();
  const size = Buffer.byteLength(code, "utf8");
  if (size > FUNCTION_SIZE_LIMIT) {
    console.error(`type-uri-negotiation: ${size} bytes, over CloudFront's ${FUNCTION_SIZE_LIMIT}-byte limit`);
    process.exit(1);
  }
  const out = join(here, "build", "type-uri-negotiation.min.js");
  mkdirSync(dirname(out), { recursive: true });
  writeFileSync(out, code);
  console.log(`wrote ${out} (${size} of ${FUNCTION_SIZE_LIMIT} bytes)`);
}
