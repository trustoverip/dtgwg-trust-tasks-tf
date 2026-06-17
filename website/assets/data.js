/* ============================================================
   Trust Tasks — Taxonomy
   ------------------------------------------------------------
   This file holds the hand-edited taxonomy (categories).
   Trust Task entries (window.TT_TASKS, window.TT_STATS) are
   generated from specs/<slug>/<version>/ by
   scripts/build-registry.mjs and written to
   tasks.generated.js, which is loaded after this file in
   index.html.

   IMPORTANT: keep `id` values in sync with the enum in
   `specs/spec.meta.schema.json#/properties/category`. The build
   pipeline validates every spec's `category` against that enum;
   a category listed here without a matching enum value is dead
   weight, and an enum value missing here makes its specs render
   without a name / color in the site (they fall through to the
   `var(--tt-navy)` default in `components.jsx::catColor`).
   ============================================================ */

window.TT_CATEGORIES = [
  {
    id: "access-control",
    name: "Access Control",
    color: "violet",
    blurb: "ACL, role, capability, and permission tasks — granting, revoking, and managing access-control privileges between parties (acl/*).",
    icon: "key"
  },
  {
    id: "ai-agents",
    name: "AI Agents",
    color: "coral",
    blurb: "Tasks specific to AI-agent interaction patterns — capability negotiation, supervised execution, audit-bound delegation.",
    icon: "cpu"
  },
  {
    id: "authentication",
    name: "Authentication",
    color: "coral",
    blurb: "Login, session, challenge/response, passkey, step-up. Covers SIOPv2 self-issued auth, WebAuthn enrollment + login, and approval flows (auth/*, confirm/*).",
    icon: "shield-check"
  },
  {
    id: "chat",
    name: "Chat",
    color: "teal",
    blurb: "Conversational messaging between AI agents and messaging-platform bridges — author-signed, hash-linked messages forming a verifiable per-conversation chain for audit and dispute resolution (chat/*).",
    icon: "message"
  },
  {
    id: "consent",
    name: "Consent",
    color: "amber",
    blurb: "Authorization-to-proceed tasks that gate whether an interaction — a connection, channel, group, or conversation — may reach a protected party such as an AI agent, with operator consent on first contact (consent/*).",
    icon: "shield"
  },
  {
    id: "credentials",
    name: "Credentials",
    color: "amber",
    blurb: "Vault / credential management — store, release, proxy-login, sync. The wallet's password / passkey / SIOP / OAuth surface (vault/*).",
    icon: "wallet"
  },
  {
    id: "data-exchange",
    name: "Data Exchange",
    color: "sky",
    blurb: "Data-transport, sync, and event-stream tasks — push notifications and incremental delta synchronisation between consumers (sync/*).",
    icon: "arrows-exchange"
  },
  {
    id: "did-management",
    name: "DID Management",
    color: "teal",
    blurb: "Lifecycle, hosting, and registry operations for DIDs hosted on a Trust-Tasks-aware service: claim a path, publish a signed log, disable or rotate a DID, manage hosting domains and the server registry.",
    icon: "id-card"
  },
  {
    id: "framework",
    name: "Framework",
    color: "navy",
    blurb: "Framework-defined response and meta types that every Trust Task ecosystem reuses (e.g. trust-task-error, trust-task-discovery).",
    icon: "anchor"
  },
  {
    id: "governance",
    name: "Governance",
    color: "navy",
    blurb: "Policy authoring, rule-engine evaluation, and decision tasks — Rego policy CRUD plus dry-run evaluation against a request context (policy/*).",
    icon: "scale"
  },
  {
    id: "identity",
    name: "Identity",
    color: "sky",
    blurb: "Identity-anchor, device-binding, and persona tasks — registration, heartbeat, disable, and remote wipe of Companion / Service consumers (device/*).",
    icon: "user-circle"
  },
  {
    id: "notifications",
    name: "Notifications",
    color: "amber",
    blurb: "Push wake-up control plane — register a device's push channel with a gateway, provision the VTA-owned trigger allowlist, and request a contentless wake (push/*). The contentless doorbell itself rides the push transport binding.",
    icon: "bell"
  },
  {
    id: "payments",
    name: "Payments",
    color: "amber",
    blurb: "Payment-flow tasks — settlement initiation, invoice exchange, payer/payee identity binding. (Reserved; no specs yet.)",
    icon: "credit-card"
  },
  {
    id: "reputation",
    name: "Reputation",
    color: "sky",
    blurb: "Reputation, attestation, and trust-score tasks. (Reserved; no specs yet.)",
    icon: "star"
  }
];
