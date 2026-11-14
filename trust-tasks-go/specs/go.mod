// The generated specification packages are a SEPARATE module from
// trust-tasks-go, so the core no longer carries the generated tree (#1112). The
// import paths do not change: this module's path is a prefix of every package's.
//
// TODO(#1112 step 3): require the first trust-tasks-go release WITHOUT specs/.
// v0.12.0 still contains these packages, so against it an importer of one sees
// it in two modules (the core and this one) and the go command refuses the
// import as ambiguous. Until step 3 moves the requirement, this module is
// unreleased and CI does not build it.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs

go 1.22

require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.12.0
