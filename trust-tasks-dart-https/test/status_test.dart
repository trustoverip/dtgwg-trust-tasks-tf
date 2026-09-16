import 'package:test/test.dart';
import 'package:trust_tasks_https/trust_tasks_https.dart';

void main() {
  test('the binding spec §4 table', () {
    const table = {
      'malformedRequest': 400,
      'permissionDenied': 403,
      'idConflict': 409,
      'unsupportedType': 422,
      'unsupportedVersion': 422,
      'expired': 422,
      'proofRequired': 422,
      'proofInvalid': 422,
      'identityMismatch': 422,
      'wrongRecipient': 422,
      'cancelled': 422,
      'taskFailed': 422,
      'unavailable': 503,
      'internalError': 500,
    };
    table.forEach((code, status) {
      expect(statusForCode(code), status, reason: code);
    });
  });

  test('codes about identity share one status, so none is an oracle', () {
    final statuses = {
      for (final code in [
        'proofRequired',
        'proofInvalid',
        'identityMismatch',
        'wrongRecipient',
      ])
        statusForCode(code),
    };
    expect(statuses, {422});
  });

  test('frozen 0.1 snake_case spellings map like their 0.2 forms', () {
    expect(statusForCode('malformed_request'), 400);
    expect(statusForCode('id_conflict'), 409);
  });

  test('extended codes are 422; an unknown standard-looking code is 500', () {
    expect(statusForCode('acl/grant:lastAuthorityProtected'), 422);
    expect(statusForCode('somethingNew'), 500);
  });

  test('newUrnUuid is a version-4 UUID URN', () {
    final re = RegExp(
      r'^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-'
      r'[0-9a-f]{12}$',
    );
    final seen = <String>{};
    for (var i = 0; i < 100; i++) {
      final id = newUrnUuid();
      expect(id, matches(re));
      seen.add(id);
    }
    expect(seen, hasLength(100));
  });
}
