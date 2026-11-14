module example.com/trust-tasks-getting-started-go

go 1.22

require (
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.12.0
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof v0.0.0-00010101000000-000000000000
)

// The example builds against this checkout's runtime, proof module and
// generator, so it tests them before they are released. An application outside
// this repository requires published versions and has no replace.
replace (
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go => ../../trust-tasks-go
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof => ../../trust-tasks-go/proof
)
