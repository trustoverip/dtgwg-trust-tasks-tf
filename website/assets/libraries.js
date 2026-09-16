/* ============================================================
   Trust Tasks — Client library registry
   ------------------------------------------------------------
   Hand-edited list of the reference client libraries, and what
   each one can actually do. Rendered by ImplementationsPage as
   a capability matrix.

   ⚠️ TWO RULES, both enforced by `checkLibraryRegistry()` in
   scripts/build-registry.mjs so this cannot quietly rot:

   1. Every library's `dir` must exist in the source tree, and
      every package named under `transports` must too. A library
      claiming a transport it does not ship is the defect this
      guard exists to catch — the matrix is a promise to a
      reader choosing a language.

   2. Every transport binding under `bindings/<slug>/` must
      appear in TT_LIBRARY_CAPABILITIES. Publishing a binding
      that no row mentions would leave the matrix silently
      incomplete, which is how `didcomm/0.2` and `didcomm-v1/0.1`
      stayed invisible on this site (see CLAUDE.md).

   ── Three states, not two ───────────────────────────────
   `capabilities` is what a library SHIPS — you can install it
   today. `foundations` is what a library could be BUILT ON:
   a published package in that language that provides the
   underlying protocol, named so the claim is checkable rather
   than aspirational. A dash means neither.

   A `foundations` entry is evidence, NOT a roadmap commitment.
   Only add one where the named package genuinely exists and
   does the thing. `tsp` on pub.dev is the Travelling Salesperson
   Problem and `tsp-sdk` on npm is an empty security placeholder,
   so a TSP foundation points at the affinidi-tsp-<lang> repo, not
   a registry package — and only where a binding is actually built
   on it (Dart today).

   ⚠️ NO VERSION NUMBERS LIVE HERE. Every install line uses its
   registry's own "latest" idiom, so the page cannot go stale
   between releases — the previous ecosystem entry claimed
   "Tracking framework 0.2 (0.2.x crate line)" while the
   workspace was at 0.21.
   ============================================================ */

/* The capability rows, in the order the matrix renders them.

   `binding` ties a row to a transport binding specification under
   `bindings/<slug>/`, which is what lets the build guard check the two lists
   against each other and lets each row link to the binding it names. */
window.TT_LIBRARY_CAPABILITIES = [
  {
    id: "types",
    group: "Core",
    label: "Generated payload types",
    note: "A typed request and response for every specification in the registry, regenerated from the schemas and guarded against drift in CI.",
  },
  {
    id: "pipeline",
    group: "Core",
    label: "SPEC §7.2 consumer pipeline",
    note: "Items 4–8 — expiry, recipient, identity cross-check, proof policy and audience binding — plus the §8.1 routing that decides where a rejection is addressed.",
  },
  {
    id: "schemas",
    group: "Core",
    label: "Embedded payload schemas",
    note: "§7.2 item 2. Each generated module carries its own schema with cross-file $refs already inlined, so validation needs no resolver and no filesystem.",
  },
  {
    id: "freshness",
    group: "Core",
    label: "Freshness bounds",
    note: "§4.2 and §7.2 item 4 — future-dating, empty validity intervals, and the acceptance window that bounds the replay record.",
  },
  {
    id: "replay",
    group: "Core",
    label: "Duplicate-execution guard",
    note: "§7.2 item 11. Every transport binding delegates replay defence to the consumer, so a library without this leaves an ordinary mediator redelivery able to execute a consequential task twice.",
  },
  {
    id: "errors",
    group: "Core",
    label: "Standard + extended error codes",
    note: "§8.3 and §8.5, including the frozen 0.1 snake_case spellings so a 0.2 consumer can still read a 0.1 peer's error response.",
  },

  {
    id: "transport-seam",
    group: "Transports",
    label: "Transport seam",
    note: "The §4.8.1 identity-precedence contract a binding plugs into. Present in every library — it is what makes a binding writable, in this repo or outside it.",
  },
  {
    id: "binding:tsp",
    group: "Transports",
    binding: "tsp",
    label: "TSP",
    note: "The preferred binding. HPKE authenticated encryption binds the sender's VID, which maps directly onto the framework's transport-authenticated identity.",
  },
  {
    id: "binding:https",
    group: "Transports",
    binding: "https",
    label: "HTTPS",
    note: "Typed server and client over HTTP, running the full §7.2 pipeline per request.",
  },
  {
    id: "binding:didcomm",
    group: "Transports",
    binding: "didcomm",
    label: "DIDComm v2",
    note: "pack/unpack over authcrypt'd JWEs; the verified sender_kid becomes the transport-authenticated peer.",
  },
  {
    id: "binding:didcomm-v1",
    group: "Transports",
    binding: "didcomm-v1",
    label: "DIDComm v1",
    note: "Carriage over the legacy DIDComm v1 envelope, for peers that have not moved to v2.",
  },
  {
    id: "binding:push",
    group: "Transports",
    binding: "push",
    label: "Push",
    note: "Carriage over a push-notification channel.",
  },

  {
    id: "proof",
    group: "Proofs",
    label: "Data Integrity verifier",
    note: "A concrete W3C Data Integrity backend behind the framework's ProofVerifier seam. Where a library ships none, the seam is still there and you supply the verifier — which is deliberate: the cryptosuite is the consumer's choice.",
  },
  {
    id: "capability-client",
    group: "Proofs",
    label: "Capability wire client",
    note: "Shared document builders and reply parsing for the governance/capability/* and git-trust/* families.",
  },
];

/* The libraries. `capabilities` lists the rows above that this library
   satisfies; `transports` names the package that provides each binding row, so
   the matrix can link a tick to the thing that implements it. */
window.TT_LIBRARIES = [
  {
    id: "rust",
    name: "trust-tasks-rs",
    language: "Rust",
    accent: "violet",
    dir: "trust-tasks-rs",
    registry: "crates.io",
    packageUrl: "https://crates.io/crates/trust-tasks-rs",
    docsUrl: "https://docs.rs/trust-tasks-rs",
    install: "cargo add trust-tasks --features https,proof-affinidi",
    tagline: "The reference implementation, and the only one with DIDComm v1 and the capability client.",
    summary:
      "A workspace of nine published crates: the core library and its generated payload types, four transport bindings, a W3C Data Integrity proof backend, and a shared capability wire client. The trust-tasks facade re-exports the rest behind Cargo features, so you pick a transport rather than a set of version numbers.",
    detail: "rust",
    capabilities: [
      "types", "pipeline", "schemas", "freshness", "replay", "errors",
      "transport-seam",
      "binding:tsp", "binding:https", "binding:didcomm", "binding:didcomm-v1",
      "proof", "capability-client",
    ],
    transports: {
      "binding:tsp": { package: "trust-tasks-tsp", dir: "trust-tasks-tsp" },
      "binding:https": { package: "trust-tasks-https", dir: "trust-tasks-https" },
      "binding:didcomm": { package: "trust-tasks-didcomm", dir: "trust-tasks-didcomm" },
      "binding:didcomm-v1": { package: "trust-tasks-didcomm-v1", dir: "trust-tasks-didcomm-v1" },
      proof: { package: "trust-tasks-proof", dir: "trust-tasks-proof" },
      "capability-client": { package: "trust-tasks-capability-client", dir: "trust-tasks-capability-client" },
    },
  },
  {
    id: "typescript",
    name: "@openvtc/trust-tasks",
    language: "TypeScript",
    accent: "teal",
    dir: "trust-tasks-ts",
    registry: "npm",
    packageUrl: "https://www.npmjs.com/package/@openvtc/trust-tasks",
    install: "npm install @openvtc/trust-tasks",
    tagline: "Types, the §7.2 pipeline, a TSP binding and a Data Integrity proof backend, for Node, browsers and workers.",
    summary:
      "Generated types for every specification plus the hand-written §7.2 consumer pipeline. The core has zero runtime dependencies — the cryptosuite and the JSON Schema engine are interfaces you supply, which is what keeps it usable in a browser or a worker. Two sibling packages build on it and interoperate with their Rust counterparts: @openvtc/trust-tasks-proof puts a W3C Data Integrity verifier and signer — rolled on @noble, byte-compatible with the Rust proof library — behind the ProofVerifier seam, and @openvtc/trust-tasks-tsp is the TSP binding on @openvtc/vti-tsp-js, sealing a document into a Trust Spanning Protocol message and running the pipeline over it with the duplicate-execution guard a mediated transport needs on by default.",
    capabilities: [
      "types", "pipeline", "schemas", "freshness", "replay", "errors",
      "transport-seam",
      "binding:tsp",
      "proof", "capability-client",
    ],
    transports: {
      "binding:tsp": { package: "@openvtc/trust-tasks-tsp", dir: "trust-tasks-ts-tsp" },
      proof: { package: "@openvtc/trust-tasks-proof", dir: "trust-tasks-ts-proof" },
      "capability-client": { package: "@openvtc/trust-tasks-capability-client", dir: "trust-tasks-ts-capability-client" },
    },
    foundations: {
      "binding:https": { package: "fetch (built in)", url: "https://developer.mozilla.org/docs/Web/API/Fetch_API" },
      "binding:didcomm": { package: "didcomm", url: "https://www.npmjs.com/package/didcomm" },
    },
  },
  {
    id: "go",
    name: "trust-tasks-go",
    language: "Go",
    accent: "sky",
    dir: "trust-tasks-go",
    registry: "Go module proxy",
    packageUrl: "https://pkg.go.dev/github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go",
    install: "go get github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go",
    tagline: "Types, the §7.2 pipeline, TSP and DIDComm v2 transport bindings, and a Data Integrity proof backend.",
    summary:
      "One package per specification, each self-contained, plus the §7.2 pipeline in the trusttasks package — with no dependencies beyond the standard library. Optional members are pointers throughout, including slices, so a member that is present and empty stays distinguishable from one that is absent. Three separate nested modules keep the core dependency-free: trust-tasks-go/tsp seals a document into a ToIP Trust Spanning Protocol message; trust-tasks-go/didcomm carries one in a DIDComm v2.1 authcrypt envelope (ECDH-1PU key agreement rolled on the standard library's crypto/ecdh); and trust-tasks-go/proof puts a W3C Data Integrity verifier and signer behind the ProofVerifier seam — the eddsa-jcs-2022 and ecdsa-jcs-2019 cryptosuites, also on the standard library. A fourth nested module, trust-tasks-go/capabilityclient, builds and parses the governance/capability/* and git-trust/* documents (no crypto). Each interoperates with its Rust counterpart.",
    detail: "go",
    capabilities: [
      "types", "pipeline", "schemas", "freshness", "replay", "errors",
      "transport-seam",
      "binding:tsp", "binding:didcomm",
      "proof", "capability-client",
    ],
    transports: {
      // Separate nested modules, so the core stays dependency-free.
      "binding:tsp": { package: "trust-tasks-go/tsp", dir: "trust-tasks-go/tsp" },
      "binding:didcomm": { package: "trust-tasks-go/didcomm", dir: "trust-tasks-go/didcomm" },
      proof: { package: "trust-tasks-go/proof", dir: "trust-tasks-go/proof" },
      "capability-client": { package: "trust-tasks-go/capabilityclient", dir: "trust-tasks-go/capabilityclient" },
    },
    foundations: {
      "binding:https": { package: "net/http (standard library)", url: "https://pkg.go.dev/net/http" },
    },
  },
  {
    id: "dart",
    name: "trust_tasks",
    language: "Dart",
    accent: "coral",
    dir: "trust-tasks-dart",
    registry: "pub.dev",
    packageUrl: "https://pub.dev/packages/trust_tasks",
    docsUrl: "https://pub.dev/documentation/trust_tasks/latest/",
    install: "dart pub add trust_tasks",
    detail: "dart",
    tagline: "Types and the §7.2 pipeline, for Dart and Flutter.",
    summary:
      "One library per specification plus the §7.2 pipeline, with no dependencies. Outcomes are a sealed hierarchy, so a switch over them is exhaustive; closed value sets are extension types rather than enums, so a value from a newer MINOR is carried through instead of throwing (§5.2). Two packages build on it and interoperate with their Rust counterparts: trust_tasks_proof puts a Data Integrity verifier and signer built on Affinidi's ssi behind the ProofVerifier seam, trust_tasks_https is the HTTPS binding — a client that runs on the web and a server that runs the §7.2 pipeline per request — and trust_tasks_didcomm is the DIDComm v2.1 binding on Affinidi's didcomm, with the duplicate-execution guard a mediated transport needs on by default.",
    capabilities: [
      "types", "pipeline", "schemas", "freshness", "replay", "errors",
      "transport-seam",
      "binding:https", "binding:didcomm",
      "proof",
    ],
    transports: {
      "binding:https": { package: "trust_tasks_https", dir: "trust-tasks-dart-https" },
      "binding:didcomm": { package: "trust_tasks_didcomm", dir: "trust-tasks-dart-didcomm" },
      proof: { package: "trust_tasks_proof", dir: "trust-tasks-dart-proof" },
    },
    // trust_tasks_tsp is built and tested (bindings/tsp) but not on pub.dev yet
    // — affinidi_tsp, the library it seals with, is not published — so it is a
    // foundation, not a shipped cell. The named evidence is the repo.
    foundations: {
      "binding:tsp": { package: "affinidi-tsp-dart", url: "https://github.com/affinidi/affinidi-tsp-dart" },
    },
  },
];
