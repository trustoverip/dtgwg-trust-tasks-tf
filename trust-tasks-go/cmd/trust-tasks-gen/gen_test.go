package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// copyApp copies testdata/app (a lock pinning acl/grant 0.2, keys/list 0.2 and
// vault/get 0.3, as examples/consumer-app does) into a temporary directory.
func copyApp(t *testing.T) string {
	t.Helper()
	dst := t.TempDir()
	err := filepath.Walk("testdata/app", func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, _ := filepath.Rel("testdata/app", path)
		target := filepath.Join(dst, rel)
		if info.IsDir() {
			return os.MkdirAll(target, 0o755)
		}
		data, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		return os.WriteFile(target, data, 0o644)
	})
	if err != nil {
		t.Fatal(err)
	}
	return dst
}

func readTree(t *testing.T, root string) map[string]string {
	t.Helper()
	out := map[string]string{}
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return err
		}
		rel, _ := filepath.Rel(root, path)
		data, err := os.ReadFile(path)
		out[filepath.ToSlash(rel)] = string(data)
		return err
	})
	if err != nil {
		t.Fatal(err)
	}
	return out
}

func TestGeneratesTheLockedPackagesDeterministically(t *testing.T) {
	app := copyApp(t)
	lock := filepath.Join(app, "trust-tasks.lock")
	out := filepath.Join(app, "internal", "tasks")
	if err := run(options{Lock: lock, Out: out}); err != nil {
		t.Fatal(err)
	}
	first := readTree(t, out)
	for _, want := range []string{
		markerFile,
		"trusttasks.go",
		"acl/grant/v0_2/payload.go",
		"keys/list/v0_2/payload.go",
		"vault/get/v0_3/payload.go",
		// The sourced task, under a directory named for its source.
		"acme/payments/settle/v1_0/payload.go",
	} {
		if _, ok := first[want]; !ok {
			t.Errorf("%s was not generated", want)
		}
	}
	if len(first) != 6 {
		t.Errorf("generated %d files, want 6: %v", len(first), first)
	}
	settle := first["acme/payments/settle/v1_0/payload.go"]
	for _, want := range []string{
		"package acmepaymentssettlev1_0\n",
		// Identity is the bundle's typeUri, not rebuilt under trusttasks.org.
		`const TypeURI = "https://tasks.example.com/spec/payments/settle/1.0"`,
		`const ResponseTypeURI = "https://tasks.example.com/spec/payments/settle/1.0#response"`,
		"References *[]string       `json:\"references,omitempty\"`",
	} {
		if !strings.Contains(settle, want) {
			t.Errorf("acme/payments/settle/v1_0 lacks %q", want)
		}
	}
	if !strings.Contains(first["trusttasks.go"], "package tasks\n") {
		t.Errorf("root package not named for the out directory")
	}
	vault := first["vault/get/v0_3/payload.go"]
	for _, want := range []string{
		"package vaultgetv0_3\n",
		`"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"`,
		// Optional slices are pointers: present-and-empty is not absent.
		"RedactedFields *[]string  `json:\"redactedFields,omitempty\"`",
		`const ResponseTypeURI = "https://trusttasks.org/spec/vault/get/0.3#response"`,
	} {
		if !strings.Contains(vault, want) {
			t.Errorf("vault/get/v0_3 lacks %q", want)
		}
	}

	// A stale file in the owned directory goes; the output is byte-identical.
	stale := filepath.Join(out, "old", "v0_1", "payload.go")
	if err := os.MkdirAll(filepath.Dir(stale), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(stale, []byte("package old\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := run(options{Lock: lock, Out: out}); err != nil {
		t.Fatal(err)
	}
	second := readTree(t, out)
	if len(second) != len(first) {
		t.Fatalf("regeneration left %d files, want %d", len(second), len(first))
	}
	for k, v := range first {
		if second[k] != v {
			t.Errorf("%s differs between runs", k)
		}
	}
}

func TestRefusesADirectoryItDidNotWrite(t *testing.T) {
	app := copyApp(t)
	out := filepath.Join(app, "src")
	if err := os.MkdirAll(out, 0o755); err != nil {
		t.Fatal(err)
	}
	keep := filepath.Join(out, "main.go")
	if err := os.WriteFile(keep, []byte("package main\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	err := run(options{Lock: filepath.Join(app, "trust-tasks.lock"), Out: out, Package: "src"})
	if err == nil || !strings.Contains(err.Error(), "no "+markerFile+" marker") {
		t.Fatalf("got %v, want a refusal naming the marker", err)
	}
	if _, err := os.Stat(keep); err != nil {
		t.Fatal("the refusal deleted a file")
	}
	if err := run(options{Lock: filepath.Join(app, "trust-tasks.lock"), Out: app}); err == nil {
		t.Fatal("generating over the lock's own directory was not refused")
	}
}

func TestRefusesATamperedBundle(t *testing.T) {
	app := copyApp(t)
	path := filepath.Join(app, "trust-tasks", "vault", "get", "0.3.bundle.json")
	data, _ := os.ReadFile(path)
	if err := os.WriteFile(path, bytes.Replace(data, []byte(`"draft"`), []byte(`"standard"`), 1), 0o644); err != nil {
		t.Fatal(err)
	}
	out := filepath.Join(app, "gen")
	err := run(options{Lock: filepath.Join(app, "trust-tasks.lock"), Out: out})
	if err == nil || !strings.Contains(err.Error(), "vendored bundle does not match trust-tasks.lock — run `trust-tasks check`") {
		t.Fatalf("got %v", err)
	}
	if _, err := os.Stat(out); err == nil {
		t.Fatal("a refused run wrote output")
	}
}

// rewriteBundle edits a vendored bundle and re-pins its digest in the lock, as
// if the registry had published it so.
func rewriteBundle(t *testing.T, app, slug, version string, edit func([]byte) []byte) {
	t.Helper()
	path := filepath.Join(app, "trust-tasks", filepath.FromSlash(slug), version+".bundle.json")
	old, _ := os.ReadFile(path)
	updated := edit(old)
	if bytes.Equal(old, updated) {
		t.Fatal("edit changed nothing")
	}
	if err := os.WriteFile(path, updated, 0o644); err != nil {
		t.Fatal(err)
	}
	digest := func(b []byte) string { s := sha256.Sum256(b); return "sha256:" + hex.EncodeToString(s[:]) }
	lockPath := filepath.Join(app, "trust-tasks.lock")
	lock, _ := os.ReadFile(lockPath)
	lock = bytes.Replace(lock, []byte(digest(old)), []byte(digest(updated)), 1)
	if err := os.WriteFile(lockPath, lock, 0o644); err != nil {
		t.Fatal(err)
	}
}

func TestRefusesAnUnknownBundleVersion(t *testing.T) {
	app := copyApp(t)
	rewriteBundle(t, app, "vault/get", "0.3", func(b []byte) []byte {
		return bytes.Replace(b, []byte(`"bundleVersion":1`), []byte(`"bundleVersion":2`), 1)
	})
	err := run(options{Lock: filepath.Join(app, "trust-tasks.lock"), Out: filepath.Join(app, "gen")})
	if err == nil || !strings.Contains(err.Error(), "bundleVersion 2") {
		t.Fatalf("got %v", err)
	}
}

func TestSkipsWithdrawnAndFrameworkVersions(t *testing.T) {
	app := copyApp(t)
	rewriteBundle(t, app, "vault/get", "0.3", func(b []byte) []byte {
		return bytes.Replace(b, []byte(`"status":"draft"`), []byte(`"status":"withdrawn"`), 1)
	})
	lockPath := filepath.Join(app, "trust-tasks.lock")
	lock, _ := os.ReadFile(lockPath)
	lock = append(lock, []byte("\n[[task]]\nslug = \"trust-task-ok\"\nversion = \"0.1\"\ntype_uri = \"https://trusttasks.org/spec/trust-task-ok/0.1\"\ndigest = \"sha256:"+strings.Repeat("0", 64)+"\"\n")...)
	if err := os.WriteFile(lockPath, lock, 0o644); err != nil {
		t.Fatal(err)
	}
	res, err := loadAndGenerate(options{Lock: lockPath, Out: filepath.Join(app, "gen")})
	if err != nil {
		t.Fatal(err)
	}
	for _, f := range res.files {
		if f.slug == "vault/get" || f.slug == "trust-task-ok" {
			t.Errorf("%s was generated", f.rel)
		}
	}
	notes := strings.Join(res.notes, "\n")
	if !strings.Contains(notes, "vault/get 0.3: skipped — withdrawn") || !strings.Contains(notes, "trust-task-ok 0.1: skipped — a framework specification") {
		t.Errorf("notes: %s", notes)
	}
}

func TestLockParser(t *testing.T) {
	good := "# Written by `trust-tasks`. Do not edit by hand.\nversion = 1\nregistry = \"https://trusttasks.org\"\n\n" +
		"[[task]]\nslug = \"vault/get\"\nversion = \"0.3\"\ntype_uri = \"https://trusttasks.org/spec/vault/get/0.3\"\ndigest = \"sha256:" + strings.Repeat("a", 64) + "\"\n"
	lock, err := parseLock(good)
	if err != nil {
		t.Fatal(err)
	}
	if len(lock.Tasks) != 1 || lock.Tasks[0].Slug != "vault/get" || lock.Registry != "https://trusttasks.org" {
		t.Fatalf("%+v", lock)
	}
	if lock, err := parseLock("version = 1\nregistry = \"x\"\n"); err != nil || len(lock.Tasks) != 0 {
		t.Fatalf("an empty lock: %v", err)
	}
	for name, bad := range map[string]string{
		"version 2":       strings.Replace(good, "version = 1", "version = 2", 1),
		"unknown member":  good + "extra = \"x\"\n",
		"unknown top":     strings.Replace(good, "registry =", "extra = \"x\"\nregistry =", 1),
		"other table":     good + "[meta]\n",
		"missing digest":  strings.Replace(good, "digest = \"sha256:"+strings.Repeat("a", 64)+"\"\n", "", 1),
		"bad version":     strings.Replace(good, "version = \"0.3\"", "version = \"0.3.1\"", 1),
		"bad slug":        strings.Replace(good, "slug = \"vault/get\"", "slug = \"_vault/get\"", 1),
		"bad digest":      strings.Replace(good, "sha256:", "md5:", 1),
		"duplicate":       good + good[strings.Index(good, "[[task]]"):],
		"unquoted string": strings.Replace(good, "slug = \"vault/get\"", "slug = vault/get", 1),
		"no registry":     strings.Replace(good, "registry = \"https://trusttasks.org\"\n", "", 1),
	} {
		if _, err := parseLock(bad); err == nil {
			t.Errorf("%s: accepted", name)
		}
	}
}

func TestIdentifiers(t *testing.T) {
	for in, want := range map[string]string{
		"sessionId":   "SessionID",
		"DIDDoc":      "DIDDoc",
		"identifier":  "Identifier",
		"1of2":        "N1of2",
		"":            "X",
		"vid-a_b":     "VIDAB",
		"sha256Hash":  "Sha256Hash",
		"allowedKeys": "AllowedKeys",
	} {
		if got := goIdent(in); got != want {
			t.Errorf("goIdent(%q) = %q, want %q", in, got, want)
		}
	}
	if got := goPackageName("did-management/resolve", "0.12"); got != "didmanagementresolvev0_12" {
		t.Errorf("goPackageName = %q", got)
	}
	if got := goString("a\"b\\c\n\x01é`"); got != "\"a\\\"b\\\\c\\n\\x01é`\"" {
		t.Errorf("goString = %s", got)
	}
}

// TestGeneratedPackagesCompile builds the generated tree — the three official
// packages and the sourced one — in a throwaway module against this checkout's
// runtime, as an application would.
func TestGeneratedPackagesCompile(t *testing.T) {
	if _, err := exec.LookPath("go"); err != nil {
		t.Skip("no go command on PATH")
	}
	app := copyApp(t)
	core, err := filepath.Abs("../..")
	if err != nil {
		t.Fatal(err)
	}
	mod := "module example.com/app\n\ngo 1.22\n\n" +
		"require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.0.0-00010101000000-000000000000\n\n" +
		"replace github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go => " + filepath.ToSlash(core) + "\n"
	if err := os.WriteFile(filepath.Join(app, "go.mod"), []byte(mod), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := run(options{Lock: filepath.Join(app, "trust-tasks.lock"), Out: filepath.Join(app, "tasks")}); err != nil {
		t.Fatal(err)
	}
	cmd := exec.Command("go", "vet", "./...")
	cmd.Dir = app
	cmd.Env = append(os.Environ(), "GOFLAGS=-mod=mod", "GOWORK=off")
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("go vet of the generated tree: %v\n%s", err, out)
	}
}

// withLock rewrites the app's lock through edit.
func withLock(t *testing.T, app string, edit func(string) string) string {
	t.Helper()
	path := filepath.Join(app, "trust-tasks.lock")
	data, _ := os.ReadFile(path)
	updated := edit(string(data))
	if updated == string(data) {
		t.Fatal("lock edit changed nothing")
	}
	if err := os.WriteFile(path, []byte(updated), 0o644); err != nil {
		t.Fatal(err)
	}
	return path
}

func TestSourceRefusals(t *testing.T) {
	expect := func(t *testing.T, lock, want string) {
		t.Helper()
		_, err := loadAndGenerate(options{Lock: lock, Out: filepath.Join(filepath.Dir(lock), "gen")})
		if err == nil || !strings.Contains(err.Error(), want) {
			t.Fatalf("got %v, want an error containing %q", err, want)
		}
	}

	t.Run("source named like an official slug's first segment", func(t *testing.T) {
		app := copyApp(t)
		lock := withLock(t, app, func(s string) string {
			s = strings.Replace(s, "[sources.acme]", "[sources.vault]", 1)
			return strings.Replace(s, `source = "acme"`, `source = "vault"`, 1)
		})
		expect(t, lock, "source `vault` has the same name as the first segment of the official task vault/get")
	})

	t.Run("framework slug from a source", func(t *testing.T) {
		app := copyApp(t)
		lock := withLock(t, app, func(s string) string {
			return strings.Replace(s, `source = "acme"
slug = "payments/settle"`, `source = "acme"
slug = "trust-task-ok"`, 1)
		})
		expect(t, lock, "trust-task-ok is a framework specification and cannot come from source `acme`")
	})

	t.Run("source with no table", func(t *testing.T) {
		app := copyApp(t)
		lock := withLock(t, app, func(s string) string {
			return strings.Replace(s, `source = "acme"`, `source = "other"`, 1)
		})
		expect(t, lock, "names source `other`, which has no [sources.other] table")
	})

	t.Run("source with both url and path", func(t *testing.T) {
		app := copyApp(t)
		lock := withLock(t, app, func(s string) string {
			return strings.Replace(s, `url = "https://tasks.example.com"`, `url = "https://tasks.example.com"
path = "../acme"`, 1)
		})
		expect(t, lock, "needs exactly one of `url` and `path`")
	})

	t.Run("sourced bundle under trusttasks.org", func(t *testing.T) {
		app := copyApp(t)
		rewriteBundle(t, app, "@acme/payments/settle", "1.0", func(b []byte) []byte {
			return bytes.Replace(b, []byte(`"typeUri":"https://tasks.example.com/`), []byte(`"typeUri":"https://trusttasks.org/`), 1)
		})
		lock := withLock(t, app, func(s string) string {
			return strings.Replace(s, `type_uri = "https://tasks.example.com/`, `type_uri = "https://trusttasks.org/`, 1)
		})
		expect(t, lock, "a task from source `acme` has the official Type URI")
	})

	t.Run("official bundle under another authority", func(t *testing.T) {
		app := copyApp(t)
		rewriteBundle(t, app, "vault/get", "0.3", func(b []byte) []byte {
			return bytes.Replace(b, []byte(`"typeUri":"https://trusttasks.org/`), []byte(`"typeUri":"https://tasks.example.com/`), 1)
		})
		lock := withLock(t, app, func(s string) string {
			return strings.Replace(s, `type_uri = "https://trusttasks.org/spec/vault/get/0.3"`, `type_uri = "https://tasks.example.com/spec/vault/get/0.3"`, 1)
		})
		expect(t, lock, "an official task's Type URI https://tasks.example.com/spec/vault/get/0.3 is not under https://trusttasks.org/")
	})
}
