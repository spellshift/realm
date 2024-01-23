package c2pb

import (
	"database/sql/driver"
	"io"

	"github.com/99designs/gqlgen/graphql"
)

// Values provides list valid values for Enum.
func (Host_Platform) Values() []string {
	values := make([]string, 0, len(Host_Platform_name))
	for _, name := range Host_Platform_name {
		values = append(values, name)
	}
	return values
}

// Value provides the DB a string from int.
func (p Host_Platform) Value() (driver.Value, error) {
	return p.String(), nil
}

// Scan tells our code how to read the enum into our type.
func (p *Host_Platform) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*p = Host_PLATFORM_UNSPECIFIED
		return nil
	}

	status, ok := Host_Platform_value[name]
	if !ok {
		*p = Host_PLATFORM_UNSPECIFIED
		return nil
	}
	*p = Host_Platform(status)

	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (p Host_Platform) MarshalGQL(w io.Writer) {
	graphql.MarshalString(p.String()).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (p *Host_Platform) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}

	return p.Scan(str)
}
