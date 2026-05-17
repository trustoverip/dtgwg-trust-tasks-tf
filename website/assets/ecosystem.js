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
    id: "trust-tasks-rs",
    name: "trust-tasks (Rust)",
    tagline: "The reference implementation.",
    role: "Reference implementation · Rust",
    tier: "reference",
    accent: "violet",
    summary:
      "A Rust workspace that turns the Trust Tasks framework into runnable code: framework primitives (envelope, TypeUri, Proof, RejectReason, TransportHandler), two transport bindings (HTTPS, DIDComm v2.1), a ProofVerifier implementation backed by Affinidi Data Integrity, and a codegen tool that materialises typed payload modules for every spec in the registry.",
    bullets: [
      "trust-tasks-rs — core library + generated specs::*",
      "trust-tasks-https — typed axum server + reqwest client",
      "trust-tasks-didcomm — pack/unpack over DIDComm v2.1",
      "trust-tasks-proof-affinidi — W3C Data Integrity verifier",
      "Pre-publication 0.1.0, tracking SPEC.md 0.1",
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
