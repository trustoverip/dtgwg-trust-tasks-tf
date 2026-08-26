---
slug: vtc/auth/admin-session
version: "0.1"
title: VTC Auth — Admin Session
summary: Exchange a bearer access token for an HttpOnly cookie session, so a browser administration UI can call the community without exposing the token to scripts.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - vtc
  - auth
  - session
  - cookie
  - admin-ui
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The presented access token was minted against a proven identity, but the token is a bearer artefact — anyone who obtains it can replay it, and the envelope proof is what binds the presentation to the party entitled to make it. Execution mints an admin session in the subject's name and returns secret session material the caller retains, so token theft and replay are the threats addressed. Requiring the proof also brings the audience-binding rule of §4.8.2 into force, which pins the session to a named recipient rather than to whoever presents the token.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An admin session is minted under the administrator's own authority and returns the session secret to the caller. A replayed request mints a second concurrent session for that administrator, the highest-privilege duplicate the community can suffer.
sideEffects:
  level: mutating
  rationale: "Mints a cookie-backed session bound to the token's identity."
exposure:
  discloses: secret
  actsAsSubject: true
  rationale: Consumes a bearer access token and returns Set-Cookie headers that authenticate subsequent requests. The caller acts on their own session.
errorCodes:
  - code: vtc/auth/admin-session:invalidToken
    meaning: The access token is malformed, expired, or was not issued by this community.
    retryable: false
---

## Abstract

The **VTC Auth — Admin Session** Trust Task bridges a bearer token into a cookie session. A browser administration UI authenticates once — over the wallet SIOP flow or any other path that yields an access token — then exchanges that token here for `HttpOnly` cookies it can no longer read.

The point is that the SPA stops holding a token in JavaScript. An `HttpOnly` cookie is unreadable by page scripts, so an XSS foothold cannot exfiltrate the session the way it could a token in memory or storage.

## Conformance

Producer: send a currently-valid `accessToken`.

Consumer: validate the token exactly as a bearer on any other route — same audience, same expiry, same session lookup. Reject a token this community did not issue with `invalidToken`. On success set the session and CSRF cookies with `HttpOnly` on the session cookie, and bind the cookie session to the same identity and expiry as the token, so the exchange does not silently extend a session's life.

The cookie side-effect is a **transport binding concern**, not payload: a non-browser consumer of this task gets the same session semantics without cookies.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is possession of a **currently-valid `accessToken` issued by this community**, validated exactly as a bearer credential on any other route — same audience, same expiry, same session lookup. A token this community did not issue is refused with `invalidToken`.

This exchange creates a cookie session, so the authority it rests on is bearer authority and carries bearer risk: whoever holds the token gets the session. The declared exposure (`discloses: secret`, `actsAsSubject: true`) is a statement about that. Binding the cookie session to the same identity and expiry as the token is what stops the exchange silently extending a session's life beyond the authority that created it.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

`exposure.discloses` is `secret` because the response *is* an authenticator — the `Set-Cookie` headers authenticate every subsequent request from that browser.

Because the CSRF cookie must be readable by the SPA (it has to echo the value in a header), only the session cookie carries `HttpOnly`. That asymmetry is the design: the readable half proves same-origin intent, the unreadable half carries the authority.
