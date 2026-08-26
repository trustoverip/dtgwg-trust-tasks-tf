#!/usr/bin/env node
/**
 * Scaffold a new Trust Task specification that passes `npm run validate`
 * unmodified.
 *
 *   npm run new-spec -- <slug> [--version 0.1] [--no-response]
 *                             [--category <id>] [--title "..."] [--summary "..."]
 *                             [--side-effects none|mutating|destructive]
 *                             [--discloses none|metadata|secret] [--acts-as-subject]
 *                             [--force]
 *
 * ── Why this exists ─────────────────────────────────────────────────────────
 *
 * There was no scaffold, so the documented path was "copy the front-matter
 * template from CONTRIBUTING-SPECS.md" — and that template omitted `sideEffects`
 * and `exposure`, both of which the meta-schema lists as `required`. Copying the
 * official template produced a spec that failed the build. So nobody copied it:
 * authors copied a neighbouring spec instead, which works and drags along that
 * neighbour's depth-specific `../../../../SPEC.md` links, its keywords, its error
 * codes and its Security & Privacy paragraph. Both halves of the registry's
 * front-matter monoculture start here.
 *
 * What this emits is therefore deliberately *complete rather than minimal*: every
 * required declaration, an `## Authorization` stub when the declared classes make
 * the task consequential, the full Security & Privacy skeleton the build now
 * lints for, root-relative `/SPEC.md#...` cross-references that do not depend on
 * how deep the slug is, and an example document that validates against the
 * framework envelope schema. The TODOs are the parts only the author can write.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS_DIR = path.join(ROOT, 'specs');
const META_SCHEMA_PATH = path.join(SPECS_DIR, 'spec.meta.schema.json');

const SLUG_RE = /^[a-z][a-z0-9]*(-[a-z0-9]+)*(\/[a-z][a-z0-9]*(-[a-z0-9]+)*)*$/;
const RESERVED_RE = /^trust-(task|ceremony)($|-|\/)/;

/* Category inference. A best guess only — a wrong category is a one-word fix,
 * whereas being made to pick from a 16-item list before you have written a line
 * is the friction this script exists to remove. Keyed on the slug's first
 * segment, following the mapping the meta-schema's own `category` description
 * spells out. */
const CATEGORY_BY_FAMILY = {
  acl: 'access-control',
  auth: 'authentication',
  confirm: 'authentication',
  chat: 'chat',
  consent: 'consent',
  'task-consent': 'consent',
  vault: 'credentials',
  vrc: 'credentials',
  sync: 'data-exchange',
  provision: 'data-exchange',
  'did-management': 'did-management',
  policy: 'governance',
  device: 'identity',
  keys: 'key-management',
  messaging: 'messaging',
  push: 'notifications',
  witness: 'reputation',
  vtc: 'identity',
  vta: 'ai-agents'
};

function usage(msg) {
  if (msg) console.error(`error: ${msg}\n`);
  console.error(`usage: npm run new-spec -- <slug> [options]

  <slug>                     lowercase, hyphen-separated, may be hierarchical (acl/grant)

  --version <MAJOR.MINOR>    default 0.1
  --no-response              fire-and-forget task: no ## Response section and no $defs.Response
  --category <id>            one of the meta-schema's category enum (inferred from the slug when omitted)
  --title <text>             default derived from the slug
  --summary <text>           default a TODO one-liner
  --side-effects <level>     none | mutating | destructive     (default mutating)
  --discloses <level>        none | metadata | secret          (default metadata)
  --acts-as-subject          declare exposure.actsAsSubject: true
  --force                    overwrite an existing version directory
`);
  process.exit(msg ? 1 : 0);
}

function parseArgs(argv) {
  const opts = {
    version: '0.1',
    response: true,
    sideEffects: 'mutating',
    discloses: 'metadata',
    actsAsSubject: false,
    force: false
  };
  const rest = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const next = () => {
      const v = argv[++i];
      if (v === undefined) usage(`${a} needs a value`);
      return v;
    };
    switch (a) {
      case '-h': case '--help': usage(); break;
      case '--version': opts.version = next(); break;
      case '--no-response': opts.response = false; break;
      case '--category': opts.category = next(); break;
      case '--title': opts.title = next(); break;
      case '--summary': opts.summary = next(); break;
      case '--side-effects': opts.sideEffects = next(); break;
      case '--discloses': opts.discloses = next(); break;
      case '--acts-as-subject': opts.actsAsSubject = true; break;
      case '--force': opts.force = true; break;
      default:
        if (a.startsWith('-')) usage(`unknown option ${a}`);
        rest.push(a);
    }
  }
  if (rest.length !== 1) usage(rest.length ? 'expected exactly one slug' : 'a slug is required');
  opts.slug = rest[0].replace(/^\/+|\/+$/g, '');
  return opts;
}

/** Latest framework version on disk, in the three-part form the canonical spec repo requires. */
function latestFrameworkVersion() {
  const dir = path.join(SPECS_DIR, '_framework');
  const versions = fs.existsSync(dir)
    ? fs.readdirSync(dir, { withFileTypes: true })
        .filter((e) => e.isDirectory() && /^\d+\.\d+$/.test(e.name))
        .map((e) => e.name)
        .sort((a, b) => {
          const [am, an] = a.split('.').map(Number), [bm, bn] = b.split('.').map(Number);
          return (am - bm) || (an - bn);
        })
    : [];
  const latest = versions[versions.length - 1] || '0.4';
  return `${latest}.0`;
}

function titleFromSlug(slug) {
  const segs = slug.split('/');
  const words = (s) => s.split('-').map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  return segs.length > 1
    ? `${words(segs[0])} — ${segs.slice(1).map(words).join(' ')}`
    : words(segs[0]);
}

/** SPEC §2: mutating/destructive, secret disclosure, or acting as the subject. */
function isConsequential({ sideEffects, discloses, actsAsSubject }) {
  return sideEffects === 'mutating' || sideEffects === 'destructive' || discloses === 'secret' || actsAsSubject;
}

/* SPEC §7.3 item 8 + checkProofFloor() in build-registry.mjs: a request that is
 * irreversible or acts with the subject's authority, and a response that carries
 * secret material, MUST declare proof REQUIRED. Deriving it here rather than
 * defaulting means the scaffold cannot emit a spec the build rejects. */
function proofFloor({ sideEffects, discloses, actsAsSubject }) {
  const requestRequired = sideEffects === 'destructive' || actsAsSubject;
  const responseRequired = discloses === 'secret';
  if (requestRequired && responseRequired) return { requirement: 'REQUIRED', why: 'both variants' };
  if (requestRequired) return { request: 'REQUIRED', response: 'RECOMMENDED', why: 'request' };
  if (responseRequired) return { request: 'RECOMMENDED', response: 'REQUIRED', why: 'response' };
  return { requirement: 'RECOMMENDED', why: null };
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const { slug } = opts;

  if (!SLUG_RE.test(slug)) usage(`'${slug}' is not a valid slug (SPEC §6.1: lowercase, hyphen-separated segments joined by '/')`);
  if (RESERVED_RE.test(slug)) {
    usage(
      `'${slug}' is in the framework-reserved namespace. SPEC §6.1 reserves every slug matching ` +
      `^trust-(task|ceremony)($|-|/) — both halves, not just trust-task*. Publishing one means ` +
      `documenting it in SPEC.md §6.1 AND adding it to the allowlist at ` +
      `specs/spec.meta.schema.json #/properties/slug/anyOf[1]/enum by hand; this scaffold will not do that for you.`
    );
  }
  if (!/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.test(opts.version)) {
    usage(`'${opts.version}' is not a MAJOR.MINOR version (SPEC §5.1 — no patch level on a spec version)`);
  }
  for (const [flag, value, allowed] of [
    ['--side-effects', opts.sideEffects, ['none', 'mutating', 'destructive']],
    ['--discloses', opts.discloses, ['none', 'metadata', 'secret']]
  ]) {
    if (!allowed.includes(value)) usage(`${flag} must be one of ${allowed.join(' | ')} (got '${value}')`);
  }

  const meta = JSON.parse(fs.readFileSync(META_SCHEMA_PATH, 'utf8'));
  const categories = meta.properties.category.enum;
  const category = opts.category || CATEGORY_BY_FAMILY[slug.split('/')[0]];
  if (!category) {
    usage(
      `could not infer a category from '${slug}'. Pass --category <id>, one of:\n  ` +
      categories.join('\n  ')
    );
  }
  if (!categories.includes(category)) {
    usage(`--category '${category}' is not in the meta-schema enum. Valid values:\n  ` + categories.join('\n  '));
  }

  const dir = path.join(SPECS_DIR, ...slug.split('/'), opts.version);
  if (fs.existsSync(dir) && !opts.force) {
    console.error(`error: ${path.relative(ROOT, dir)} already exists (pass --force to overwrite)`);
    process.exit(1);
  }

  const ctx = {
    ...opts,
    category,
    title: opts.title || titleFromSlug(slug),
    summary: opts.summary || `TODO: one sentence saying what ${slug} achieves and for whom.`,
    targetFrameworkVersion: latestFrameworkVersion(),
    typeUri: `https://trusttasks.org/spec/${slug}/${opts.version}`,
    consequential: isConsequential(opts),
    proof: proofFloor(opts)
  };

  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, 'spec.md'), renderSpecMd(ctx));
  fs.writeFileSync(path.join(dir, 'payload.schema.json'), renderPayloadSchema(ctx));

  const rel = path.relative(ROOT, dir);
  console.log(`Scaffolded ${rel}/`);
  console.log(`  spec.md              ${ctx.consequential ? 'with' : 'without'} an ## Authorization section (task is ${ctx.consequential ? '' : 'not '}consequential)`);
  console.log(`  payload.schema.json  ${ctx.response ? 'request + $defs.Response' : 'request only (fire-and-forget)'}`);
  console.log('');
  console.log('It validates as generated — run `npm run validate` to confirm — but every TODO is');
  console.log('load-bearing prose only you can write. Start with ## Abstract and the four');
  console.log('## Security & Privacy sub-headings; the build lints for those by name.');
  console.log('');
  console.log('When the schema is final: regenerate the bindings and bump both libraries');
  console.log('in the same PR (see CONTRIBUTING-SPECS.md → "Regenerate the bindings").');
}

/* ── Templates ───────────────────────────────────────────────────────────── */

function renderProofRequirement(proof) {
  const rationale =
    proof.why === null
      ? 'TODO: one sentence on why your threat model needs this strength. RECOMMENDED is the right default where the transport already guarantees integrity; REQUIRED commits every conforming consumer to reject a proofless document.'
      : `TODO: name what makes the ${proof.why} evidentiary. This level is the floor SPEC §7.3 item 8 derives from the declarations below — the build rejects anything weaker.`;
  if (proof.requirement) {
    return [
      'proofRequirement:',
      `  requirement: ${proof.requirement}`,
      `  rationale: >-`,
      `    ${rationale}`
    ].join('\n');
  }
  return [
    'proofRequirement:',
    `  request: ${proof.request}`,
    `  response: ${proof.response}`,
    `  rationale: >-`,
    `    ${rationale}`
  ].join('\n');
}

/*
 * SPEC §7.3 item 17. A consequential task MUST require `issuedAt`, and the
 * scaffold's claim is that what it emits validates and conforms — so it
 * declares it rather than leaving the author a floor to trip over. A
 * non-consequential task gets nothing: absent means the §4.2 SHOULD applies,
 * and inventing a RECOMMENDED restatement of the baseline would be noise.
 *
 * Returns a leading newline with the block, or '' — so the caller can splice
 * it into the front matter without leaving a blank line behind.
 */
function renderIssuedAtRequirement(ctx) {
  if (!ctx.consequential) return '';
  const triggers = [`sideEffects.level: ${ctx.sideEffects}`]
    .concat(ctx.discloses === 'secret' ? ['exposure.discloses: secret'] : [])
    .concat(ctx.actsAsSubject ? ['exposure.actsAsSubject: true'] : [])
    .join(', ');
  return '\n' + [
    'issuedAtRequirement:',
    '  requirement: REQUIRED',
    '  rationale: >-',
    `    TODO: name what makes this task's documents worth placing in time. REQUIRED is the`,
    `    floor SPEC §7.3 item 17 sets for a consequential Trust Task (${triggers}) — §7.2`,
    `    item 11 can only absorb a duplicate inside a bounded window, and a document with no`,
    `    issuedAt cannot be placed in one. The build rejects anything weaker.`
  ].join('\n');
}

/* YAML-quote a scalar. Every author-facing default here contains prose, and
 * prose contains ": " — which YAML reads as a nested mapping and rejects. The
 * scaffold's whole claim is that its output parses, so quote unconditionally
 * rather than guessing which strings are safe. */
function yq(s) {
  return JSON.stringify(String(s));
}

function renderSpecMd(ctx) {
  const { slug, version, title, summary, category, typeUri } = ctx;
  const exposureRationale =
    ctx.discloses === 'secret' || ctx.actsAsSubject
      ? '\n  rationale: >-\n    TODO: name the disclosed material or the authority exercised. REQUIRED by the meta-schema whenever discloses is secret or actsAsSubject is true.'
      : '';

  const authorization = ctx.consequential
    ? `## Authorization

TODO — **required**: this task is *consequential* ([SPEC §2](/SPEC.md#2-terminology)), because it declares \`sideEffects.level: ${ctx.sideEffects}\`${ctx.discloses === 'secret' ? ', `exposure.discloses: secret`' : ''}${ctx.actsAsSubject ? ', `exposure.actsAsSubject: true`' : ''}. [SPEC §7.3 item 15](/SPEC.md#73-specification-requirements) requires it to describe the class of authorization evidence a consumer needs.

Name the **authority**, not the pipeline step. "The consumer verifies the proof, then executes" describes a check; it never says what entitles the producer to the outcome. Write the entitlement in one sentence — ownership of the resource, a held capability, membership of the exchange named in \`parties\`, an accepted prior proposal, possession of a token — and say which conformance rule enforces it.

Then distinguish it from identity and proof validation: per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying a VID, \`issuer\`, \`recipient\`, transport identity or \`proof\` establishes *who* and *unaltered*, never *authorized*. If this task is open to any caller, say so explicitly — that is a legitimate design and item 15 asks for it to be stated rather than inferred from silence.

Keep it descriptive: a specification **MUST NOT** declare that consent, human approval or a step-up is required. That policy belongs to the consumer.

`
    : '';

  const requestSection = `## Request

TODO: one sentence naming the producer and the recipient, and pointing at the top-level schema in [\`payload.schema.json\`](payload.schema.json).

### TODO: what this example demonstrates

\`\`\`json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "${typeUri}${ctx.response ? '#request' : ''}",
  "issuer": "did:example:producer",
  "recipient": "did:example:recipient",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "subject": "did:example:subject"
  }
}
\`\`\`
`;

  const responseSection = ctx.response
    ? `
## Response

TODO: one sentence naming the producer (the *recipient* of the request, now responding) and pointing at the sub-schema reachable via \`$anchor: "response"\`. Describe each payload member, and note that failures use \`trust-task-error\` rather than a \`#response\` document.

### TODO: what this example demonstrates

\`\`\`json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "${typeUri}#response",
  "issuer": "did:example:recipient",
  "recipient": "did:example:producer",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "outcome": "accepted"
  }
}
\`\`\`
`
    : `
<!-- This is a fire-and-forget task (SPEC §4.4.1): no success-response document,
     and payload.schema.json therefore carries no $defs.Response. Failures still
     use trust-task-error. Delete this comment once the Abstract says so. -->
`;

  return `---
slug: ${slug}
version: "${version}"
title: ${yq(title)}
summary: ${yq(summary)}
status: draft
targetFrameworkVersion: "${ctx.targetFrameworkVersion}"
category: ${category}
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong \u2014 a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [${[...new Set(slug.split('/').flatMap((x) => x.split('-')))].join(', ')}, a-term-a-searcher-would-use]
#   authors:
#     - Your Name (https://github.com/your-handle)
parties:
  - role: "TODO: the party that issues the request"
    requirement: REQUIRED
    member: issuer
    # identifierScope: pairwise   # framework 0.5.0, OPTIONAL: pairwise | public | any.
                                  # \`public\` says the counterparty must recognise a reusable
                                  # identifier, which forecloses pairwise identifiers for every
                                  # producer — the build warns unless the prose justifies it.
  - role: "TODO: the party that acts on it"
    requirement: REQUIRED
    member: recipient
${renderProofRequirement(ctx.proof)}${renderIssuedAtRequirement(ctx)}
sideEffects:
  level: ${ctx.sideEffects}
  rationale: >-
    TODO: why this level.${ctx.sideEffects === 'destructive' ? ' For destructive, name the irreversible effect.' : ''}
exposure:
  discloses: ${ctx.discloses}
  actsAsSubject: ${ctx.actsAsSubject}${exposureRationale}
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — \`personal\` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # \`personal\` or \`secret\` makes exposure.rationale REQUIRED.
# retention:              # framework 0.5.0, OPTIONAL: how long what the recipient receives lives.
#   class: exchange       # transient (discarded in flight) | exchange (life of the session/thread/
#                         # enactment) | durable (kept beyond it, because it is a record).
#                         # An absent declaration reads as \`durable\`.
#   rationale: >-
#     TODO: why this class. For durable, name what makes the record worth keeping.
errorCodes: []
related: []
---

## Abstract

TODO: two or three sentences. What outcome do the two parties agree to achieve, and why does it need to be a Trust Task rather than an API call?

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version ${ctx.targetFrameworkVersion} and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

${authorization}## Definitions

TODO: define each payload member — what it means, who chooses its value, and what a consumer does with it. Members are lowerCamelCase ([SPEC §4.10](/SPEC.md#410-naming-conventions)).

${requestSection}${responseSection}
## Security & Privacy

### Data carried

TODO: what the request and the response actually move. Name the personal or sensitive members explicitly, and say what a producer **MUST NOT** put in the free-form ones. State the smallest payload that still answers the task — this is where data minimisation gets written down.

### Correlation

TODO: what an observer, an intermediary, or the recipient can join across documents — subject identifiers, \`threadId\`, stable handles, request timing. Say which of those are unavoidable given the task and which a producer can vary.

### Retention

TODO: how long a recipient needs to keep what it receives, and what this document's evidentiary value implies about deleting it.

### Consent/purpose

TODO: the purpose the data is collected for and the limit on reusing it. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required.
`;
}

function renderPayloadSchema(ctx) {
  const schema = {
    $schema: 'https://json-schema.org/draft/2020-12/schema',
    $id: ctx.typeUri,
    title: `${ctx.title} — payload`,
    description: `TODO: what the request payload of ${ctx.slug} carries. The outer document members (id, type, issuer, recipient, issuedAt, expiresAt, proof) are owned by the framework — SPEC §6.3.`,
    type: 'object',
    additionalProperties: false,
    required: ['subject'],
    properties: {
      subject: {
        type: 'string',
        minLength: 1,
        description: 'TODO: replace with this task\'s real members. A VID identifying the subject of the task, if it has one — declare `subjectPath: /subject` in front matter when it does, so a delegated-execution policy engine can find it without per-task code.'
      }
    }
  };
  if (ctx.response) {
    schema.$defs = {
      Response: {
        $anchor: 'response',
        type: 'object',
        additionalProperties: false,
        required: ['outcome'],
        description: 'TODO: the success-response payload. Failures use trust-task-error, not this shape.',
        properties: {
          outcome: {
            type: 'string',
            enum: ['accepted'],
            description: 'TODO: replace with the response members this task returns.'
          }
        }
      }
    };
  }
  return JSON.stringify(schema, null, 2) + '\n';
}

main();
