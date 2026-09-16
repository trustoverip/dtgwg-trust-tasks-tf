# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.2 — 2026-09-16

## 0.1.1 — 2026-09-16


### Fixed

- **dart**: Take trust_tasks to 160/160 on pub.dev (#473)

pub.dev scored the published 0.1.0 at 140/160, with two specific deductions.

  -10 "The package description is too long." The field was 196 characters against
  a 60-180 window; search results only render the first part of it, which is why
  pub.dev treats an over-long one as a failed pubspec check rather than a style
  note. Now 148, with the window recorded in a comment above it so the next edit
  does not silently fall outside it again.

  -10 "No example found." Adds example/main.dart, which is runnable rather than
  illustrative: it decodes an inbound document, runs it through consumeInbound
  against the generated acl/grant policy, and delivers it three times to show both
  halves of SPEC §7.2 item 11 —

    first delivery    : handled -> .../acl/grant/0.1#response
    identical resend  : duplicate (already executed, inFlight=false)
    escalated resend  : rejected -> idConflict
    handler ran 1 time(s) — the effect happened exactly once.

  Writing it as something that runs is what caught a bug in it: the first draft
  omitted the proof acl/grant declares REQUIRED, so all three deliveries returned
  proofRequired and the example demonstrated a rejection rather than the pipeline.

  Verified with pana 0.23.19 — the analyzer pub.dev runs — against the working
  tree: Points: 160/160. `dart analyze --fatal-infos`, the format check and the
  74 tests all still pass with example/ included in the analysed set.

  The score will not move until a release carries these files, so the improvement
  lands with 0.1.1 — which is also the first version the trusted-publishing
  automation will publish end to end.

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `lib/src/runtime/`.
