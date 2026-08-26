# Changelog

## [0.2.0] - 2026-08-26

### Changed

- **`trust-tasks-https` requirement moved to `0.16`.** That release makes
  `HttpsServer::on` require `RequestPayload` and adds `on_ack`; this crate
  re-exports the type, so the leading component moves with it. No change to
  this crate's own API.

