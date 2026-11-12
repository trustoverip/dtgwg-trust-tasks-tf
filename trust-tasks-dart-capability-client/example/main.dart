// Build capability requests, hold their threads, and classify the replies.
//
// Run it:
//   dart run example/main.dart
//
// This package owns the documents, not a transport, and does no signing. Here
// the replies a community service would send are built in place so the example
// runs on its own; in an application they arrive over your transport (DIDComm,
// TSP, HTTPS) and the request is signed with package:trust_tasks_proof first.

import 'dart:convert';

import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_capability_client/trust_tasks_capability_client.dart';

const authority = 'did:web:authority.example';
const registry = 'did:web:registry.example';
const subject = 'did:key:z6MkExampleSubject';
const community = 'did:web:community.example';

void main() {
  // --- a git-trust write ---------------------------------------------------
  final grant = buildGitTrustGrant(authority, registry, subject, 'openvtc');
  // Hold the thread from the moment you send: only a reply on it is an answer.
  final thread = correlationThread(grant);
  print('send: ${jsonEncode(grant.toJson((p) => p))}');

  // The registry says the subject is already trusted. That is the end state
  // the write wanted, so it is a success, decided from the extended error code
  // (SPEC §8.5), never from the free-text message.
  final alreadyGranted = TrustTaskDocument<Map<String, dynamic>>(
    id: 'urn:uuid:5f0c2a4e-3b1d-4c8e-9a7f-1e2d3c4b5a69',
    type: trustTaskErrorTypeUri,
    threadId: thread,
    issuer: registry,
    recipient: authority,
    issuedAt: systemClock(),
    payload: {
      'code': gitTrustAlreadyGrantedCodeCamel,
      'retryable': false,
    },
  );
  switch (classifyGitTrustReply(alreadyGranted, thread)) {
    case WriteSuccess():
    case WriteIdempotentSuccess():
      print('grant: done');
    case WriteRejected(:final code, :final message):
      print('grant: rejected $code ${message ?? ''}');
    case null:
      print('grant: not an answer to this request; keep waiting');
  }

  // Sending it again after a timeout? A bit-for-bit resend of `grant` is a
  // retry the consumer absorbs. Anything else is a new attempt, with a fresh id.
  final again = newAttempt(grant);
  print('new attempt: ${again.id} (was ${grant.id})');

  // --- a governance listing ------------------------------------------------
  final list = buildListDocument(authority, community);
  final listThread = correlationThread(list);
  final inbound = TrustTaskDocument<Map<String, dynamic>>(
    id: 'urn:uuid:0b7e6d5c-4a3f-4e21-8d9c-7b6a5f4e3d2c',
    type: '$capabilityListType#response',
    threadId: listThread,
    issuer: community,
    recipient: authority,
    issuedAt: systemClock(),
    payload: {
      'capabilities': [
        {
          'enabled': true,
          'manifest': {
            'capability': 'git-trust',
            'title': 'Git Trust',
            'version': '0.1',
          },
        },
      ],
    },
  ).toJson((p) => p); // what a transport hands you: a decoded JSON body

  switch (parseEnvelopeReply(inbound, listThread)) {
    case CapabilityListing(:final entries):
      for (final e in entries) {
        print('capability ${e.slug}/${e.version}: '
            '${e.enabled ? 'enabled' : 'disabled'}');
      }
    case final other:
      print('list: $other');
  }
}
