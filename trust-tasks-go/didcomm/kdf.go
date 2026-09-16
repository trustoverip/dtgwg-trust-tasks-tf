package didcomm

import (
	"crypto/sha256"
	"encoding/binary"
)

// concatKDF is the NIST SP 800-56A single-step (Concat) KDF with SHA-256, as
// DIDComm v2 ECDH-1PU key agreement uses it (RFC 7518 §4.6 +
// draft-madden-jose-ecdh-1pu-04 §2.3). For a key of 256 bits or fewer a single
// SHA-256 iteration suffices, which is every case this binding produces (a
// 256-bit A256KW key-wrapping key), so the loop is written for the general case
// but never runs more than a handful of rounds.
//
//	OtherInfo = AlgorithmID ‖ PartyUInfo ‖ PartyVInfo ‖ SuppPubInfo [‖ SuppPrivInfo]
//
// where AlgorithmID, PartyUInfo and PartyVInfo are each length-prefixed
// (a big-endian uint32 length followed by the bytes), SuppPubInfo is the
// key-data length in bits as a bare big-endian uint32, and SuppPrivInfo — the
// ECDH-1PU content-encryption authentication tag `ccTag` — is appended
// length-prefixed when non-empty. An empty ccTag is byte-identical to the plain
// ECDH-ES Concat KDF (RFC 7518 §4.6). This tag-in-KDF binding is what every
// interoperable DIDComm v2 implementation (aries-askar, didcomm-python,
// didcomm-rust, affinidi) does, and is what makes this binding wire-compatible
// with them.
func concatKDF(z []byte, algorithmID, partyUInfo, partyVInfo []byte, keyLenBits int, ccTag []byte) []byte {
	var other []byte
	other = append(other, lengthPrefixed(algorithmID)...)
	other = append(other, lengthPrefixed(partyUInfo)...)
	other = append(other, lengthPrefixed(partyVInfo)...)
	other = append(other, uint32be(uint32(keyLenBits))...)
	if len(ccTag) > 0 {
		other = append(other, lengthPrefixed(ccTag)...)
	}

	keyLenBytes := (keyLenBits + 7) / 8
	out := make([]byte, 0, keyLenBytes)
	for counter := uint32(1); len(out) < keyLenBytes; counter++ {
		h := sha256.New()
		h.Write(uint32be(counter))
		h.Write(z)
		h.Write(other)
		out = h.Sum(out)
	}
	return out[:keyLenBytes]
}

func lengthPrefixed(b []byte) []byte {
	return append(uint32be(uint32(len(b))), b...)
}

func uint32be(v uint32) []byte {
	var b [4]byte
	binary.BigEndian.PutUint32(b[:], v)
	return b[:]
}
