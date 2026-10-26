module example.com/trust-tasks-consumer-app-go

go 1.22

require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.0.0-00010101000000-000000000000

// The example builds against this checkout's runtime and generator, so it
// tests them before they are released. An application outside this repository
// requires a published version and has no replace.
replace github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go => ../../trust-tasks-go
