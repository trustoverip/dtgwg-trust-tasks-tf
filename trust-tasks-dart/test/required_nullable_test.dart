// A required member whose schema admits null must keep its key when it is
// null. vta/contexts/update-did 1.2 requires `did` and reads `"did": null` as
// "leave the context with no DID of its own"; a document without `did` fails
// the schema's `required`. The generated field is nullable (null is JSON
// null), and toJson must write it whatever its value, or a null DID drops the
// key. This pins scripts/build-dart-bindings.mjs; test/generator_test.dart's
// 'required nullable' group pins the per-module generator.

import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks/specs/acl/update/v0_3/payload.dart' as acl_update;
import 'package:trust_tasks/specs/vta/contexts/update_did/v1_2/payload.dart'
    as update_did;

void main() {
  test('a null required nullable member is written as null', () {
    const p = update_did.Payload(id: 'ctx/payments', did: null);
    expect(jsonEncode(p.toJson()), '{"id":"ctx/payments","did":null}');
  });

  test('a required nullable member round-trips null and a value', () {
    for (final wire in [
      '{"id":"ctx/payments","did":null}',
      '{"id":"ctx/payments","did":"did:web:payments.example"}',
    ]) {
      final p =
          update_did.Payload.fromJson(jsonDecode(wire) as Map<String, dynamic>);
      expect(jsonEncode(p.toJson()), wire);
    }
  });

  // The other half of the rule: an optional nullable member is still omitted
  // when null. acl/update 0.3 reads an absent `label` as "leave it unchanged"
  // (and `"label": null` as "clear it", which a Dart client cannot send:
  // docs/runtime-parity.md).
  test('a null optional nullable member writes no key', () {
    const p = acl_update.Payload(subject: 'did:example:alice');
    final json = p.toJson();
    expect(json.containsKey('label'), isFalse);
    expect(json['subject'], 'did:example:alice');
  });
}
