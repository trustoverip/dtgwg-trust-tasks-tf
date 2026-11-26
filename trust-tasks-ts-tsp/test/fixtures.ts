// Fixed TSP messages for the interop tests in tsp.test.ts. Both carry the
// binding's {type, document} envelope from did:example:alice to did:example:bob,
// sealed with the test parties' keys: alice signs with the Ed25519 seed of 32
// 0x01 bytes, and bob's X25519 private key is 32 0x66 (102) bytes — `party(…, 1)`
// and `party(…, 2)` in tsp.test.ts.

/** A TSP 1.0 (`YTSP-BAA`) Direct message packed by the Rust `affinidi-tsp`
 * 0.5.0 — the crate the Rust `trust-tasks-tsp` binding seals with — through
 * `affinidi_tsp::message::direct::pack(body, MessageType::Direct, "did:example:alice",
 * "did:example:bob", &alice_signing_seed, &bob_x25519_public)`, with `body` the
 * `serde_json` bytes of the envelope. The generating program unpacked it with the
 * same crate before printing it. Not reproducible byte for byte: HPKE draws a
 * fresh ephemeral key on every seal. Its document: */
export const AFFINIDI_TSP_1_0_DOCUMENT = {
  id: "urn:uuid:00000000-0000-4000-8000-0000000000a1",
  type: "https://trusttasks.org/spec/example/echo/0.1",
  issuedAt: "2026-06-01T12:00:00Z",
  payload: { text: "from affinidi-tsp" },
};
export const AFFINIDI_TSP_1_0_HEX =
  "f8408061348ff81000e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c653a626f62e05070dcf39858172f2f081019d8b304436fcd20facc9141e75b3f6c86ebcb0c958f417d4b830b6e10d4ecbf10e157aba5e3c6617f8b29f4be9feb9afa07504ee530c8906b7219e42bc1b37295d3b39f52bc5c806ad316e1e20a7e8237343c6b14cf72e92172a6f5cc30999beb2a3effcb88c18889dff54591796a86b8f9dc82142056d94a965154f737e6115be11a0edc7600ced661fbb77ac2de0c8a84f9c87079b2d3aa4284b05dc5005fdc62d8eee75ff181d57c0330a2abe170ff8bbb3dc81a63bb2db5fa23289d773b1da9411e5ac348b26453980b97af243cbe4e768637d2bb3c5092f28cdf84eb600e985cf74dc17f3a3ba2d89b97e4a313ee9d26e5b468f0df1bb0561ae1dc35499b840b88d2b756ca212c93702480ec3a8e5eabc69ffff53b316ad6c7495f596d7681eee31ae7894348cae252c4971e4990155645751bbec2e53dfcad96902eb8e607c32aad7c64f82017f8a0160400f907598aae27a4762637af72f2f233c9e4b34f2bcdea5c60b465fc32720fe0521ae78280fb57dd0268ea35ac08e1dae08152e5ac7e6116d1c8aad18b1ea0c703";

/** A TSP Rev 2 (`YTSP-AAB`) Direct message, HPKE-Auth sealed, packed by
 * `@openvtc/vti-tsp-js` 0.2.0 — the version this package depended on before it
 * moved to TSP 1.0 — with alice's X25519 private key of 32 0x65 (101) bytes as
 * the HPKE-Auth sender key. It stands for a peer that has not upgraded. */
export const VTI_TSP_JS_0_2_REV2_HEX =
  "f8401061348ff80001e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c653a626f625c0000e060674a8d67c84e02956663e9b8349532d9117d55f25a757553e916663b5067b1c085d81341b8239eaceaa32bd58cc2708cd5bfc53db86d409e8c1230e01081b0b5bb4e3d4724c66ddc1ef7c1849ce124148ec3f1a012a638e0c321826e0df592c2dfc64ac622d974bc1012608670a2b7ca51392a65332cb3c740b5408c8ab9d74a474d07c3a71ffc2f79070001e17d6b650b38dfb08f5283ecb8647114564bf62207e869afa01491ed0b49dc0dc2ffe970d12de025743b0595ff1594ab95816421d7e42cec61b55d57c83fa8be372d85f5c76f1a4b1b661d0695d366ceeb4d6760ec9e5c83a7eff71c959b18d68f7ff6ea6f64c419ac02493b4a14d6f99d076e1d3140ae44d00c8551d34a54abd5ff4f310f7e7b198f4a85086ce1dd5cca1bf1c002ed041d527b116f2d350a54989bfdc65e3597552b29f82016f8a016d010b694a72107551cc44102175cd59e4eaf4913ae3460707dac1f634e01bb289f4026f862d60930944ea09b62f2d621be936fb429118571de7038951feaa071ef05";
