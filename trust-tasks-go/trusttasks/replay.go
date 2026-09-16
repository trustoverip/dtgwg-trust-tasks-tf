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
	id          string
	digest      string
	retainUntil time.Time
	bounded     bool
	response    json.RawMessage
	completed   bool
}

// InMemoryReplayGuard is a bounded, in-process [ReplayGuard]: an LRU map from id
// to the digest accepted under it, its retention deadline, and the response it
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
// Eviction is by capacity as well as by retainUntil: a burst of distinct
// documents can push an older record out before its deadline, and a replay
// arriving after that would be accepted. Size the capacity above the number of
// distinct documents the widest acceptance window can hold.
//
// Safe for concurrent use, which is what makes claim-and-record atomic as the
// [ReplayGuard] contract requires.
type InMemoryReplayGuard struct {
	mu       sync.Mutex
	capacity int
	order    *list.List               // front = least recently used
	entries  map[string]*list.Element // id -> element holding *replayEntry
}

// DefaultReplayCapacity is the record count [NewInMemoryReplayGuard] retains when
// given no explicit capacity.
const DefaultReplayCapacity = 10_000

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
		capacity: capacity,
		order:    list.New(),
		entries:  make(map[string]*list.Element),
	}
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
	for element := g.order.Front(); element != nil; {
		next := element.Next()
		entry := element.Value.(*replayEntry)
		if entry.bounded && !entry.retainUntil.After(now) {
			g.remove(element)
		}
		element = next
	}
}

// Claim implements [ReplayGuard].
func (g *InMemoryReplayGuard) Claim(
	_ context.Context,
	id, digest string,
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
				// A conflicting document is not a use of the record, so it does not
				// refresh recency — otherwise a flood of conflicts could pin an
				// entry and evict live ones.
				return ReplayVerdict{Kind: VerdictConflict}, nil
			}
			g.order.MoveToBack(element)
			return ReplayVerdict{
				Kind:          VerdictDuplicate,
				PriorResponse: entry.response,
				InFlight:      !entry.completed,
			}, nil
		}
	}

	element := g.order.PushBack(&replayEntry{
		id:          id,
		digest:      digest,
		retainUntil: retainUntil,
		bounded:     !retainUntil.IsZero(),
	})
	g.entries[id] = element
	for len(g.entries) > g.capacity {
		oldest := g.order.Front()
		if oldest == nil {
			break
		}
		g.remove(oldest)
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
}
