package epb

import (
	"database/sql/driver"
	"io"
	"sort"

	"github.com/99designs/gqlgen/graphql"
)

// Values provides list valid values for Enum.
func (Credential_Kind) Values() []string {
	values := make([]string, 0, len(Process_Status_name))
	for _, name := range Process_Status_name {
		values = append(values, name)
	}
	sort.Strings(values)
	return values
}

// Value provides the DB a string from int.
func (c Credential_Kind) Value() (driver.Value, error) {
	return c.String(), nil
}

// Scan tells our code how to read the enum into our type.
func (c *Credential_Kind) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*c = Credential_KIND_UNSPECIFIED
		return nil
	}

	if name == "" {
		*c = Credential_KIND_UNSPECIFIED
		return nil
	}

	kind, ok := Credential_Kind_value[name]
	if !ok {
		*c = Credential_KIND_UNSPECIFIED
		return nil
	}
	*c = Credential_Kind(kind)

	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (c Credential_Kind) MarshalGQL(w io.Writer) {
	graphql.MarshalString(c.String()).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (c *Credential_Kind) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}

	return c.Scan(str)
}
