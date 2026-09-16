import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:ssi/ssi.dart';
import 'package:trust_tasks/trust_tasks.dart';

/// A did:key VID that can seal, sign, verify and open.
Future<PrivateVid> identity(KeyType keyType) async {
  final wallet = PersistentWallet(InMemoryKeyStore());
  final manager = DidKeyManager(wallet: wallet, store: InMemoryDidStore());
  await wallet.generateKey(keyId: 'key-1', keyType: keyType);
  await manager.addVerificationMethod('key-1');
  return manager.toTspPrivateVid();
}

const echoType = 'https://trusttasks.org/spec/example/echo/0.1';
const echoSpec = SpecPolicy(typeUri: echoType);

class Echo {
  const Echo(this.text);
  factory Echo.fromJson(Map<String, dynamic> json) =>
      Echo(json['text'] as String);
  final String text;
  Map<String, dynamic> toJson() => {'text': text};
}

final fixedNow = DateTime.utc(2026, 6, 1, 12);
const fixedIssuedAt = '2026-06-01T12:00:00Z';

Map<String, dynamic> echoDoc({
  String id = 'urn:uuid:00000000-0000-4000-8000-000000000001',
  String text = 'hello',
  String? issuer,
  String? recipient,
  String? threadId,
  Map<String, dynamic>? proof,
}) => <String, dynamic>{
  'id': id,
  'type': echoType,
  if (threadId != null) 'threadId': threadId,
  if (issuer != null) 'issuer': issuer,
  if (recipient != null) 'recipient': recipient,
  'issuedAt': fixedIssuedAt,
  'payload': <String, dynamic>{'text': text},
  if (proof != null) 'proof': proof,
};

/// A resolver over a fixed set of known VIDs — no network, for tests.
final class MapResolver implements VidResolver {
  MapResolver(this._byId);
  final Map<String, PublicVid> _byId;
  @override
  Future<PublicVid> resolve(String vid) async {
    final v = _byId[vid];
    if (v == null) throw TspUnsupportedException('unknown VID $vid');
    return v;
  }
}

Future<MapResolver> resolverFor(List<PrivateVid> parties) async {
  final ssi = SsiVidResolver();
  return MapResolver({for (final p in parties) p.id: await ssi.resolve(p.id)});
}
