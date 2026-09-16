/* ============================================================
   Ecosystem projects — verifiable-trust stack
   ============================================================
   Add new projects here. Tier values:
     "core"      — foundational specifications & implementations
     "reference" — reference implementations of the core
     "tooling"   — end-user / community tooling
     "adjacent"  — related projects in the broader ecosystem
*/
window.TT_ECOSYSTEM = [
  {
    id: "trust-tasks",
    name: "Trust Tasks",
    tagline: "The vocabulary.",
    role: "Specifications",
    tier: "core",
    accent: "violet",
    self: true,
    summary:
      "JSON-based, transport-agnostic specifications for the verifiable work that happens between two parties — KYC handoffs, consent receipts, payment commitments, credential issuance. Each task is a typed, major.minor-versioned reference (https://trusttasks.org/spec/<task>/<major.minor>) that any implementation can target.",
    bullets: [
      "Self-contained, transport-agnostic, JSON-based",
      "Major.minor versioned type URIs",
      "Public review under Trust Over IP DTGWG",
    ],
    primary: { label: "trusttasks.org", href: "https://trusttasks.org" },
    repo: "https://github.com/trustoverip/dtgwg-trust-tasks-tf",
  },
  {
    id: "trust-tasks-libraries",
    name: "Trust Tasks client libraries",
    tagline: "The reference implementations.",
    role: "Reference implementations · Rust, TypeScript, Go, Dart",
    tier: "reference",
    accent: "violet",
    summary:
      "Four reference client libraries, each generated from the same registry and published to its own language's package registry. All four carry a typed payload for every specification and implement the SPEC §7.2 consumer pipeline; a cross-language conformance check holds them to the same verdict on the same document. Transport bindings ship in Rust (TSP, HTTPS, DIDComm v2 and v1) and Dart (HTTPS, DIDComm v2), each with a Data Integrity proof backend, and Go ships a TSP binding as a separate module; the implementations interoperate across languages.",
    // No version numbers: the previous entry said "Tracking framework 0.2
    // (0.2.x crate line)" while the workspace was at 0.21. Each registry states
    // its own current version, and the capability matrix states what each
    // library can do — both are checked against the tree by
    // `checkLibraryRegistry()` in scripts/build-registry.mjs.
    bullets: [
      "trust-tasks-rs — crates.io · the workspace with the transport bindings",
      "@openvtc/trust-tasks — npm · types + §7.2 pipeline, zero dependencies",
      "trust-tasks-go — Go module proxy · one package per specification",
      "trust_tasks — pub.dev · Dart and Flutter, with trust_tasks_https, trust_tasks_didcomm and trust_tasks_proof beside it",
      "See the capability matrix for which library supports which transport",
    ],
    primary: { label: "Implementations · trusttasks.org", href: "/implementations" },
    repo: "https://github.com/trustoverip/dtgwg-trust-tasks-tf",
  },
  {
    id: "dtg-credentials",
    name: "Decentralized Trust Graph (DTG) Credentials",
    tagline: "The credential types.",
    role: "Credentials · reference implementation",
    tier: "core",
    accent: "teal",
    summary:
      "The credential types that connect Trust Task participants into a graph of verifiable relationships. Implements the DTGWG credential specifications — Membership (VMC), Relationship (VRC), Persona (VPC), Endorsement (VEC), Witness (VWC), Invitation (VIC) — built on W3C VC 1.1 and 2.0. A Rust library for creating, signing, and verifying these credentials.",
    bullets: [
      "Six credential types from the DTGWG spec",
      "W3C VC 1.1 / 2.0 conformant",
      "Rust library; pluggable signing & verification",
    ],
    primary: { label: "github.com/OpenVTC/dtg-credentials", href: "https://github.com/OpenVTC/dtg-credentials" },
    spec: { label: "DTGWG cred-tf spec", href: "https://github.com/trustoverip/dtgwg-cred-tf" },
  },
  {
    id: "vti",
    name: "Verifiable Trust Infrastructure (VTI)",
    tagline: "The runtime.",
    role: "Infrastructure · reference implementation",
    tier: "core",
    accent: "coral",
    summary:
      "The runtime that holds keys, DIDs, and access-control policies on behalf of a community. The Verifiable Trust Agent (VTA) is an always-on service — runnable locally or inside an AWS Nitro Enclave — that mints DIDs from declarative templates, issues authorization credentials, and brokers DIDComm protocol management. Ships with a sealed-transfer wire format (HPKE + ASCII armor) and Personal/Community Network Manager CLIs (PNM, CNM).",
    bullets: [
      "VTA service: keys, DIDs, ACLs",
      "Local or AWS Nitro Enclave deployment",
      "Sealed-transfer (HPKE-AEAD + ASCII armor)",
      "Declarative DID templates",
    ],
    primary: { label: "github.com/OpenVTC/verifiable-trust-infrastructure", href: "https://github.com/OpenVTC/verifiable-trust-infrastructure" },
  },
  {
    id: "openvtc",
    name: "OpenVTC",
    tagline: "The community tool.",
    role: "Community tooling · reference implementation",
    tier: "tooling",
    accent: "amber",
    summary:
      "The end-user CLI for participating in a Verifiable Trust Community. Establishes first-person trust between developers via Personhood Credentials (PHCs) and Verifiable Relationship Credentials (VRCs), backed by the First Person Project. Uses did:webvh for portable, self-certifying identity, and includes did-git-sign for signing git commits with your DID.",
    bullets: [
      "did:webvh persona DIDs",
      "PHCs and VRCs over DIDComm",
      "did-git-sign for git commit signing",
      "Hardware-token (OpenPGP card) support",
    ],
    primary: { label: "github.com/OpenVTC/openvtc", href: "https://github.com/OpenVTC/openvtc" },
    spec: { label: "First Person Project white paper", href: "https://www.firstperson.network/white-paper" },
  },
];
