package trusttasks

// Forward-minor routing along a declared extension (SPEC §5.2 "Declared
// extension").
//
// A specification version may declare `extends: "<M.K>"`: it keeps everything
// version M.K defines and only adds to it, and the registry build verifies that
// promise. A consumer implementing M.K SHOULD then accept a document of the
// extending version by ignoring the payload members it does not recognise.
// Without a declaration it SHOULD NOT — a draft may break in a MINOR.
//
// The point of carrying this in the library is that an upgrade (a patch release)
// carries the knowledge that x/0.3 extends x/0.2, so an application that still
// implements only x/0.2 starts accepting x/0.3 documents without a code change:
//
//	reading, refusal, err := trusttasks.ReadForSpec(received, xv0_2.Spec)
//	if err != nil { /* not a Trust Task document at all */ }
//	var raw trusttasks.Document[json.RawMessage]
//	_ = json.Unmarshal(received, &raw) // for routing a refusal, as today
//	if refusal != nil { emit(trusttasks.Reject(transport, &raw, newID(), *refusal, nil)) }
//	var doc trusttasks.Document[xv0_2.Payload]
//	if err := json.Unmarshal(reading, &doc); err != nil { /* malformedRequest */ }
//	outcome, err := trusttasks.ConsumeInbound(ctx, trusttasks.ConsumeOptions[...]{
//	    Spec:     xv0_2.Spec,
//	    Doc:      &doc,
//	    Received: received, // the proof is verified over THESE bytes, never `reading`
//	    ...
//	})
//
// The pipeline answers under the received document's own type (x/0.3#response):
// [RespondWith] and the error builders already take the type from the document,
// and ReadForSpec leaves it untouched. A response shaped for x/0.2 is valid for
// x/0.3, because x/0.3 only added to it.

import (
	"bytes"
	"encoding/json"
	"fmt"
	"reflect"
	"regexp"
	"strings"

	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs"
)

// ExtendsLookup returns the MAJOR.MINOR that the specification version named by
// a Type URI declares it extends, or "" when it declares none. A fragment on the
// argument is to be ignored.
//
// [RegistryExtends] is the one backed by the generated registry index. The type
// exists so a test, or a deployment pinning its own view of the registry, can
// supply another.
type ExtendsLookup func(typeURI string) string

// RegistryExtends is the [ExtendsLookup] over the generated registry index
// (package specs, regenerated with the specification packages), so it reflects
// every specification this release of the library knows — including versions
// whose packages the application never imports.
func RegistryExtends(typeURI string) string { return specs.ExtendsOf(typeURI) }

// maxExtendsChain bounds a chain walk, so a lookup that loops cannot hang the
// consumer. Real chains are a handful of MINORs long.
const maxExtendsChain = 64

// UnsupportedVersionWireMessage is the message [ReadForSpec] refuses a document
// with when its version is neither the implemented one nor declared to extend it.
const UnsupportedVersionWireMessage = "this consumer does not implement the document's " +
	"specification version, and that version declares no extension of one it does (SPEC §5.2)"

// UnsupportedTypeWireMessage is the message [ReadForSpec] refuses a document with
// when it names a different specification from the one implemented.
const UnsupportedTypeWireMessage = "document type is not the specification this consumer implements"

// routingKey reads a Type URI for routing: its routing variant collapses
// #request into no fragment (SPEC §4.4.1).
func routingKey(typeURI string) (TypeURI, error) {
	parsed, err := ParseTypeURI(typeURI)
	if err != nil {
		return TypeURI{}, err
	}
	if parsed.Variant() == VariantRequest {
		parsed = parsed.WithVariant(VariantNone)
	}
	return parsed, nil
}

// sameRoute reports whether two routing keys name the same specification (same
// authority, form and slug — §6.1) and the same variant.
func sameRoute(a, b TypeURI) bool {
	return a.SameSpecification(b) && a.Variant() == b.Variant()
}

// ExtendsChainReaches reports whether the specification version named by
// receivedTypeURI is the one named by implementedTypeURI, or declares — directly
// or along a chain of declarations, per lookup — that it extends it.
//
// Both may be in any SPEC §6.1 form, and must name the same specification — the
// same authority, form and slug, since the same slug under two authorities is
// two specifications — and the same variant. Each link of the chain is looked up
// under the received URI's own authority and form, so a chain never crosses to
// another authority. A nil lookup means [RegistryExtends].
func ExtendsChainReaches(receivedTypeURI, implementedTypeURI string, lookup ExtendsLookup) bool {
	received, err := routingKey(receivedTypeURI)
	if err != nil {
		return false
	}
	implemented, err := routingKey(implementedTypeURI)
	if err != nil || !sameRoute(received, implemented) {
		return false
	}
	return chainReaches(received, implemented, lookup)
}

// chainReaches walks from's declared-extension chain, looking each link up as a
// bare sibling of from (only the version changed), until it reaches to's version.
func chainReaches(from, to TypeURI, lookup ExtendsLookup) bool {
	if lookup == nil {
		lookup = RegistryExtends
	}
	at := from.Bare()
	for i := 0; i <= maxExtendsChain; i++ {
		if at.Major() == to.Major() && at.Minor() == to.Minor() {
			return true
		}
		major, minor, ok := parseVersion(lookup(at.String()))
		if !ok {
			return false // no declaration, or not a MAJOR.MINOR
		}
		at = at.WithVersion(major, minor)
	}
	return false
}

// ReadForSpec re-reads a received document for the specification version the
// application implements — spec, a generated package's Spec (or ResponseSpec) —
// along the registry's declared extensions. See [ReadForSpecWith].
func ReadForSpec(received json.RawMessage, spec SpecPolicy) (json.RawMessage, *RejectReason, error) {
	return ReadForSpecWith(received, spec, RegistryExtends)
}

// ReadForSpecWith is [ReadForSpec] with an explicit [ExtendsLookup] (nil means
// [RegistryExtends]).
//
// It returns exactly one of:
//
//   - the document to decode as spec's version. For a document of that exact
//     version this is received itself, byte for byte — nothing changes. For a
//     document of a later version whose extends chain reaches spec's, it is the
//     document with its payload pruned to the members spec's PayloadSchema knows
//     (see below), so the typed decode and §7.2 item 2 both see a payload of the
//     implemented version. Every other member — the type included — is kept, so
//     the response is typed as the received version.
//   - a refusal: unsupportedVersion when the received version's chain does not
//     reach spec's (an undeclared later MINOR, an earlier version, another
//     MAJOR); unsupportedType when the document names another specification —
//     another slug, or the same slug under another authority or form (§6.1) —
//     or another variant. Route it with [Reject] over the document as received.
//     Every §6.1 form is read, so a third-party (§6.5) specification routes
//     along its own chain, never to a registry specification sharing its slug.
//   - an error, when received is not a JSON object with a string type, or spec's
//     TypeURI is not a Trust Task Type URI — there is no document to answer.
//
// Proofs: the returned reading is NOT the signed document, and nothing derived
// from it may be handed to a proof verifier. Pass received as
// [ConsumeOptions.Received]; [ConsumeInbound] then verifies over those bytes, and
// checks that the decoded document is a reading of them.
//
// Pruning walks the payload alongside the schema: an object member is dropped
// where the schema in force has `additionalProperties: false` (or
// `unevaluatedProperties: false`) and neither names it in `properties` nor
// matches it with `patternProperties`; it recurses through `properties`,
// `additionalProperties`, `patternProperties`, `items` and `prefixItems`,
// follows local `$ref`s into the `$defs` the schema embeds, and applies every
// `allOf` branch. For a `oneOf` / `anyOf` it prunes against the one branch the
// value can plausibly match — by type, `const` / `enum`, required members and
// `const` / `enum` discriminator members — and leaves the value untouched when
// that is not exactly one branch: an unsure prune could delete what the right
// branch needs, whereas an unpruned value is refused by item 2, which is the
// conservative failure. Number literals are carried through verbatim.
func ReadForSpecWith(
	received json.RawMessage,
	spec SpecPolicy,
	lookup ExtendsLookup,
) (json.RawMessage, *RejectReason, error) {
	var members map[string]json.RawMessage
	if err := json.Unmarshal(received, &members); err != nil {
		return nil, nil, fmt.Errorf("trusttasks: received document is not a JSON object: %w", err)
	}
	var typeURI string
	if err := json.Unmarshal(members["type"], &typeURI); err != nil || typeURI == "" {
		return nil, nil, fmt.Errorf("trusttasks: received document carries no string type")
	}
	implemented, err := routingKey(spec.TypeURI)
	if err != nil {
		return nil, nil, fmt.Errorf("trusttasks: spec.TypeURI: %w", err)
	}
	receivedType, err := routingKey(typeURI)
	if err != nil || !sameRoute(receivedType, implemented) {
		return nil, &RejectReason{Code: CodeUnsupportedType, Message: UnsupportedTypeWireMessage}, nil
	}
	if receivedType.Major() == implemented.Major() && receivedType.Minor() == implemented.Minor() {
		return received, nil, nil
	}
	if !chainReaches(receivedType, implemented, lookup) {
		return nil, &RejectReason{Code: CodeUnsupportedVersion, Message: UnsupportedVersionWireMessage}, nil
	}

	payload, present := members["payload"]
	if !present || spec.PayloadSchema == "" {
		// Nothing to prune against (a hand-written policy without a schema), or no
		// payload at all — which the typed decode and item 2 will refuse in their
		// own terms.
		return received, nil, nil
	}
	var schema any
	if err := json.Unmarshal([]byte(spec.PayloadSchema), &schema); err != nil {
		return nil, nil, fmt.Errorf("trusttasks: spec.PayloadSchema is not JSON: %w", err)
	}
	value, err := decodeNumbers(payload)
	if err != nil {
		return nil, nil, fmt.Errorf("trusttasks: received payload is not JSON: %w", err)
	}
	p := pruner{root: schema}
	pruned, err := json.Marshal(p.prune(value, schema, 0))
	if err != nil {
		return nil, nil, fmt.Errorf("trusttasks: re-encode pruned payload: %w", err)
	}
	members["payload"] = pruned
	reading, err := json.Marshal(members)
	if err != nil {
		return nil, nil, fmt.Errorf("trusttasks: re-encode pruned document: %w", err)
	}
	return reading, nil, nil
}

// decodeNumbers decodes JSON keeping every number as its json.Number literal, so
// a pruned payload re-encodes its numbers exactly as they arrived.
func decodeNumbers(data []byte) (any, error) {
	decoder := json.NewDecoder(bytes.NewReader(data))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return nil, err
	}
	return value, nil
}

// maxPruneDepth bounds schema recursion that consumes no value — a `$ref` cycle.
const maxPruneDepth = 256

type pruner struct {
	root any
}

// resolve follows a local JSON Pointer `$ref` ("#/$defs/X") within the root
// schema. Anything else is reported unresolved, and the caller then leaves the
// value alone.
func (p *pruner) resolve(ref string) (any, bool) {
	if !strings.HasPrefix(ref, "#") {
		return nil, false
	}
	node := p.root
	pointer := strings.TrimPrefix(ref, "#")
	if pointer == "" {
		return node, true
	}
	if !strings.HasPrefix(pointer, "/") {
		return nil, false
	}
	for _, token := range strings.Split(pointer[1:], "/") {
		token = strings.ReplaceAll(strings.ReplaceAll(token, "~1", "/"), "~0", "~")
		object, ok := node.(map[string]any)
		if !ok {
			return nil, false
		}
		if node, ok = object[token]; !ok {
			return nil, false
		}
	}
	return node, true
}

// prune returns value with every member schema does not admit removed.
func (p *pruner) prune(value, schema any, depth int) any {
	node, ok := schema.(map[string]any)
	if !ok || depth > maxPruneDepth {
		return value // `true`, `false`, or unbounded: nothing to learn
	}

	// Each applicator applies independently, as validation does, so applying
	// them one after another prunes to what all of them admit.
	if ref, ok := node["$ref"].(string); ok {
		if target, found := p.resolve(ref); found {
			value = p.prune(value, target, depth+1)
		}
	}
	if branches, ok := node["allOf"].([]any); ok {
		for _, branch := range branches {
			value = p.prune(value, branch, depth+1)
		}
	}
	var chosen any
	for _, keyword := range []string{"oneOf", "anyOf"} {
		branches, ok := node[keyword].([]any)
		if !ok {
			continue
		}
		var candidates []any
		for _, branch := range branches {
			if p.plausible(value, branch, depth+1) {
				candidates = append(candidates, branch)
			}
		}
		if len(candidates) == 1 {
			chosen = candidates[0]
			value = p.prune(value, chosen, depth+1)
		}
	}

	switch v := value.(type) {
	case map[string]any:
		return p.pruneObject(v, node, chosen, depth)
	case []any:
		return p.pruneArray(v, node, depth)
	}
	return value
}

func (p *pruner) pruneObject(object map[string]any, node map[string]any, chosen any, depth int) any {
	properties, _ := node["properties"].(map[string]any)
	patterns, _ := node["patternProperties"].(map[string]any)
	additional, hasAdditional := node["additionalProperties"]

	out := make(map[string]any, len(object))
	for name, member := range object {
		if sub, ok := properties[name]; ok {
			out[name] = p.prune(member, sub, depth+1)
			continue
		}
		if sub, ok := matchPattern(patterns, name); ok {
			out[name] = p.prune(member, sub, depth+1)
			continue
		}
		if hasAdditional {
			if allowed, isBool := additional.(bool); isBool && !allowed {
				continue // the schema in force does not know this member
			}
			out[name] = p.prune(member, additional, depth+1)
			continue
		}
		out[name] = member
	}

	if closed, ok := node["unevaluatedProperties"].(bool); ok && !closed {
		for name := range out {
			if !p.evaluates(node, name, depth) && !(chosen != nil && p.evaluates(chosen, name, depth)) {
				delete(out, name)
			}
		}
	}
	return out
}

// evaluates approximates whether schema (with its `$ref` and `allOf`) evaluates
// the member name, for `unevaluatedProperties`.
func (p *pruner) evaluates(schema any, name string, depth int) bool {
	node, ok := schema.(map[string]any)
	if !ok || depth > maxPruneDepth {
		return schema == true
	}
	if properties, ok := node["properties"].(map[string]any); ok {
		if _, ok := properties[name]; ok {
			return true
		}
	}
	if patterns, ok := node["patternProperties"].(map[string]any); ok {
		if _, ok := matchPattern(patterns, name); ok {
			return true
		}
	}
	if additional, ok := node["additionalProperties"]; ok && additional != false {
		return true
	}
	if ref, ok := node["$ref"].(string); ok {
		if target, found := p.resolve(ref); found && p.evaluates(target, name, depth+1) {
			return true
		}
	}
	if branches, ok := node["allOf"].([]any); ok {
		for _, branch := range branches {
			if p.evaluates(branch, name, depth+1) {
				return true
			}
		}
	}
	return false
}

// matchPattern returns the subschema of the first patternProperties entry
// matching name. A pattern Go's regexp cannot compile (ECMA-262 has constructs
// RE2 lacks) is treated as matching, so its member is kept rather than dropped.
func matchPattern(patterns map[string]any, name string) (any, bool) {
	for pattern, sub := range patterns {
		re, err := regexp.Compile(pattern)
		if err != nil {
			return true, true
		}
		if re.MatchString(name) {
			return sub, true
		}
	}
	return nil, false
}

func (p *pruner) pruneArray(array []any, node map[string]any, depth int) any {
	prefix, _ := node["prefixItems"].([]any)
	items, hasItems := node["items"]
	out := make([]any, len(array))
	for i, item := range array {
		switch {
		case i < len(prefix):
			out[i] = p.prune(item, prefix[i], depth+1)
		case hasItems:
			out[i] = p.prune(item, items, depth+1)
		default:
			out[i] = item
		}
	}
	return out
}

// plausible reports whether value could match schema, judging only what decides
// a oneOf / anyOf branch in practice: the JSON type, `const` / `enum`, the
// required members, and members whose subschema is a `const` / `enum`
// discriminator. Members the branch does not know do not disqualify it — they
// are exactly what is being pruned.
func (p *pruner) plausible(value, schema any, depth int) bool {
	node, ok := schema.(map[string]any)
	if !ok || depth > maxPruneDepth {
		return schema != false
	}
	if ref, ok := node["$ref"].(string); ok {
		if target, found := p.resolve(ref); found && !p.plausible(value, target, depth+1) {
			return false
		}
	}
	if branches, ok := node["allOf"].([]any); ok {
		for _, branch := range branches {
			if !p.plausible(value, branch, depth+1) {
				return false
			}
		}
	}
	if declared, ok := node["type"]; ok && !typeMatches(value, declared) {
		return false
	}
	if constant, ok := node["const"]; ok && !jsonEqual(value, constant) {
		return false
	}
	if enum, ok := node["enum"].([]any); ok {
		found := false
		for _, candidate := range enum {
			if jsonEqual(value, candidate) {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}
	if object, ok := value.(map[string]any); ok {
		if required, ok := node["required"].([]any); ok {
			for _, name := range required {
				if s, ok := name.(string); ok {
					if _, present := object[s]; !present {
						return false
					}
				}
			}
		}
		if properties, ok := node["properties"].(map[string]any); ok {
			for name, sub := range properties {
				member, present := object[name]
				subNode, isObject := sub.(map[string]any)
				if !present || !isObject {
					continue
				}
				_, hasConst := subNode["const"]
				_, hasEnum := subNode["enum"]
				if (hasConst || hasEnum) && !p.plausible(member, subNode, depth+1) {
					return false
				}
			}
		}
	}
	return true
}

// typeMatches reports whether value is of the JSON Schema `type` declared.
func typeMatches(value, declared any) bool {
	var names []string
	switch d := declared.(type) {
	case string:
		names = []string{d}
	case []any:
		for _, n := range d {
			if s, ok := n.(string); ok {
				names = append(names, s)
			}
		}
	default:
		return true
	}
	for _, name := range names {
		switch v := value.(type) {
		case nil:
			if name == "null" {
				return true
			}
		case bool:
			if name == "boolean" {
				return true
			}
		case string:
			if name == "string" {
				return true
			}
		case json.Number:
			if name == "number" {
				return true
			}
			if name == "integer" {
				if _, err := v.Int64(); err == nil {
					return true
				}
				if f, err := v.Float64(); err == nil && f == float64(int64(f)) {
					return true
				}
			}
		case map[string]any:
			if name == "object" {
				return true
			}
		case []any:
			if name == "array" {
				return true
			}
		}
	}
	return false
}

// jsonEqual compares two decoded JSON values, numbers by value.
func jsonEqual(a, b any) bool {
	ea, errA := json.Marshal(a)
	eb, errB := json.Marshal(b)
	if errA != nil || errB != nil {
		return false
	}
	var va, vb any
	if json.Unmarshal(ea, &va) != nil || json.Unmarshal(eb, &vb) != nil {
		return false
	}
	return reflect.DeepEqual(va, vb)
}

// isReadingOf reports whether doc is a reading of received: every envelope
// member equal, and a payload whose every member appears, with the same value,
// in the received payload. That is what [ReadForSpec] produces — and what a
// plain typed decode of received produces — so a proof over received vouches for
// doc. A doc that is not a reading of received has nothing vouching for it.
func isReadingOf[P any](doc *Document[P], received json.RawMessage) bool {
	var raw Document[json.RawMessage]
	if err := json.Unmarshal(received, &raw); err != nil {
		return false
	}
	envelope := func(d Document[json.RawMessage]) (any, bool) {
		d.Payload = nil
		encoded, err := json.Marshal(d)
		if err != nil {
			return nil, false
		}
		v, err := decodeNumbers(encoded)
		return v, err == nil
	}
	typed := Document[json.RawMessage]{
		ID: doc.ID, ThreadID: doc.ThreadID, ParentThreadID: doc.ParentThreadID,
		Ceremony: doc.Ceremony, Type: doc.Type, Issuer: doc.Issuer, Recipient: doc.Recipient,
		IssuedAt: doc.IssuedAt, ExpiresAt: doc.ExpiresAt, Context: doc.Context,
		Proof: doc.Proof, Extra: doc.Extra,
	}
	a, okA := envelope(typed)
	b, okB := envelope(raw)
	if !okA || !okB || !jsonSubset(a, b) || !jsonSubset(b, a) {
		return false
	}

	encoded, err := json.Marshal(doc.Payload)
	if err != nil {
		return false
	}
	mine, errMine := decodeNumbers(encoded)
	theirs, errTheirs := decodeNumbers(raw.Payload)
	if errMine != nil || errTheirs != nil {
		return false
	}
	return jsonSubset(mine, theirs)
}

// jsonSubset reports whether a is b with zero or more object members removed,
// at any depth.
func jsonSubset(a, b any) bool {
	switch av := a.(type) {
	case map[string]any:
		bv, ok := b.(map[string]any)
		if !ok {
			return false
		}
		for name, member := range av {
			other, present := bv[name]
			if !present || !jsonSubset(member, other) {
				return false
			}
		}
		return true
	case []any:
		bv, ok := b.([]any)
		if !ok || len(av) != len(bv) {
			return false
		}
		for i := range av {
			if !jsonSubset(av[i], bv[i]) {
				return false
			}
		}
		return true
	case json.Number:
		bv, ok := b.(json.Number)
		return ok && numberEqual(av, bv)
	}
	return reflect.DeepEqual(a, b)
}

// numberEqual compares two number literals by value — exactly for integers, so
// a change past float64's 53 bits is still a change; as float64 otherwise, since
// a typed re-encoding may spell a decimal differently ("2.50" as "2.5").
func numberEqual(a, b json.Number) bool {
	if a == b {
		return true
	}
	ai, errA := a.Int64()
	bi, errB := b.Int64()
	if errA == nil && errB == nil {
		return ai == bi
	}
	af, errA := a.Float64()
	bf, errB := b.Float64()
	return errA == nil && errB == nil && af == bf
}
