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
// trust-tasks-go/proof module, sealed with trust-tasks-go/tsp (a TSP 1.0 Direct
// message: HPKE-Base, signed), opened, and run through the SPEC §7.2 pipeline
// (proof verified over the document as received, payload validated against its
// schema) before anything acts on it. Nothing touches the network.
package main

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
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

// start is where the demo's clock begins: 2026-11-25T10:00:00Z.
var start = time.Date(2026, 11, 25, 10, 0, 0, 0, time.UTC)

// iso is an instant as the documents write it: whole seconds,
// `YYYY-MM-DDTHH:MM:SSZ`. The repository's rule is "never write the system time
// into a document": every instant the demo writes comes from its injected
// Clock, which holds whole seconds only, and goes through this one function.
func iso(t time.Time) string {
	if t.Nanosecond() != 0 {
		panic("the demo's instants are whole seconds")
	}
	return t.UTC().Format("2006-01-02T15:04:05Z")
}

// Clock is a fixed clock the demo controls: it starts at start and moves one
// second per document issued. It never reads the system time.
type Clock struct{ t time.Time }

// Now is the clock's instant, for the pipeline and the proof verifier.
func (c *Clock) Now() time.Time { return c.t }

// ISO is the clock's instant as the documents write it.
func (c *Clock) ISO() string { return iso(c.t) }

// Plus is the instant seconds after now.
func (c *Clock) Plus(seconds int64) string { return iso(c.t.Add(time.Duration(seconds) * time.Second)) }

// Tick moves the clock one second and returns the new instant: the
// trusttasks.Clock a built document is stamped with.
func (c *Clock) Tick() string {
	c.t = c.t.Add(time.Second)
	return c.ISO()
}

// Advance lets time pass without issuing a document (a client waiting out
// retryAfter, or a test).
func (c *Clock) Advance(seconds int64) { c.t = c.t.Add(time.Duration(seconds) * time.Second) }

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
	// invalidToken is the text of every hcp:invalidToken.
	invalidToken = "The access token is not valid."
	// subject is the person, by the pseudonym the provider uses with this client (HCP 3.4).
	subject = "hcp-sub-demo-alex"
	// AfterApprovalWindow is how long after the response a resubmission still
	// gets the same grant, in seconds (hcp/grants/request "After approval").
	AfterApprovalWindow = 3600
	// HoldLifetime is the hold's lifetime, provider policy: how long a request
	// the person has not decided stays held, in seconds from the held request's
	// issuedAt. When it passes, the policy denies (hcp/grants/request "Holding"
	// rule 6).
	HoldLifetime = 300
)

// entry is the person's context, as the toy provider stores it (HCP 4.2, much simplified).
type entry struct{ ref, category, value string }

var entries = []entry{
	{"hcp-ref-0001", "travel_preferences", "Prefers aisle seats and early-morning flights."},
	{"hcp-ref-0002", "travel_preferences", "Avoids overnight layovers."},
	{"hcp-ref-0003", "health_context", "Severe peanut allergy."},
}

// registration is what the provider knows of a registered client, and shows the person (HCP 6.1.5, 6.1.6).
type registration struct {
	name     string
	operator grantrequest.Named
}

// grant is a grant in the provider's table.
type grant struct {
	grantID, tokenExpiresAt string
	// tokenDigests are the SHA-256 digests (hex) of every token issued under
	// the grant. The provider keeps no token itself (hcp/grants/request "The
	// access token"), so it can check a presented token but never send one
	// again.
	tokenDigests         map[string]bool
	client               string // the client's VID
	access               []grantrequest.AccessDetail
	categories, purposes []string
	issuedAt, expiresAt  string
	status               grantrequest.GrantStatus // active | revoked
	revokedAt            string
}

// holdState is where a held grant request stands.
type holdState int

const (
	pending holdState = iota
	approved
	granted
	declined
	expired
)

// hold is a grant request on one thread (hcp/grants/request "Holding"): held,
// then approved by the person and granted, or ended: declined by the person,
// or expired when its lifetime passed undecided. An ended thread is never
// reopened.
type hold struct {
	issuer      string
	payload     grantrequest.Payload
	canonical   string    // the JCS form of the payload, to compare resubmissions
	heldAt      time.Time // the held request's issuedAt: the lifetime counts from it
	state       holdState
	grantID     string
	respondedAt time.Time // when the response was first sent
}

// HeldGrant is what the Context Client keeps for a grant it holds: the token,
// and the refs of the copies disclosed under it.
type HeldGrant struct {
	Token  string
	Copies []string
}

// ── The world: three parties, one transport, one clock ─────────────────────

// World holds the three parties, the shared clock and counter, the toy
// provider's state, the Context Client's state and the transcript.
type World struct {
	Clock         *Clock
	IDs           *IDs
	Provider      *Party
	ContextClient *Party
	Management    *Party

	// ── The provider's state ──
	// registeredClients is its one registered client, by VID.
	registeredClients map[string]registration
	grants            []*grant
	// held is the grant requests by threadId (the SPEC §8.6 hold).
	held   map[string]*hold
	tokens int

	// ── The Context Client's state ──
	// Wallet is the grants it holds, by grantId.
	Wallet map[string]*HeldGrant
	// ProcessedNotices is the revocation notices it has processed: noticeId →
	// grantId (hcp/grants/revoked "Deduplication").
	ProcessedNotices map[string]string

	transcript strings.Builder
	verifier   *proof.Verifier
	// One TSP consumer per receiving party: its replay guard is that party's
	// duplicate-execution record.
	consumers map[string]*tsp.Consumer
}

// NewWorld builds the parties, the provider's registry and the consumers.
func NewWorld() *World {
	w := &World{
		Clock:            &Clock{t: start},
		IDs:              &IDs{},
		Provider:         newParty("Provider", 0x11, 0x12),
		ContextClient:    newParty("Context Client", 0x21, 0x22),
		Management:       newParty("Management Client", 0x31, 0x32),
		held:             map[string]*hold{},
		Wallet:           map[string]*HeldGrant{},
		ProcessedNotices: map[string]string{},
		consumers:        map[string]*tsp.Consumer{},
	}
	w.registeredClients = map[string]registration{
		w.ContextClient.VID: {name: "Wayfare booking agent", operator: grantrequest.Named{ID: "booking.example", Name: "Wayfare Travel Ltd"}},
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

func (w *World) grantByID(grantID string) *grant {
	for _, g := range w.grants {
		if g.grantID == grantID {
			return g
		}
	}
	return nil
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
// sealed reply back at from. It checks the reply (CheckReply, then its proof
// and its payload's schema) and returns it exactly as it arrived.
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
	var sent struct {
		ID       string  `json:"id"`
		ThreadID *string `json:"threadId"`
	}
	must(json.Unmarshal(doc, &sent))
	thread := sent.ID
	if sent.ThreadID != nil {
		thread = *sent.ThreadID
	}
	must(CheckReply(reply, to, thread))
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

// pipelineCodes are the codes the receiving end's SPEC §7.2 pipeline raises
// before any handler runs. The library builds those error responses itself,
// unsigned; every other reply (each response, next step and handler refusal)
// is signed.
var pipelineCodes = []string{
	"malformedRequest",
	"unsupportedType",
	"unsupportedVersion",
	"unsupportedExtension",
	"expired",
	"proofRequired",
	"proofInvalid",
	"wrongRecipient",
	"identityMismatch",
	"idConflict",
}

// CheckReply is the sender's check on a reply before it trusts it: who it is
// from, which thread, and that it is signed unless the library built it. The
// proof itself and the payload's schema are checked after it.
func CheckReply(reply json.RawMessage, from *Party, threadID string) error {
	var r struct {
		Type     string          `json:"type"`
		Issuer   *string         `json:"issuer"`
		ThreadID *string         `json:"threadId"`
		Proof    json.RawMessage `json:"proof"`
		Payload  struct {
			Code string `json:"code"`
		} `json:"payload"`
	}
	if err := json.Unmarshal(reply, &r); err != nil {
		return fmt.Errorf("reply: %w", err)
	}
	if r.Issuer == nil || *r.Issuer != from.VID {
		return fmt.Errorf("reply issuer %v is not %s", deref(r.Issuer), from.VID)
	}
	if r.ThreadID == nil || *r.ThreadID != threadID {
		return fmt.Errorf("reply threadId %v is not %s", deref(r.ThreadID), threadID)
	}
	libraryError := r.Type == tt.TrustTaskErrorTypeURI && slices.Contains(pipelineCodes, r.Payload.Code)
	if r.Proof == nil && !libraryError {
		return errors.New("reply " + r.Type + " carries no proof")
	}
	return nil
}

func deref(s *string) string {
	if s == nil {
		return "(absent)"
	}
	return *s
}

// route consumes an unpacked document addressed to me.
type route func(ctx context.Context, w *World, me *Party, u *tsp.Unpacked) (*tsp.Received[json.RawMessage], error)

// serve runs the §7.2 pipeline for one specification and, once it accepts the
// document, hands it to handle. Every reply a handler builds is signed, so the
// reply type is the signed document itself.
func serve[P any](spec tt.SpecPolicy, decode func(json.RawMessage) (P, error), handle func(w *World, me *Party, doc *tt.Document[P]) (*tt.Document[json.RawMessage], error)) route {
	return func(ctx context.Context, w *World, me *Party, u *tsp.Unpacked) (*tsp.Received[json.RawMessage], error) {
		return tsp.Consume(ctx, w.consumers[me.VID], u, spec, decode,
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

// withJCS is a decoded payload together with the payload as received and its
// JCS form, for a handler that compares payloads. It marshals as the payload as
// received, so the pipeline's schema check sees the received payload.
type withJCS[P any] struct {
	Value P
	Raw   json.RawMessage
	JCS   string
}

// MarshalJSON is the payload as received.
func (p withJCS[P]) MarshalJSON() ([]byte, error) { return p.Raw, nil }

// decodeWithJCS decodes as decode does and keeps the payload's canonical form.
// tt.CanonicalJSON orders members recursively and drops insignificant
// whitespace: it is equality-equivalent to JCS, which is all a comparison
// needs. (trust-tasks-go/proof's byte-exact JCS is not exported.)
func decodeWithJCS[P any](raw json.RawMessage) (withJCS[P], error) {
	p, err := decode[P](raw)
	if err != nil {
		return withJCS[P]{}, err
	}
	canonical, err := tt.CanonicalJSON(raw)
	return withJCS[P]{Value: p, Raw: append(json.RawMessage(nil), raw...), JCS: string(canonical)}, err
}

// routes is what each receiving party serves, by bare Type URI.
var routes = map[string]route{
	discovery.TypeURI:    serve(discovery.Spec, decode[discovery.Payload], handleDiscovery),
	grantrequest.TypeURI: serve(grantrequest.Spec, decodeWithJCS[grantrequest.Payload], handleGrantRequest),
	query.TypeURI:        serve(query.Spec, decode[query.Payload], handleQuery),
	revoke.TypeURI:       serve(revoke.Spec, decode[revoke.Payload], handleRevoke),
	revoked.TypeURI:      serve(revoked.Spec, decode[revoked.Payload], handleRevoked),
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
// (HCP 6.1.6, SPEC §8.6), then the grant, and "After approval". The rules are
// applied in SCENARIO.md's order.
func handleGrantRequest(w *World, me *Party, doc *tt.Document[withJCS[grantrequest.Payload]]) (*tt.Document[json.RawMessage], error) {
	issuer, threadID := *doc.Issuer, *doc.ThreadID
	// 1. Only a registered client may ask (hcp/grants/request "Authorization").
	if _, ok := w.registeredClients[issuer]; !ok {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	h := w.held[threadID]
	// 2. A new thread: hold it.
	if h == nil {
		w.held[threadID] = &hold{issuer: issuer, payload: doc.Payload.Value, canonical: doc.Payload.JCS, heldAt: w.Clock.Now(), state: pending}
		return w.nextStep(me, doc)
	}
	// 3, 4. A resubmission is matched by (threadId, issuer), and must be the held
	// payload (JCS forms compared); a refusal leaves the hold as it is.
	if h.issuer != issuer {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	if h.canonical != doc.Payload.JCS {
		return refuse(w, me, doc, "malformedRequest", "The resubmission differs from the held request.")
	}

	// 5. A pending hold whose lifetime has passed ends.
	w.expireIfDue(h)
	switch h.state {
	case pending: // 5. still with the person
		return w.nextStep(me, doc)
	case declined, expired: // 5. the thread has ended and is never reopened: the same refusal as an unknown person
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	case approved: // 6. the person approved (the toy approves everything asked, read only): issue the one grant
		g := &grant{
			grantID:      w.IDs.Next(),
			tokenDigests: map[string]bool{},
			client:       issuer,
			issuedAt:     w.Clock.ISO(),
			status:       grantrequest.GrantStatusActive,
		}
		for _, a := range h.payload.Access {
			g.categories = append(g.categories, a.Categories...)
			var purposes []grantrequest.Purpose
			if a.Purposes != nil {
				for _, p := range *a.Purposes {
					purposes = append(purposes, grantrequest.Purpose{Code: p.Code, Description: p.Description})
					g.purposes = append(g.purposes, p.Code)
				}
			}
			actions := make([]grantrequest.AccessDetailActionsItem, len(a.Actions))
			for i, x := range a.Actions {
				actions[i] = grantrequest.AccessDetailActionsItem(x)
			}
			g.access = append(g.access, grantrequest.AccessDetail{Type: a.Type, Actions: actions, Categories: a.Categories, Purposes: purposes}) // approved as asked
		}
		expiresIn := int64(4 * 3600) // HCP 6.1.3 default
		if h.payload.ExpiresIn != nil {
			expiresIn = *h.payload.ExpiresIn
		}
		g.expiresAt = w.Clock.Plus(expiresIn)
		// No refresh in 0.1, so the token lasts as long as the grant (hcp/grants/request "Response").
		g.tokenExpiresAt = g.expiresAt
		w.grants = append(w.grants, g)
		h.state, h.grantID, h.respondedAt = granted, g.grantID, w.Clock.Now()
		return reply(w, me, doc, w.grantResponse(g, w.issueToken(g)))
	default: // 7. granted: the same grant within the hour, never a second one; after it, permissionDenied
		// The provider kept no copy of the token it sent, so it issues a new one
		// under the same grant; every token issued under the grant stays valid
		// until the grant ends (HCP 6.2.2).
		g := w.grantByID(h.grantID)
		late := w.Clock.Now().Sub(h.respondedAt) > AfterApprovalWindow*time.Second
		if late || g.status != grantrequest.GrantStatusActive {
			return refuse(w, me, doc, "permissionDenied", neutralDenial)
		}
		return reply(w, me, doc, w.grantResponse(g, w.issueToken(g)))
	}
}

// nextStep is the hold: a next step naming the request Type URI, continuation
// resubmit (SPEC §8.6).
func (w *World) nextStep(me *Party, doc *tt.Document[withJCS[grantrequest.Payload]]) (*tt.Document[json.RawMessage], error) {
	id := w.IDs.Next()
	issuedAt := w.Clock.Tick()
	reason := "The request is with the person, who has not decided yet."
	hint := map[string]json.RawMessage{"retryAfter": json.RawMessage("1")}
	return asSigned(w.sign(me, tt.Document[nextstep.Payload]{
		ID: id, Type: nextstep.TypeURI, Issuer: &me.VID, Recipient: doc.Issuer,
		IssuedAt: &issuedAt, ThreadID: doc.ThreadID,
		Payload: nextstep.Payload{
			Expects:      []nextstep.PayloadExpectsItem{{TypeURI: grantrequest.TypeURI + "#request", Reason: &reason, Hint: &hint}},
			Continuation: quoted("resubmit"),
			InResponseTo: &nextstep.PayloadInResponseTo{TypeURI: doc.Type, ID: &doc.ID},
		},
	})), nil
}

// TokenDigest is the SHA-256 digest of a token, in hex: what the provider
// keeps in place of the token.
func TokenDigest(token string) string {
	sum := sha256.Sum256([]byte(token))
	return hex.EncodeToString(sum[:])
}

// issueToken mints a new token under g, keeps only its digest, and returns the
// token to send.
func (w *World) issueToken(g *grant) string {
	w.tokens++
	token := fmt.Sprintf("hcp_at_demo_%04d", w.tokens) // DEMO ONLY: a real token is random
	g.tokenDigests[TokenDigest(token)] = true
	return token
}

// grantResponse is a newly issued token and HCP's grant object, as
// hcp/grants/read-own would return it.
func (w *World) grantResponse(g *grant, token string) grantrequest.Response {
	r := w.registeredClients[g.client]
	expiresAt := g.expiresAt
	return grantrequest.Response{
		AccessToken:    token,
		TokenExpiresAt: g.tokenExpiresAt,
		Grant: grantrequest.Grant{
			GrantID:              g.grantID,
			Subject:              grantrequest.GrantParty{ID: subject},
			AuthorizingPrincipal: grantrequest.AuthorizingPrincipal{ID: subject},
			Client:               grantrequest.GrantClient{ID: g.client, Name: r.name, Operator: r.operator},
			Access:               g.access,
			GrantType:            grantrequest.GrantGrantTypeTemporary,
			IssuedAt:             g.issuedAt,
			ExpiresAt:            &expiresAt,
			Status:               g.status,
		},
	}
}

// Steps 3 and 5 — hcp/entries/query: the grant is checked on every request
// (HCP 5.2.1, 6.2.2), in SCENARIO.md's order.
func handleQuery(w *World, me *Party, doc *tt.Document[query.Payload]) (*tt.Document[json.RawMessage], error) {
	p := doc.Payload
	var g *grant
	digest := TokenDigest(p.AccessToken)
	for _, x := range w.grants {
		if x.tokenDigests[digest] {
			g = x
			break
		}
	}
	if g == nil {
		return refuse(w, me, doc, "hcp:invalidToken", invalidToken)
	}
	// A token issued to another client is no use from this VID (query "The
	// access token"); checked before the status, so it tells that VID nothing
	// about the grant.
	if doc.Issuer == nil || g.client != *doc.Issuer {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	if g.status != grantrequest.GrantStatusActive {
		return refuse(w, me, doc, "hcp:invalidToken", invalidToken)
	}
	asked := g.categories
	if p.Categories != nil {
		asked = *p.Categories
	}
	if !slices.Contains(g.purposes, p.Purpose) || slices.ContainsFunc(asked, func(c string) bool { return !slices.Contains(g.categories, c) }) {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	limit := int64(5)
	if p.MaxEntries != nil {
		limit = *p.MaxEntries
	}
	limit = min(limit, 20)
	// A copy is usable for an hour, but never past the grant it was disclosed
	// under (V0's rule). Both are whole-second UTC instants, so they order as strings.
	useUntil := min(w.Clock.Plus(3600), g.expiresAt)
	disclosed := []json.RawMessage{}
	for _, e := range entries {
		if int64(len(disclosed)) == limit || !slices.Contains(asked, e.category) {
			continue
		}
		d, err := json.Marshal(query.Disclosure{
			Ref:            e.ref,
			Version:        1,
			Content:        query.Content{Profile: "https://hcp.example/spec/profiles/general-context/2.0", Value: quoted(e.value), Language: tt.Value("en")},
			Assertion:      query.Assertion{DeclarationType: query.AssertionDeclarationTypePersonDeclared},
			Classification: query.Classification{Categories: []string{e.category}},
			Standing:       query.Standing{Assurance: query.AssuranceAuthenticated, Confirmed: true},
			UseUntil:       useUntil,
		})
		must(err)
		disclosed = append(disclosed, d)
	}
	directives := []query.Directive{} // a Context Client always gets the list; the person set none (HCP 6.4)
	return reply(w, me, doc, query.Response{
		Entries:            disclosed,
		RetrievalID:        w.IDs.Next(),
		Minimization:       query.MinimizationReport{Method: "lexical"},
		CategoriesSearched: asked,
		Directives:         &directives,
		RespondedAt:        w.Clock.ISO(),
	})
}

// Step 4 — hcp/grants/revoke: only the person's Management Client may (HCP 6.1.9, 6.2.1).
func handleRevoke(w *World, me *Party, doc *tt.Document[revoke.Payload]) (*tt.Document[json.RawMessage], error) {
	if doc.Payload.AccessToken == nil || *doc.Payload.AccessToken != ManagementToken {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	g := w.grantByID(doc.Payload.GrantID)
	if g == nil {
		return refuse(w, me, doc, "permissionDenied", neutralDenial)
	}
	if g.status == grantrequest.GrantStatusActive {
		g.status = grantrequest.GrantStatusRevoked // immediate (HCP 6.2.2); a second revoke is idempotent
		g.revokedAt = w.Clock.ISO()
	}
	return reply(w, me, doc, revoke.Response{})
}

// Step 6 — hcp/grants/revoked, at the Context Client (HCP 6.2.2, 9.9;
// hcp/grants/revoked "Delivery rules").
func handleRevoked(w *World, me *Party, doc *tt.Document[revoked.Payload]) (*tt.Document[json.RawMessage], error) {
	p := doc.Payload
	// Deduplicate on noticeId: a later attempt of a processed notice is
	// acknowledged, not acted on again. (A real client keeps the record at
	// least 25 hours; the demo keeps it for the run.)
	if _, seen := w.ProcessedNotices[p.NoticeID]; !seen {
		w.ProcessedNotices[p.NoticeID] = p.GrantID
		w.ForgetGrant(p.GrantID)
	}
	return reply(w, me, doc, revoked.Response{})
}

// expireIfDue ends a pending hold whose lifetime has passed: the policy denies
// (hcp/grants/request "Holding" rule 6).
func (w *World) expireIfDue(h *hold) {
	if h.state == pending && w.Clock.Now().Sub(h.heldAt) > HoldLifetime*time.Second {
		h.state = expired
	}
}

// personDecides is the person deciding, in the provider's own modality. Only a
// pending hold changes; an ended thread stays ended.
func (w *World) personDecides(threadID, verb string, state holdState) {
	h := w.held[threadID]
	w.expireIfDue(h)
	line := fmt.Sprintf("[person] %s the grant request on thread %s in the provider's app", verb, threadID)
	if h.state == pending {
		h.state = state
		fmt.Fprintf(&w.transcript, "%s\n", line)
	} else {
		fmt.Fprintf(&w.transcript, "%s, but the hold has ended: nothing changes\n", line)
	}
}

// PersonApproves is the person approving (the toy approves everything asked).
func (w *World) PersonApproves(threadID string) { w.personDecides(threadID, "approves", approved) }

// PersonDeclines is the person saying no.
func (w *World) PersonDeclines(threadID string) { w.personDecides(threadID, "declines", declined) }

// PersonDoesNotDecide is the person never deciding: the hold's lifetime passes
// with no document sent.
func (w *World) PersonDoesNotDecide(threadID string) {
	fmt.Fprintf(&w.transcript, "[person] does not decide on thread %s; the hold's lifetime (%d s) passes\n", threadID, HoldLifetime)
	w.Clock.Advance(HoldLifetime)
}

// ForgetGrant is the Context Client no longer using a grant: it drops the
// token and every copy disclosed under it (HCP 6.2.2, 7.6).
func (w *World) ForgetGrant(grantID string) { delete(w.Wallet, grantID) }

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

// replyOf decodes a reply document's type and payload.
func replyOf[P any](raw json.RawMessage) tt.Document[P] {
	var doc tt.Document[P]
	must(json.Unmarshal(raw, &doc))
	return doc
}

// ── The eight steps, one function each ─────────────────────────────────────

// Ask is what the Context Client asks for in step 2.
func Ask() grantrequest.Payload {
	description := "Book trips that match how you like to travel"
	expiresIn := int64(14400)
	subjectHint := "acct:alex@provider.example"
	return grantrequest.Payload{
		Access: []grantrequest.AccessRequest{{
			Type:       "https://hcp.example/spec/authorization/context-access",
			Actions:    []grantrequest.GrantAction{grantrequest.GrantActionRead},
			Categories: []string{"travel_preferences"},
			Purposes:   &[]grantrequest.RequestedPurpose{{Code: Purpose, Description: description}},
		}},
		ExpiresIn:   &expiresIn,
		SubjectHint: &subjectHint,
	}
}

// QueryPayload is the query the Context Client sends in steps 3 and 5, under
// the grant's token.
func QueryPayload(accessToken string) query.Payload {
	maxEntries := int64(5)
	return query.Payload{Query: "How does Alex like to fly?", Categories: &[]string{"travel_preferences"}, Purpose: Purpose, MaxEntries: &maxEntries, AccessToken: accessToken}
}

// StepDiscover is step 1, Discover: does the provider speak hcp/*?
// (trust-task-discovery 0.4, SPEC §10)
func StepDiscover(w *World) {
	P, C := w.Provider, w.ContextClient
	w.step("1. Discover (trust-task-discovery 0.4)")
	found := replyOf[discovery.Response](w.Exchange(C, P, Request(w, C, P, discovery.TypeURI, discovery.Payload{Patterns: &[]string{"hcp/*"}}, "")))
	if !slices.ContainsFunc(found.Payload.SupportedTypes, func(t json.RawMessage) bool { return string(t) == string(quoted(grantrequest.TypeURI)) }) {
		panic("provider does not serve grant requests")
	}
}

// DefaultRetryAfter is how long the client waits before it resubmits when the
// next step's hint is absent or fails NextStepHint.
const DefaultRetryAfter = 5

// RetryAfterOf is the seconds to wait before resubmitting, read from a next
// step's `hint` (nil when absent). The client validates the hint against
// `$defs.NextStepHint` in hcp/grants/request's schema (an object whose only
// member, `retryAfter`, is an integer from 1 to 3600) and ignores a hint that
// fails, as if it were absent (hcp/grants/request "Holding" rule 5,
// trust-task-next-step).
func RetryAfterOf(hint json.RawMessage) int64 {
	var h struct {
		RetryAfter int64 `json:"retryAfter"`
	}
	if hint == nil || !validNextStepHint(hint) || json.Unmarshal(hint, &h) != nil {
		return DefaultRetryAfter
	}
	return h.RetryAfter
}

// retryAfterOfNextStep is RetryAfterOf the hint of a next step's first
// expectation.
func retryAfterOfNextStep(p nextstep.Payload) int64 {
	var hint json.RawMessage
	if len(p.Expects) > 0 && p.Expects[0].Hint != nil {
		hint, _ = json.Marshal(*p.Expects[0].Hint)
	}
	return RetryAfterOf(hint)
}

// StepGrantRequest is step 2, Grant request: held with a next step while the
// person decides, then resubmitted on the same thread (HCP 6.1.6, 6.1.7; SPEC
// §8.6). The client keeps the token in its wallet. It returns the thread and
// the grant.
func StepGrantRequest(w *World) (threadID, grantID string) {
	P, C := w.Provider, w.ContextClient
	w.step("2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step")
	first := Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), "")
	held := replyOf[nextstep.Payload](w.Exchange(C, P, first))
	if held.Type != nextstep.TypeURI {
		panic("expected a next step")
	}
	threadID = *replyOf[json.RawMessage](first).ThreadID
	w.PersonApproves(threadID)
	// The client SHOULD NOT resubmit sooner than the next step's retryAfter
	// (hcp/grants/request "Holding").
	w.Clock.Advance(retryAfterOfNextStep(held.Payload))
	// Resubmission: a new document, fresh id, same threadId and payload.
	granted := replyOf[grantrequest.Response](w.Exchange(C, P, Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), threadID)))
	if granted.Type != grantrequest.ResponseTypeURI {
		panic("expected the grant")
	}
	grantID = granted.Payload.Grant.GrantID
	w.Wallet[grantID] = &HeldGrant{Token: granted.Payload.AccessToken, Copies: []string{}}
	return threadID, grantID
}

// StepQuery is step 3, Query under the grant (hcp/entries/query 0.1, HCP
// 5.2.1). The client keeps the refs of the copies.
func StepQuery(w *World, grantID string) {
	P, C := w.Provider, w.ContextClient
	w.step("3. Query (hcp/entries/query 0.1)")
	held := w.Wallet[grantID]
	answer := replyOf[query.Response](w.Exchange(C, P, Request(w, C, P, query.TypeURI+"#request", QueryPayload(held.Token), "")))
	if answer.Type != query.ResponseTypeURI {
		panic("expected entries")
	}
	for _, e := range answer.Payload.Entries {
		var d struct {
			Ref string `json:"ref"`
		}
		must(json.Unmarshal(e, &d))
		held.Copies = append(held.Copies, d.Ref)
	}
}

// StepRevoke is step 4, Revoke from the person's Management Client
// (hcp/grants/revoke 0.1, HCP 6.2.1).
func StepRevoke(w *World, grantID string) {
	P, M := w.Provider, w.Management
	w.step("4. Revoke (hcp/grants/revoke 0.1) from the Management Client")
	token := ManagementToken
	if done := replyOf[json.RawMessage](w.Exchange(M, P, Request(w, M, P, revoke.TypeURI+"#request", revoke.Payload{GrantID: grantID, AccessToken: &token}, ""))); done.Type != revoke.ResponseTypeURI {
		panic("expected the revocation")
	}
}

// StepQueryAfterRevoke is step 5: the same query, sent before any notice
// arrives, fails with hcp:invalidToken. The grant is checked on every request,
// so revocation takes effect at once (HCP 6.2.2). The client acts on that at
// once too: it drops the token and the copies (hcp/grants/revoked
// "Independence"). It returns the reply.
func StepQueryAfterRevoke(w *World, grantID string) tt.Document[tt.ErrorPayload] {
	P, C := w.Provider, w.ContextClient
	w.step("5. Query after revoke (hcp/entries/query 0.1) fails with hcp:invalidToken")
	held := w.Wallet[grantID]
	refused := replyOf[tt.ErrorPayload](w.Exchange(C, P, Request(w, C, P, query.TypeURI+"#request", QueryPayload(held.Token), "")))
	if refused.Type == tt.TrustTaskErrorTypeURI && refused.Payload.Code == "hcp:invalidToken" {
		w.ForgetGrant(grantID)
	}
	return refused
}

// StepNotice is step 6, Revocation notice to the grant's client
// (hcp/grants/revoked 0.1, HCP 6.2.3, 9.9). The client records the noticeId and
// the grant and stops using the grant; it never sends the old token again. It
// returns the notice sent.
func StepNotice(w *World, grantID string) tt.Document[revoked.Payload] {
	P := w.Provider
	w.step("6. Revocation notice (hcp/grants/revoked 0.1)")
	g := w.grantByID(grantID)
	client := w.byVID(g.client)
	// noticeId is HCP's jti: minted once per notice, before the notice's own id,
	// and kept on every delivery attempt.
	payload := revoked.Payload{GrantID: grantID, NoticeID: w.IDs.Next(), RevokedAt: g.revokedAt}
	notice := Request(w, P, client, revoked.TypeURI+"#request", payload, "")
	if ack := replyOf[json.RawMessage](w.Exchange(P, client, notice)); ack.Type != revoked.ResponseTypeURI {
		panic("expected the acknowledgement")
	}
	return replyOf[revoked.Payload](notice)
}

// expectDenied panics unless reply is the fixed permissionDenied every refusal
// of a grant request carries.
func expectDenied(reply tt.Document[json.RawMessage]) {
	var p tt.ErrorPayload
	if reply.Type != tt.TrustTaskErrorTypeURI || json.Unmarshal(reply.Payload, &p) != nil || p.Code != "permissionDenied" {
		panic("expected permissionDenied")
	}
}

// Declined is what step 7 produces: the thread and the refusal.
type Declined struct {
	ThreadID string
	Refused  tt.Document[json.RawMessage]
}

// StepDecline is step 7: the person declines (HCP 6.1.7), so the client's
// next resubmission on the thread is refused with the fixed permissionDenied,
// and no grant is issued (hcp/grants/request "Errors").
func StepDecline(w *World) Declined {
	P, C := w.Provider, w.ContextClient
	w.step("7. Grant request declined (hcp/grants/request 0.1): the person says no")
	first := Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), "")
	held := replyOf[nextstep.Payload](w.Exchange(C, P, first))
	if held.Type != nextstep.TypeURI {
		panic("expected a next step")
	}
	threadID := *replyOf[json.RawMessage](first).ThreadID
	w.PersonDeclines(threadID)
	w.Clock.Advance(retryAfterOfNextStep(held.Payload))
	refused := replyOf[json.RawMessage](w.Exchange(C, P, Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), threadID)))
	expectDenied(refused)
	return Declined{ThreadID: threadID, Refused: refused}
}

// ExpiredHold is what step 8 produces: both threads and the refusals.
type ExpiredHold struct {
	ThreadID, NewThreadID string
	Refused               []tt.Document[json.RawMessage]
}

// StepExpiredHold is step 8: the hold's lifetime passes with no decision, so the
// policy denies. The next resubmission gets the same permissionDenied; the
// thread is never reopened, so a late approval changes nothing and a further
// resubmission is refused too; a new request on a new thread is put to the
// person again (hcp/grants/request "Holding" rule 6).
func StepExpiredHold(w *World) ExpiredHold {
	P, C := w.Provider, w.ContextClient
	w.step("8. Grant request expired (hcp/grants/request 0.1): the hold's lifetime passes")
	ask := func(threadID string) json.RawMessage {
		return w.Exchange(C, P, Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), threadID))
	}
	first := Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), "")
	if replyOf[json.RawMessage](w.Exchange(C, P, first)).Type != nextstep.TypeURI {
		panic("expected a next step")
	}
	threadID := *replyOf[json.RawMessage](first).ThreadID
	w.PersonDoesNotDecide(threadID)
	refused := []tt.Document[json.RawMessage]{replyOf[json.RawMessage](ask(threadID))}
	w.PersonApproves(threadID) // too late: the thread has ended
	refused = append(refused, replyOf[json.RawMessage](ask(threadID)))
	for _, r := range refused {
		expectDenied(r)
	}
	// A client that still needs a grant asks again, on a new thread.
	again := Request(w, C, P, grantrequest.TypeURI+"#request", Ask(), "")
	if replyOf[json.RawMessage](w.Exchange(C, P, again)).Type != nextstep.TypeURI {
		panic("expected a next step")
	}
	return ExpiredHold{ThreadID: threadID, NewThreadID: *replyOf[json.RawMessage](again).ThreadID, Refused: refused}
}

// Result is what a run produces.
type Result struct {
	Transcript      string
	ThreadID        string
	GrantID         string
	PostRevokeError tt.Document[tt.ErrorPayload]
	Notice          tt.Document[revoked.Payload]
	Declined        Declined
	ExpiredHold     ExpiredHold
}

// RunDemo plays the eight steps, in order, on w (a fresh world when nil).
func RunDemo(w *World) Result {
	if w == nil {
		w = NewWorld()
	}
	StepDiscover(w)
	threadID, grantID := StepGrantRequest(w)
	StepQuery(w, grantID)
	StepRevoke(w, grantID)
	postRevokeError := StepQueryAfterRevoke(w, grantID)
	notice := StepNotice(w, grantID)
	declined := StepDecline(w)
	expiredHold := StepExpiredHold(w)
	return Result{Transcript: w.transcript.String(), ThreadID: threadID, GrantID: grantID, PostRevokeError: postRevokeError, Notice: notice, Declined: declined, ExpiredHold: expiredHold}
}
