# Changelog — `@openvtc/trust-tasks-gen`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the
`trust-tasks-ts-gen-v<version>` tag; see `RELEASING.md`.

## 0.1.0

- Initial release: `trust-tasks-gen-ts` generates TypeScript modules for the
  versions pinned in `trust-tasks.lock`, importing the runtime from
  `@openvtc/trust-tasks/runtime` (0.27.8 or later). Published by hand.
