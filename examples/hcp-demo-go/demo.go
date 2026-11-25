// The Human Context Protocol (HCP) carried as Trust Tasks, end to end, in one
// process. A DEMONSTRATION, not a reference provider: see README.md and
// ../hcp-demo-ts/SCENARIO.md (the language-neutral script every port of this
// demo follows).
//
// Three parties, each with its own did:key and TSP keys:
//   - the HCP Provider (a toy, in-memory: a few entries, a grant table);
//   - a Context Client (a booking agent that wants the person's context);
//   - a Management Client (the person's own app, which can revoke grants).
//
// Every document is built with the generated types, signed with the
// trust-tasks-go/proof module, sealed with trust-tasks-go/tsp (a Direct TSP
// message, signed and sealed), opened, and run through the SPEC §7.2 pipeline
// (proof verified over the document as received, payload validated against its
// schema) before anything acts on it. Nothing touches the network.
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"slices"
	"sort"
	"strings"
	"time"

	atsp "github.com/affinidi/affinidi-tsp-go"
	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof"
	discovery "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/trust-task-discovery/v0_4"
	nextstep "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/trust-task-next-step/v0_1"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp"

	"example.com/trust-tasks-hcp-demo-go/internal/tasks"
	query "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/entries/query/v0_1"
	grantrequest "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/request/v0_1"
	revoke "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/revoke/v0_1"
	revoked "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/revoked/v0_1"
)

// ── Deterministic inputs ───────────────────────────────────────────────────

// start is where the demo's clock begins.
var start = time.Date(2026, 11, 25, 10, 0, 0, 0, time.UTC)

// Clock is a fixed clock the demo controls: it starts at start and moves one
// second per document issued. Instants are whole seconds (`…Z`), never the
// system time.
type Clock struct{ t time.Time }

// Now is the clock's instant, for the pipeline and the proof verifier.
func (c *Clock) Now() time.Time { return c.t }

// ISO is the clock's instant as the documents write it.
func (c *Clock) ISO() string { return instant(c.t) }

// Tick moves the clock one second and returns the new instant: the
// trusttasks.Clock a built document is stamped with.
func (c *Clock) Tick() string {
	c.t = c.t.Add(time.Second)
	return c.ISO()
}

func instant(t time.Time) string { return t.UTC().Format(time.RFC3339) }

// IDs hands out sequential urn:uuid identifiers: …-000000000001, …-000000000002, …
type IDs struct{ n int }

// Next is the next identifier. Every party shares the one counter.
func (i *IDs) Next() string {
	i.n++
	return fmt.Sprintf("urn:uuid:00000000-0000-4000-8000-%012x", i.n)
}

// ── Parties ────────────────────────────────────────────────────────────────

// Party is one participant. DEMO ONLY keys: each is 32 bytes of one repeated
// value. Never use such keys for anything real. The VID is the did:key of the
// Ed25519 signing key, and the same VID names the party in TSP.
type Party struct {
	Name   string
	VID    string
	Signer *proof.Signer
	TSP    *atsp.PrivateIdentity
}

func repeated(b byte) []byte { return bytes.Repeat([]byte{b}, 32) } // DEMO ONLY

func newParty(name string, signingSeed, encryptionSeed byte) *Party {
	signer, err := proof.SignerFromPrivateKey(proof.Ed25519, repeated(signingSeed))
	if err != nil {
		panic(err)
	}
	id, err := atsp.NewPrivateIdentity(signer.DID, atsp.SigKeyEd25519, repeated(signingSeed), atsp.EncKeyX25519, repeated(encryptionSeed))
	if err != nil {
		panic(err)
	}
	return &Party{Name: name, VID: signer.DID, Signer: signer, TSP: id}
}

// ── The toy provider's data ────────────────────────────────────────────────

const (
	// Purpose is the one purpose the Context Client asks for and uses.
	Purpose = "https://purposes.example/travel-booking"
	// ManagementToken is issued to the person's app at setup. DEMO ONLY.
	ManagementToken = "hcp_mt_demo_management"
	// neutralDenial is one fixed text for every refusal (HCP 5.5).
	neutralDenial = "The request is not authorized."
)

// entry is the person's context, as the toy provider stores it (HCP 4.2, much simplified).
type entry struct{ ref, category, value string }

var entries = []entry{
	{"hcp-ref-0001", "travel_preferences", "Prefers aisle seats and early-morning flights."},
	{"hcp-ref-0002", "travel_preferences", "Avoids overnight layovers."},
	{"hcp-ref-0003", "health_context", "Severe peanut allergy."},
}

type grant struct {
	grantID, token, client string
	categories, purposes   []string
	status                 string // "active" | "revoked"
}

// hold is a grant request the person has not decided yet (SPEC §8.6 hold).
type hold struct {
	payload grantrequest.Payload
	decided bool
}

// ── The world: three parties, one transport, one clock ─────────────────────

// World holds the three parties, the shared clock and counter, the toy
// provider's state and the transcript.
type World struct {
	Clock         *Clock
	IDs           *IDs
	Provider      *Party
	ContextClient *Party
	Management    *Party

	transcript strings.Builder
	grants     []*grant
	held       map[string]*hold
	tokens     int
	verifier   *proof.Verifier
	// One TSP consumer per receiving party: its replay guard is that party's
	// duplicate-execution record.
	consumers map[string]*tsp.Consumer
}

// NewWorld builds the parties and their consumers.
func NewWorld() *World {
	w := &World{
		Clock:         &Clock{t: start},
		IDs:           &IDs{},
		Provider:      newParty("Provider", 0x11, 0x12),
		ContextClient: newParty("Context Client", 0x21, 0x22),
		Management:    newParty("Management Client", 0x31, 0x32),
		held:          map[string]*hold{},
		consumers:     map[string]*tsp.Consumer{},
	}
	w.verifier = proof.NewVerifier(proof.WithClock(w.Clock.Now))
	for _, me := range []*Party{w.Provider, w.ContextClient} {
		c := tsp.NewConsumer(me.TSP)
		c.ProofVerifier = w.verifier
		c.PayloadValidator = schemaValidator{}
		c.Now = w.Clock.Now
		c.NewID = w.IDs.Next
		c.Extends = tasks.ExtendsOf
		w.consumers[me.VID] = c
	}
	return w
}

func (w *World) byVID(vid string) *Party {
	for _, p := range []*Party{w.Provider, w.ContextClient, w.Management} {
		if p.VID == vid {
			return p
		}
	}
	panic("unknown VID " + vid)
}

// sign signs a built document as by, with the proof's created equal to the clock.
func (w *World) sign(by *Party, doc any) json.RawMessage {
	unsigned, err := json.Marshal(doc)
	if err != nil {
		panic(err)
	}
	signed, err := by.Signer.Sign(unsigned, proof.SignOptions{Created: w.Clock.Now()})
	if err != nil {
		panic(err)
	}
	return signed
}

// Request builds and signs a request; party members and issuedAt are set
// BEFORE signing. threadID is empty for a new exchange, whose thread is the
// request's own id.
func Request[P any](w *World, from, to *Party, typeURI string, payload P, threadID string) json.RawMessage {
	id := w.IDs.Next()
	issuedAt := w.Clock.Tick()
	if threadID == "" {
		threadID = id
	}
	return w.sign(from, tt.Document[P]{
		ID: id, Type: typeURI, Issuer: &from.VID, Recipient: &to.VID,
		IssuedAt: &issuedAt, ThreadID: &threadID, Payload: payload,
	})
}

// Exchange seals doc over TSP from → to, lets to consume it, and opens the
// sealed reply back at from. It returns the reply exactly as it arrived.
func (w *World) Exchange(from, to *Party, doc json.RawMessage) json.RawMessage {
	w.print(from, to, doc)
	wire, err := tsp.PackTrustTask(doc, from.TSP, to.TSP.Public())
	must(err)

	// ── receiving side: open, route on the type, then the §7.2 pipeline ──
	sender := w.byVID(tsp.AdvertisedSender(wire))
	unpacked, err := tsp.UnpackTrustTask(wire, to.TSP, sender.TSP.Public())
	must(err)
	var head struct {
		Type string `json:"type"`
	}
	must(json.Unmarshal(unpacked.Document, &head))
	route, ok := routes[bare(head.Type)]
	if !ok {
		panic("no handler for " + head.Type)
	}
	received, err := route(context.Background(), w, to, unpacked)
	must(err)
	if received.Reply == nil {
		panic(fmt.Sprintf("no reply: %s", received.Outcome.Kind))
	}

	// ── back at the sender: open the sealed reply and check it before use ──
	replyWire, err := tsp.PackReply(to.TSP, from.TSP.Public(), received.Reply)
	must(err)
	opened, err := tsp.UnpackTrustTask(replyWire, from.TSP, to.TSP.Public())
	must(err)
	reply := opened.Document
	var shape struct {
		Type    string          `json:"type"`
		Proof   json.RawMessage `json:"proof"`
		Payload json.RawMessage `json:"payload"`
	}
	must(json.Unmarshal(reply, &shape))
	if shape.Proof != nil {
		if err := w.verifier.Verify(context.Background(), reply); err != nil {
			panic("reply proof does not verify: " + err.Error())
		}
	}
	must(validateReply(shape.Type, shape.Payload))
	w.print(to, from, reply)
	return reply
}

// route consumes an unpacked document addressed to me.
type route func(ctx context.Context, w *World, me *Party, u *tsp.Unpacked) (*tsp.Received[json.RawMessage], error)

// serve runs the §7.2 pipeline for one specification and, once it accepts the
// document, hands it to handle. Every reply a handler builds is signed, so the
// reply type is the signed document itself.
func serve[P any](spec tt.SpecPolicy, handle func(w *World, me *Party, doc *tt.Document[P]) (*tt.Document[json.RawMessage], error)) route {
	return func(ctx context.Context, w *World, me *Party, u *tsp.Unpacked) (*tsp.Received[json.RawMessage], error) {
		return tsp.Consume(ctx, w.consumers[me.VID], u, spec, decode[P],
			func(_ context.Context, doc *tt.Document[P], _ tt.ResolvedParties) (*tt.Document[json.RawMessage], error) {
				return handle(w, me, doc)
			})
	}
}

// decode reads a payload, refusing a member encoding/json would match by case alone.
func decode[P any](raw json.RawMessage) (P, error) {
	var p P
	if err := tt.CheckMemberNames[P](raw); err != nil {
		return p, err
	}
	err := json.Unmarshal(raw, &p)
	return p, err
}

// routes is what each receiving party serves, by bare Type URI.
var routes = map[string]route{
	discovery.TypeURI:    serve(discovery.Spec, handleDiscovery),
	grantrequest.TypeURI: serve(grantrequest.Spec, handleGrantRequest),
	query.TypeURI:        serve(query.Spec, handleQuery),
	revoke.TypeURI:       serve(revoke.Spec, handleRevoke),
	revoked.TypeURI:      serve(revoked.Spec, handleRevoked),
}

// ── Handlers: what each receiving party does with an accepted document ────

// reply builds the success response to doc, signed by me.
func reply[P, R any](w *World, me *Party, doc *tt.Document[P], payload R) (*tt.Document[json.RawMessage], error) {
	return asSigned(w.sign(me, tt.RespondWith(doc, w.IDs.Next(), payload, w.Clock.Tick))), nil
}

// refuse builds a trust-task-error response to doc, signed by me.
func refuse[P any](w *World, me *Party, doc *tt.Document[P], code, message string) (*tt.Document[json.RawMessage], error) {
	built := tt.RejectWith(doc, w.IDs.Next(), tt.ErrorPayload{Code: code, Message: &message, Retryable: false}, w.Clock.Tick)
	var signed tt.ErrorResponse
	must(json.Unmarshal(w.sign(me, built), &signed))
	return nil, &tt.Refusal{Response: &signed}
}

func asSigned(raw json.RawMessage) *tt.Document[json.RawMessage] {
	var doc tt.Document[json.RawMessage]
	must(json.Unmarshal(raw, &doc))
	return &doc
}

// Step 1 — trust-task-discovery: what the provider serves (SPEC §10).
func handleDiscovery(w *World, me *Party, doc *tt.Document[discovery.Payload]) (*tt.Document[json.RawMessage], error) {
	var supported []json.RawMessage
	for _, t := range []string{grantrequest.TypeURI, query.TypeURI, revoke.TypeURI, revoked.TypeURI} {
		supported = append(supported, quoted(t))
	}
	return reply(w, me, doc, discovery.Response{SupportedTypes: supported})
}

// Step 2 — hcp/grants/request: hold with a next step until the person decides
// (HCP 6.1.6, SPEC §8.6).
func handleGrantRequest(w *World, me *Party, doc *tt.Document[grantrequest.Payload]) (*tt.Document[json.RawMessage], error) {
	threadID := *doc.ThreadID
	h := w.held[threadID]
	if h == nil {
		w.held[threadID] = &hold{payload: doc.Payload}
	}
	if h == nil || !h.decided {
		id := w.IDs.Next()
		issuedAt := w.Clock.Tick()
		reason := "The request is with the person, who has not decided yet."
		hint := map[string]json.RawMessage{"retryAfter": json.RawMessage("5")}
		return asSigned(w.sign(me, tt.Document[nextstep.Payload]{
			ID: id, Type: nextstep.TypeURI, Issuer: &me.VID, Recipient: doc.Issuer,
			IssuedAt: &issuedAt, ThreadID: &threadID,
			Payload: nextstep.Payload{
				Expects:      []nextstep.PayloadExpectsItem{{TypeURI: grantrequest.TypeURI + "#request", Reason: &reason, Hint: &hint}},
				Continuation: quoted("resubmit"),
				InResponseTo: &nextstep.PayloadInResponseTo{TypeURI: doc.Type, ID: &doc.ID},
			},
		})), nil
	}

	// The person approved (the toy approves everything asked, read only).
	g := &grant{grantID: w.IDs.Next(), client: *doc.Issuer, status: "active"}
	w.tokens++
	g.token = fmt.Sprintf("hcp_at_demo_%04d", w.tokens) // DEMO ONLY
	var access []grantrequest.GrantedAccess
	for _, a := range h.payload.Access {
		g.categories = append(g.categories, a.Categories...)
		var purposes []grantrequest.Purpose
		if a.Purposes != nil {
			purposes = *a.Purposes
		}
		for _, p := range purposes {
			g.purposes = append(g.purposes, p.Code)
		}
		access = append(access, grantrequest.GrantedAccess{Type: a.Type, Actions: a.Actions, Categories: a.Categories, Purposes: purposes}) // approved as asked
	}
	w.grants = append(w.grants, g)
	expiresIn := int64(4 * 3600) // HCP 6.1.3 default
	if h.payload.ExpiresIn != nil {
		expiresIn = *h.payload.ExpiresIn
	}
	grantExpiresAt := w.plus(expiresIn)
	return reply(w, me, doc, grantrequest.Response{
		AccessToken:    g.token,
		TokenExpiresAt: w.plus(3600), // HCP 13.4 default
		GrantID:        g.grantID,
		Access:         access,
		GrantType:      grantrequest.ResponseGrantTypeTemporary,
		GrantedAt:      w.Clock.ISO(),
		GrantExpiresAt: &grantExpiresAt,
	})
}

func (w *World) plus(seconds int64) string {
	return instant(w.Clock.Now().Add(time.Duration(seconds) * time.Second))
}

// Step 3 — hcp/entries/query: the grant is checked on every request (HCP 5.2.1, 6.2.2).
func handleQuery(w *World, me *Party, doc *tt.Document[query.Payload]) (*tt.Document[json.RawMessage], error) {
	p := doc.Payload
	var g *grant
	for _, x := range w.grants {
		if x.token == p.AccessToken && doc.Issuer != nil && x.client == *doc.Issuer {
			g = x
			break
		}
	}
	if g == nil || g.status != "active" {
		return refuse(w, me, doc, "hcp:invalidToken", "The access token is not valid.")
	}
	asked := g.categories
	if p.Categories != nil {
		asked = *p.Categories
	}
	for _, c := range asked {
		if !slices.Contains(g.categories, c) {
			return refuse(w, me, doc, "permissionDenied", neutralDenial)
		}
	}
	if !slices.Contains(g.purposes, p.Purpose) {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	limit := int64(5)
	if p.MaxEntries != nil {
		limit = *p.MaxEntries
	}
	limit = min(limit, 20)
	useUntil := w.plus(3600)
	disclosed := []json.RawMessage{}
	for _, e := range entries {
		if int64(len(disclosed)) == limit || !slices.Contains(asked, e.category) {
			continue
		}
		language := "en"
		d, err := json.Marshal(query.Disclosure{
			Ref:            e.ref,
			Version:        1,
			Content:        query.Content{Profile: "https://hcp.example/spec/profiles/general-context/2.0", Value: quoted(e.value), Language: &language},
			Assertion:      query.Assertion{DeclarationType: query.AssertionDeclarationTypePersonDeclared},
			Classification: query.Classification{Categories: []string{e.category}},
			Standing:       query.Standing{Assurance: query.AssuranceAuthenticated, Confirmed: true},
			UseUntil:       useUntil,
		})
		must(err)
		disclosed = append(disclosed, d)
	}
	return reply(w, me, doc, query.Response{
		Entries:            disclosed,
		RetrievalID:        w.IDs.Next(),
		Minimization:       query.MinimizationReport{Method: "lexical"},
		CategoriesSearched: asked,
		RespondedAt:        w.Clock.ISO(),
	})
}

// Step 4 — hcp/grants/revoke: only the person's Management Client may (HCP 6.1.9, 6.2.1).
func handleRevoke(w *World, me *Party, doc *tt.Document[revoke.Payload]) (*tt.Document[json.RawMessage], error) {
	if doc.Payload.AccessToken != ManagementToken {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	for _, g := range w.grants {
		if g.grantID == doc.Payload.GrantID {
			g.status = "revoked" // immediate (HCP 6.2.2); a second revoke is idempotent
			return reply(w, me, doc, revoke.Response{})
		}
	}
	return refuse(w, me, doc, "permissionDenied", neutralDenial)
}

// Step 5 — hcp/grants/revoked, at the Context Client: stop using what it holds (HCP 6.2.2, 9.9).
func handleRevoked(w *World, me *Party, doc *tt.Document[revoked.Payload]) (*tt.Document[json.RawMessage], error) {
	return reply(w, me, doc, revoked.Response{})
}

// PersonApproves is the person deciding, in the provider's own modality (here:
// approving at once).
func (w *World) PersonApproves(threadID string) {
	w.held[threadID].decided = true
	fmt.Fprintf(&w.transcript, "[person] approves the grant request on thread %s in the provider's app\n", threadID)
}

// ── Transcript ─────────────────────────────────────────────────────────────

func (w *World) step(title string) { fmt.Fprintf(&w.transcript, "\n=== %s ===\n", title) }

func (w *World) print(from, to *Party, doc json.RawMessage) {
	value := parse(doc)
	typ, _ := value.(map[string]any)["type"].(string)
	fmt.Fprintf(&w.transcript, "--> %s -> %s (TSP) %s\n%s\n", from.Name, to.Name, typ, Pretty(redact(value), ""))
}

// redact shows a secret as present, never in full: the first 6 characters and an ellipsis.
func redact(value any) any {
	switch v := value.(type) {
	case []any:
		out := make([]any, len(v))
		for i, x := range v {
			out[i] = redact(x)
		}
		return out
	case map[string]any:
		out := make(map[string]any, len(v))
		for k, x := range v {
			if s, ok := x.(string); ok && k == "accessToken" {
				r := []rune(s)
				out[k] = string(r[:min(6, len(r))]) + "…"
			} else {
				out[k] = redact(x)
			}
		}
		return out
	}
	return value
}

// Pretty is JSON with object keys sorted by code point (as JCS orders them)
// and two-space indents, strings escaped as JSON.stringify does: the same in
// every language.
func Pretty(value any, indent string) string {
	next := indent + "  "
	switch v := value.(type) {
	case []any:
		if len(v) == 0 {
			return "[]"
		}
		items := make([]string, len(v))
		for i, x := range v {
			items[i] = next + Pretty(x, next)
		}
		return "[\n" + strings.Join(items, ",\n") + "\n" + indent + "]"
	case map[string]any:
		if len(v) == 0 {
			return "{}"
		}
		keys := make([]string, 0, len(v))
		for k := range v {
			keys = append(keys, k)
		}
		sort.Strings(keys) // byte order of UTF-8 is code-point order
		items := make([]string, len(keys))
		for i, k := range keys {
			items[i] = next + jsString(k) + ": " + Pretty(v[k], next)
		}
		return "{\n" + strings.Join(items, ",\n") + "\n" + indent + "}"
	case string:
		return jsString(v)
	case json.Number:
		return v.String()
	case bool:
		if v {
			return "true"
		}
		return "false"
	case nil:
		return "null"
	}
	panic(fmt.Sprintf("unexpected JSON value %T", value))
}

// jsString quotes s as JSON.stringify does: only ", \ and control characters
// are escaped, everything else (<, >, &, U+2028, non-ASCII) is left as is —
// unlike encoding/json, which escapes HTML characters and U+2028/U+2029.
func jsString(s string) string {
	var b strings.Builder
	b.WriteByte('"')
	for _, r := range s {
		switch r {
		case '"':
			b.WriteString(`\"`)
		case '\\':
			b.WriteString(`\\`)
		case '\b':
			b.WriteString(`\b`)
		case '\f':
			b.WriteString(`\f`)
		case '\n':
			b.WriteString(`\n`)
		case '\r':
			b.WriteString(`\r`)
		case '\t':
			b.WriteString(`\t`)
		default:
			if r < 0x20 {
				fmt.Fprintf(&b, `\u%04x`, r)
			} else {
				b.WriteRune(r)
			}
		}
	}
	b.WriteByte('"')
	return b.String()
}

// ── helpers ────────────────────────────────────────────────────────────────

func parse(raw json.RawMessage) any {
	dec := json.NewDecoder(bytes.NewReader(raw))
	dec.UseNumber()
	var v any
	must(dec.Decode(&v))
	return v
}

func quoted(s string) json.RawMessage {
	var b bytes.Buffer
	enc := json.NewEncoder(&b)
	enc.SetEscapeHTML(false)
	must(enc.Encode(s))
	return bytes.TrimRight(b.Bytes(), "\n")
}

func bare(typeURI string) string {
	if i := strings.IndexByte(typeURI, '#'); i >= 0 {
		return typeURI[:i]
	}
	return typeURI
}

func must(err error) {
	if err != nil {
		panic(err)
	}
}

// ── The five steps ─────────────────────────────────────────────────────────

// Result is what a run produces: the transcript and step 6's reply.
type Result struct {
	Transcript     string
	PostRevokeCode string
	PostRevokeType string
}

// replyOf decodes a reply document's type and payload.
func replyOf[P any](raw json.RawMessage) tt.Document[P] {
	var doc tt.Document[P]
	must(json.Unmarshal(raw, &doc))
	return doc
}

// Run plays the five steps on w (a fresh world when nil).
func Run(w *World) Result {
	if w == nil {
		w = NewWorld()
	}
	P, C, M := w.Provider, w.ContextClient, w.Management

	// 1. Discover: does the provider speak hcp/*? (trust-task-discovery 0.4, SPEC §10)
	w.step("1. Discover (trust-task-discovery 0.4)")
	found := replyOf[discovery.Response](w.Exchange(C, P, Request(w, C, P, discovery.TypeURI, discovery.Payload{Patterns: &[]string{"hcp/*"}}, "")))
	if !slices.ContainsFunc(found.Payload.SupportedTypes, func(t json.RawMessage) bool { return string(t) == string(quoted(grantrequest.TypeURI)) }) {
		panic("provider does not serve grant requests")
	}

	// 2. Grant request: held with a next step while the person decides, then
	// resubmitted (HCP 6.1.6, 6.1.7; SPEC §8.6)
	w.step("2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step")
	description := "Book trips that match how you like to travel"
	expiresIn := int64(14400)
	subjectHint := "acct:alex@provider.example"
	ask := grantrequest.Payload{
		Access: []grantrequest.AccessRequest{{
			Type:       "https://hcp.example/spec/authorization/context-access",
			Actions:    []grantrequest.GrantAction{grantrequest.GrantActionRead},
			Categories: []string{"travel_preferences"},
			Purposes:   &[]grantrequest.Purpose{{Code: Purpose, Description: &description}},
		}},
		ExpiresIn:   &expiresIn,
		SubjectHint: &subjectHint,
	}
	first := Request(w, C, P, grantrequest.TypeURI+"#request", ask, "")
	held := replyOf[json.RawMessage](w.Exchange(C, P, first))
	if held.Type != nextstep.TypeURI {
		panic("expected a next step")
	}
	thread := *replyOf[json.RawMessage](first).ThreadID
	w.PersonApproves(thread)
	// Resubmission: a new document, fresh id, same threadId and payload.
	granted := replyOf[grantrequest.Response](w.Exchange(C, P, Request(w, C, P, grantrequest.TypeURI+"#request", ask, thread)))
	if granted.Type != grantrequest.ResponseTypeURI {
		panic("expected the grant")
	}
	accessToken, grantID := granted.Payload.AccessToken, granted.Payload.GrantID

	// 3. Query under the grant (hcp/entries/query 0.1, HCP 5.2.1)
	w.step("3. Query (hcp/entries/query 0.1)")
	maxEntries := int64(5)
	queryPayload := query.Payload{Query: "How does Alex like to fly?", Categories: &[]string{"travel_preferences"}, Purpose: Purpose, MaxEntries: &maxEntries, AccessToken: accessToken}
	if answer := replyOf[json.RawMessage](w.Exchange(C, P, Request(w, C, P, query.TypeURI+"#request", queryPayload, ""))); answer.Type != query.ResponseTypeURI {
		panic("expected entries")
	}

	// 4. Revoke from the person's Management Client (hcp/grants/revoke 0.1, HCP 6.2.1)
	w.step("4. Revoke (hcp/grants/revoke 0.1) from the Management Client")
	if done := replyOf[json.RawMessage](w.Exchange(M, P, Request(w, M, P, revoke.TypeURI+"#request", revoke.Payload{GrantID: grantID, AccessToken: ManagementToken}, ""))); done.Type != revoke.ResponseTypeURI {
		panic("expected the revocation")
	}

	// 5. Revocation notice to the Context Client (hcp/grants/revoked 0.1, HCP 6.2.3, 9.9)
	w.step("5. Revocation notice (hcp/grants/revoked 0.1)")
	// noticeId is HCP's jti, minted once per notice and kept on every delivery attempt.
	notice := revoked.Payload{GrantID: grantID, NoticeID: w.IDs.Next(), RevokedAt: w.Clock.ISO()}
	if ack := replyOf[json.RawMessage](w.Exchange(P, C, Request(w, P, C, revoked.TypeURI+"#request", notice, ""))); ack.Type != revoked.ResponseTypeURI {
		panic("expected the acknowledgement")
	}

	// After: the same query again fails — the grant is checked on every request (HCP 6.2.2)
	w.step("After revoke: the same query fails with hcp:invalidToken")
	refused := replyOf[tt.ErrorPayload](w.Exchange(C, P, Request(w, C, P, query.TypeURI+"#request", queryPayload, "")))

	return Result{Transcript: w.transcript.String(), PostRevokeType: refused.Type, PostRevokeCode: refused.Payload.Code}
}
