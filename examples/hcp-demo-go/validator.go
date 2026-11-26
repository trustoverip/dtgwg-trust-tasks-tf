package main

// Payload validation with a full JSON Schema 2020-12 engine
// (github.com/santhosh-tekuri/jsonschema/v6), as
// examples/getting-started-go/validator.go recommends for a real application.
// The HCP schemas lean on `pattern` (urn:uuid identifiers), `format` and
// `uniqueItems`, which that example's structural stand-in ignores.
//
// Each generated package embeds its payload schema with cross-file `$ref`s
// already inlined, so no resolver is needed.

import (
	"bytes"
	"encoding/json"
	"fmt"
	"strings"
	"sync"

	"github.com/santhosh-tekuri/jsonschema/v6"

	discovery "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/trust-task-discovery/v0_4"
	taskerror "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/trust-task-error/v0_6"
	nextstep "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/trust-task-next-step/v0_1"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"

	query "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/entries/query/v0_1"
	grantrequest "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/request/v0_1"
	revoke "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/revoke/v0_1"
	revoked "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/revoked/v0_1"
)

var compiled sync.Map // schema JSON → *jsonschema.Schema

func compile(schemaJSON string) (*jsonschema.Schema, error) {
	if s, ok := compiled.Load(schemaJSON); ok {
		return s.(*jsonschema.Schema), nil
	}
	doc, err := jsonschema.UnmarshalJSON(strings.NewReader(schemaJSON))
	if err != nil {
		return nil, fmt.Errorf("schema: %w", err)
	}
	// One compiler per schema: request and response schemas may share an $id.
	c := jsonschema.NewCompiler()
	c.DefaultDraft(jsonschema.Draft2020)
	c.AssertFormat() // `format` is an assertion here, as ajv-formats makes it in the TypeScript demo
	const location = "urn:trust-tasks:payload-schema"
	if err := c.AddResource(location, doc); err != nil {
		return nil, fmt.Errorf("schema: %w", err)
	}
	schema, err := c.Compile(location)
	if err != nil {
		return nil, fmt.Errorf("schema: %w", err)
	}
	compiled.Store(schemaJSON, schema)
	return schema, nil
}

func check(schemaJSON string, payload json.RawMessage) error {
	schema, err := compile(schemaJSON)
	if err != nil {
		return err
	}
	value, err := jsonschema.UnmarshalJSON(bytes.NewReader(payload))
	if err != nil {
		return fmt.Errorf("payload: %w", err)
	}
	return schema.Validate(value)
}

// schemaValidator is the validator the receiving end's SPEC §7.2 pipeline runs (item 2).
type schemaValidator struct{}

// Validate implements trusttasks.PayloadValidator.
func (schemaValidator) Validate(schemaJSON string, payload json.RawMessage) error {
	return check(schemaJSON, payload)
}

var _ tt.PayloadValidator = schemaValidator{}

// replySchemas is the schema a reply's payload must satisfy, by its `type`.
var replySchemas = map[string]string{
	discovery.ResponseTypeURI:    discovery.ResponsePayloadSchemaJSON,
	nextstep.TypeURI:             nextstep.PayloadSchemaJSON,
	taskerror.TypeURI:            taskerror.PayloadSchemaJSON,
	grantrequest.ResponseTypeURI: grantrequest.ResponsePayloadSchemaJSON,
	query.ResponseTypeURI:        query.ResponsePayloadSchemaJSON,
	revoke.ResponseTypeURI:       revoke.ResponsePayloadSchemaJSON,
	revoked.ResponseTypeURI:      revoked.ResponsePayloadSchemaJSON,
}

// validateReply is a producer checking the reply it receives before acting on
// it, as a consumer checks a request.
func validateReply(typeURI string, payload json.RawMessage) error {
	schema, ok := replySchemas[typeURI]
	if !ok {
		return fmt.Errorf("unexpected reply type %s", typeURI)
	}
	if err := check(schema, payload); err != nil {
		return fmt.Errorf("reply %s fails its schema: %w", typeURI, err)
	}
	return nil
}

// nextStepHintSchema is hcp/grants/request's `$defs.NextStepHint`, taken from
// the pinned payload schema.
var nextStepHintSchema = sync.OnceValue(func() string {
	var schema struct {
		Defs map[string]json.RawMessage `json:"$defs"`
	}
	if err := json.Unmarshal([]byte(grantrequest.PayloadSchemaJSON), &schema); err != nil || schema.Defs["NextStepHint"] == nil {
		panic("hcp/grants/request's schema has no NextStepHint")
	}
	return string(schema.Defs["NextStepHint"])
})

// validNextStepHint is whether a next step's `hint` satisfies
// hcp/grants/request's `$defs.NextStepHint`.
func validNextStepHint(hint json.RawMessage) bool {
	return check(nextStepHintSchema(), hint) == nil
}
