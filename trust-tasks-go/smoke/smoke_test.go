// The smoke test across the seam between the generated packages and the
// hand-written runtime — the Go counterpart of `npm run smoke` in
// trust-tasks-ts.
//
// The test itself is testdata/smoke/smoke_test.go. It imports generated
// packages, which are no longer in this module: they are the
// trust-tasks-go/specs module (#1112), and that module cannot yet require a
// published core without specs/ (every published core still contains the
// tree, so an import of a spec package would be ambiguous). So this test builds
// a throwaway copy of the specs module holding only the packages the smoke test
// imports, points it at this checkout's runtime with a replace — as the
// examples do — and runs the smoke test inside it. It drives the generated
// code exactly as before, under the same import paths, and keeps the smoke in
// the core's `go test ./...`.
//
// When the specs module is released and gets its own CI job (#1112 step 3),
// the smoke test can move into that module and this driver can go.
package smoke

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

const (
	coreModule  = "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go"
	specsModule = coreModule + "/specs"
)

// smokePackages are the generated packages testdata/smoke imports, relative to
// trust-tasks-go/specs.
var smokePackages = []string{
	"acl/grant/v0_2",
	"auth/authenticate/v0_3",
	"vta/contexts/update/v1_0",
}

func TestGeneratedPackagesDriveTheRuntime(t *testing.T) {
	if _, err := exec.LookPath("go"); err != nil {
		t.Skip("no go command on PATH")
	}
	core, err := filepath.Abs("..")
	if err != nil {
		t.Fatal(err)
	}
	// A copy of this module outside the repository — the module cache, or
	// `go test all` in an application — has no specs/ beside it.
	if _, err := os.Stat(filepath.Join(core, "specs", "go.mod")); err != nil {
		t.Skip("the trust-tasks-go/specs module is not beside this core (not a repository checkout)")
	}
	mod := t.TempDir()

	// The smoke test imports exactly smokePackages; a package it starts to
	// import without being listed fails the build below, by name.
	for _, pkg := range smokePackages {
		copyGoFiles(t, filepath.Join(core, "specs", filepath.FromSlash(pkg)), filepath.Join(mod, filepath.FromSlash(pkg)))
	}
	copyGoFiles(t, filepath.Join("testdata", "smoke"), filepath.Join(mod, "smoke"))

	goMod := "module " + specsModule + "\n\ngo 1.22\n\n" +
		"require " + coreModule + " v0.0.0-00010101000000-000000000000\n\n" +
		"replace " + coreModule + " => " + filepath.ToSlash(core) + "\n"
	if err := os.WriteFile(filepath.Join(mod, "go.mod"), []byte(goMod), 0o644); err != nil {
		t.Fatal(err)
	}

	args := []string{"test", "-count=1"}
	if testing.Verbose() {
		args = append(args, "-v")
	}
	cmd := exec.Command("go", append(args, "./smoke")...)
	cmd.Dir = mod
	cmd.Env = append(os.Environ(), "GOFLAGS=-mod=mod", "GOWORK=off")
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("the smoke test failed in a copy of the specs module: %v\n%s", err, out)
	}
	if testing.Verbose() {
		t.Logf("%s", out)
	}
}

// copyGoFiles copies the .go files of src (not its subdirectories) into dst.
func copyGoFiles(t *testing.T, src, dst string) {
	t.Helper()
	entries, err := os.ReadDir(src)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(dst, 0o755); err != nil {
		t.Fatal(err)
	}
	copied := 0
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".go") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(src, e.Name()))
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(dst, e.Name()), data, 0o644); err != nil {
			t.Fatal(err)
		}
		copied++
	}
	if copied == 0 {
		t.Fatalf("no .go files in %s", src)
	}
}
