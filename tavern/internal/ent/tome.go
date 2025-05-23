// Code generated by ent, DO NOT EDIT.

package ent

import (
	"fmt"
	"strings"
	"time"

	"entgo.io/ent"
	"entgo.io/ent/dialect/sql"
	"realm.pub/tavern/internal/ent/repository"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/internal/ent/user"
)

// Tome is the model entity for the Tome schema.
type Tome struct {
	config `json:"-"`
	// ID of the ent.
	ID int `json:"id,omitempty"`
	// Timestamp of when this ent was created
	CreatedAt time.Time `json:"created_at,omitempty"`
	// Timestamp of when this ent was last updated
	LastModifiedAt time.Time `json:"last_modified_at,omitempty"`
	// Name of the tome
	Name string `json:"name,omitempty"`
	// Information about the tome
	Description string `json:"description,omitempty"`
	// Name of the author who created the tome.
	Author string `json:"author,omitempty"`
	// Information about the tomes support model.
	SupportModel tome.SupportModel `json:"support_model,omitempty"`
	// MITRE ATT&CK tactic provided by the tome.
	Tactic tome.Tactic `json:"tactic,omitempty"`
	// JSON string describing what parameters are used with the tome. Requires a list of JSON objects, one for each parameter.
	ParamDefs string `json:"param_defs,omitempty"`
	// A SHA3 digest of the eldritch field
	Hash string `json:"hash,omitempty"`
	// Eldritch script that will be executed when the tome is run
	Eldritch string `json:"eldritch,omitempty"`
	// Edges holds the relations/edges for other nodes in the graph.
	// The values are being populated by the TomeQuery when eager-loading is set.
	Edges           TomeEdges `json:"edges"`
	tome_uploader   *int
	tome_repository *int
	selectValues    sql.SelectValues
}

// TomeEdges holds the relations/edges for other nodes in the graph.
type TomeEdges struct {
	// Any files required for tome execution that will be bundled and provided to the agent for download
	Files []*File `json:"files,omitempty"`
	// User who uploaded the tome (may be null).
	Uploader *User `json:"uploader,omitempty"`
	// Repository from which this Tome was imported (may be null).
	Repository *Repository `json:"repository,omitempty"`
	// loadedTypes holds the information for reporting if a
	// type was loaded (or requested) in eager-loading or not.
	loadedTypes [3]bool
	// totalCount holds the count of the edges above.
	totalCount [3]map[string]int

	namedFiles map[string][]*File
}

// FilesOrErr returns the Files value or an error if the edge
// was not loaded in eager-loading.
func (e TomeEdges) FilesOrErr() ([]*File, error) {
	if e.loadedTypes[0] {
		return e.Files, nil
	}
	return nil, &NotLoadedError{edge: "files"}
}

// UploaderOrErr returns the Uploader value or an error if the edge
// was not loaded in eager-loading, or loaded but was not found.
func (e TomeEdges) UploaderOrErr() (*User, error) {
	if e.Uploader != nil {
		return e.Uploader, nil
	} else if e.loadedTypes[1] {
		return nil, &NotFoundError{label: user.Label}
	}
	return nil, &NotLoadedError{edge: "uploader"}
}

// RepositoryOrErr returns the Repository value or an error if the edge
// was not loaded in eager-loading, or loaded but was not found.
func (e TomeEdges) RepositoryOrErr() (*Repository, error) {
	if e.Repository != nil {
		return e.Repository, nil
	} else if e.loadedTypes[2] {
		return nil, &NotFoundError{label: repository.Label}
	}
	return nil, &NotLoadedError{edge: "repository"}
}

// scanValues returns the types for scanning values from sql.Rows.
func (*Tome) scanValues(columns []string) ([]any, error) {
	values := make([]any, len(columns))
	for i := range columns {
		switch columns[i] {
		case tome.FieldID:
			values[i] = new(sql.NullInt64)
		case tome.FieldName, tome.FieldDescription, tome.FieldAuthor, tome.FieldSupportModel, tome.FieldTactic, tome.FieldParamDefs, tome.FieldHash, tome.FieldEldritch:
			values[i] = new(sql.NullString)
		case tome.FieldCreatedAt, tome.FieldLastModifiedAt:
			values[i] = new(sql.NullTime)
		case tome.ForeignKeys[0]: // tome_uploader
			values[i] = new(sql.NullInt64)
		case tome.ForeignKeys[1]: // tome_repository
			values[i] = new(sql.NullInt64)
		default:
			values[i] = new(sql.UnknownType)
		}
	}
	return values, nil
}

// assignValues assigns the values that were returned from sql.Rows (after scanning)
// to the Tome fields.
func (t *Tome) assignValues(columns []string, values []any) error {
	if m, n := len(values), len(columns); m < n {
		return fmt.Errorf("mismatch number of scan values: %d != %d", m, n)
	}
	for i := range columns {
		switch columns[i] {
		case tome.FieldID:
			value, ok := values[i].(*sql.NullInt64)
			if !ok {
				return fmt.Errorf("unexpected type %T for field id", value)
			}
			t.ID = int(value.Int64)
		case tome.FieldCreatedAt:
			if value, ok := values[i].(*sql.NullTime); !ok {
				return fmt.Errorf("unexpected type %T for field created_at", values[i])
			} else if value.Valid {
				t.CreatedAt = value.Time
			}
		case tome.FieldLastModifiedAt:
			if value, ok := values[i].(*sql.NullTime); !ok {
				return fmt.Errorf("unexpected type %T for field last_modified_at", values[i])
			} else if value.Valid {
				t.LastModifiedAt = value.Time
			}
		case tome.FieldName:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field name", values[i])
			} else if value.Valid {
				t.Name = value.String
			}
		case tome.FieldDescription:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field description", values[i])
			} else if value.Valid {
				t.Description = value.String
			}
		case tome.FieldAuthor:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field author", values[i])
			} else if value.Valid {
				t.Author = value.String
			}
		case tome.FieldSupportModel:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field support_model", values[i])
			} else if value.Valid {
				t.SupportModel = tome.SupportModel(value.String)
			}
		case tome.FieldTactic:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field tactic", values[i])
			} else if value.Valid {
				t.Tactic = tome.Tactic(value.String)
			}
		case tome.FieldParamDefs:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field param_defs", values[i])
			} else if value.Valid {
				t.ParamDefs = value.String
			}
		case tome.FieldHash:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field hash", values[i])
			} else if value.Valid {
				t.Hash = value.String
			}
		case tome.FieldEldritch:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field eldritch", values[i])
			} else if value.Valid {
				t.Eldritch = value.String
			}
		case tome.ForeignKeys[0]:
			if value, ok := values[i].(*sql.NullInt64); !ok {
				return fmt.Errorf("unexpected type %T for edge-field tome_uploader", value)
			} else if value.Valid {
				t.tome_uploader = new(int)
				*t.tome_uploader = int(value.Int64)
			}
		case tome.ForeignKeys[1]:
			if value, ok := values[i].(*sql.NullInt64); !ok {
				return fmt.Errorf("unexpected type %T for edge-field tome_repository", value)
			} else if value.Valid {
				t.tome_repository = new(int)
				*t.tome_repository = int(value.Int64)
			}
		default:
			t.selectValues.Set(columns[i], values[i])
		}
	}
	return nil
}

// Value returns the ent.Value that was dynamically selected and assigned to the Tome.
// This includes values selected through modifiers, order, etc.
func (t *Tome) Value(name string) (ent.Value, error) {
	return t.selectValues.Get(name)
}

// QueryFiles queries the "files" edge of the Tome entity.
func (t *Tome) QueryFiles() *FileQuery {
	return NewTomeClient(t.config).QueryFiles(t)
}

// QueryUploader queries the "uploader" edge of the Tome entity.
func (t *Tome) QueryUploader() *UserQuery {
	return NewTomeClient(t.config).QueryUploader(t)
}

// QueryRepository queries the "repository" edge of the Tome entity.
func (t *Tome) QueryRepository() *RepositoryQuery {
	return NewTomeClient(t.config).QueryRepository(t)
}

// Update returns a builder for updating this Tome.
// Note that you need to call Tome.Unwrap() before calling this method if this Tome
// was returned from a transaction, and the transaction was committed or rolled back.
func (t *Tome) Update() *TomeUpdateOne {
	return NewTomeClient(t.config).UpdateOne(t)
}

// Unwrap unwraps the Tome entity that was returned from a transaction after it was closed,
// so that all future queries will be executed through the driver which created the transaction.
func (t *Tome) Unwrap() *Tome {
	_tx, ok := t.config.driver.(*txDriver)
	if !ok {
		panic("ent: Tome is not a transactional entity")
	}
	t.config.driver = _tx.drv
	return t
}

// String implements the fmt.Stringer.
func (t *Tome) String() string {
	var builder strings.Builder
	builder.WriteString("Tome(")
	builder.WriteString(fmt.Sprintf("id=%v, ", t.ID))
	builder.WriteString("created_at=")
	builder.WriteString(t.CreatedAt.Format(time.ANSIC))
	builder.WriteString(", ")
	builder.WriteString("last_modified_at=")
	builder.WriteString(t.LastModifiedAt.Format(time.ANSIC))
	builder.WriteString(", ")
	builder.WriteString("name=")
	builder.WriteString(t.Name)
	builder.WriteString(", ")
	builder.WriteString("description=")
	builder.WriteString(t.Description)
	builder.WriteString(", ")
	builder.WriteString("author=")
	builder.WriteString(t.Author)
	builder.WriteString(", ")
	builder.WriteString("support_model=")
	builder.WriteString(fmt.Sprintf("%v", t.SupportModel))
	builder.WriteString(", ")
	builder.WriteString("tactic=")
	builder.WriteString(fmt.Sprintf("%v", t.Tactic))
	builder.WriteString(", ")
	builder.WriteString("param_defs=")
	builder.WriteString(t.ParamDefs)
	builder.WriteString(", ")
	builder.WriteString("hash=")
	builder.WriteString(t.Hash)
	builder.WriteString(", ")
	builder.WriteString("eldritch=")
	builder.WriteString(t.Eldritch)
	builder.WriteByte(')')
	return builder.String()
}

// NamedFiles returns the Files named value or an error if the edge was not
// loaded in eager-loading with this name.
func (t *Tome) NamedFiles(name string) ([]*File, error) {
	if t.Edges.namedFiles == nil {
		return nil, &NotLoadedError{edge: name}
	}
	nodes, ok := t.Edges.namedFiles[name]
	if !ok {
		return nil, &NotLoadedError{edge: name}
	}
	return nodes, nil
}

func (t *Tome) appendNamedFiles(name string, edges ...*File) {
	if t.Edges.namedFiles == nil {
		t.Edges.namedFiles = make(map[string][]*File)
	}
	if len(edges) == 0 {
		t.Edges.namedFiles[name] = []*File{}
	} else {
		t.Edges.namedFiles[name] = append(t.Edges.namedFiles[name], edges...)
	}
}

// Tomes is a parsable slice of Tome.
type Tomes []*Tome
