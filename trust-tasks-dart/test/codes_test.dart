// SPEC.md §8.3 and §8.5 error-code tests.
//
// Mirror trust-tasks-ts/test/codes.test.ts, trust-tasks-go/trusttasks and the
// error.rs tests in trust-tasks-rs.

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

const grant = 'https://trusttasks.org/spec/acl/grant/0.1';
const grantResponse = 'https://trusttasks.org/spec/acl/grant/0.1#response';
const didDelete = 'https://trusttasks.org/spec/did-management/did/delete/0.1';
const discovery = 'https://trusttasks.org/spec/trust-task-discovery/0.1';

void main() {
  group('§8.3 standard codes', () {
    test('accepts the 0.2 lowerCamelCase spellings', () {
      for (final code in <String>[
        'proofRequired',
        'identityMismatch',
        'expired'
      ]) {
        expect(isStandardCode(code), isTrue, reason: code);
      }
    });

    test('accepts the frozen 0.1 snake_case spellings and normalizes them', () {
      // A 0.2 consumer must still read an error response from a 0.1 peer;
      // otherwise proof_required reads as an unrecognized extended code and
      // falls through to taskFailed (§8.5), losing the meaning.
      expect(isStandardCode('proof_required'), isTrue);
      expect(normalizeCode('proof_required'), 'proofRequired');
      expect(normalizeCode('identity_mismatch'), 'identityMismatch');
      // `canceled` is the 0.1 spelling of `cancelled`, not a separate code.
      expect(normalizeCode('canceled'), 'cancelled');
    });

    test('leaves an extended code alone', () {
      const extended = 'acl/grant:roleNotRecognized';
      expect(normalizeCode(extended), extended);
      expect(isStandardCode(extended), isFalse);
    });

    test('lists every §8.3 code', () {
      expect(StandardCode.values, hasLength(14));
      expect(StandardCode.values.first, StandardCode.malformedRequest);
    });

    test(
        'an unrecognised code does not throw — SPEC §5.2 forward compatibility',
        () {
      // The whole reason StandardCode is an extension type and not an enum: a
      // peer on a newer MINOR must not crash this consumer's parse.
      const fromFuture = StandardCode('quarantined');
      expect(fromFuture.value, 'quarantined');
      expect(StandardCode.values.contains(fromFuture), isFalse);
      expect(isStandardCode(fromFuture.value), isFalse);
    });
  });

  group('§8.5 extended codes', () {
    test('sources the namespace from the Type URI', () {
      expect(extendedCode(grant, 'roleNotRecognized'),
          'acl/grant:roleNotRecognized');
    });

    test('works for a single-segment slug', () {
      expect(
        extendedCode(discovery, 'filterUnsupported'),
        'trust-task-discovery:filterUnsupported',
      );
    });

    test('strips the #response fragment before deriving the slug', () {
      // An error raised while handling a response still belongs to the bare
      // slug; namespacing it acl/grant#response would name nothing.
      expect(
        extendedCode(grantResponse, 'roleNotRecognized'),
        'acl/grant:roleNotRecognized',
      );
    });

    test('accepts both casings of the local part', () {
      expect(
          extendedCode(grant, 'documentRevoked'), 'acl/grant:documentRevoked');
      expect(extendedCode(grant, 'document_revoked'),
          'acl/grant:document_revoked');
    });

    test('rejects a leading capital in the local part', () {
      // Only the first character must be lowercase; the resulting code would
      // otherwise fail to round-trip.
      expect(() => extendedCode(grant, 'BadLocal'), throwsArgumentError);
    });
  });

  group('§8.5 rule 2 — family namespaces', () {
    test('accepts each path prefix of the slug', () {
      expect(
        familyCode(didDelete, 'did-management', 'unknown_domain'),
        'did-management:unknown_domain',
      );
      expect(
        familyCode(didDelete, 'did-management/did', 'unknown_domain'),
        'did-management/did:unknown_domain',
      );
    });

    test('accepts the full slug, making it a superset of extendedCode', () {
      expect(
        familyCode(didDelete, 'did-management/did/delete', 'notOwner'),
        'did-management/did/delete:notOwner',
      );
    });

    test('rejects namespaces that are not prefixes', () {
      // A sibling shares a prefix but is not itself one — exactly the confusion
      // §8.5 forbids. `ac` is a string prefix of `acl/grant` but names nothing.
      for (final namespace in <String>['acl/revoke', 'vault', 'ac']) {
        expect(
          () => familyCode(grant, namespace, 'somethingElse'),
          throwsArgumentError,
          reason: namespace,
        );
      }
    });

    test('strips the #response fragment before checking the prefix', () {
      expect(
        familyCode(grantResponse, 'acl', 'permissionDenied'),
        'acl:permissionDenied',
      );
    });
  });

  group('slugFromTypeUri', () {
    test('drops the version segment and any fragment', () {
      expect(slugFromTypeUri(grant), 'acl/grant');
      expect(slugFromTypeUri(grantResponse), 'acl/grant');
      expect(slugFromTypeUri(didDelete), 'did-management/did/delete');
    });

    test('refuses a URI outside the Trust Tasks namespace', () {
      expect(
        () => slugFromTypeUri('https://example.com/spec/acl/grant/0.1'),
        throwsArgumentError,
      );
    });
  });
}
