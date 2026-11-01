package schema

import (
	"encoding/json"
	"fmt"
	"strings"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// TagRule holds the schema definition for the TagRule entity.
// A TagRule attaches an existing Tag to hosts whose attributes match
// a set of Field + Predicate + Value rows (see TAGRULE_DECISIONS.md).
// Rows AND together; multi-value rows OR within the row.
type TagRule struct {
	ent.Schema
}

// Fields of the TagRule.
func (TagRule) Fields() []ent.Field {
	return []ent.Field{
		field.Int("priority").
			Default(0).
			Annotations(
				entgql.OrderField("PRIORITY"),
			).
			Comment("Evaluation order; lower values evaluate first. Ties break by ID."),
		field.Text("conditions").
			SchemaType(map[string]string{
				dialect.MySQL: "LONGTEXT",
			}).
			Validate(validateTagRuleConditions).
			Comment("JSON-encoded TagRule rows: [{field, predicate, values}]. Allowed fields: platform, primaryIP, externalIP, host (name)."),
	}
}

// Edges of the TagRule.
func (TagRule) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("tag", Tag.Type).
			Unique().
			Required().
			Comment("Existing tag applied when conditions match. Tag kind (group/service) fills that kind slot."),
	}
}

// Annotations describes additional information for the ent.
func (TagRule) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entgql.RelayConnection(),
		entgql.MultiOrder(),
		entsql.Annotation{
			Collation: "utf8mb4_general_ci",
		},
	}
}

// Mixin defines common shared properties for the ent.
func (TagRule) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // createdAt, lastModifiedAt
	}
}

var tagRuleAllowedFields = map[string]bool{
	"platform":   true,
	"primaryIP":  true,
	"externalIP": true,
	"host":       true,
}

var tagRuleAllowedPredicates = map[string]bool{
	"is":               true,
	"is-not":           true,
	"is-in":            true,
	"is-not-in":        true,
	"contains":         true,
	"does-not-contain": true,
}

type tagRuleConditionJSON struct {
	Field     string   `json:"field"`
	Predicate string   `json:"predicate"`
	Values    []string `json:"values"`
}

func validateTagRuleConditions(data string) error {
	var conds []tagRuleConditionJSON
	if err := json.Unmarshal([]byte(data), &conds); err != nil {
		return fmt.Errorf("conditions must be a JSON array of {field, predicate, values}: %w", err)
	}
	if len(conds) == 0 {
		return fmt.Errorf("conditions must contain at least one row")
	}
	for i, c := range conds {
		if !tagRuleAllowedFields[c.Field] {
			return fmt.Errorf("conditions[%d]: unsupported field %q (allowed: platform, primaryIP, externalIP, host)", i, c.Field)
		}
		if !tagRuleAllowedPredicates[c.Predicate] {
			return fmt.Errorf("conditions[%d]: unsupported predicate %q", i, c.Predicate)
		}
		if c.Field == "platform" && (c.Predicate == "contains" || c.Predicate == "does-not-contain") {
			return fmt.Errorf("conditions[%d]: predicate %q not allowed for enum field platform", i, c.Predicate)
		}
		if len(c.Values) == 0 {
			return fmt.Errorf("conditions[%d]: values must not be empty", i)
		}
		for _, v := range c.Values {
			if strings.TrimSpace(v) == "" {
				return fmt.Errorf("conditions[%d]: values must not contain empty strings", i)
			}
		}
		single := c.Predicate == "is" || c.Predicate == "is-not" || c.Predicate == "contains" || c.Predicate == "does-not-contain"
		if single && len(c.Values) != 1 {
			return fmt.Errorf("conditions[%d]: predicate %q requires exactly one value", i, c.Predicate)
		}
	}
	return nil
}
