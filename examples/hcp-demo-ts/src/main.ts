// The HCP-over-Trust-Tasks demo: run the six steps and print every document.
//
//   npm run all      # generate from trust-tasks.lock, build, run, test
//
// A DEMONSTRATION, not a reference provider. See README.md and SCENARIO.md.

import { runDemo } from "./demo.js";

const { transcript, postRevokeError } = await runDemo();
process.stdout.write(transcript);
const code = (postRevokeError.payload as { code: string }).code;
if (code !== "hcp:invalidToken") throw new Error(`expected hcp:invalidToken after revoke, got ${code}`);
console.log("\nhcp-demo-ts: discover, grant request (held, then granted), query, revoke, post-revoke refusal and notice all OK");
