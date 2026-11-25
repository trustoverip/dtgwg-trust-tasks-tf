// Payload validation with a full JSON Schema 2020-12 engine (package:json_schema),
// as examples/getting-started-dart/lib/structural_validator.dart recommends for
// a real application. The HCP schemas lean on `pattern` (urn:uuid identifiers),
// `format` and `uniqueItems`, which that example's structural stand-in ignores.
//
// Each generated library embeds its payload schema with cross-file `$ref`s
// already inlined, so no resolver is needed.

import 'package:json_schema/json_schema.dart';
import 'package:trust_tasks/specs/trust_task_discovery/v0_4/payload.dart'
    as discovery;
import 'package:trust_tasks/specs/trust_task_error/v0_6/payload.dart' as error;
import 'package:trust_tasks/specs/trust_task_next_step/v0_1/payload.dart'
    as next_step;
import 'package:trust_tasks/trust_tasks.dart';

import 'trust_tasks/hcp/entries/query/v0_1/payload.dart' as query;
import 'trust_tasks/hcp/grants/request/v0_1/payload.dart' as grant_request;
import 'trust_tasks/hcp/grants/revoke/v0_1/payload.dart' as revoke;
import 'trust_tasks/hcp/grants/revoked/v0_1/payload.dart' as revoked;

final Map<String, JsonSchema> _compiled = {};

List<String> _check(String schemaJson, Object? payload) {
  final schema = _compiled.putIfAbsent(
    schemaJson,
    () => JsonSchema.create(
      schemaJson,
      schemaVersion: SchemaVersion.draft2020_12,
    ),
  );
  // 2020-12 makes `format` an annotation by default; assert it, as ajv-formats
  // does in the TypeScript demo.
  final result = schema.validate(payload, validateFormats: true);
  return [
    for (final e in result.errors) 'payload${e.instancePath} ${e.message}',
  ];
}

/// The validator the receiving end's SPEC §7.2 pipeline runs (item 2).
final class SchemaValidator implements PayloadValidator {
  const SchemaValidator();

  @override
  String? validate(String schemaJson, Object? payload) {
    final errors = _check(schemaJson, payload);
    return errors.isEmpty ? null : errors.join('; ');
  }
}

/// The schema a reply's payload must satisfy, by its `type`.
const Map<String, String> _replySchemas = {
  discovery.responseTypeUri: discovery.responsePayloadSchemaJson,
  next_step.typeUri: next_step.payloadSchemaJson,
  error.typeUri: error.payloadSchemaJson,
  grant_request.responseTypeUri: grant_request.responsePayloadSchemaJson,
  query.responseTypeUri: query.responsePayloadSchemaJson,
  revoke.responseTypeUri: revoke.responsePayloadSchemaJson,
  revoked.responseTypeUri: revoked.responsePayloadSchemaJson,
};

/// A producer checks the reply it receives before acting on it, as a consumer
/// checks a request.
void validateReply(Map<String, dynamic> reply) {
  final type = reply['type'];
  final schema = _replySchemas[type];
  if (schema == null) throw StateError('unexpected reply type $type');
  final errors = _check(schema, reply['payload']);
  if (errors.isNotEmpty) {
    throw StateError('reply $type fails its schema: ${errors.join('; ')}');
  }
}
