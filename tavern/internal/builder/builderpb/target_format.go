package builderpb

import (
	"database/sql/driver"
	"fmt"
	"io"
	"sort"

	"github.com/99designs/gqlgen/graphql"
)

const (
	// DefaultInterval is the default callback interval in seconds for the IMIX agent.
	DefaultInterval = 5
)

// TargetFormat represents the output format for a build.
type TargetFormat string

const (
	TargetFormatBin            TargetFormat = "BIN"
	TargetFormatCdylib         TargetFormat = "CDYLIB"
	TargetFormatWindowsService TargetFormat = "WINDOWS_SERVICE"
)

// targetFormatName maps each TargetFormat to its string representation.
var targetFormatName = map[TargetFormat]string{
	TargetFormatBin:            "BIN",
	TargetFormatCdylib:         "CDYLIB",
	TargetFormatWindowsService: "WINDOWS_SERVICE",
}

// targetFormatValue maps string names to TargetFormat values.
var targetFormatValue = map[string]TargetFormat{
	"BIN":             TargetFormatBin,
	"CDYLIB":          TargetFormatCdylib,
	"WINDOWS_SERVICE": TargetFormatWindowsService,
}

// Values provides list of valid values for the ent enum interface.
func (TargetFormat) Values() []string {
	values := make([]string, 0, len(targetFormatName))
	for _, name := range targetFormatName {
		values = append(values, name)
	}
	sort.Strings(values)
	return values
}

// Value provides the DB a string from the enum (implements driver.Valuer).
func (f TargetFormat) Value() (driver.Value, error) {
	return string(f), nil
}

// Scan tells our code how to read the enum from the database (implements sql.Scanner).
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
		return fmt.Errorf("unsupported type for TargetFormat: %T", val)
	}

	tf, ok := targetFormatValue[name]
	if !ok {
		return fmt.Errorf("invalid TargetFormat value: %q", name)
	}
	*f = tf
	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (f TargetFormat) MarshalGQL(w io.Writer) {
	graphql.MarshalString(string(f)).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (f *TargetFormat) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}
	return f.Scan(str)
}
