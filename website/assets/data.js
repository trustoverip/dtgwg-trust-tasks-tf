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
    id: "tt-0001-kyc-handoff",
    slug: "kyc-handoff",
    number: "TT-0001",
    title: "KYC Handoff",
    summary: "A counterparty proves it has performed Know-Your-Customer verification on a subject and conveys the result to a relying party.",
    category: "identity",
    keywords: ["kyc", "aml", "onboarding", "compliance", "subject", "verification"],
    status: "standard",
    version: "1.0.0",
    updated: "2026-03-14",
    authors: ["DTGWG Identity Subgroup"],
    parties: ["Verifier (KYC provider)", "Relying party"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/schemas/kyc-handoff/1.0.0.json",
      "title": "KYC Handoff Trust Task",
      "type": "object",
      "required": ["taskId", "subject", "result", "evidence", "issuedAt"],
      "properties": {
        "taskId":   { "type": "string", "format": "uri" },
        "subject":  { "type": "string", "description": "DID of the natural or legal person verified." },
        "result":   { "enum": ["passed", "failed", "review"] },
        "level":    { "enum": ["LOA1", "LOA2", "LOA3"] },
        "evidence": { "type": "array", "items": { "type": "string", "format": "uri" } },
        "issuedAt": { "type": "string", "format": "date-time" },
        "expiresAt":{ "type": "string", "format": "date-time" }
      }
    },
    example: {
      "taskId": "did:web:trusttasks.org#tt-0001-kyc-handoff/4f3c",
      "subject": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSdoVdb...",
      "result": "passed",
      "level": "LOA2",
      "evidence": ["did:web:kyc-provider.example#receipt/9a2c"],
      "issuedAt": "2026-04-12T09:31:00Z",
      "expiresAt": "2027-04-12T09:31:00Z"
    },
    related: ["tt-0004-credential-issuance", "tt-0005-trust-registry-query"]
  },
  {
    id: "tt-0002-consent-receipt",
    slug: "consent-receipt",
    number: "TT-0002",
    title: "Consent Receipt",
    summary: "A subject grants scoped, time-bound consent for a processor to use specific data, with a portable, revocable receipt.",
    category: "data-exchange",
    keywords: ["consent", "gdpr", "privacy", "purpose", "receipt", "data sharing"],
    status: "candidate",
    version: "0.9.2",
    updated: "2026-04-22",
    authors: ["DTGWG Data Exchange TF"],
    parties: ["Subject", "Data processor"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/schemas/consent-receipt/0.9.2.json",
      "title": "Consent Receipt Trust Task",
      "type": "object",
      "required": ["taskId", "subject", "processor", "scope", "purpose", "grantedAt"],
      "properties": {
        "taskId":    { "type": "string", "format": "uri" },
        "subject":   { "type": "string" },
        "processor": { "type": "string" },
        "scope":     { "type": "array", "items": { "type": "string" } },
        "purpose":   { "type": "string" },
        "grantedAt": { "type": "string", "format": "date-time" },
        "expiresAt": { "type": "string", "format": "date-time" },
        "revocation":{ "type": "string", "format": "uri" }
      }
    },
    example: {
      "taskId": "did:web:trusttasks.org#tt-0002-consent-receipt/8b1a",
      "subject": "did:key:z6Mki...A1",
      "processor": "did:web:analytics.example",
      "scope": ["profile.email", "profile.country"],
      "purpose": "Onboarding analytics, 30 days",
      "grantedAt": "2026-04-22T15:04:00Z",
      "expiresAt": "2026-05-22T15:04:00Z",
      "revocation": "https://analytics.example/consent/8b1a/revoke"
    },
    related: ["tt-0006-agent-authorization"]
  },
  {
    id: "tt-0003-payment-commitment",
    slug: "payment-commitment",
    number: "TT-0003",
    title: "Payment Commitment",
    summary: "Two parties commit to a payment with conditions of release, settlement rail, and tolerance window — settlement happens off-protocol.",
    category: "payments",
    keywords: ["payment", "escrow", "settlement", "commitment", "iso20022", "rail"],
    status: "draft",
    version: "0.4.0",
    updated: "2026-04-29",
    authors: ["DTGWG Payments TF"],
    parties: ["Payer", "Payee", "Settlement agent (optional)"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/schemas/payment-commitment/0.4.0.json",
      "title": "Payment Commitment Trust Task",
      "type": "object",
      "required": ["taskId", "payer", "payee", "amount", "currency", "rail", "conditions"],
      "properties": {
        "taskId":   { "type": "string", "format": "uri" },
        "payer":    { "type": "string" },
        "payee":    { "type": "string" },
        "amount":   { "type": "string", "pattern": "^[0-9]+(\\.[0-9]+)?$" },
        "currency": { "type": "string", "pattern": "^[A-Z]{3}$" },
        "rail":     { "enum": ["sepa-instant", "fednow", "swift", "onchain", "internal"] },
        "conditions": { "type": "array", "items": { "type": "string" } },
        "tolerance":  { "type": "string", "description": "ISO 8601 duration" }
      }
    },
    example: {
      "taskId": "did:web:trusttasks.org#tt-0003-payment-commitment/1d04",
      "payer": "did:web:payer.example",
      "payee": "did:web:payee.example",
      "amount": "12500.00",
      "currency": "EUR",
      "rail": "sepa-instant",
      "conditions": ["tt-0004-credential-issuance.completed"],
      "tolerance": "PT72H"
    },
    related: ["tt-0004-credential-issuance"]
  },
  {
    id: "tt-0004-credential-issuance",
    slug: "credential-issuance",
    number: "TT-0004",
    title: "Credential Issuance",
    summary: "An issuer commits to producing a verifiable credential for a holder against a published schema, with status notification on completion.",
    category: "credentials",
    keywords: ["credential", "vc", "issuance", "issuer", "holder", "schema"],
    status: "standard",
    version: "1.1.0",
    updated: "2026-02-18",
    authors: ["DTGWG Credentials TF"],
    parties: ["Issuer", "Holder"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/schemas/credential-issuance/1.1.0.json",
      "title": "Credential Issuance Trust Task",
      "type": "object",
      "required": ["taskId", "issuer", "holder", "credentialSchema", "format"],
      "properties": {
        "taskId":   { "type": "string", "format": "uri" },
        "issuer":   { "type": "string" },
        "holder":   { "type": "string" },
        "credentialSchema": { "type": "string", "format": "uri" },
        "format":   { "enum": ["vc-jwt", "vc-jose-cose", "sd-jwt-vc", "mdoc"] },
        "claims":   { "type": "object", "additionalProperties": true },
        "deliveryEndpoint": { "type": "string", "format": "uri" }
      }
    },
    example: {
      "taskId": "did:web:trusttasks.org#tt-0004-credential-issuance/c0a1",
      "issuer": "did:web:university.example",
      "holder": "did:key:z6Mki...A1",
      "credentialSchema": "https://schemas.example/diploma/v2.json",
      "format": "sd-jwt-vc",
      "claims": { "degree": "BSc Computer Science", "issuedYear": 2026 },
      "deliveryEndpoint": "https://wallet.example/inbox/c0a1"
    },
    related: ["tt-0001-kyc-handoff", "tt-0005-trust-registry-query"]
  },
  {
    id: "tt-0005-trust-registry-query",
    slug: "trust-registry-query",
    number: "TT-0005",
    title: "Trust Registry Query",
    summary: "A relying party asks a trust registry whether a given entity is authorized to perform a given role under a governance framework.",
    category: "governance",
    keywords: ["registry", "trqp", "authorization", "ecosystem", "governance", "egf"],
    status: "candidate",
    version: "0.7.1",
    updated: "2026-04-02",
    authors: ["DTGWG Governance TF"],
    parties: ["Relying party", "Trust registry operator"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/schemas/trust-registry-query/0.7.1.json",
      "title": "Trust Registry Query Trust Task",
      "type": "object",
      "required": ["taskId", "registry", "entity", "role", "framework"],
      "properties": {
        "taskId":   { "type": "string", "format": "uri" },
        "registry": { "type": "string", "format": "uri" },
        "entity":   { "type": "string" },
        "role":     { "type": "string" },
        "framework":{ "type": "string", "format": "uri" },
        "asOf":     { "type": "string", "format": "date-time" }
      }
    },
    example: {
      "taskId": "did:web:trusttasks.org#tt-0005-trust-registry-query/77ee",
      "registry": "https://registry.example/trqp",
      "entity": "did:web:university.example",
      "role": "credential-issuer",
      "framework": "https://egf.example/education/v3",
      "asOf": "2026-04-02T08:00:00Z"
    },
    related: ["tt-0004-credential-issuance"]
  },
  {
    id: "tt-0006-agent-authorization",
    slug: "agent-authorization",
    number: "TT-0006",
    title: "Agent Authorization",
    summary: "A principal delegates a scoped authority to an AI agent for a finite window, with revocation and audit-log commitments.",
    category: "ai-agents",
    keywords: ["agent", "delegation", "authority", "scope", "ai", "principal"],
    status: "draft",
    version: "0.3.0",
    updated: "2026-05-01",
    authors: ["DTGWG AI Agents TF"],
    parties: ["Principal", "Agent"],
    schema: {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "https://trusttasks.org/schemas/agent-authorization/0.3.0.json",
      "title": "Agent Authorization Trust Task",
      "type": "object",
      "required": ["taskId", "principal", "agent", "scope", "expiresAt"],
      "properties": {
        "taskId":    { "type": "string", "format": "uri" },
        "principal": { "type": "string" },
        "agent":     { "type": "string" },
        "scope":     { "type": "array", "items": { "type": "string" } },
        "expiresAt": { "type": "string", "format": "date-time" },
        "auditLog":  { "type": "string", "format": "uri" },
        "revocation":{ "type": "string", "format": "uri" }
      }
    },
    example: {
      "taskId": "did:web:trusttasks.org#tt-0006-agent-authorization/aa7b",
      "principal": "did:key:z6Mki...A1",
      "agent": "did:web:assistant.example",
      "scope": ["calendar:read", "email:send-on-behalf:colleagues"],
      "expiresAt": "2026-06-01T00:00:00Z",
      "auditLog": "https://principal.example/audit/aa7b",
      "revocation": "https://principal.example/agent/aa7b/revoke"
    },
    related: ["tt-0002-consent-receipt"]
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
