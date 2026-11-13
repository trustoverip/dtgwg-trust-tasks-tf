# Security policy

This repository publishes libraries that sit on security-relevant paths: proof
creation and verification, transport bindings (HTTPS, DIDComm, TSP), and the
inbound processing pipeline that decides whether a Trust Task document is
accepted. Please report a suspected vulnerability in any of them privately, as
described below, and not in a public issue, pull request or discussion.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting: open the repository's
**Security** tab and choose **Report a vulnerability**
(<https://github.com/trustoverip/dtgwg-trust-tasks-tf/security/advisories/new>).
The report is visible only to the maintainers, and the fix, the advisory and
any CVE are coordinated in the same place.

Please include:

- the package (or specification) and version affected;
- what an attacker can do, and the conditions it needs;
- a proof of concept or a failing test, if you have one;
- whether, and when, you intend to disclose it yourself.

We aim to acknowledge a report within five working days and to agree a fix and
disclosure timeline with you after that. We will credit you in the advisory
unless you ask us not to.

A weakness in a **specification** rather than in the code — a task whose
payload invites a confused-deputy attack, say, or a framework rule that cannot
be implemented safely — is also welcome through the same channel. The
framework specification itself is maintained in
[trustoverip/dtgwg-trust-tasks-spec](https://github.com/trustoverip/dtgwg-trust-tasks-spec);
report it here if you are unsure, and we will route it.

## Supported versions

Security fixes are made in the **latest release** of each package, and released
as a new version of it. Earlier releases are not patched; upgrade to receive the
fix. That applies to every package this repository publishes:

| Ecosystem | Packages |
|---|---|
| crates.io | `trust-tasks` (the front-door crate), `trust-tasks-rs`, and the other `trust-tasks-*` crates (the HTTPS, DIDComm, TSP, proof and capability-client bindings, `trust-tasks-build`, `trust-tasks-cli`, `trust-tasks-ceremony`) |
| npm | `@openvtc/trust-tasks` and its `@openvtc/trust-tasks-*` siblings |
| Go | `github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go` and its nested modules (`proof`, `didcomm`, `capabilityclient`) |
| pub.dev | `trust_tasks` and its `trust_tasks_*` siblings |

All of them are below 1.0, so a fix may arrive in a release that is also a
breaking one under the library's own semver rules; its changelog says so.

**Go releases cannot be withdrawn.** `proxy.golang.org` caches every tag
permanently, so a vulnerable Go version stays downloadable after the fix ships.
Go fixes therefore go forward only: a new tag, plus a `retract` directive in the
module's `go.mod` naming the affected versions, so that `go get` and
`go list -m -u` steer users away from them. crates.io (yank) and npm
(deprecate) releases are marked the same way where the registry allows it.
