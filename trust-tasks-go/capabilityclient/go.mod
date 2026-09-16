// Client-side wire helpers for the capability Trust Task families
// (governance/capability/* and git-trust/*). A SEPARATE module from
// trust-tasks-go like tsp/proof/didcomm, but pure wire logic — no crypto, no
// transport, and no third-party dependency. Its only requirement is the core
// module whose Document type and Type-URI helpers it builds on.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/capabilityclient

go 1.22

require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.1.4
