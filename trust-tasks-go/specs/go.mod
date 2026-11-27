// The generated specification packages are a SEPARATE module from
// trust-tasks-go, so the core no longer carries the generated tree (#1112). The
// import paths do not change: this module's path is a prefix of every package's.
//
// It must require a core published WITHOUT specs/ (v0.13.0 is the first).
// Against an older core, which still contains these packages, an importer sees
// each one in two modules and the go command refuses the import as ambiguous.
//
// It must also require a core that has every runtime API the generated tree
// uses: trusttasks.Nullable is in v0.14.0 and later.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs

go 1.24

require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.14.0
