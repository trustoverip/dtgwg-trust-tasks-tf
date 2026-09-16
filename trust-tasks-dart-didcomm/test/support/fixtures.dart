import 'package:ssi/ssi.dart';
import 'package:trust_tasks/trust_tasks.dart';

/// A did:key identity with a key-agreement key.
Future<DidKeyManager> party(KeyType keyType) async {
  final wallet = PersistentWallet(InMemoryKeyStore());
  final manager = DidKeyManager(wallet: wallet, store: InMemoryDidStore());
  await wallet.generateKey(keyId: 'key-1', keyType: keyType);
  await manager.addVerificationMethod('key-1');
  return manager;
}

Future<String> didOf(DidManager m) async => (await m.getDidDocument()).id;

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
  String? parentThreadId,
  Map<String, dynamic>? proof,
}) =>
    <String, dynamic>{
      'id': id,
      'type': echoType,
      if (threadId != null) 'threadId': threadId,
      if (parentThreadId != null) 'parentThreadId': parentThreadId,
      if (issuer != null) 'issuer': issuer,
      if (recipient != null) 'recipient': recipient,
      'issuedAt': fixedIssuedAt,
      'payload': <String, dynamic>{'text': text},
      if (proof != null) 'proof': proof,
    };
