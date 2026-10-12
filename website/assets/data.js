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
    id: "automation",
    name: "Automation",
    color: "sky",
    blurb: "Scheduled and event-triggered execution with no human in the loop — recurring jobs, one-shot timers, trigger registration, and the run history that shows what fired and when (scheduler/*).",
    icon: "zap"
  },
  {
    id: "chat",
    name: "Chat",
    color: "teal",
    blurb: "Conversational messaging between AI agents and messaging-platform bridges — author-signed, hash-linked messages forming a verifiable per-conversation chain for audit and dispute resolution (chat/*).",
    icon: "message"
  },
  {
    id: "collaboration",
    name: "Collaboration",
    color: "violet",
    blurb: "Concurrent multi-party editing of shared state — a canvas or document several parties mutate at the same time, with per-party attribution and an explicit conflict-resolution rule (canvas/*).",
    icon: "users"
  },
  {
    id: "communications",
    name: "Communications",
    color: "coral",
    blurb: "Real-time synchronous session setup and teardown — offer/answer negotiation, participant management, and call lifecycle. Distinct from the asynchronous message exchange of chat/* and from the transport administration of messaging/* (call/*).",
    icon: "phone"
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
    id: "key-management",
    name: "Key Management",
    color: "navy",
    blurb: "Custody and lifecycle of cryptographic key material held on a producer's behalf — create, import, list, rename, revoke, and the signing-oracle surface that exercises a key without ever exporting it (keys/*).",
    icon: "key"
  },
  {
    id: "matchmaking",
    name: "Matchmaking",
    color: "coral",
    blurb: "Mutual-interest discovery and introduction — a party declares what it seeks and an introduction is made only when the interest is reciprocal, so neither side learns of an interest the other did not return (matchmaking/*).",
    icon: "handshake"
  },
  {
    id: "media",
    name: "Media",
    color: "violet",
    blurb: "Custody and delivery of recorded audio, video, image, and book assets — ingest, transcode, streaming release, and playback position, held on behalf of the party that owns the asset (media/*).",
    icon: "film"
  },
  {
    id: "messaging",
    name: "Messaging",
    color: "teal",
    blurb: "Messaging-infrastructure operation of a mediator or relay — account and ACL administration, queue-limit management, and liveness/capability ping. Operator-facing control of the message-transport substrate itself, distinct from the application-level access-control of acl/* (messaging/*).",
    icon: "server"
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
    id: "prediction",
    name: "Prediction",
    color: "amber",
    blurb: "A claim about the future together with the accuracy record that later settles it. Reputation scores the past and provenance records it; neither can hold the claim half (forecast/*).",
    icon: "trending-up"
  },
  {
    id: "privacy",
    name: "Privacy",
    color: "teal",
    blurb: "Data-subject rights exercised against a controller — access, rectification, erasure, portability, and objection. Distinct from consent, which covers a subject granting permission rather than compelling a controller who would rather not comply (data-subject/*).",
    icon: "eye-off"
  },
  {
    id: "provenance",
    name: "Provenance",
    color: "violet",
    blurb: "Evidence of how an artifact or a flow came to be \u2014 process attestation bound to a Verifier-supplied challenge, ceremony receipts, and other records asserting that something was produced the way it is claimed to have been.",
    icon: "anchor"
  },
  {
    id: "reputation",
    name: "Reputation",
    color: "sky",
    blurb: "Reputation, attestation, and trust-score tasks. (Reserved; no specs yet.)",
    icon: "star"
  },
  {
    id: "storage",
    name: "Storage",
    color: "navy",
    blurb: "Document custody on behalf of another party — upload, listing, retrieval, and deletion of files a producer holds for a consumer. Distinct from governance, which decides who may reach them (files/*).",
    icon: "folder"
  },
  {
    id: "task-management",
    name: "Task Management",
    color: "amber",
    blurb: "Human work tracking and assignment — creating, assigning, transitioning, and closing items of work whose completion is judged by a person rather than by a machine (todo/*).",
    icon: "check-square"
  },
  {
    id: "temporal-coordination",
    name: "Temporal Coordination",
    color: "teal",
    blurb: "Negotiating when an interaction happens, between parties — availability exchange, proposal and counter-proposal, confirmation, and rescheduling of a shared point in time (calendar/*).",
    icon: "calendar"
  }
];
