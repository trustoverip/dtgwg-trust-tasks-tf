/* ============================================================
   Trust Tasks — Taxonomy
   ------------------------------------------------------------
   This file holds the hand-edited taxonomy (categories).
   Trust Task entries (window.TT_TASKS, window.TT_STATS) are
   generated from specs/<slug>/<version>/ by
   scripts/build-registry.mjs and written to
   tasks.generated.js, which is loaded after this file in
   index.html.
   ============================================================ */

window.TT_CATEGORIES = [
  {
    id: "identity",
    name: "Identity",
    color: "coral",
    blurb: "Establishing, proving, and binding identifiers between entities.",
    icon: "id"
  },
  {
    id: "credentials",
    name: "Credentials",
    color: "teal",
    blurb: "Issuing, presenting, and verifying portable claims.",
    icon: "credential"
  },
  {
    id: "data-exchange",
    name: "Data Exchange",
    color: "violet",
    blurb: "Structured request/response patterns for sharing data with consent.",
    icon: "exchange"
  },
  {
    id: "governance",
    name: "Governance",
    color: "amber",
    blurb: "Trust registry queries, policy checks, and accreditation.",
    icon: "scale"
  },
  {
    id: "payments",
    name: "Payments",
    color: "sky",
    blurb: "Settlement, escrow, and value-transfer commitments.",
    icon: "coin"
  },
  {
    id: "ai-agents",
    name: "AI Agents",
    color: "navy",
    blurb: "Agent-to-agent commitments, delegation, and authorization.",
    icon: "agent"
  },
  {
    id: "reputation",
    name: "Reputation",
    color: "coral",
    blurb: "Endorsements, attestations, and verifiable history.",
    icon: "star"
  }
];
