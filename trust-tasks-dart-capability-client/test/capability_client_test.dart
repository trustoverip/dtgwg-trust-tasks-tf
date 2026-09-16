import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_capability_client/trust_tasks_capability_client.dart';

const errorType = 'https://trusttasks.org/spec/trust-task-error/0.5';

CapabilityDocument reply(
  String type,
  String threadId,
  Map<String, dynamic> payload,
) =>
    TrustTaskDocument<Map<String, dynamic>>(
      id: 'urn:uuid:reply',
      type: type,
      threadId: threadId,
      payload: payload,
    );

/// A JSON envelope body (what an inbound dispatch holds).
Map<String, dynamic> body(CapabilityDocument doc) => doc.toJson((p) => p);

void main() {
  test('builders are addressed and typed', () {
    final list = buildListDocument('did:example:me', 'did:example:vtc');
    expect(slugOf(list.type), 'governance/capability/list');
    expect(list.issuer, 'did:example:me');
    expect(list.recipient, 'did:example:vtc');
    expect(list.payload['status'], 'all');

    final enable = buildToggleDocument(
        'did:example:me', 'did:example:vtc', 'git-trust', '0.1',
        enable: true);
    expect(slugOf(enable.type), 'governance/capability/enable');
    expect(enable.payload['config'], {'authority': 'did:example:vtc'});

    final disable = buildToggleDocument(
        'did:example:me', 'did:example:vtc', 'git-trust', '0.1',
        enable: false);
    expect(slugOf(disable.type), 'governance/capability/disable');
    expect(disable.payload, {'capability': 'git-trust'});

    final grant = buildGitTrustGrant('did:a', 'did:r', 'did:s', 'openvtc');
    expect(slugOf(grant.type), 'git-trust/grant');
    expect(grant.payload['subject'], 'did:s');

    final revoke = buildGitTrustRevoke('did:a', 'did:r', 'did:s', 'openvtc',
        reason: 'ended');
    expect(revoke.payload['reason'], 'ended');
    final revokeNoReason =
        buildGitTrustRevoke('did:a', 'did:r', 'did:s', 'openvtc');
    expect(revokeNoReason.payload.containsKey('reason'), isFalse);
  });

  test('build mints a fresh id and issuedAt each call', () {
    final a = buildListDocument('did:me', 'did:vtc');
    final b = buildListDocument('did:me', 'did:vtc');
    expect(a.id, isNot(b.id));
    expect(a.id, startsWith('urn:uuid:'));
    expect(a.issuedAt, isNotNull);
  });

  test('newAttempt: fresh id and issuedAt, no proof, same content', () {
    final previous = buildGitTrustGrant('did:a', 'did:r', 'did:s', 'openvtc')
        .copyWith(threadId: 'urn:thread:1');
    final next = newAttempt(previous);
    expect(next.id, isNot(previous.id));
    expect(next.proof, isNull);
    expect(next.payload, previous.payload);
    expect(next.type, previous.type);
    expect(next.threadId, 'urn:thread:1'); // an explicit threadId is preserved
  });

  test('typeUri slug and response variant', () {
    expect(slugOf(gitTrustGrantType), 'git-trust/grant');
    expect(isResponseVariant(gitTrustGrantType), isFalse);
    expect(isResponseVariant('$gitTrustGrantType#response'), isTrue);
    expect(isResponseVariant('$gitTrustGrantType#request'), isFalse);
    expect(slugOf(errorType), 'trust-task-error');
    expect(slugOf('not a uri'), isNull);
  });

  test('correlationThread and repliesTo', () {
    final noThread = TrustTaskDocument<Map<String, dynamic>>(
        id: 'urn:doc:1', type: gitTrustGrantType, payload: {});
    expect(correlationThread(noThread), 'urn:doc:1');
    expect(correlationThread(reply(gitTrustGrantType, 'urn:thread:1', {})),
        'urn:thread:1');
    expect(repliesTo(reply(gitTrustGrantType, 't', {}), 't'), isTrue);
    expect(repliesTo(reply(gitTrustGrantType, 't', {}), 'other'), isFalse);
  });

  test('parseEnvelopeDocument requires a threaded document', () {
    final doc = reply(gitTrustGrantType, 'urn:thread:1', {});
    final parsed = parseEnvelopeDocument(body(doc));
    expect(parsed?.$1, 'urn:thread:1');
    expect(parseEnvelopeDocument({'id': 'x', 'type': 'y', 'payload': {}}),
        isNull); // no threadId
    expect(parseEnvelopeDocument({'hello': 'world'}), isNull);
    expect(parseEnvelopeDocumentFor(body(doc), 'urn:thread:1'), isNotNull);
    expect(parseEnvelopeDocumentFor(body(doc), 'other'), isNull);
  });

  test('classifyGitTrustReply: success', () {
    final doc = reply('$gitTrustGrantType#response', 'urn:thread:1', {});
    expect(classifyGitTrustReply(doc, 'urn:thread:1'), isA<WriteSuccess>());
  });

  test('classifyGitTrustReply: idempotent success from the extended code', () {
    for (final code in [
      gitTrustAlreadyGrantedCode,
      gitTrustAlreadyGrantedCodeCamel,
      gitTrustNotGrantedCode,
    ]) {
      final doc = reply(errorType, 'urn:thread:1', {'code': code});
      expect(classifyGitTrustReply(doc, 'urn:thread:1'),
          isA<WriteIdempotentSuccess>(),
          reason: code);
    }
  });

  test('classifyGitTrustReply: any other error is rejected', () {
    final doc = reply(errorType, 'urn:thread:1',
        {'code': 'notAuthorized', 'message': 'nope'});
    final out = classifyGitTrustReply(doc, 'urn:thread:1');
    expect(out, isA<WriteRejected>());
    expect((out as WriteRejected).code, 'notAuthorized');
    expect(out.message, 'nope');
  });

  test('classifyGitTrustReply: an uncorrelated reply is not an answer', () {
    final doc = reply('$gitTrustGrantType#response', 'urn:thread:OTHER', {});
    expect(classifyGitTrustReply(doc, 'urn:thread:1'), isNull);
  });

  test('classifyGitTrustReply: free-text idempotence only under the policy',
      () {
    final doc = reply(errorType, 'urn:thread:1',
        {'code': 'taskFailed', 'message': 'already_granted: did:s on x'});
    expect(classifyGitTrustReply(doc, 'urn:thread:1'), isA<WriteRejected>());
    expect(
        classifyGitTrustReply(doc, 'urn:thread:1',
            policy: const ReplyPolicy(acceptLegacyFreeTextIdempotence: true)),
        isA<WriteIdempotentSuccess>());
  });

  test('parseCapabilityReply: a listing', () {
    final doc = reply('$capabilityListType#response', 'urn:thread:1', {
      'capabilities': [
        {
          'enabled': true,
          'enabledAt': '2026-01-01T00:00:00Z',
          'delegate': 'did:del',
          'manifest': {
            'capability': 'git-trust',
            'title': 'Git Trust',
            'version': '0.1'
          },
        },
        {
          'enabled': false,
          'manifest': {'capability': 'audit', 'version': '1.0'}
        },
        {'enabled': true}, // no manifest — dropped
      ],
    });
    final r = parseCapabilityReply(doc, 'urn:thread:1');
    expect(r, isA<CapabilityListing>());
    final entries = (r as CapabilityListing).entries;
    expect(entries.length, 2);
    expect(entries[0].slug, 'git-trust');
    expect(entries[0].title, 'Git Trust');
    expect(entries[0].enabled, isTrue);
    expect(entries[0].enabledAt, '2026-01-01T00:00:00Z');
  });

  test('parseCapabilityReply: a toggle acknowledgement and an error', () {
    final toggled = reply('$capabilityEnableType#response', 'urn:thread:1',
        {'capability': 'git-trust', 'enabled': true});
    final r = parseCapabilityReply(toggled, 'urn:thread:1');
    expect(r, isA<CapabilityToggled>());
    expect((r as CapabilityToggled).capability, 'git-trust');
    expect(r.enabled, isTrue);

    final err = reply(errorType, 'urn:thread:1', {'code': 'notAuthorized'});
    final e = parseCapabilityReply(err, 'urn:thread:1');
    expect(e, isA<CapabilityRejected>());
    expect((e as CapabilityRejected).code, 'notAuthorized');
    expect(e.message, isNull);
  });

  test('parseCapabilityReply: correlation and family gating', () {
    final doc = reply('$capabilityListType#response', 'urn:thread:1',
        {'capabilities': <Object?>[]});
    expect(parseCapabilityReply(doc, 'other'), isNull);
    final request = reply(capabilityListType, 'urn:thread:1', {});
    expect(parseCapabilityReply(request, 'urn:thread:1'), isNull);
  });

  test('parseEnvelopeReply: body to reply in one step', () {
    final listing = reply('$capabilityListType#response', 'urn:thread:9',
        {'capabilities': <Object?>[]});
    expect(parseEnvelopeReply(body(listing), 'urn:thread:9'),
        isA<CapabilityListing>());
    expect(parseEnvelopeReply(body(listing), 'wrong'), isNull);
    // A git-trust revoke #response is a write reply, not a capability reply.
    final write = reply('$gitTrustRevokeType#response', 'urn:thread:9', {});
    expect(parseEnvelopeReply(body(write), 'urn:thread:9'), isNull);
  });
}
