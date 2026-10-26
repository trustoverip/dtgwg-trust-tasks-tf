package main

// Schema → Go type emission.
//
// An independent port of the emitter in scripts/build-go-bindings.mjs, which
// generates the prebuilt packages under trust-tasks-go/specs. The repository
// keeps its generators independent on purpose (CLAUDE.md: "Do not factor the
// generators together"), and this one is held to that one by the equivalence
// test in equivalence_test.go rather than by sharing code with it.
//
// The rules it ports, briefly (the .mjs explains each at length):
//
//   - No component hoisting: every package inlines what it reaches. Go is
//     nominally typed, so a shared VaultEntry would merge type identities.
//   - Optional members are pointers — including slices and maps, because
//     `omitempty` on a bare slice collapses "present and empty" into "absent",
//     which `acl`'s allowedKeys documents as a privilege-escalation defect.
//     json.RawMessage is the one exception: an explicit null survives in it.
//   - A closed set of strings is a named string type with one constant per
//     value; any other union is json.RawMessage, because Go has no sum type.
//   - The types enforce no constraint; PayloadSchemaJSON carries them.

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

// initialisms are segments conventionally written all-caps in Go.
var initialisms = map[string]bool{}

func init() {
	for _, s := range strings.Fields(`acl api ascii cpu css did dns eof guid html
		http https id ip json jwe jwk jws jwt lhs mac ok otp qps ram rhs rpc sdk sha
		sla smtp sql ssh tcp tls ttl udp uid uri url usb utc uuid vc vid vm vp xml
		xsrf xss`) {
		initialisms[s] = true
	}
}

var (
	camelBoundary   = regexp.MustCompile(`([a-z0-9])([A-Z])`)
	acronymBoundary = regexp.MustCompile(`([A-Z]+)([A-Z][a-z])`)
	nonAlnum        = regexp.MustCompile(`[^A-Za-z0-9]+`)
)

// segments splits a schema name into lowercase word segments, breaking
// camelCase, PascalCase and acronym runs (`DIDDoc` → did, doc).
func segments(name string) []string {
	s := camelBoundary.ReplaceAllString(name, "$1 $2")
	s = acronymBoundary.ReplaceAllString(s, "$1 $2")
	var out []string
	for _, part := range nonAlnum.Split(s, -1) {
		if part != "" {
			out = append(out, strings.ToLower(part))
		}
	}
	return out
}

// goIdent is an exported Go identifier for name. A leading digit gets an `N`.
func goIdent(name string) string {
	var b strings.Builder
	for _, s := range segments(name) {
		if initialisms[s] {
			b.WriteString(strings.ToUpper(s))
		} else {
			b.WriteString(strings.ToUpper(s[:1]) + s[1:])
		}
	}
	ident := b.String()
	if ident == "" {
		return "X"
	}
	if ident[0] >= '0' && ident[0] <= '9' {
		return "N" + ident
	}
	return ident
}

// goPackageName is the lowercase, underscore-free package name for a slug at a
// version: acl/grant 0.1 → aclgrantv0_1.
func goPackageName(slug, version string) string {
	var b strings.Builder
	for _, part := range strings.Split(slug, "/") {
		for _, s := range segments(part) {
			b.WriteString(s)
		}
	}
	return b.String() + "v" + strings.ReplaceAll(version, ".", "_")
}

// versionDir is the on-disk directory for a version: 0.1 → v0_1.
func versionDir(version string) string {
	return "v" + strings.ReplaceAll(version, ".", "_")
}

// goString quotes s as a Go interpreted string literal, raw UTF-8 for
// everything printable and escapes only where Go requires them.
func goString(s string) string {
	var b strings.Builder
	b.WriteByte('"')
	for _, r := range s {
		switch {
		case r == '"':
			b.WriteString(`\"`)
		case r == '\\':
			b.WriteString(`\\`)
		case r == '\n':
			b.WriteString(`\n`)
		case r == '\r':
			b.WriteString(`\r`)
		case r == '\t':
			b.WriteString(`\t`)
		case r < 0x20 || r == 0x7f:
			fmt.Fprintf(&b, `\x%02x`, r)
		default:
			b.WriteRune(r)
		}
	}
	b.WriteByte('"')
	return b.String()
}

// isJSSpace is JavaScript's `\s` (and String.prototype.trim's) whitespace set,
// which is not unicode.IsSpace's: it includes U+FEFF and excludes U+0085.
func isJSSpace(r rune) bool {
	switch r {
	case '\t', '\n', '\v', '\f', '\r', ' ', 0xa0, 0xfeff, 0x2028, 0x2029:
		return true
	}
	return unicode.Is(unicode.Zs, r)
}

func jsTrim(s string) string { return strings.TrimFunc(s, isJSSpace) }

// utf16Len is a JavaScript string's `.length`, which the doc-comment wrap
// width is measured in.
func utf16Len(s string) int {
	n := 0
	for _, r := range s {
		if r >= 0x10000 {
			n += 2 // a surrogate pair
		} else {
			n++
		}
	}
	return n
}

// docComment renders text as a Go doc comment on name, wrapped at 88 columns.
// A non-string or blank text gives no comment at all.
func docComment(name string, text any, indent string) []string {
	s, ok := text.(string)
	if !ok || jsTrim(s) == "" {
		return nil
	}
	collapsed := strings.Join(strings.FieldsFunc(s, isJSSpace), " ")
	words := strings.Split(jsTrim(collapsed), " ")
	var lines []string
	line := ""
	if name != "" {
		line = name + " "
	}
	for _, word := range words {
		if utf16Len(line)+utf16Len(word)+1 > 88 && jsTrim(line) != name {
			lines = append(lines, strings.TrimRightFunc(line, isJSSpace))
			line = ""
		}
		line += word + " "
	}
	if jsTrim(line) != "" {
		lines = append(lines, strings.TrimRightFunc(line, isJSSpace))
	}
	out := make([]string, len(lines))
	for i, l := range lines {
		out[i] = indent + "// " + l
	}
	return out
}

// firstPresent is `a ?? b ?? …`: the first value that is neither absent nor
// null, or nil.
func firstPresent(vals ...any) any {
	for _, v := range vals {
		if v != nil {
			return v
		}
	}
	return nil
}

// emitter is one package under construction.
type emitter struct {
	decls     []string
	names     map[string]bool
	needsJSON bool
}

func newEmitter() *emitter { return &emitter{names: map[string]bool{}} }

// reserve takes preferred as a type name, suffixing 2, 3, … on a collision.
func (em *emitter) reserve(preferred string) string {
	name := preferred
	for n := 2; em.names[name]; n++ {
		name = fmt.Sprintf("%s%d", preferred, n)
	}
	em.names[name] = true
	return name
}

// mergeAllOf folds `allOf` branches that are plain object schemas into the
// node. A branch it cannot absorb leaves the node alone.
func mergeAllOf(node *object) *object {
	branches, ok := node.val("allOf").([]any)
	if !ok {
		return node
	}
	merged := node.clone()
	merged.del("allOf")
	for _, b := range branches {
		if !truthy(b) {
			return node
		}
		var branch *object
		switch t := b.(type) {
		case *object:
			branch = t
		case []any:
			branch = newObject()
		default:
			return node
		}
		if _, has := branch.get("$ref"); has {
			return node
		}
		props := newObject()
		if p, ok := merged.val("properties").(*object); ok {
			for _, k := range p.jsKeys() {
				props.set(k, p.vals[k])
			}
		}
		if p, ok := branch.val("properties").(*object); ok {
			for _, k := range p.jsKeys() {
				props.set(k, p.vals[k])
			}
		}
		merged.set("properties", props)

		var req []any
		seen := map[string]bool{}
		for _, list := range []any{merged.val("required"), branch.val("required")} {
			arr, _ := list.([]any)
			for _, r := range arr {
				key := fmt.Sprintf("%T:%v", r, r)
				if !seen[key] {
					seen[key] = true
					req = append(req, r)
				}
			}
		}
		if req == nil {
			req = []any{}
		}
		merged.set("required", req)

		for _, k := range []string{"type", "description"} {
			if v, has := branch.get(k); has && merged.val(k) == nil {
				merged.set(k, v)
			}
		}
	}
	return merged
}

// typeOf is a node's declared JSON types with "null" removed, and whether
// "null" was among them.
func typeOf(node any) (types []any, nullable bool) {
	o, ok := node.(*object)
	if !ok {
		return nil, false
	}
	raw, has := o.get("type")
	if !has {
		return nil, false
	}
	list, isList := raw.([]any)
	if !isList {
		list = []any{raw}
	}
	for _, t := range list {
		if t == "null" {
			nullable = true
		} else {
			types = append(types, t)
		}
	}
	return types, nullable
}

// goTypeFor is the Go type for node, declaring any named types it needs.
// hint names a newly declared type; defNames maps `#/$defs/<name>` to the Go
// name already reserved for it.
func (em *emitter) goTypeFor(node any, hint string, defNames map[string]string) string {
	raw := func() string {
		em.needsJSON = true
		return "json.RawMessage"
	}
	o, ok := node.(*object)
	if !ok {
		return raw()
	}
	if ref, ok := o.val("$ref").(string); ok {
		if name, ok := defNames[ref]; ok {
			return name
		}
		return raw()
	}

	o = mergeAllOf(o)
	types, _ := typeOf(o)

	if len(types) > 1 || (len(types) == 0 && (truthy(o.val("oneOf")) || truthy(o.val("anyOf")) || truthy(o.val("not")))) {
		branches := firstPresent(o.val("oneOf"), o.val("anyOf"))
		if list, ok := branches.([]any); ok && len(types) == 0 && len(list) > 0 {
			allStrings := true
			for _, b := range list {
				bo, ok := b.(*object)
				if !ok || !(bo.val("type") == "string" || truthy(bo.val("enum")) || truthy(bo.val("pattern"))) {
					allStrings = false
					break
				}
			}
			if allStrings {
				return "string"
			}
		}
		return raw()
	}

	var typ any
	if len(types) == 1 {
		typ = types[0]
	}
	switch typ {
	case "string":
		if values, ok := o.val("enum").([]any); ok && len(values) > 0 {
			allStrings := true
			for _, v := range values {
				if _, ok := v.(string); !ok {
					allStrings = false
					break
				}
			}
			if allStrings {
				return em.declareEnum(o, values, hint)
			}
		}
		return "string"
	case "integer":
		return "int64"
	case "number":
		return "float64"
	case "boolean":
		return "bool"
	case "array":
		items, has := o.get("items")
		if !has || items == nil {
			items = true
		}
		return "[]" + em.goTypeFor(items, hint+"Item", defNames)
	case "object":
		props, _ := o.val("properties").(*object)
		if props == nil || len(props.keys) == 0 {
			if ap := o.val("additionalProperties"); truthy(ap) {
				switch ap.(type) {
				case *object, []any:
					return "map[string]" + em.goTypeFor(ap, hint+"Value", defNames)
				}
			}
			em.needsJSON = true
			return "map[string]json.RawMessage"
		}
		return em.declareStruct(o, props, hint, defNames)
	}
	return raw()
}

// declareEnum declares a named string type with one constant per value.
func (em *emitter) declareEnum(node *object, values []any, hint string) string {
	name := em.reserve(hint)
	var lines []string
	lines = append(lines, docComment(name, firstPresent(node.val("description"), name+" is a closed set of string values."), "")...)
	lines = append(lines, "type "+name+" string", "", "// Values "+name+" may take, per this specification's schema.", "const (")
	seen := map[string]bool{}
	for _, v := range values {
		s := v.(string)
		constName := name + goIdent(s)
		for seen[constName] {
			constName += "_"
		}
		seen[constName] = true
		lines = append(lines, "\t"+constName+" "+name+" = "+goString(s))
	}
	lines = append(lines, ")")
	em.decls = append(em.decls, strings.Join(lines, "\n"))
	return name
}

type field struct {
	name, typ, tag string
	doc            any
}

// declareStruct declares a named struct for an object schema.
func (em *emitter) declareStruct(node, props *object, hint string, defNames map[string]string) string {
	name := em.reserve(hint)
	required := map[string]bool{}
	if req, ok := node.val("required").([]any); ok {
		for _, r := range req {
			if s, ok := r.(string); ok {
				required[s] = true
			}
		}
	}
	var fields []field
	fieldNames := map[string]bool{}
	for _, wireName := range props.jsKeys() {
		member := props.vals[wireName]
		fieldName := goIdent(wireName)
		for fieldNames[fieldName] {
			fieldName += "_"
		}
		fieldNames[fieldName] = true

		isRequired := required[wireName]
		memberNode := member
		if member == true {
			memberNode = newObject()
		}
		_, nullable := typeOf(memberNode)
		goType := em.goTypeFor(member, name+fieldName, defNames)

		// Optional → pointer, slices and maps included; json.RawMessage
		// already distinguishes absent from null.
		if (!isRequired || nullable) && goType != "json.RawMessage" {
			goType = "*" + goType
		}
		tag := wireName
		if !isRequired || nullable {
			tag += ",omitempty"
		}
		var doc any
		if mo, ok := memberNode.(*object); ok {
			doc = mo.val("description")
		}
		fields = append(fields, field{name: fieldName, typ: goType, tag: "`json:" + goString(tag) + "`", doc: doc})
	}

	var lines []string
	lines = append(lines, docComment(name, firstPresent(node.val("description"), node.val("title"), name+" is a generated payload type."), "")...)
	lines = append(lines, "type "+name+" struct {")
	for i, f := range fields {
		doc := docComment("", f.doc, "\t")
		if i > 0 && len(doc) > 0 {
			lines = append(lines, "")
		}
		lines = append(lines, doc...)
		lines = append(lines, "\t"+f.name+" "+f.typ+" "+f.tag)
	}
	lines = append(lines, "}")
	em.decls = append(em.decls, strings.Join(lines, "\n"))
	return name
}

// emitNamed emits node under the already reserved name target.
func (em *emitter) emitNamed(node any, target string, defNames map[string]string) string {
	delete(em.names, target)
	produced := em.goTypeFor(node, target, defNames)
	em.names[target] = true
	return produced
}

// emitDeclarations declares a type for every `$defs` entry, then the root
// `Payload`. Names are reserved before any body is emitted, so a `$def` that
// names a sibling resolves whatever order they come in.
//
// The order the `$defs` come in is the one thing a bundle cannot reproduce: its
// schema is key-sorted, and `propertyOrder` restores the order of `properties`
// only. So the declarations here follow `$defs` in sorted order, where the
// prebuilt packages follow the authored order — the same declarations, in a
// different sequence (see equivalence_test.go).
func (em *emitter) emitDeclarations(schema *object, rootHint string) (map[string]string, string) {
	defs, _ := schema.val("$defs").(*object)
	defNames := map[string]string{}
	for _, defName := range defs.jsKeys() {
		defNames["#/$defs/"+defName] = em.reserve(goIdent(defName))
	}
	rootName := em.reserve(rootHint)

	for _, defName := range defs.jsKeys() {
		defNode := defs.vals[defName]
		target := defNames["#/$defs/"+defName]
		produced := em.emitNamed(defNode, target, defNames)
		if produced != target {
			var desc any
			if do, ok := defNode.(*object); ok {
				desc = do.val("description")
			}
			lines := docComment(target, firstPresent(desc, target+" is defined by this specification's schema."), "")
			em.decls = append(em.decls, strings.Join(append(lines, "type "+target+" = "+produced), "\n"))
		}
	}

	produced := em.emitNamed(schema, rootName, defNames)
	if produced != rootName {
		lines := docComment(rootName, firstPresent(schema.val("description"), rootName+" is this specification's payload."), "")
		em.decls = append(em.decls, strings.Join(append(lines, "type "+rootName+" = "+produced), "\n"))
	}
	return defNames, rootName
}

// markDeprecated adds a `// Deprecated: <note>` paragraph to the declaration of
// name.
func markDeprecated(decls []string, name, note string) {
	for i, d := range decls {
		lines := strings.Split(d, "\n")
		at := -1
		for j, l := range lines {
			if strings.HasPrefix(l, "type "+name+" ") {
				at = j
				break
			}
		}
		if at < 0 {
			continue
		}
		var insert []string
		if at > 0 && strings.HasPrefix(lines[at-1], "//") {
			insert = append(insert, "//")
		}
		insert = append(insert, "// Deprecated: "+note)
		lines = append(lines[:at], append(insert, lines[at:]...)...)
		decls[i] = strings.Join(lines, "\n")
		return
	}
}
