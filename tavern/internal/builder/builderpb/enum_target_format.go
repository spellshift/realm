package builderpb

import (
	"database/sql/driver"
	"io"
	"sort"

	"github.com/99designs/gqlgen/graphql"
)

// Values provides list valid values for Enum.
func (TargetFormat) Values() []string {
	values := make([]string, 0, len(TargetFormat_name))
	for _, name := range TargetFormat_name {
		values = append(values, name)
	}
	sort.Strings(values)
	return values
}

// Value provides the DB a string from int.
func (f TargetFormat) Value() (driver.Value, error) {
	return f.String(), nil
}

// Scan tells our code how to read the enum into our type.
func (f *TargetFormat) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*f = TargetFormat_TARGET_FORMAT_UNSPECIFIED
		return nil
	}

	status, ok := TargetFormat_value[name]
	if !ok {
		*f = TargetFormat_TARGET_FORMAT_UNSPECIFIED
		return nil
	}
	*f = TargetFormat(status)

	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (f TargetFormat) MarshalGQL(w io.Writer) {
	graphql.MarshalString(f.String()).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (f *TargetFormat) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}

	return f.Scan(str)
}
