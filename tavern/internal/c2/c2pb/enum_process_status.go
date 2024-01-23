package c2pb

import (
	"database/sql/driver"
	"io"

	"github.com/99designs/gqlgen/graphql"
)

// Values provides list valid values for Enum.
func (Process_Status) Values() []string {
	values := make([]string, 0, len(Process_Status_name))
	for _, name := range Process_Status_name {
		values = append(values, name)
	}
	return values
}

// Value provides the DB a string from int.
func (p Process_Status) Value() (driver.Value, error) {
	return p.String(), nil
}

// Scan tells our code how to read the enum into our type.
func (p *Process_Status) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*p = Process_STATUS_UNKNOWN
		return nil
	}

	status, ok := Process_Status_value[name]
	if !ok {
		*p = Process_STATUS_UNKNOWN
		return nil
	}
	*p = Process_Status(status)

	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (p *Process_Status) MarshalGQL(w io.Writer) {
	graphql.MarshalString(p.String()).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (p *Process_Status) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}

	return p.Scan(str)
}
