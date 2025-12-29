package c2pb

import (
	"database/sql/driver"
	"io"
	"sort"

	"github.com/99designs/gqlgen/graphql"
)

// Values provides list valid values for Enum.
func (ActiveTransport_Type) Values() []string {
	values := make([]string, 0, len(ActiveTransport_Type_name))
	for _, name := range ActiveTransport_Type_name {
		values = append(values, name)
	}
	sort.Strings(values)
	return values
}

// Value provides the DB a string from int.
func (p ActiveTransport_Type) Value() (driver.Value, error) {
	return p.String(), nil
}

// Scan tells our code how to read the enum into our type.
func (p *ActiveTransport_Type) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*p = ActiveTransport_TRANSPORT_UNSPECIFIED
		return nil
	}

	status, ok := ActiveTransport_Type_value[name]
	if !ok {
		*p = ActiveTransport_TRANSPORT_UNSPECIFIED
		return nil
	}
	*p = ActiveTransport_Type(status)

	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (p ActiveTransport_Type) MarshalGQL(w io.Writer) {
	graphql.MarshalString(p.String()).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (p *ActiveTransport_Type) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}

	return p.Scan(str)
}
