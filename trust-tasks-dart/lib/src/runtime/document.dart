/// The Trust Task document envelope (SPEC.md §4) and the framework-level checks
/// that operate on it.
///
/// Hand-written. Mirrors `document.rs` in trust-tasks-rs, `_runtime/document.ts`
/// in @openvtc/trust-tasks and `trusttasks/document.go` in trust-tasks-go.
library;

import 'codes.dart';

/// A W3C Data Integrity Proof object (SPEC.md §4.7). Opaque to the framework,
/// which never inspects it beyond noticing that it is present.
class Proof {
  const Proof({
    required this.type,
    required this.cryptosuite,
    required this.verificationMethod,
    required this.created,
    required this.proofPurpose,
    required this.proofValue,
    this.extra = const <String, Object?>{},
  });

  factory Proof.fromJson(Map<String, dynamic> json) {
    const known = <String>{
      'type',
      'cryptosuite',
      'verificationMethod',
      'created',
      'proofPurpose',
      'proofValue',
    };
    return Proof(
      type: json['type'] as String,
      cryptosuite: json['cryptosuite'] as String,
      verificationMethod: json['verificationMethod'] as String,
      created: json['created'] as String,
      proofPurpose: json['proofPurpose'] as String,
      proofValue: json['proofValue'] as String,
      extra: <String, Object?>{
        for (final e in json.entries)
          if (!known.contains(e.key)) e.key: e.value,
      },
    );
  }

  final String type;
  final String cryptosuite;
  final String verificationMethod;
  final String created;
  final String proofPurpose;
  final String proofValue;

  /// Any further members the cryptosuite defines, preserved on round-trip so a
  /// proof this framework does not understand still verifies against the bytes
  /// it arrived as.
  final Map<String, Object?> extra;

  Map<String, dynamic> toJson() => <String, dynamic>{
        ...extra,
        'type': type,
        'cryptosuite': cryptosuite,
        'verificationMethod': verificationMethod,
        'created': created,
        'proofPurpose': proofPurpose,
        'proofValue': proofValue,
      };
}

/// A reference to a predecessor step (SPEC.md §4.11).
class CeremonyPrev {
  const CeremonyPrev({required this.id, required this.digestMultibase});

  factory CeremonyPrev.fromJson(Map<String, dynamic> json) => CeremonyPrev(
        id: json['id'] as String,
        digestMultibase: json['digestMultibase'] as String,
      );

  /// The predecessor document's `id` (§4.3), so a verifier can locate what the
  /// digest is over.
  final String id;

  /// A multibase-encoded multihash over the predecessor document, salted per
  /// enactment — salted because many steps carry near-zero-entropy payloads, and
  /// an unsalted digest over one is a confirmation oracle for any party handed
  /// it, which a chain does by design.
  final String digestMultibase;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'digestMultibase': digestMultibase,
      };
}

/// Records that a document is a step of a Trust Ceremony — a flow composed of
/// several Trust Tasks (SPEC.md §4.11).
///
/// Optional in every sense: no specification declares anything about ceremonies,
/// a document without it is fully conforming, and a consumer that does not
/// implement ceremonies processes the document unchanged. Ignoring it is always
/// safe, because §4.11.4 forbids deriving authority from it — there is nothing a
/// ceremony-aware consumer may do that an unaware one omits.
class Ceremony {
  const Ceremony({
    required this.enactment,
    required this.step,
    this.definition,
    this.definitionDigest,
    this.parentEnactment,
    this.round,
    this.terminal,
    this.prev,
  });

  factory Ceremony.fromJson(Map<String, dynamic> json) => Ceremony(
        enactment: json['enactment'] as String,
        step: json['step'] as String,
        definition: json['definition'] as String?,
        definitionDigest: json['definitionDigest'] as String?,
        parentEnactment: json['parentEnactment'] as String?,
        round: json['round'] as int?,
        terminal: json['terminal'] as bool?,
        prev: (json['prev'] as List<dynamic>?)
            ?.map((e) => CeremonyPrev.fromJson(e as Map<String, dynamic>))
            .toList(),
      );

  /// Identifies one run of a ceremony. Globally unique and never reused, on the
  /// same terms as the document `id` and unlike `threadId` — evidence about a
  /// flow needs a stable anchor.
  final String enactment;

  /// Names this step within the ceremony. The step name, not the Type URI, is
  /// the step's identity: one Type URI may serve several steps whose meaning
  /// differs by context.
  final String step;

  /// The ceremony definition this step is enacted under (§6.7). Requires
  /// [definitionDigest].
  final String? definition;

  /// A multibase-multihash over the JCS canonicalization of the definition. It
  /// pins the definition by content rather than by name: a URI alone would leave
  /// the flow's rules mutable by whoever controls it, retroactively and for
  /// every enactment already performed.
  final String? definitionDigest;

  /// The enactment containing this one. One level, navigation only — but the
  /// pointers chain, so nesting is unbounded.
  final String? parentEnactment;

  /// Distinguishes repetitions of the same step by the same party. Absent means 1.
  final int? round;

  /// Marks a step that ends the enactment; a set with none so marked is a
  /// prefix, not a completed flow.
  final bool? terminal;

  /// The steps this one follows. A set, so concurrent branches are expressible.
  final List<CeremonyPrev>? prev;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'enactment': enactment,
        'step': step,
        if (definition != null) 'definition': definition,
        if (definitionDigest != null) 'definitionDigest': definitionDigest,
        if (parentEnactment != null) 'parentEnactment': parentEnactment,
        if (round != null) 'round': round,
        if (terminal != null) 'terminal': terminal,
        if (prev != null) 'prev': prev!.map((e) => e.toJson()).toList(),
      };
}

/// A single Trust Task document, per SPEC.md §4.2.
///
/// Optional members are nullable because every framework check in §7.2 turns on
/// whether a member is *present*: a recipient of `''` and an absent recipient
/// lead to different verdicts, and collapsing them would make this runtime
/// disagree with the Rust, TypeScript and Go ones.
///
/// Timestamps are strings, not [DateTime], for the same parity reason. A
/// malformed `expiresAt` must be reportable as `malformedRequest` by
/// [validateBasic]; parsing it at decode time would instead throw, and the
/// consumer would have no document to build an error response about.
class TrustTaskDocument<P> {
  const TrustTaskDocument({
    required this.id,
    required this.type,
    required this.payload,
    this.threadId,
    this.parentThreadId,
    this.ceremony,
    this.issuer,
    this.recipient,
    this.issuedAt,
    this.expiresAt,
    this.context,
    this.proof,
    this.extra = const <String, Object?>{},
  });

  /// Decode a document whose payload is read by [payloadFromJson].
  ///
  /// Dart cannot recover `P` at runtime, so the payload's own decoder is passed
  /// in — the standard shape for a generic JSON model here. A generated library
  /// supplies `Payload.fromJson`.
  factory TrustTaskDocument.fromJson(
    Map<String, dynamic> json,
    P Function(Object? payload) payloadFromJson,
  ) {
    const known = <String>{
      'id',
      'threadId',
      'parentThreadId',
      'ceremony',
      'type',
      'issuer',
      'recipient',
      'issuedAt',
      'expiresAt',
      'payload',
      '@context',
      'proof',
    };
    return TrustTaskDocument<P>(
      id: json['id'] as String,
      type: json['type'] as String,
      payload: payloadFromJson(json['payload']),
      threadId: json['threadId'] as String?,
      parentThreadId: json['parentThreadId'] as String?,
      ceremony: json['ceremony'] == null
          ? null
          : Ceremony.fromJson(json['ceremony'] as Map<String, dynamic>),
      issuer: json['issuer'] as String?,
      recipient: json['recipient'] as String?,
      issuedAt: json['issuedAt'] as String?,
      expiresAt: json['expiresAt'] as String?,
      context: json['@context'],
      proof: json['proof'] == null
          ? null
          : Proof.fromJson(json['proof'] as Map<String, dynamic>),
      extra: <String, Object?>{
        for (final e in json.entries)
          if (!known.contains(e.key)) e.key: e.value,
      },
    );
  }

  /// Globally unique to this instance (§4.3).
  final String id;

  /// Correlates this document with others in the same exchange (§4.9).
  final String? threadId;

  /// The `threadId` of the exchange containing this one, where this exchange is
  /// conducted inside another (§4.9.2).
  ///
  /// A navigation aid. It records one level of containment and does **not**
  /// change which exchange attests an event — §4.9.1 governs that, and holds
  /// whether or not this member is present. Like [threadId] it carries no
  /// normative validation semantics: a consumer MUST NOT reject a document on
  /// the basis of `parentThreadId` alone.
  final String? parentThreadId;

  /// Records that this document is a step of a Trust Ceremony (§4.11).
  ///
  /// Carries no authority: membership is an assertion by the issuer, not a
  /// verified fact, so every authorization decision still rests on [issuer],
  /// [proof] and local policy (§4.11.4).
  final Ceremony? ceremony;

  /// The Type URI identifying specification and version (§4.4).
  final String type;

  /// VID of the party responsible for the content (§4.8).
  final String? issuer;

  /// VID of the party expected to act on the document (§4.8).
  final String? recipient;

  /// When the document was produced (§4.2), as RFC 3339.
  final String? issuedAt;

  /// The instant after which the document is no longer valid (§4.2), RFC 3339.
  final String? expiresAt;

  /// The task-specific body, whose structure is defined by the specification
  /// [type] names.
  final P payload;

  /// The optional JSON-LD context (§4.6). Held untyped because §4.6 permits a
  /// string, an array or an object and the framework never interprets it.
  final Object? context;

  /// The optional Data Integrity proof binding the document to its issuer (§4.7).
  final Proof? proof;

  /// Top-level members this class does not name. §7.2 says a consumer SHOULD
  /// preserve but MUST NOT act upon them, and §7.1 asks a forwarding producer to
  /// carry them through — so they survive a round trip here rather than being
  /// dropped by the decoder.
  final Map<String, Object?> extra;

  /// Serialize, folding [extra] back in.
  ///
  /// A member of [extra] that collides with one this class names is dropped: the
  /// named member is authoritative, and emitting both would produce a document
  /// with a duplicate key.
  Map<String, dynamic> toJson(Object? Function(P payload) payloadToJson) =>
      <String, dynamic>{
        ...extra,
        'id': id,
        if (threadId != null) 'threadId': threadId,
        if (parentThreadId != null) 'parentThreadId': parentThreadId,
        if (ceremony != null) 'ceremony': ceremony!.toJson(),
        'type': type,
        if (issuer != null) 'issuer': issuer,
        if (recipient != null) 'recipient': recipient,
        if (issuedAt != null) 'issuedAt': issuedAt,
        if (expiresAt != null) 'expiresAt': expiresAt,
        'payload': payloadToJson(payload),
        if (context != null) '@context': context,
        if (proof != null) 'proof': proof!.toJson(),
      };

  /// A copy with the named members replaced. Used by the response builders.
  TrustTaskDocument<P> copyWith({
    String? id,
    String? type,
    String? threadId,
    String? parentThreadId,
    String? issuer,
    String? recipient,
    String? issuedAt,
  }) =>
      TrustTaskDocument<P>(
        id: id ?? this.id,
        type: type ?? this.type,
        payload: payload,
        threadId: threadId ?? this.threadId,
        parentThreadId: parentThreadId ?? this.parentThreadId,
        ceremony: ceremony,
        issuer: issuer ?? this.issuer,
        recipient: recipient ?? this.recipient,
        issuedAt: issuedAt ?? this.issuedAt,
        expiresAt: expiresAt,
        context: context,
        proof: proof,
        extra: extra,
      );
}

/// Names the Trust Task document an [ErrorPayload] reports on (§8.2).
class InResponseTo {
  const InResponseTo({required this.typeUri, this.id});

  factory InResponseTo.fromJson(Map<String, dynamic> json) => InResponseTo(
        typeUri: json['typeUri'] as String,
        id: json['id'] as String?,
      );

  /// The reported-on document's `type`, including any `#request` / `#response`
  /// fragment — that fragment is what tells a consumer which variant's semantics
  /// apply.
  final String typeUri;

  /// The reported-on document's `id`. Globally unique and never reused (§4.3),
  /// so it names one instance where `threadId` names an exchange.
  ///
  /// Omitted under `identityMismatch`: per §8.1 the response goes to the
  /// transport-authenticated sender rather than the in-band issuer, and that
  /// party did not necessarily compose the document.
  final String? id;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'typeUri': typeUri,
        if (id != null) 'id': id,
      };
}

/// The `payload` of an error response (SPEC.md §8.2).
class ErrorPayload {
  const ErrorPayload({
    required this.code,
    required this.retryable,
    this.inResponseTo,
    this.message,
    this.retryAfter,
    this.details,
  });

  factory ErrorPayload.fromJson(Map<String, dynamic> json) => ErrorPayload(
        code: json['code'] as String,
        retryable: json['retryable'] as bool,
        inResponseTo: json['inResponseTo'] == null
            ? null
            : InResponseTo.fromJson(
                json['inResponseTo'] as Map<String, dynamic>),
        message: json['message'] as String?,
        retryAfter: json['retryAfter'] as String?,
        details: json['details'] as Map<String, dynamic>?,
      );

  final String code;

  /// Identifies the document this error reports on (§8.2).
  ///
  /// `threadId` correlates the exchange for a party that saw the request and
  /// identifies nothing to anyone else, so without this a retained error names
  /// neither the specification the failure occurred under nor the instance. The
  /// builders below populate it.
  final InResponseTo? inResponseTo;

  final String? message;

  /// Not nullable: §8.2 makes it REQUIRED, and omitting it would leave a
  /// producer unable to tell a final refusal from an invitation to re-send.
  final bool retryable;

  final String? retryAfter;
  final Map<String, Object?>? details;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'code': code,
        if (inResponseTo != null) 'inResponseTo': inResponseTo!.toJson(),
        if (message != null) 'message': message,
        'retryable': retryable,
        if (retryAfter != null) 'retryAfter': retryAfter,
        if (details != null) 'details': details,
      };
}

/// A `trust-task-error` document.
typedef ErrorResponse = TrustTaskDocument<ErrorPayload>;

/// The per-specification declarations a consumer needs to apply SPEC §7.2 items
/// 5b, 7 and 8. Generated libraries export this as `spec` / `responseSpec`.
class SpecPolicy {
  const SpecPolicy({
    required this.typeUri,
    this.isBearer = false,
    this.isProofRequired = false,
    this.isRecipientRequired = false,
    this.isIssuedAtRequired = false,
    this.payloadSchema,
  });

  final String typeUri;

  /// §4.8.3 — the specification opts out of the §4.8.2 audience-binding rule.
  final bool isBearer;

  /// §7.3 item 8 — `proofRequirement` is REQUIRED.
  final bool isProofRequired;

  /// §7.3 item 5 — the party filling `recipient` is REQUIRED.
  final bool isRecipientRequired;

  /// §7.3 item 17 — `issuedAtRequirement` is REQUIRED, raising §4.2's `issuedAt`
  /// SHOULD to a MUST for this specification's documents. A specification
  /// defining a *consequential Trust Task* (§2) must declare it, so that §7.2
  /// item 11's duplicate-execution record always has a window to sit in.
  ///
  /// This is the *specification's* requirement, published in the registry —
  /// distinct from a consumer's own freshness posture, which is chosen at the
  /// call site and applied to every document.
  final bool isIssuedAtRequired;

  /// This variant's payload schema as JSON text (§7.2 item 2), carried so a
  /// consumer has something to validate against. Null only for a hand-written
  /// policy that omits it.
  final String? payloadSchema;
}

/// Why a consumer rejected a document, and the §8.3 code it maps to.
class RejectReason {
  const RejectReason({
    required this.code,
    required this.message,
    this.retryable = false,
    this.retryAfter,
    this.details,
  });

  final StandardCode code;
  final String message;
  final bool retryable;
  final String? retryAfter;
  final Map<String, Object?>? details;
}

/// The Type URI a consumer emits error responses under.
///
/// The single source of truth for the emitted `trust-task-error` version on this
/// side. Build error responses with [rejectWith] / [rejectWithRecipient], or
/// read the URI from here; do not spell it out again.
///
/// The counterparts are `trust_task_error_type_uri()` in trust-tasks-rs,
/// `TRUST_TASK_ERROR_TYPE_URI` in @openvtc/trust-tasks and
/// `TrustTaskErrorTypeURI` in trust-tasks-go, all kept equal to this value.
/// `scripts/check-bindings-conformance.mjs` enforces it.
///
/// `0.5` because this runtime populates the `inResponseTo` member of §8.2 and
/// can emit `idConflict` (§8.3), which is absent from `0.3`'s code enum and does
/// not match its extended-code pattern — a document carrying it would not
/// validate as `0.3`. Per §5.2 forward-minor compatibility a `0.3` consumer
/// SHOULD accept it.
const String trustTaskErrorTypeUri =
    'https://trusttasks.org/spec/trust-task-error/0.5';

/// Returns the timestamp a built response is stamped with. Injectable so tests
/// are deterministic.
typedef Clock = String Function();

/// Stamps responses with the current time in RFC 3339, to the second.
String systemClock() =>
    '${DateTime.now().toUtc().toIso8601String().split('.').first}Z';

/// SPEC §7.2 items 4 and 5a — expiry and wrong-recipient.
///
/// ⚠ This is *not* the full §7.2 check. Items 1–3 (framework schema, payload
/// schema, unknown `type`) belong to the caller's parse and dispatch; items 5b,
/// 7 and 8 need the specification's policy and live in [enforceSpecPolicy].
/// [consumeInbound] bundles 4–8.
///
/// Returns null when the document passes.
RejectReason? validateBasic<P>(
  TrustTaskDocument<P> doc,
  DateTime now,
  String myVid,
) {
  final expiresAtRaw = doc.expiresAt;
  if (expiresAtRaw != null) {
    final expiresAt = DateTime.tryParse(expiresAtRaw);
    if (expiresAt == null) {
      return const RejectReason(
        code: StandardCode.malformedRequest,
        message: 'expiresAt is not a valid RFC 3339 timestamp',
      );
    }
    // §4.2 / §7.2 item 4: inclusive bound — `now >= expiresAt` is expired.
    if (!expiresAt.isAfter(now)) {
      return RejectReason(
        code: StandardCode.expired,
        message: 'document expired at $expiresAtRaw',
      );
    }
  }

  final recipient = doc.recipient;
  if (recipient != null && recipient != myVid) {
    return const RejectReason(
      code: StandardCode.wrongRecipient,
      message: 'in-band recipient does not identify this consumer',
    );
  }

  return null;
}

/// SPEC §7.2 item 8 — a proof-bearing document on a non-bearer specification
/// must carry an in-band `recipient`, so the proof binds the audience as well as
/// the content (§4.8.2).
RejectReason? enforceAudienceBinding<P>(
  TrustTaskDocument<P> doc,
  SpecPolicy spec,
) {
  if (doc.proof != null && doc.recipient == null && !spec.isBearer) {
    return const RejectReason(
      code: StandardCode.malformedRequest,
      message: 'proof present with no in-band recipient on a non-bearer '
          'specification (SPEC §4.8.2 audience binding)',
    );
  }
  return null;
}

/// The policy-driven subset of SPEC §7.2 — items 5b, 7 clause A, and 8, plus
/// §7.3 item 17's `issuedAt` requirement.
///
/// Single source of truth for the flag-driven checks, so a binding-specific
/// pipeline and [consumeInbound] cannot diverge on the check set. Ordering
/// matches `enforce_spec_policy` in trust-tasks-rs: recipient, then proof, then
/// issuedAt (§7.3 item 17), then audience binding.
RejectReason? enforceSpecPolicy<P>(
  TrustTaskDocument<P> doc,
  SpecPolicy spec,
) {
  if (doc.recipient == null && spec.isRecipientRequired) {
    return const RejectReason(
      code: StandardCode.malformedRequest,
      message: 'specification declares recipient REQUIRED but the document '
          'carries no in-band recipient',
    );
  }
  if (doc.proof == null && spec.isProofRequired) {
    return const RejectReason(
      code: StandardCode.proofRequired,
      message:
          'specification declares proof REQUIRED but the document carries none',
    );
  }
  if (doc.issuedAt == null && spec.isIssuedAtRequired) {
    return const RejectReason(
      code: StandardCode.malformedRequest,
      message: 'issuedAt is required by this Trust Task specification '
          '(SPEC §7.3 item 17)',
    );
  }
  return enforceAudienceBinding(doc, spec);
}

/// Build the error response for [request], addressed to an explicit [recipient].
///
/// ⚠ Prefer [rejectWith] for ordinary refusals. This form exists for
/// `identityMismatch`, where §8.1 forbids addressing the contested in-band
/// issuer — see [reject], which applies that rule.
ErrorResponse rejectWithRecipient<P>(
  TrustTaskDocument<P> request,
  String id,
  ErrorPayload payload,
  String? recipient, {
  Clock? clock,
}) {
  // §8.2 — name the document this error reports on, so it means something to a
  // party that did not see the request. Filled here rather than left to the
  // caller because the builder is the only place that reliably has the
  // originating document in hand; a caller-supplied value is kept.
  var withOrigin = payload;
  if (payload.inResponseTo == null) {
    // §8.1/§8.2 — under identityMismatch the response is addressed to the
    // transport-authenticated sender, not the in-band issuer. That party did not
    // necessarily compose the document, so its id is not echoed back.
    final isMismatch =
        normalizeCode(payload.code) == StandardCode.identityMismatch.value;
    withOrigin = ErrorPayload(
      code: payload.code,
      retryable: payload.retryable,
      inResponseTo: InResponseTo(
        typeUri: request.type,
        id: isMismatch ? null : request.id,
      ),
      message: payload.message,
      retryAfter: payload.retryAfter,
      details: payload.details,
    );
  }

  return ErrorResponse(
    id: id,
    // §4.9: continue the thread, falling back to the request's own id.
    threadId: request.threadId ?? request.id,
    type: trustTaskErrorTypeUri,
    issuer: request.recipient,
    recipient: recipient,
    issuedAt: (clock ?? systemClock)(),
    payload: withOrigin,
    // §4.9.2 — the whole exchange shares one parent, so the error response stays
    // inside the same enclosing exchange.
    parentThreadId: request.parentThreadId,
  );
}

/// Build the error response for [request], addressed to its original producer.
///
/// ⚠ Not safe under a rejection that contests the in-band identity: it copies
/// `request.issuer` into `recipient`, which under `identityMismatch` is the
/// contested value §8.1 says MUST NOT be addressed. Use [reject] for those.
ErrorResponse rejectWith<P>(
  TrustTaskDocument<P> request,
  String id,
  ErrorPayload payload, {
  Clock? clock,
}) =>
    rejectWithRecipient(request, id, payload, request.issuer, clock: clock);

/// Turn a [RejectReason] into the §8.2 payload it maps to.
ErrorPayload toErrorPayload(RejectReason reason) => ErrorPayload(
      code: reason.code.value,
      message: reason.message,
      retryable: reason.retryable,
      retryAfter: reason.retryAfter,
      details: reason.details,
    );

/// Build the success-response document for [request], per SPEC §4.4.1 — the
/// request's Type URI with the `#response` fragment, the parties swapped, and
/// the thread continued.
TrustTaskDocument<R> respondWith<P, R>(
  TrustTaskDocument<P> request,
  String id,
  R payload, {
  Clock? clock,
}) =>
    TrustTaskDocument<R>(
      id: id,
      threadId: request.threadId ?? request.id,
      type: '${bareTypeUri(request.type)}#response',
      issuer: request.recipient,
      recipient: request.issuer,
      issuedAt: (clock ?? systemClock)(),
      payload: payload,
      // §4.9.2 — the whole exchange shares one parent.
      parentThreadId: request.parentThreadId,
    );

/// A Type URI with any fragment removed.
String bareTypeUri(String typeUri) {
  final hash = typeUri.indexOf('#');
  return hash < 0 ? typeUri : typeUri.substring(0, hash);
}
