package trusttasks

// Duplicate-execution protection — SPEC §7.2 item 11, §8.4, §10.1.
//
// Mirrors replay.rs in trust-tasks-rs and replay.ts in @openvtc/trust-tasks,
// check for check.
//
// # The rule
//
// §7.2 item 11 is normative and unconditional for a consequential Trust Task:
// once a consumer has accepted a document with a given id for execution,
// receiving that same document again MUST NOT cause the consequential effect a
// second time, and receiving a different document under the same id MUST be
// rejected with idConflict.
//
// §8.4 is the same mechanism from the producer's end: a retry is a bit-for-bit
// identical resend, and it is safe precisely because item 11 obliges the consumer
// to absorb it. Every transport binding in this repo delegates replay defence to
// the consumer — bindings/https/0.2 §5 says "Freshness / replay: None", and the
// DIDComm and TSP bindings say the same — so if the consumer does not do it,
// nobody does, and an ordinary mediator redelivery grants an ACL entry twice by
// accident. §10.1: "The rule deliberately does not distinguish a hostile replay
// from a legitimate transport retry, because at the document layer the two are
// indistinguishable."
//
// # The key
//
// The id alone — "Transport request identifiers, transport message identifiers,
// and execution handles MUST NOT substitute" — plus a digest of the canonical
// serialization, because "an id alone cannot distinguish the retry it must absorb
// from the conflict it must reject".
//
// [DocumentDigest] hashes the canonicalization of the whole document rather than
// the octets as received: a re-indented body or a member order chosen by an
// intermediary would otherwise make a legitimate §8.4 retry look like a different
// document.
//
// The digest covers the entire document, proof included. That differs
// deliberately from the §4.9.3 task digest, computed over JCS(document ∖ proof),
// and the spec spells the distinction out: "Item 11 and §8.4 ask which
// serialization arrived, so a re-signed proof over identical content makes a
// different document — that is the idConflict case, and the distinction is the
// whole point of the rule."

import (
	"container/list"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"time"
)

// DocumentDigest computes the content identity of a document, per SPEC §7.2's
// keying paragraph: SHA-256 over the canonical serialization of the whole
// document, as hex.
//
// Consumer-local. Never on the wire, and not the §4.9.3 task digest.
//
// Fails only if the payload cannot be marshalled, which for a document that
// arrived over any transport cannot happen — but a caller-constructed
// Document[P] over a hand-written json.Marshaler can, and a guard that silently
// keyed on a truncated document would be worse than one that refuses.
func DocumentDigest[P any](doc *Document[P]) (string, error) {
	encoded, err := json.Marshal(doc)
	if err != nil {
		return "", fmt.Errorf("trusttasks: document digest: %w", err)
	}
	canonical, err := CanonicalJSON(encoded)
	if err != nil {
		return "", err
	}
	return SHA256Hex(canonical), nil
}

// receivedDigest is [DocumentDigest] over a document as it arrived: SHA-256 over
// its canonical serialization, as hex. Members a typed decode would drop are
// part of it.
func receivedDigest(received json.RawMessage) (string, error) {
	canonical, err := CanonicalJSON(received)
	if err != nil {
		return "", fmt.Errorf("trusttasks: document digest: %w", err)
	}
	return SHA256Hex(canonical), nil
}

// VerdictKind is what a [ReplayGuard] says about a document offered for
// execution.
type VerdictKind string

const (
	// VerdictFresh means not seen before. The caller may execute.
	VerdictFresh VerdictKind = "fresh"

	// VerdictDuplicate means already accepted under the same digest — a §8.4
	// retry, or a replay. The caller MUST NOT execute again.
	VerdictDuplicate VerdictKind = "duplicate"

	// VerdictConflict means already accepted under a different digest. §7.2 item
	// 11 requires idConflict, and requires that this not be treated as a retry.
	VerdictConflict VerdictKind = "conflict"
)

// ReplayVerdict is a [VerdictKind] plus, for a duplicate, what is known about the
// first execution.
type ReplayVerdict struct {
	Kind VerdictKind

	// PriorResponse is the response the first execution produced, where one was
	// retained. Nil otherwise.
	PriorResponse json.RawMessage

	// InFlight reports whether that first execution is still running.
	InFlight bool
}

// ReplayGuard is the consumer-side record that makes SPEC §7.2 item 11 true.
//
// An interface, so a deployment can back it with Redis, Postgres, or anything
// that survives a process restart. [InMemoryReplayGuard] is the default and is
// correct for a single-process consumer; it is not correct behind a load
// balancer, where two replicas would each accept the same document once.
type ReplayGuard interface {
	// Claim claims id for execution on behalf of a document with identity digest.
	//
	// retainUntil is the instant past which the record may be dropped —
	// [RecordExpiry], which SPEC §7.2 makes the same instant as the end of the
	// consumer's willingness to execute the document. An implementation SHOULD
	// treat a record whose retainUntil has passed as absent, so the key is
	// released rather than conflicting forever with a document nobody would
	// execute.
	//
	// Implementations MUST make claim-and-record atomic with respect to
	// concurrent calls: two simultaneous deliveries of the same document must not
	// both receive [VerdictFresh]. That is the whole guarantee.
	//
	// Returning an error means the record could not be consulted.
	// [ConsumeInbound] fails closed on that, mapping it to unavailable with
	// Retryable true — a consumer that cannot establish whether a document is a
	// duplicate has not satisfied item 11, and executing anyway is the double
	// execution the rule forbids.
	Claim(ctx context.Context, id, digest string, retainUntil time.Time, now time.Time) (ReplayVerdict, error)

	// RecordResponse attaches the response a completed execution produced, so a
	// later duplicate can be answered with it per §7.2 (*Disposition of a
	// duplicate*) rather than merely absorbed in silence.
	//
	// A guard that records nothing still satisfies item 11 — the effect does not
	// happen twice — and is the right shape for a fire-and-forget specification,
	// which has no response to return. Implement it as a no-op returning nil.
	RecordResponse(ctx context.Context, id string, response json.RawMessage) error

	// Release releases a claim whose execution is not to stand — for example a
	// refusal the consumer marked retryable, where §8.4 has just invited the
	// producer to re-send the same bytes. Without this, that invited retry would
	// come back as an absorbed duplicate carrying the same failure forever.
	//
	// Implement it as a no-op returning nil if the store cannot support it; that
	// is safe but leaves the record in place until retainUntil.
	Release(ctx context.Context, id, digest string) error
}

// ReplayPolicyKind selects how a consumer applies SPEC §7.2 item 11.
type ReplayPolicyKind string

const (
	// ReplayGuarded applies item 11 using the policy's Guard. Correct for any
	// consequential spec.
	ReplayGuarded ReplayPolicyKind = "guard"

	// ReplayNotConsequential keeps no duplicate-execution record. Conformant only
	// where the task is not consequential (§2), or where the specification
	// "explicitly declares repeated execution safe and intended" — a property of
	// the operation, not of the consumer's convenience.
	ReplayNotConsequential ReplayPolicyKind = "notConsequential"
)

// ReplayPolicy is how a consumer applies (or knowingly disapplies) item 11.
type ReplayPolicy struct {
	Kind  ReplayPolicyKind
	Guard ReplayGuard
}

type replayEntry struct {
	issuer      string
	id          string
	digest      string
	retainUntil time.Time
	bounded     bool
	response    json.RawMessage
	completed   bool
}

// InMemoryReplayGuard is a bounded, in-process [ReplayGuard]: a map from id to
// the digest accepted under it, its retention deadline, and the response it
// produced.
//
// Suitable when one process is the sole consumer for the recipient VID it serves
// and losing the record on restart is acceptable.
//
// Not suitable when the consumer is replicated: two replicas hold separate maps,
// so a document accepted by replica A is fresh at replica B and the effect
// happens twice — the exact failure item 11 exists to prevent. Replicated
// deployments MUST back the guard with a shared store.
//
// When it is full: a record is dropped only once its retainUntil has passed —
// never to make room. When a new id arrives and every slot holds a record still
// inside its retention window, Claim returns [ErrReplayGuardFull], which
// [ConsumeInbound] answers unavailable (retryable): the guard refuses to execute
// a document it could not then remember, rather than forgetting one it promised
// to. Records whose window has closed are reclaimed first. The capacity is
// therefore a ceiling on distinct documents per retention window; see
// [DefaultReplayCapacity] for sizing. A document carrying an expiresAt but no
// issuedAt is not bounded by the expiry horizon (it runs from issuedAt), so
// under a policy that does not require issuedAt its record lives until its
// expiresAt.
//
// One party's share: so that one party cannot fill the guard and hold every slot
// for the length of the expiry horizon, no issuer may hold more than a share of
// the capacity — a tenth by default, rounded up ([InMemoryReplayGuard.WithPerIssuerLimit]
// changes it). A party at its share is refused with [ErrReplayGuardIssuerShare]
// (unavailable) for its own new documents; others are unaffected. The issuer is
// the one [ConsumeInbound] resolved (SPEC §4.8.1) and passes to
// [InMemoryReplayGuard.ClaimFor]; documents with no resolved issuer, and direct
// calls to Claim, share one allowance. The share is per identifier: where anyone
// can mint identifiers, a party using many is limited only by the capacity.
//
// Memory: at most capacity records, each its id, a digest and the response
// cached for duplicates — roughly capacity × (largest id + largest response) at
// worst.
//
// Safe for concurrent use, which is what makes claim-and-record atomic as the
// [ReplayGuard] contract requires.
type InMemoryReplayGuard struct {
	mu        sync.Mutex
	capacity  int
	perIssuer int
	held      map[string]int           // live records per issuer ("" = none)
	order     *list.List               // insertion order
	entries   map[string]*list.Element // id -> element holding *replayEntry
	// earliest is no later than any bounded record's retainUntil (it may be
	// earlier, after a removal). Lets a full guard skip the reclaiming scan when
	// nothing can have expired yet. Zero means no bounded record.
	earliest time.Time
}

// DefaultReplayCapacity is the live-record count [NewInMemoryReplayGuard]
// retains when given no explicit capacity: 100 000.
//
// Enough for the consequential policy's five-minute acceptance window at ~330
// distinct documents per second. Documents carrying an expiresAt hold their
// record until then (up to the expiry horizon, 30 days by default), so a
// consumer accepting them should size for that window instead — when the guard
// is full of live records it refuses new documents rather than forgetting one.
// Mirrors Rust's DEFAULT_REPLAY_CAPACITY.
const DefaultReplayCapacity = 100_000

// ErrReplayGuardFull is returned by [InMemoryReplayGuard.Claim] when every slot
// holds a record still inside its retention window.
var ErrReplayGuardFull = errors.New("trusttasks: in-memory replay guard is full of records inside their retention window")

// ErrReplayGuardIssuerShare is returned by [InMemoryReplayGuard.ClaimFor] when
// the document's issuer already holds its share of live records.
var ErrReplayGuardIssuerShare = errors.New("trusttasks: in-memory replay guard: this issuer holds its share of records")

// IssuerScopedReplayGuard is a [ReplayGuard] that is told which party a document
// is attributed to: the issuer SPEC §4.8.1 resolved for it, "" when there is
// none. [ConsumeInbound] calls ClaimFor in place of Claim on a guard that
// implements it, so the guard can share its capacity out between parties.
type IssuerScopedReplayGuard interface {
	ReplayGuard
	ClaimFor(ctx context.Context, issuer, id, digest string, retainUntil, now time.Time) (ReplayVerdict, error)
}

// NewInMemoryReplayGuard builds a guard retaining at most capacity records.
//
// A capacity of zero or less takes [DefaultReplayCapacity]. It is not treated as
// "retain nothing": a guard that retains nothing answers fresh to everything,
// which is a silent total defeat of item 11 rather than a visible
// misconfiguration.
func NewInMemoryReplayGuard(capacity int) *InMemoryReplayGuard {
	if capacity < 1 {
		capacity = DefaultReplayCapacity
	}
	return &InMemoryReplayGuard{
		capacity:  capacity,
		perIssuer: (capacity + 9) / 10,
		held:      make(map[string]int),
		order:     list.New(),
		entries:   make(map[string]*list.Element),
	}
}

// WithPerIssuerLimit lets one issuer hold at most limit live records (default: a
// tenth of the capacity, rounded up), clamped to 1..capacity. It returns g.
func (g *InMemoryReplayGuard) WithPerIssuerLimit(limit int) *InMemoryReplayGuard {
	g.mu.Lock()
	defer g.mu.Unlock()
	g.perIssuer = min(max(limit, 1), g.capacity)
	return g
}

// Size reports how many records are currently retained. For tests and metrics.
func (g *InMemoryReplayGuard) Size() int {
	g.mu.Lock()
	defer g.mu.Unlock()
	return len(g.entries)
}

// PurgeExpired drops every record whose retainUntil has passed.
func (g *InMemoryReplayGuard) PurgeExpired(now time.Time) {
	g.mu.Lock()
	defer g.mu.Unlock()
	g.purgeExpired(now)
}

func (g *InMemoryReplayGuard) purgeExpired(now time.Time) {
	var earliest time.Time
	for element := g.order.Front(); element != nil; {
		next := element.Next()
		entry := element.Value.(*replayEntry)
		if entry.bounded {
			if !entry.retainUntil.After(now) {
				g.remove(element)
			} else if earliest.IsZero() || entry.retainUntil.Before(earliest) {
				earliest = entry.retainUntil
			}
		}
		element = next
	}
	g.earliest = earliest
}

// Claim implements [ReplayGuard].
func (g *InMemoryReplayGuard) Claim(
	ctx context.Context,
	id, digest string,
	retainUntil time.Time,
	now time.Time,
) (ReplayVerdict, error) {
	return g.ClaimFor(ctx, "", id, digest, retainUntil, now)
}

// ClaimFor implements [IssuerScopedReplayGuard]: [InMemoryReplayGuard.Claim],
// counting the record against issuer's share.
func (g *InMemoryReplayGuard) ClaimFor(
	_ context.Context,
	issuer, id, digest string,
	retainUntil time.Time,
	now time.Time,
) (ReplayVerdict, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	if element, found := g.entries[id]; found {
		entry := element.Value.(*replayEntry)
		// An expired record is treated as absent: the consumer would refuse the
		// document under §7.2 item 4 or the acceptance window anyway, so holding
		// the key would only manufacture a permanent idConflict for an id nobody
		// can use.
		if entry.bounded && !entry.retainUntil.After(now) {
			g.remove(element)
		} else {
			if entry.digest != digest {
				return ReplayVerdict{Kind: VerdictConflict}, nil
			}
			return ReplayVerdict{
				Kind:          VerdictDuplicate,
				PriorResponse: entry.response,
				InFlight:      !entry.completed,
			}, nil
		}
	}

	// Full: reclaim the records whose window has closed, and if every slot still
	// holds a live record, refuse rather than drop one. A record dropped inside
	// its window is a replay accepted later.
	// Likewise for one issuer's share, so that one party cannot hold every slot
	// and refuse everyone else.
	if len(g.entries) >= g.capacity || g.held[issuer] >= g.perIssuer {
		if !g.earliest.IsZero() && !g.earliest.After(now) {
			g.purgeExpired(now)
		}
		if len(g.entries) >= g.capacity {
			return ReplayVerdict{}, ErrReplayGuardFull
		}
		if g.held[issuer] >= g.perIssuer {
			return ReplayVerdict{}, ErrReplayGuardIssuerShare
		}
	}

	bounded := !retainUntil.IsZero()
	element := g.order.PushBack(&replayEntry{
		issuer:      issuer,
		id:          id,
		digest:      digest,
		retainUntil: retainUntil,
		bounded:     bounded,
	})
	g.entries[id] = element
	g.held[issuer]++
	if bounded && (g.earliest.IsZero() || retainUntil.Before(g.earliest)) {
		g.earliest = retainUntil
	}
	return ReplayVerdict{Kind: VerdictFresh}, nil
}

// RecordResponse implements [ReplayGuard].
func (g *InMemoryReplayGuard) RecordResponse(
	_ context.Context,
	id string,
	response json.RawMessage,
) error {
	g.mu.Lock()
	defer g.mu.Unlock()
	element, found := g.entries[id]
	if !found {
		return nil
	}
	entry := element.Value.(*replayEntry)
	entry.response = response
	entry.completed = true
	return nil
}

// Release implements [ReplayGuard].
func (g *InMemoryReplayGuard) Release(_ context.Context, id, digest string) error {
	g.mu.Lock()
	defer g.mu.Unlock()
	element, found := g.entries[id]
	if !found {
		return nil
	}
	entry := element.Value.(*replayEntry)
	// Only release the claim this digest made, and only while it is unfinished: a
	// concurrent arrival that legitimately holds the key must not have it taken
	// away by another document's cleanup.
	if entry.digest == digest && !entry.completed {
		g.remove(element)
	}
	return nil
}

// remove drops one element. The caller holds g.mu.
func (g *InMemoryReplayGuard) remove(element *list.Element) {
	entry := element.Value.(*replayEntry)
	delete(g.entries, entry.id)
	g.order.Remove(element)
	if g.held[entry.issuer] <= 1 {
		delete(g.held, entry.issuer)
	} else {
		g.held[entry.issuer]--
	}
}
