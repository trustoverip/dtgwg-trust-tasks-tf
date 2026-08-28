---
slug: consent/approve-request
version: "0.1"
title: "Consent — Approve Request"
summary: "An agent's home service asks a designated human approver to decide whether the agent may act on one conversation, issuing the challenge that binds their answer to this request."
status: draft
targetFrameworkVersion: "0.5.0"
category: consent
parties:
  - role: Requesting service
    requirement: REQUIRED
    member: issuer
  - role: Approver
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The document is rendered to a person and asks them to authorise an agent's
    access to a conversation. Without a proof the only thing vouching for it is
    the transport, so anything that can reach the approver's inbox can pose a
    question in the service's name, and the approver has no way to tell. The
    prompt is also the sole carrier of the challenge a decision is bound to,
    so an unauthenticated one lets an attacker choose the value the approver
    will sign.
sideEffects:
  level: none
  rationale: >-
    Asks for a decision and issues a challenge. Nothing is granted here — the
    approver answers with a separate consent/decision, and it is that document
    which changes what the agent may do.
subjectPath: /subject/agent
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: personal
  rationale: >-
    The prompt tells the approver's device which platform a conversation lives
    on, what kind of conversation it is, and — through displayHint — a
    human-readable name for it. That is a description of the subject's
    communications, and it arrives on a device the requesting service does not
    control, which is why it is declared as personal on ingest rather than as
    the metadata it discloses back.
retention:
  class: exchange
  rationale: >-
    The prompt is worth keeping only until the matching decision is made or the
    challenge expires. Its evidentiary value lives in the signed decision, which
    echoes the challenge; retaining prompts beyond that accumulates a log of who
    the subject talks to on a device chosen for its convenience, not for being a
    good place to store one.
errorCodes:
  - code: consent/approve-request:subjectUnknown
    meaning: The approver does not speak for the agent named in subject.agent.
    retryable: false
  - code: consent/approve-request:scopeUnsupported
    meaning: The approver cannot render or answer a decision at the requested scope.
    retryable: false
  - code: consent/approve-request:challengeReplayed
    meaning: The challenge has been seen before. A challenge is single-use; a repeat is either a retry that must not be answered twice or a replay.
    retryable: false
related:
  - consent/request
  - consent/decision
---

## Abstract

An agent wants to act on a conversation — read it, or read and reply. A human
who is not the agent's operator has to decide whether it may. This task carries
that question to the approver's device and issues the single-use challenge their
answer must echo; the answer itself is a separate, separately-signed
`consent/decision`.

It is a Trust Task rather than an API call because the approver is reached
across a trust boundary, on a device the requesting service does not control and
often cannot address directly. There is no session to authenticate the question
and no channel whose integrity the approver can assume, so the document has to
carry its own authentication — which is what makes the prompt a thing the
approver can verify rather than a thing that merely arrived.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

**subject** — what the decision is about: one conversation, for one agent. Chosen
by the requesting service. Its members are defined below.

**subject.platform** — the messaging platform the conversation lives on, as the
bridge names it. Opaque to the framework.

**subject.conversationRef** — the bridge's handle for the conversation. It is
deliberately **opaque**: a consent record naming raw platform addresses is a
stored directory of who the subject talks to, readable by anyone who reaches the
store. A producer **MUST NOT** put a phone number, a platform handle or a group
invite link here.

**subject.kind** — `dm`, `group` or `channel`. It is not decoration: consenting
to an agent reading a 1:1 conversation exposes two parties; consenting on a group
exposes everyone in it, most of whom are not being asked. An approver **SHOULD**
be shown it.

**subject.agent** — the DID of the agent the decision is about. Consent is
granted to an agent, not to the service hosting it, so revoking one agent's
access does not revoke another's.

**scope** — `receive` (the agent sees inbound messages) or `converse` (it may
also reply). These are distinct values rather than a boolean because replying
means the agent can speak to the other parties *as the subject*, which is a
materially different thing to agree to.

**challenge** — a single-use, unpredictable value chosen by the requesting
service. The approver **MUST** echo it in the `consent/decision` they sign. A
consumer **MUST** reject a decision carrying a challenge it did not issue, and
**MUST NOT** accept the same challenge twice.

**displayHint** — an advisory human-readable label for the approver's screen.
Chosen by the requesting service, so a renderer **MUST** treat it as untrusted
text: escape it, bound its length, and never let it displace the `subject`
members as the basis of the decision. Present because `conversationRef` is
opaque by design — without a hint the approver is asked to decide about an
identifier that means nothing to them, and will end up always allowing or always
denying.

**firstMessageDigest** — a digest of the inbound message that prompted the
request, when there is one. A digest rather than the content: the approver is
deciding whether the agent may *read* the conversation, and showing them the
message in order to ask would disclose the very thing the decision gates.

## Request

The requesting service (`issuer`) sends this to the approver (`recipient`). The
payload is defined by the top-level schema in
[`payload.schema.json`](payload.schema.json).

A consumer **MUST** verify the proof before rendering anything to a person. A
prompt that fails verification, or carries none, is not a prompt — rendering it
and letting the human decide moves the authentication decision onto someone who
has no way to make it.

### A `converse` request on a named group, with a prompting message

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/consent/approve-request/0.1",
  "issuer": "did:example:producer",
  "recipient": "did:example:recipient",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "subject": {
      "platform": "signal",
      "conversationRef": "9f2c1a7e-4d31-4a2b-9c88-0e6b1d5f3a44",
      "kind": "group",
      "agent": "did:example:agent"
    },
    "scope": "converse",
    "challenge": "0e3f5a91c7b24d8e",
    "displayHint": "Signal group 'Family'",
    "firstMessageDigest": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-01-01T00:00:00Z",
    "verificationMethod": "did:example:producer#key-0",
    "proofPurpose": "assertionMethod",
    "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
  }
}
```

The `conversationRef` is a bridge-assigned identifier, not a group link. The
approver recognises the conversation from `displayHint`; `conversationRef` exists
so the decision can be matched back without the matching key being personal
data.

## Security & Privacy

### Data carried

The request moves a description of one conversation onto the approver's device:
the platform, an opaque reference, the interaction kind, the agent's DID, and a
human-readable label. `displayHint` is the personal member — it is chosen to be
recognisable to a human, which is exactly what makes it identifying. It is bounded
at 256 characters and a producer **MUST NOT** put message content, participant
lists, or platform addresses in it.

`conversationRef` **MUST** be opaque (see Definitions). `firstMessageDigest` is a
digest precisely so the prompt does not carry the message.

The smallest payload that still answers the task is `subject`, `scope` and
`challenge`: enough to say what is being asked, about which agent and
conversation, and to bind the answer. `displayHint` and `firstMessageDigest` are
optional and exist to make the decision *informed* — a producer that can ask a
recognisable question without them **SHOULD** omit them.

There is no response document. The approver answers with `consent/decision`,
which is where any decision-bearing data lives.

### Correlation

`subject.agent` and `subject.conversationRef` are stable across every prompt about
the same agent and conversation, and they have to be — a decision that could not be
matched to the conversation it was about would grant nothing. So an approver, and
anyone who reads their device, accumulates a picture of which conversations an
agent is asking about and how often.

What a producer can vary: `challenge` is single-use and unpredictable by
requirement, and `id` is per-document, so neither links prompts together. A
producer **SHOULD** use a pairwise identifier as `issuer` where its ecosystem
supports one, so the prompt does not also join the approver to the service's
other relationships.

What it cannot vary: request timing. A prompt is sent when a message arrives, so
their arrival pattern discloses conversation activity to anyone observing the
approver's transport, whether or not they can read the payload.

### Retention

An approver needs the prompt until they answer it or the challenge expires,
whichever is first. After that it has no evidentiary value: what a party would
later want to show is the signed `consent/decision`, and that document echoes the
challenge, so the decision is self-supporting.

Keeping prompts beyond the exchange therefore buys nothing and accumulates a log
of who the subject talks to, on a device chosen for being convenient to prompt on
rather than for being a good place to store one. A consumer **SHOULD** discard the
prompt once the decision is sent.

### Consent/purpose

The data is carried for one purpose: to let a human decide, on this occasion,
whether an agent may act on this conversation. It is not a directory of the
subject's conversations, and a recipient **SHOULD NOT** reuse it to build one —
in particular, the accumulated `subject` members across prompts **SHOULD NOT** be
retained as a contact graph once the individual decisions are made.

Note what this document is and is not. It is the *carrier* of a question, and the
`consent/decision` is the record of an answer. Neither is what authorises the
agent: that is the ecosystem's own policy, which decides what weight a decision
carries and for how long. This specification describes how the question travels
and how an answer is bound to it.
