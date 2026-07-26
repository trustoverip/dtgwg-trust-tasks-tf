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
  requirement: OPTIONAL
  rationale: The presented access token is itself the credential; it was minted against a proven identity.
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

## Security & Privacy

`exposure.discloses` is `secret` because the response *is* an authenticator — the `Set-Cookie` headers authenticate every subsequent request from that browser.

Because the CSRF cookie must be readable by the SPA (it has to echo the value in a header), only the session cookie carries `HttpOnly`. That asymmetry is the design: the readable half proves same-origin intent, the unreadable half carries the authority.
