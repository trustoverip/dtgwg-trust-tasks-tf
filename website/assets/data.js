/* ============================================================
   Trust Tasks — Seed registry data + taxonomy
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

window.TT_TASKS = [
  {
    id: "kyc-handoff",
    slug: "kyc-handoff",
    title: "KYC Handoff",
    summary: "A counterparty proves it has performed Know-Your-Customer verification on a subject and conveys the result to a relying party.",
    category: "identity",
    keywords: ["kyc", "aml", "onboarding", "compliance", "subject", "verification"],
    status: "standard",
    version: "1.0",
    updated: "2026-03-14",
    authors: ["DTGWG Identity Subgroup"],
    parties: ["Verifier (KYC provider)", "Relying party"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/spec/kyc-handoff/1.0",
      "title": "KYC Handoff — payload",
      "type": "object",
      "required": ["subject", "result"],
      "properties": {
        "subject":  { "type": "string", "description": "DID of the natural or legal person verified." },
        "result":   { "enum": ["passed", "failed", "review"] },
        "level":    { "enum": ["LOA1", "LOA2", "LOA3"] },
        "evidence": { "type": "array", "items": { "type": "string", "format": "uri" } }
      }
    },
    related: ["credential-issuance", "trust-registry-query"]
  },
  {
    id: "consent-receipt",
    slug: "consent-receipt",
    title: "Consent Receipt",
    summary: "A subject grants scoped, time-bound consent for a processor to use specific data, with a portable, revocable receipt.",
    category: "data-exchange",
    keywords: ["consent", "gdpr", "privacy", "purpose", "receipt", "data sharing"],
    status: "candidate",
    version: "0.9",
    updated: "2026-04-22",
    authors: ["DTGWG Data Exchange TF"],
    parties: ["Subject", "Data processor"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/spec/consent-receipt/0.9",
      "title": "Consent Receipt — payload",
      "type": "object",
      "required": ["subject", "processor", "scope", "purpose", "grantedAt"],
      "properties": {
        "subject":   { "type": "string" },
        "processor": { "type": "string" },
        "scope":     { "type": "array", "items": { "type": "string" } },
        "purpose":   { "type": "string" },
        "grantedAt": { "type": "string", "format": "date-time" },
        "revocation":{ "type": "string", "format": "uri" }
      }
    },
    related: ["agent-authorization"]
  },
  {
    id: "payment-commitment",
    slug: "payment-commitment",
    title: "Payment Commitment",
    summary: "Two parties commit to a payment with conditions of release, settlement rail, and tolerance window — settlement happens off-protocol.",
    category: "payments",
    keywords: ["payment", "escrow", "settlement", "commitment", "iso20022", "rail"],
    status: "draft",
    version: "0.4",
    updated: "2026-04-29",
    authors: ["DTGWG Payments TF"],
    parties: ["Payer", "Payee", "Settlement agent (optional)"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/spec/payment-commitment/0.4",
      "title": "Payment Commitment — payload",
      "type": "object",
      "required": ["payer", "payee", "amount", "currency", "rail", "conditions"],
      "properties": {
        "payer":    { "type": "string" },
        "payee":    { "type": "string" },
        "amount":   { "type": "string", "pattern": "^[0-9]+(\\.[0-9]+)?$" },
        "currency": { "type": "string", "pattern": "^[A-Z]{3}$" },
        "rail":     { "enum": ["sepa-instant", "fednow", "swift", "onchain", "internal"] },
        "conditions": { "type": "array", "items": { "type": "string" } },
        "tolerance":  { "type": "string", "description": "ISO 8601 duration" }
      }
    },
    related: ["credential-issuance"]
  },
  {
    id: "credential-issuance",
    slug: "credential-issuance",
    title: "Credential Issuance",
    summary: "An issuer commits to producing a verifiable credential for a holder against a published schema, with status notification on completion.",
    category: "credentials",
    keywords: ["credential", "vc", "issuance", "issuer", "holder", "schema"],
    status: "standard",
    version: "1.1",
    updated: "2026-02-18",
    authors: ["DTGWG Credentials TF"],
    parties: ["Issuer", "Holder"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/spec/credential-issuance/1.1",
      "title": "Credential Issuance — payload",
      "type": "object",
      "required": ["issuer", "holder", "credentialSchema", "format"],
      "properties": {
        "issuer":   { "type": "string" },
        "holder":   { "type": "string" },
        "credentialSchema": { "type": "string", "format": "uri" },
        "format":   { "enum": ["vc-jwt", "vc-jose-cose", "sd-jwt-vc", "mdoc"] },
        "claims":   { "type": "object", "additionalProperties": true },
        "deliveryEndpoint": { "type": "string", "format": "uri" }
      }
    },
    related: ["kyc-handoff", "trust-registry-query"]
  },
  {
    id: "trust-registry-query",
    slug: "trust-registry-query",
    title: "Trust Registry Query",
    summary: "A relying party asks a trust registry whether a given entity is authorized to perform a given role under a governance framework.",
    category: "governance",
    keywords: ["registry", "trqp", "authorization", "ecosystem", "governance", "egf"],
    status: "candidate",
    version: "0.7",
    updated: "2026-04-02",
    authors: ["DTGWG Governance TF"],
    parties: ["Relying party", "Trust registry operator"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/spec/trust-registry-query/0.7",
      "title": "Trust Registry Query — payload",
      "type": "object",
      "required": ["registry", "entity", "role", "framework"],
      "properties": {
        "registry": { "type": "string", "format": "uri" },
        "entity":   { "type": "string" },
        "role":     { "type": "string" },
        "framework":{ "type": "string", "format": "uri" },
        "asOf":     { "type": "string", "format": "date-time" }
      }
    },
    related: ["credential-issuance"]
  },
  {
    id: "agent-authorization",
    slug: "agent-authorization",
    title: "Agent Authorization",
    summary: "A principal delegates a scoped authority to an AI agent for a finite window, with revocation and audit-log commitments.",
    category: "ai-agents",
    keywords: ["agent", "delegation", "authority", "scope", "ai", "principal"],
    status: "draft",
    version: "0.3",
    updated: "2026-05-01",
    authors: ["DTGWG AI Agents TF"],
    parties: ["Principal", "Agent"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/spec/agent-authorization/0.3",
      "title": "Agent Authorization — payload",
      "type": "object",
      "required": ["principal", "agent", "scope"],
      "properties": {
        "principal": { "type": "string" },
        "agent":     { "type": "string" },
        "scope":     { "type": "array", "items": { "type": "string" } },
        "auditLog":  { "type": "string", "format": "uri" },
        "revocation":{ "type": "string", "format": "uri" }
      }
    },
    related: ["consent-receipt"]
  }
];

/* derived counts */
window.TT_STATS = (function () {
  const tasks = window.TT_TASKS;
  const byStatus = tasks.reduce((acc, t) => { acc[t.status] = (acc[t.status] || 0) + 1; return acc; }, {});
  const orgs = new Set();
  tasks.forEach(t => t.authors.forEach(a => orgs.add(a)));
  const latest = tasks.reduce((a, b) => (a.updated > b.updated ? a : b));
  return {
    total: tasks.length,
    byStatus,
    categories: window.TT_CATEGORIES.length,
    orgs: orgs.size,
    latest: latest.updated,
    latestTitle: latest.title
  };
})();
