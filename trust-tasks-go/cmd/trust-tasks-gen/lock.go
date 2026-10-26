package main

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"
)

// lockEntry is one `[[task]]` of trust-tasks.lock.
type lockEntry struct {
	Source  string // "" for the official registry; else a [sources.<name>] table
	Slug    string
	Version string
	TypeURI string
	Digest  string
	line    int
}

// lockFile is trust-tasks.lock.
type lockFile struct {
	Registry string
	Sources  map[string]lockSource
	Tasks    []lockEntry
}

// lockSource is a `[sources.<name>]` table: a third-party registry (SPEC §6.5
// private specifications). Its url / path tell the CLI where to fetch from;
// generation reads only the vendored bundles, so the generator just checks the
// table is well formed.
type lockSource struct {
	URL  string
	Path string
	line int
}

const lockVersion = 1

var (
	slugPattern    = regexp.MustCompile(`^[a-z0-9][a-z0-9-]*(/[a-z0-9][a-z0-9-]*)*$`)
	versionPattern = regexp.MustCompile(`^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$`)
	digestPattern  = regexp.MustCompile(`^sha256:[0-9a-f]{64}$`)
	keyPattern     = regexp.MustCompile(`^([a-z_]+)[ \t]*=[ \t]*(.*)$`)
	sourcePattern  = regexp.MustCompile(`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`)
	sourceHeader   = regexp.MustCompile(`^\[sources\.([^\]]*)\]$`)
)

// parseLock reads trust-tasks.lock.
//
// The lockfile is TOML, but only ever written by the `trust-tasks` CLI, in one
// fixed shape (trust-tasks-cli/src/lock.rs):
//
//	# Written by `trust-tasks`. Do not edit by hand.
//	version = 1
//	registry = "https://trusttasks.org"
//
//	[[task]]
//	slug = "vault/upsert"
//	version = "0.4"
//	type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//	digest = "sha256:<hex>"
//
// So this is a strict parser of exactly that shape — comments, blank lines,
// `key = value` with a basic string (or the integer `version` at the top), and
// `[[task]]` headers — rather than a TOML dependency the core module would then
// carry for everyone. Anything else is refused, never guessed at, as the CLI's
// own `deny_unknown_fields` parser refuses it.
func parseLock(text string) (*lockFile, error) {
	lock := &lockFile{Sources: map[string]lockSource{}}
	var (
		src       *lockSource
		srcName   string
		cur       *lockEntry
		seenTop   = map[string]bool{}
		seenTask  map[string]bool
		gotVer    bool
		gotReg    bool
		finishErr error
	)
	finish := func() {
		if src != nil && finishErr == nil {
			if (src.URL == "") == (src.Path == "") {
				finishErr = fmt.Errorf("line %d: [sources.%s] needs exactly one of `url` and `path`", src.line, srcName)
				return
			}
			lock.Sources[srcName] = *src
			src = nil
		}
		if cur == nil || finishErr != nil {
			return
		}
		for _, k := range []string{"slug", "version", "type_uri", "digest"} {
			if !seenTask[k] {
				finishErr = fmt.Errorf("line %d: [[task]] has no `%s`", cur.line, k)
				return
			}
		}
		lock.Tasks = append(lock.Tasks, *cur)
		cur = nil
	}

	for i, raw := range strings.Split(strings.ReplaceAll(text, "\r\n", "\n"), "\n") {
		n := i + 1
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if line == "[[task]]" {
			finish()
			if finishErr != nil {
				return nil, finishErr
			}
			cur = &lockEntry{line: n}
			seenTask = map[string]bool{}
			continue
		}
		if m := sourceHeader.FindStringSubmatch(line); m != nil {
			finish()
			if finishErr != nil {
				return nil, finishErr
			}
			if !sourcePattern.MatchString(m[1]) {
				return nil, fmt.Errorf("line %d: `%s` is not a source name ([a-z][a-z0-9]*, `-`-separated)", n, m[1])
			}
			if _, dup := lock.Sources[m[1]]; dup {
				return nil, fmt.Errorf("line %d: [sources.%s] appears twice", n, m[1])
			}
			src, srcName = &lockSource{line: n}, m[1]
			seenTask = map[string]bool{}
			continue
		}
		if strings.HasPrefix(line, "[") {
			return nil, fmt.Errorf("line %d: unexpected table %s (only [sources.<name>] and [[task]] are allowed)", n, line)
		}
		m := keyPattern.FindStringSubmatch(line)
		if m == nil {
			return nil, fmt.Errorf("line %d: not a `key = value` line: %s", n, line)
		}
		key, rawVal := m[1], strings.TrimSpace(m[2])

		if src != nil {
			if seenTask[key] {
				return nil, fmt.Errorf("line %d: `%s` appears twice in [sources.%s]", n, key, srcName)
			}
			seenTask[key] = true
			v, err := parseBasicString(rawVal)
			if err != nil {
				return nil, fmt.Errorf("line %d: %s: %w", n, key, err)
			}
			switch key {
			case "url":
				src.URL = v
			case "path":
				src.Path = v
			default:
				return nil, fmt.Errorf("line %d: unknown [sources.%s] member `%s`", n, srcName, key)
			}
			continue
		}
		if cur == nil {
			if seenTop[key] {
				return nil, fmt.Errorf("line %d: `%s` appears twice", n, key)
			}
			seenTop[key] = true
			switch key {
			case "version":
				v, err := strconv.ParseUint(stripComment(rawVal), 10, 64)
				if err != nil {
					return nil, fmt.Errorf("line %d: `version` must be an integer", n)
				}
				if v != lockVersion {
					return nil, fmt.Errorf("lockfile version %d; this trust-tasks-gen reads version %d — upgrade the trust-tasks-go module", v, lockVersion)
				}
				gotVer = true
			case "registry":
				s, err := parseBasicString(rawVal)
				if err != nil {
					return nil, fmt.Errorf("line %d: registry: %w", n, err)
				}
				lock.Registry = s
				gotReg = true
			default:
				return nil, fmt.Errorf("line %d: unknown member `%s`", n, key)
			}
			continue
		}

		if seenTask[key] {
			return nil, fmt.Errorf("line %d: `%s` appears twice in one [[task]]", n, key)
		}
		seenTask[key] = true
		s, err := parseBasicString(rawVal)
		if err != nil {
			return nil, fmt.Errorf("line %d: %s: %w", n, key, err)
		}
		switch key {
		case "source":
			cur.Source = s
		case "slug":
			cur.Slug = s
		case "version":
			cur.Version = s
		case "type_uri":
			cur.TypeURI = s
		case "digest":
			cur.Digest = s
		default:
			return nil, fmt.Errorf("line %d: unknown [[task]] member `%s`", n, key)
		}
	}
	finish()
	if finishErr != nil {
		return nil, finishErr
	}
	if !gotVer {
		return nil, fmt.Errorf("lockfile has no `version`")
	}
	if !gotReg {
		return nil, fmt.Errorf("lockfile has no `registry`")
	}

	seen := map[string]bool{}
	officialRoots := map[string]string{}
	for _, t := range lock.Tasks {
		if t.Source == "" {
			officialRoots[strings.SplitN(t.Slug, "/", 2)[0]] = t.Slug
		}
	}
	for _, t := range lock.Tasks {
		if t.Source != "" {
			if _, ok := lock.Sources[t.Source]; !ok {
				return nil, fmt.Errorf("line %d: %s names source `%s`, which has no [sources.%s] table", t.line, t.Slug, t.Source, t.Source)
			}
			if frameworkSlug.MatchString(t.Slug) {
				return nil, fmt.Errorf("line %d: %s is a framework specification and cannot come from source `%s` (SPEC §6.5 item 2)", t.line, t.Slug, t.Source)
			}
		}
		if !slugPattern.MatchString(t.Slug) {
			return nil, fmt.Errorf("line %d: `%s` is not a specification slug", t.line, t.Slug)
		}
		if !versionPattern.MatchString(t.Version) {
			return nil, fmt.Errorf("line %d: %s: `%s` is not a MAJOR.MINOR version", t.line, t.Slug, t.Version)
		}
		if !digestPattern.MatchString(t.Digest) {
			return nil, fmt.Errorf("line %d: %s %s: digest is not sha256:<64 hex>", t.line, t.Slug, t.Version)
		}
		key := t.Source + "@" + t.Slug + "/" + t.Version
		if seen[key] {
			return nil, fmt.Errorf("%s %s is locked twice", t.Label(), t.Version)
		}
		seen[key] = true
	}
	for name := range lock.Sources {
		if slug, clash := officialRoots[name]; clash {
			return nil, fmt.Errorf("source `%s` has the same name as the first segment of the official task %s; both would generate into %s/ — rename the source", name, slug, name)
		}
	}
	return lock, nil
}

// stripComment drops a trailing `# …` from a bare (unquoted) value.
func stripComment(s string) string {
	if i := strings.IndexByte(s, '#'); i >= 0 {
		s = s[:i]
	}
	return strings.TrimSpace(s)
}

// parseBasicString reads one TOML basic string ("…") and refuses anything
// after it but a comment.
func parseBasicString(s string) (string, error) {
	if !strings.HasPrefix(s, `"`) {
		return "", fmt.Errorf("expected a quoted string")
	}
	var b strings.Builder
	i := 1
	for i < len(s) {
		c := s[i]
		switch {
		case c == '"':
			if rest := stripComment(s[i+1:]); rest != "" {
				return "", fmt.Errorf("unexpected %q after the string", rest)
			}
			return b.String(), nil
		case c == '\\':
			if i+1 >= len(s) {
				return "", fmt.Errorf("unterminated escape")
			}
			e := s[i+1]
			i += 2
			switch e {
			case '"':
				b.WriteByte('"')
			case '\\':
				b.WriteByte('\\')
			case 'b':
				b.WriteByte('\b')
			case 't':
				b.WriteByte('\t')
			case 'n':
				b.WriteByte('\n')
			case 'f':
				b.WriteByte('\f')
			case 'r':
				b.WriteByte('\r')
			case 'u', 'U':
				width := 4
				if e == 'U' {
					width = 8
				}
				if i+width > len(s) {
					return "", fmt.Errorf("short \\%c escape", e)
				}
				cp, err := strconv.ParseUint(s[i:i+width], 16, 32)
				if err != nil {
					return "", fmt.Errorf("bad \\%c escape", e)
				}
				b.WriteRune(rune(cp))
				i += width
			default:
				return "", fmt.Errorf("unknown escape \\%c", e)
			}
		case c < 0x20 && c != '\t':
			return "", fmt.Errorf("control character in string")
		default:
			b.WriteByte(c)
			i++
		}
	}
	return "", fmt.Errorf("unterminated string")
}

// Label is how messages name the task: `acme:payments/settle` for a sourced
// one, the bare slug for an official one.
func (e lockEntry) Label() string {
	if e.Source == "" {
		return e.Slug
	}
	return e.Source + ":" + e.Slug
}
