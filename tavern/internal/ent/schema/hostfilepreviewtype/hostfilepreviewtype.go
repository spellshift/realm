package hostfilepreviewtype

import (
	"database/sql/driver"
	"fmt"
	"io"

	"github.com/99designs/gqlgen/graphql"
)

// HostFilePreviewType defines the type of preview available for a host file.
type HostFilePreviewType string

const (
	Text  HostFilePreviewType = "TEXT"
	Image HostFilePreviewType = "IMAGE"
	None  HostFilePreviewType = "NONE"
)

// Values provides the list of valid values for the enum (required by ent).
func (HostFilePreviewType) Values() []string {
	return []string{
		string(Text),
		string(Image),
		string(None),
	}
}

// Value provides the DB a string from the enum (implements driver.Valuer).
func (p HostFilePreviewType) Value() (driver.Value, error) {
	return string(p), nil
}

// Scan tells our code how to read the enum from the DB (implements sql.Scanner).
func (p *HostFilePreviewType) Scan(val any) error {
	var name string
	switch v := val.(type) {
	case nil:
		return nil
	case string:
		name = v
	case []uint8:
		name = string(v)
	default:
		*p = None
		return nil
	}

	switch name {
	case string(Text):
		*p = Text
	case string(Image):
		*p = Image
	case string(None):
		*p = None
	case "":
		*p = None
	default:
		return fmt.Errorf("invalid HostFilePreviewType: %q", name)
	}
	return nil
}

// MarshalGQL writes a formatted string value for GraphQL.
func (p HostFilePreviewType) MarshalGQL(w io.Writer) {
	graphql.MarshalString(string(p)).MarshalGQL(w)
}

// UnmarshalGQL parses a GraphQL string representation into the enum.
func (p *HostFilePreviewType) UnmarshalGQL(v interface{}) error {
	str, err := graphql.UnmarshalString(v)
	if err != nil {
		return err
	}
	return p.Scan(str)
}
