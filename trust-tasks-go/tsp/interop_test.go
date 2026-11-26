package tsp_test

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"testing"
	"time"

	atsp "github.com/affinidi/affinidi-tsp-go"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp"
)

// Fixed TSP messages from other implementations. All three carry the binding's
// {type, document} envelope from did:example:alice to did:example:bob, sealed
// with the same parties' keys: alice signs with the Ed25519 seed of 32 0x01
// bytes, and bob's X25519 private key is 32 0x66 (102) bytes. They are the
// fixtures trust-tasks-ts-tsp's test/fixtures.ts uses (#1387), plus a Rev 3
// message from the affinidi-tsp-go this module required before TSP 1.0. HPKE
// draws a fresh ephemeral key on every seal, so none is reproducible byte for
// byte.

// rustTSP10Hex is a TSP 1.0 (YTSP-BAA) Direct message packed by the Rust
// affinidi-tsp 0.5.0 — the crate the Rust trust-tasks-tsp binding seals with —
// through affinidi_tsp::message::direct::pack(body, MessageType::Direct,
// "did:example:alice", "did:example:bob", &alice_signing_seed,
// &bob_x25519_public), with body the serde_json bytes of the envelope. The
// generating program unpacked it with the same crate before printing it.
const rustTSP10Hex = "f8408061348ff81000e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c653a626f62e05070dcf39858172f2f081019d8b304436fcd20facc9141e75b3f6c86ebcb0c958f417d4b830b6e10d4ecbf10e157aba5e3c6617f8b29f4be9feb9afa07504ee530c8906b7219e42bc1b37295d3b39f52bc5c806ad316e1e20a7e8237343c6b14cf72e92172a6f5cc30999beb2a3effcb88c18889dff54591796a86b8f9dc82142056d94a965154f737e6115be11a0edc7600ced661fbb77ac2de0c8a84f9c87079b2d3aa4284b05dc5005fdc62d8eee75ff181d57c0330a2abe170ff8bbb3dc81a63bb2db5fa23289d773b1da9411e5ac348b26453980b97af243cbe4e768637d2bb3c5092f28cdf84eb600e985cf74dc17f3a3ba2d89b97e4a313ee9d26e5b468f0df1bb0561ae1dc35499b840b88d2b756ca212c93702480ec3a8e5eabc69ffff53b316ad6c7495f596d7681eee31ae7894348cae252c4971e4990155645751bbec2e53dfcad96902eb8e607c32aad7c64f82017f8a0160400f907598aae27a4762637af72f2f233c9e4b34f2bcdea5c60b465fc32720fe0521ae78280fb57dd0268ea35ac08e1dae08152e5ac7e6116d1c8aad18b1ea0c703"

// rustTSP10Document is the document rustTSP10Hex carries.
var rustTSP10Document = json.RawMessage(`{"id":"urn:uuid:00000000-0000-4000-8000-0000000000a1","type":"https://trusttasks.org/spec/example/echo/0.1","issuedAt":"2026-06-01T12:00:00Z","payload":{"text":"from affinidi-tsp"}}`)

// goRev3Hex is a TSP Rev 3 (YTSP-AAC) Direct message packed by affinidi-tsp-go
// v0.0.0-20260916130837-d32ca7cdde20 — the pseudo-version this module required
// before it moved to TSP 1.0 — with tsp.Pack(alice, bob.Public(), &tsp.SCS{Data:
// body}, &tsp.PackOptions{Scheme: tsp.SchemeHPKEBase}); alice's X25519 key is 32
// 0x65 bytes. It stands for a Go or Dart peer of this repository that has not
// upgraded. The generating program opened it with the same library first.
const goRev3Hex = "f8407d61348ff80002e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c653a626f62e0506d05e98823214d48869ab16c103da5127bb8a9ae817f0056d92531695cc2ee8a776c43cdaf474e235166b8ab51f19703845f86e04d492bf2d1cbb16c2227de3c8fb536b2aa279d41c8703e00ce4e9d218f82b4357bd855304bbc85752371928a219380cc802c678bf64317c273d0fe1099a55c342289e348dce91a554b80d3bbf651f5e80c5b041bcf66fc9bc91ee017fb6121e5e499cf62fc715336768031399dd85677b26159fcc427e4ce41e8f8667d7b3954f1c3adc9e8559740f7fbf4081db8344b16fd5c8cae1c1b6c59387f47f020a8d9c8b5698ffe0fd88b266c0f5f6cb47c32aa544cfac271b970a6def8645b728d97ac4c394804e95c61738051b7bdcb7409ccdf264e2c71d6ade9651e0e740344e83e998d8cb34a3b7cdcd3a1f12556f66292b0f055f93c6ee0ebe7b479cab7e9cb8ade8fd38257ee0fa481a11f4c7de8713ae13bb3f82017f8a01604000d3c3aa590b545eb838f579da49ba05918bf6554dd3ef8d8a51aa08903582974d78dae43e304e1ac0ad03a3eb4f3d6b8a7111a56e69cf348d45ab64d64a6a205"

// vtiTSPJSRev2Hex is a TSP Rev 2 (YTSP-AAB) Direct message, HPKE-Auth sealed,
// packed by @openvtc/vti-tsp-js 0.2.0 with alice's X25519 private key of 32 0x65
// bytes as the HPKE-Auth sender key: #1387's VTI_TSP_JS_0_2_REV2_HEX.
const vtiTSPJSRev2Hex = "f8401061348ff80001e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c653a626f625c0000e060674a8d67c84e02956663e9b8349532d9117d55f25a757553e916663b5067b1c085d81341b8239eaceaa32bd58cc2708cd5bfc53db86d409e8c1230e01081b0b5bb4e3d4724c66ddc1ef7c1849ce124148ec3f1a012a638e0c321826e0df592c2dfc64ac622d974bc1012608670a2b7ca51392a65332cb3c740b5408c8ab9d74a474d07c3a71ffc2f79070001e17d6b650b38dfb08f5283ecb8647114564bf62207e869afa01491ed0b49dc0dc2ffe970d12de025743b0595ff1594ab95816421d7e42cec61b55d57c83fa8be372d85f5c76f1a4b1b661d0695d366ceeb4d6760ec9e5c83a7eff71c959b18d68f7ff6ea6f64c419ac02493b4a14d6f99d076e1d3140ae44d00c8551d34a54abd5ff4f310f7e7b198f4a85086ce1dd5cca1bf1c002ed041d527b116f2d350a54989bfdc65e3597552b29f82016f8a016d010b694a72107551cc44102175cd59e4eaf4913ae3460707dac1f634e01bb289f4026f862d60930944ea09b62f2d621be936fb429118571de7038951feaa071ef05"

func mustHex(t *testing.T, s string) []byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatalf("fixture hex: %v", err)
	}
	return b
}

// fixtureParties are alice and bob with the fixtures' keys. Only alice's public
// identity and bob's private one take part in opening a message.
func fixtureParties(t *testing.T) (alice, bob *atsp.PrivateIdentity) {
	t.Helper()
	fill := func(b byte) []byte { return bytes.Repeat([]byte{b}, 32) }
	alice, err := atsp.NewPrivateIdentity("did:example:alice", atsp.SigKeyEd25519, fill(0x01), atsp.EncKeyX25519, fill(0x65))
	if err != nil {
		t.Fatal(err)
	}
	bob, err = atsp.NewPrivateIdentity("did:example:bob", atsp.SigKeyEd25519, fill(0x02), atsp.EncKeyX25519, fill(0x66))
	if err != nil {
		t.Fatal(err)
	}
	return alice, bob
}

func TestSealsTSP10(t *testing.T) {
	if tsp.TSPVersion != "1.0" {
		t.Fatalf("TSPVersion = %q", tsp.TSPVersion)
	}
	alice, bob := fixtureParties(t)
	wire, err := tsp.PackTrustTask(rustTSP10Document, alice, bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	env, err := atsp.Peek(wire)
	if err != nil {
		t.Fatal(err)
	}
	if env.Version != (atsp.Version{Major: 1, Minor: 0}) {
		t.Fatalf("sealed in TSP %v, want 1.0", env.Version)
	}
}

// A TSP 1.0 message sealed by the Rust affinidi-tsp crate opens here, both
// through UnpackTrustTask and through the guarded consumer.
func TestOpensTSP10MessagePackedByRust(t *testing.T) {
	alice, bob := fixtureParties(t)
	wire := mustHex(t, rustTSP10Hex)

	if got := tsp.AdvertisedSender(wire); got != alice.VID {
		t.Fatalf("advertised sender = %q, want %q", got, alice.VID)
	}
	unpacked, err := tsp.UnpackTrustTask(wire, bob, alice.Public())
	if err != nil {
		t.Fatal(err)
	}
	if !jsonEqual(unpacked.Document, rustTSP10Document) {
		t.Fatalf("document:\n got %s\nwant %s", unpacked.Document, rustTSP10Document)
	}
	if unpacked.Transport.Peer != alice.VID || unpacked.Transport.Local != bob.VID {
		t.Fatalf("transport = %+v", unpacked.Transport)
	}

	consumer := tsp.NewConsumer(bob)
	consumer.Now = func() time.Time { return fixedNow }
	got, err := tsp.Receive(context.Background(), consumer, wire, alice.Public(), echoSpec, decodeEcho,
		func(_ context.Context, d *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
			if d.Payload.Text != "from affinidi-tsp" {
				t.Errorf("payload text = %q", d.Payload.Text)
			}
			return tt.RespondWith[echo, echo](d, "urn:uuid:00000000-0000-4000-8000-0000000000b1", echo{Text: "ok"}, nil), nil
		})
	if err != nil {
		t.Fatal(err)
	}
	if got.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("outcome = %v", got.Outcome.Kind)
	}
	// The reply goes back to the Rust peer in TSP 1.0.
	reply, err := tsp.PackReply(bob, alice.Public(), got.Reply)
	if err != nil {
		t.Fatal(err)
	}
	if env, err := atsp.Peek(reply); err != nil || env.Version.Major != 1 {
		t.Fatalf("reply version: %v, %v", env, err)
	}
}

// A peer still on a TSP draft is refused from the version marker, before any key
// is used: with the right sender identity, with none, and through the consumer.
// Its sender cannot be read, so a caller cannot resolve one for it.
func TestRefusesTSPDraftPeers(t *testing.T) {
	alice, bob := fixtureParties(t)
	for _, tc := range []struct{ name, hex string }{
		{"Rev 3 from affinidi-tsp-go d32ca7cd", goRev3Hex},
		{"Rev 2 from vti-tsp-js 0.2", vtiTSPJSRev2Hex},
	} {
		t.Run(tc.name, func(t *testing.T) {
			wire := mustHex(t, tc.hex)
			if got := tsp.AdvertisedSender(wire); got != "" {
				t.Fatalf("advertised sender = %q, want none for a draft", got)
			}
			_, err := tsp.UnpackTrustTask(wire, bob, alice.Public())
			e := assertFailure(t, err, tsp.FailUnsupportedTSPVersion)
			if e.Sender != "" {
				t.Fatalf("sender = %q: nothing is authenticated", e.Sender)
			}
			if atsp.KindOf(err) != atsp.KindVersion {
				t.Fatalf("cause kind = %v, want version", atsp.KindOf(err))
			}
			_, err = tsp.UnpackTrustTask(wire, bob, nil)
			assertFailure(t, err, tsp.FailUnsupportedTSPVersion)

			consumer := tsp.NewConsumer(bob)
			_, err = tsp.Receive(context.Background(), consumer, wire, alice.Public(), echoSpec, decodeEcho,
				func(context.Context, *tt.Document[echo], tt.ResolvedParties) (*tt.Document[echo], error) {
					t.Fatal("handler ran for a draft-version message")
					return nil, nil
				})
			assertFailure(t, err, tsp.FailUnsupportedTSPVersion)
		})
	}
}

// MINOR never gates a message (TSP 1.0 §3.1): a 1.x message with a MINOR this
// binding does not know is opened like any other, so one that then fails to open
// is FailNotForThisReceiver, not a version refusal. Bytes 6..8 are the version
// code, f8 10 00 for 1.0; the last carries MINOR's low bits. The signature
// covers the header, so the bumped message no longer verifies.
func TestUnknownTSPMinorIsNotAVersionRefusal(t *testing.T) {
	alice, bob := fixtureParties(t)
	wire, err := tsp.PackTrustTask(rustTSP10Document, alice, bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(wire[6:9], []byte{0xf8, 0x10, 0x00}) {
		t.Fatalf("version code = % x, want f8 10 00", wire[6:9])
	}
	bumped := bytes.Clone(wire)
	bumped[8] = 0x05
	env, err := atsp.Peek(bumped)
	if err != nil {
		t.Fatalf("a 1.x MINOR must frame: %v", err)
	}
	if env.Version != (atsp.Version{Major: 1, Minor: 5}) {
		t.Fatalf("version = %v, want 1.5", env.Version)
	}
	if got := tsp.AdvertisedSender(bumped); got != alice.VID {
		t.Fatalf("advertised sender = %q, want %q", got, alice.VID)
	}
	_, err = tsp.UnpackTrustTask(bumped, bob, alice.Public())
	assertFailure(t, err, tsp.FailNotForThisReceiver)
	_, err = tsp.Receive(context.Background(), tsp.NewConsumer(bob), bumped, alice.Public(), echoSpec, decodeEcho,
		func(context.Context, *tt.Document[echo], tt.ResolvedParties) (*tt.Document[echo], error) {
			t.Fatal("handler ran for a message that does not verify")
			return nil, nil
		})
	assertFailure(t, err, tsp.FailNotForThisReceiver)
}
