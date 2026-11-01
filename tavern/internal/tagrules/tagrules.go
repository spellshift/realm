package tagrules

import (
	"encoding/json"
	"fmt"
	"strings"
)

// Condition mirrors one HostFilterRow for the TagRule field subset:
// field is one of platform, primaryIP, externalIP, host (host name).
// predicate is one of is, is-not, is-in, is-not-in, contains, does-not-contain.
// Rows AND together; multi-value rows OR within the row.
type Condition struct {
	Field     string   `json:"field"`
	Predicate string   `json:"predicate"`
	Values    []string `json:"values"`
}

// HostSnapshot is the host attributes a TagRule can match on.
type HostSnapshot struct {
	Platform   string
	PrimaryIP  string
	ExternalIP string
	Name       string
}

// Allowed values (mirrors ent schema validation).
var allowedFields = map[string]bool{
	"platform":   true,
	"primaryIP":  true,
	"externalIP": true,
	"host":       true,
}

var allowedPredicates = map[string]bool{
	"is":               true,
	"is-not":           true,
	"is-in":            true,
	"is-not-in":        true,
	"contains":         true,
	"does-not-contain": true,
}

// ParseConditions parses the JSON-encoded conditions column.
func ParseConditions(data string) ([]Condition, error) {
	var conds []Condition
	if err := json.Unmarshal([]byte(data), &conds); err != nil {
		return nil, fmt.Errorf("invalid tag rule conditions: %w", err)
	}
	if len(conds) == 0 {
		return nil, fmt.Errorf("tag rule conditions must contain at least one row")
	}
	for i, c := range conds {
		if err := ValidateCondition(c); err != nil {
			return nil, fmt.Errorf("conditions[%d]: %w", i, err)
		}
	}
	return conds, nil
}

// ValidateCondition checks one row.
func ValidateCondition(c Condition) error {
	if !allowedFields[c.Field] {
		return fmt.Errorf("unsupported field %q (allowed: platform, primaryIP, externalIP, host)", c.Field)
	}
	if !allowedPredicates[c.Predicate] {
		return fmt.Errorf("unsupported predicate %q", c.Predicate)
	}
	if c.Field == "platform" && (c.Predicate == "contains" || c.Predicate == "does-not-contain") {
		return fmt.Errorf("predicate %q not allowed for enum field platform", c.Predicate)
	}
	if len(c.Values) == 0 {
		return fmt.Errorf("values must not be empty")
	}
	for _, v := range c.Values {
		if strings.TrimSpace(v) == "" {
			return fmt.Errorf("values must not contain empty strings")
		}
	}
	switch c.Predicate {
	case "is", "is-not", "contains", "does-not-contain":
		if len(c.Values) != 1 {
			return fmt.Errorf("predicate %q requires exactly one value", c.Predicate)
		}
	}
	return nil
}

// MarshalConditions encodes rows for storage.
func MarshalConditions(conds []Condition) (string, error) {
	for i, c := range conds {
		if err := ValidateCondition(c); err != nil {
			return "", fmt.Errorf("conditions[%d]: %w", i, err)
		}
	}
	out, err := json.Marshal(conds)
	if err != nil {
		return "", err
	}
	return string(out), nil
}

// fieldValue extracts the snapshot value for a rule field.
// Note: `host` matches against the host NAME (per TAGRULE_DECISIONS.md),
// unlike the Hosts page where exact `host` rows filter by host ID.
func fieldValue(h HostSnapshot, field string) string {
	switch field {
	case "platform":
		return h.Platform
	case "primaryIP":
		return h.PrimaryIP
	case "externalIP":
		return h.ExternalIP
	case "host":
		return h.Name
	default:
		return ""
	}
}

func matchSingle(value, predicate, operand string) bool {
	switch predicate {
	case "is":
		return value == operand
	case "is-not":
		return value != operand
	case "contains":
		return strings.Contains(value, operand)
	case "does-not-contain":
		return !strings.Contains(value, operand)
	default:
		return false
	}
}

// MatchRow evaluates one row (OR within multi-value rows).
// Mirrors constructHostFilterQuery: `is`/`is not` use a single value,
// `is in`/`is not in` OR across values, `contains` is a single substring
// match, `does not contain` its negation.
func MatchRow(h HostSnapshot, c Condition) bool {
	value := fieldValue(h, c.Field)
	switch c.Predicate {
	case "is", "is-not", "contains", "does-not-contain":
		if len(c.Values) == 0 {
			return false
		}
		return matchSingle(value, c.Predicate, c.Values[0])
	case "is-in":
		for _, v := range c.Values {
			if value == v {
				return true
			}
		}
		return false
	case "is-not-in":
		for _, v := range c.Values {
			if value == v {
				return false
			}
		}
		return true
	default:
		return false
	}
}

// SweepMode controls whether a sweep only fills empty kind slots
// or overwrites the current tag with the winning rule's tag.
type SweepMode string

const (
	// SweepFillEmpty only fills kind slots that have no tag yet.
	SweepFillEmpty SweepMode = "fill-empty"
	// SweepOverwrite replaces the current tag in a kind slot with the winner.
	SweepOverwrite SweepMode = "overwrite"
)

// ParseSweepMode normalizes user input ("fill-empty", "fill_empty",
// "FILL_EMPTY", "overwrite", "OVERWRITE", ...) to a SweepMode.
func ParseSweepMode(s string) (SweepMode, error) {
	switch s {
	case "fill-empty", "fill_empty", "FILL_EMPTY", "fillEmpty", "fill", "FILL-EMPTY":
		return SweepFillEmpty, nil
	case "overwrite", "OVERWRITE", "Overwrite", "OVERWRITE_MODE", "overwrite-mode":
		return SweepOverwrite, nil
	default:
		return "", fmt.Errorf("unknown sweep mode %q (expected fill-empty or overwrite)", s)
	}
}

// Matches evaluates all rows (AND across rows).
func Matches(h HostSnapshot, conds []Condition) bool {
	for _, c := range conds {
		if !MatchRow(h, c) {
			return false
		}
	}
	return true
}

// MatchesJSON parses conditions JSON then evaluates.
func MatchesJSON(h HostSnapshot, data string) (bool, error) {
	conds, err := ParseConditions(data)
	if err != nil {
		return false, err
	}
	return Matches(h, conds), nil
}
