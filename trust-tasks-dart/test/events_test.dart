// The observability hook's guard (#1116), the core `newUrnUuid` and the JSON
// duplicate record (#1118).
import 'dart:async';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

void main() {
  group('emitServerEvent', () {
    const event = InternalFailure(error: 'x', stackTrace: StackTrace.empty);

    test('calls the hook with the event', () {
      final seen = <ServerEvent>[];
      emitServerEvent(seen.add, event);
      expect(seen, [same(event)]);
    });

    test('does nothing without a hook', () {
      expect(() => emitServerEvent(null, event), returnsNormally);
    });

    test('swallows a hook that throws synchronously', () {
      expect(
        () => emitServerEvent((_) => throw StateError('broken'), event),
        returnsNormally,
      );
    });

    test('swallows a hook whose future fails, without an uncaught error',
        () async {
      final uncaught = <Object>[];
      await runZonedGuarded(() async {
        emitServerEvent((_) async => throw StateError('later'), event);
        emitServerEvent(
            (_) => Future<void>.error(StateError('rejected')), event);
        await Future<void>.delayed(Duration.zero);
      }, (error, _) => uncaught.add(error));
      expect(uncaught, isEmpty);
    });

    test('does not wait for an asynchronous hook', () {
      final never = Completer<void>();
      // Returns at once, although the hook's future never completes.
      emitServerEvent((_) => never.future, event);
    });
  });

  group('newUrnUuid', () {
    test('is a version-4 UUID URN, fresh each call', () {
      final shape = RegExp(
        r'^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-'
        r'[0-9a-f]{12}$',
      );
      final ids = {for (var i = 0; i < 256; i++) newUrnUuid()};
      expect(ids, hasLength(256));
      for (final id in ids) {
        expect(id, matches(shape));
      }
    });
  });

  group('DuplicateOutcome.isError', () {
    test('tells a recorded refusal from a recorded success', () {
      expect(const DuplicateOutcome<Object?>().isError, isFalse);
      expect(
        const DuplicateOutcome<Object?>(priorResponse: {
          'type': 'https://trusttasks.org/spec/acl/grant/0.3#response',
          'payload': {'code': 'looks-like-an-error'},
        }).isError,
        isFalse,
      );
      expect(
        const DuplicateOutcome<Object?>(
          priorResponse: {'type': trustTaskErrorTypeUri},
        ).isError,
        isTrue,
      );
    });
  });
}
