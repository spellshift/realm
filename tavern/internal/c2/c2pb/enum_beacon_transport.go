package c2pb

import (
	"database/sql/driver"
	"io"
	"sort"

	"github.com/99designs/gqlgen/graphql"
)

// Values provides list valid values for Enum.
func (Beacon_Transport) Values() []string {
	values := make([]string, 0, len(Beacon_Transport_name))
	for _, name := range Beacon_Transport_name {
		values = append(values, name)
	}
	sort.Strings(values)
	return values
}

// Value provides the DB a string from int.
func (p Beacon_Transport) Value() (driver.Value, error) {
	return p.String(), nil
}

// Scan tells our code how to read the enum into our type.
func (p *Beacon_Transport) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*p = Beacon_TRANSPORT_UNSPECIFIED
		return nil
	}

	status, ok := Beacon_Transport_value[name]
	if !ok {
		*p = Beacon_TRANSPORT_UNSPECIFIED
		return nil
	}
	*p = Beacon_Transport(status)

	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (p Beacon_Transport) MarshalGQL(w io.Writer) {
	graphql.MarshalString(p.String()).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (p *Beacon_Transport) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}

	return p.Scan(str)
}
