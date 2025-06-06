// Code generated by ent, DO NOT EDIT.

package ent

import (
	"fmt"
	"strings"
	"time"

	"entgo.io/ent"
	"entgo.io/ent/dialect/sql"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/host"
)

// Beacon is the model entity for the Beacon schema.
type Beacon struct {
	config `json:"-"`
	// ID of the ent.
	ID int `json:"id,omitempty"`
	// Timestamp of when this ent was created
	CreatedAt time.Time `json:"created_at,omitempty"`
	// Timestamp of when this ent was last updated
	LastModifiedAt time.Time `json:"last_modified_at,omitempty"`
	// A human readable identifier for the beacon.
	Name string `json:"name,omitempty"`
	// The identity the beacon is authenticated as (e.g. 'root')
	Principal string `json:"principal,omitempty"`
	// Unique identifier for the beacon. Unique to each instance of the beacon.
	Identifier string `json:"identifier,omitempty"`
	// Identifies the agent that the beacon is running as (e.g. 'imix').
	AgentIdentifier string `json:"agent_identifier,omitempty"`
	// Timestamp of when a task was last claimed or updated for the beacon.
	LastSeenAt time.Time `json:"last_seen_at,omitempty"`
	// Duration until next callback, in seconds.
	Interval uint64 `json:"interval,omitempty"`
	// Edges holds the relations/edges for other nodes in the graph.
	// The values are being populated by the BeaconQuery when eager-loading is set.
	Edges        BeaconEdges `json:"edges"`
	beacon_host  *int
	selectValues sql.SelectValues
}

// BeaconEdges holds the relations/edges for other nodes in the graph.
type BeaconEdges struct {
	// Host this beacon is running on.
	Host *Host `json:"host,omitempty"`
	// Tasks that have been assigned to the beacon.
	Tasks []*Task `json:"tasks,omitempty"`
	// Shells that have been created by the beacon.
	Shells []*Shell `json:"shells,omitempty"`
	// loadedTypes holds the information for reporting if a
	// type was loaded (or requested) in eager-loading or not.
	loadedTypes [3]bool
	// totalCount holds the count of the edges above.
	totalCount [3]map[string]int

	namedTasks  map[string][]*Task
	namedShells map[string][]*Shell
}

// HostOrErr returns the Host value or an error if the edge
// was not loaded in eager-loading, or loaded but was not found.
func (e BeaconEdges) HostOrErr() (*Host, error) {
	if e.Host != nil {
		return e.Host, nil
	} else if e.loadedTypes[0] {
		return nil, &NotFoundError{label: host.Label}
	}
	return nil, &NotLoadedError{edge: "host"}
}

// TasksOrErr returns the Tasks value or an error if the edge
// was not loaded in eager-loading.
func (e BeaconEdges) TasksOrErr() ([]*Task, error) {
	if e.loadedTypes[1] {
		return e.Tasks, nil
	}
	return nil, &NotLoadedError{edge: "tasks"}
}

// ShellsOrErr returns the Shells value or an error if the edge
// was not loaded in eager-loading.
func (e BeaconEdges) ShellsOrErr() ([]*Shell, error) {
	if e.loadedTypes[2] {
		return e.Shells, nil
	}
	return nil, &NotLoadedError{edge: "shells"}
}

// scanValues returns the types for scanning values from sql.Rows.
func (*Beacon) scanValues(columns []string) ([]any, error) {
	values := make([]any, len(columns))
	for i := range columns {
		switch columns[i] {
		case beacon.FieldID, beacon.FieldInterval:
			values[i] = new(sql.NullInt64)
		case beacon.FieldName, beacon.FieldPrincipal, beacon.FieldIdentifier, beacon.FieldAgentIdentifier:
			values[i] = new(sql.NullString)
		case beacon.FieldCreatedAt, beacon.FieldLastModifiedAt, beacon.FieldLastSeenAt:
			values[i] = new(sql.NullTime)
		case beacon.ForeignKeys[0]: // beacon_host
			values[i] = new(sql.NullInt64)
		default:
			values[i] = new(sql.UnknownType)
		}
	}
	return values, nil
}

// assignValues assigns the values that were returned from sql.Rows (after scanning)
// to the Beacon fields.
func (b *Beacon) assignValues(columns []string, values []any) error {
	if m, n := len(values), len(columns); m < n {
		return fmt.Errorf("mismatch number of scan values: %d != %d", m, n)
	}
	for i := range columns {
		switch columns[i] {
		case beacon.FieldID:
			value, ok := values[i].(*sql.NullInt64)
			if !ok {
				return fmt.Errorf("unexpected type %T for field id", value)
			}
			b.ID = int(value.Int64)
		case beacon.FieldCreatedAt:
			if value, ok := values[i].(*sql.NullTime); !ok {
				return fmt.Errorf("unexpected type %T for field created_at", values[i])
			} else if value.Valid {
				b.CreatedAt = value.Time
			}
		case beacon.FieldLastModifiedAt:
			if value, ok := values[i].(*sql.NullTime); !ok {
				return fmt.Errorf("unexpected type %T for field last_modified_at", values[i])
			} else if value.Valid {
				b.LastModifiedAt = value.Time
			}
		case beacon.FieldName:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field name", values[i])
			} else if value.Valid {
				b.Name = value.String
			}
		case beacon.FieldPrincipal:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field principal", values[i])
			} else if value.Valid {
				b.Principal = value.String
			}
		case beacon.FieldIdentifier:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field identifier", values[i])
			} else if value.Valid {
				b.Identifier = value.String
			}
		case beacon.FieldAgentIdentifier:
			if value, ok := values[i].(*sql.NullString); !ok {
				return fmt.Errorf("unexpected type %T for field agent_identifier", values[i])
			} else if value.Valid {
				b.AgentIdentifier = value.String
			}
		case beacon.FieldLastSeenAt:
			if value, ok := values[i].(*sql.NullTime); !ok {
				return fmt.Errorf("unexpected type %T for field last_seen_at", values[i])
			} else if value.Valid {
				b.LastSeenAt = value.Time
			}
		case beacon.FieldInterval:
			if value, ok := values[i].(*sql.NullInt64); !ok {
				return fmt.Errorf("unexpected type %T for field interval", values[i])
			} else if value.Valid {
				b.Interval = uint64(value.Int64)
			}
		case beacon.ForeignKeys[0]:
			if value, ok := values[i].(*sql.NullInt64); !ok {
				return fmt.Errorf("unexpected type %T for edge-field beacon_host", value)
			} else if value.Valid {
				b.beacon_host = new(int)
				*b.beacon_host = int(value.Int64)
			}
		default:
			b.selectValues.Set(columns[i], values[i])
		}
	}
	return nil
}

// Value returns the ent.Value that was dynamically selected and assigned to the Beacon.
// This includes values selected through modifiers, order, etc.
func (b *Beacon) Value(name string) (ent.Value, error) {
	return b.selectValues.Get(name)
}

// QueryHost queries the "host" edge of the Beacon entity.
func (b *Beacon) QueryHost() *HostQuery {
	return NewBeaconClient(b.config).QueryHost(b)
}

// QueryTasks queries the "tasks" edge of the Beacon entity.
func (b *Beacon) QueryTasks() *TaskQuery {
	return NewBeaconClient(b.config).QueryTasks(b)
}

// QueryShells queries the "shells" edge of the Beacon entity.
func (b *Beacon) QueryShells() *ShellQuery {
	return NewBeaconClient(b.config).QueryShells(b)
}

// Update returns a builder for updating this Beacon.
// Note that you need to call Beacon.Unwrap() before calling this method if this Beacon
// was returned from a transaction, and the transaction was committed or rolled back.
func (b *Beacon) Update() *BeaconUpdateOne {
	return NewBeaconClient(b.config).UpdateOne(b)
}

// Unwrap unwraps the Beacon entity that was returned from a transaction after it was closed,
// so that all future queries will be executed through the driver which created the transaction.
func (b *Beacon) Unwrap() *Beacon {
	_tx, ok := b.config.driver.(*txDriver)
	if !ok {
		panic("ent: Beacon is not a transactional entity")
	}
	b.config.driver = _tx.drv
	return b
}

// String implements the fmt.Stringer.
func (b *Beacon) String() string {
	var builder strings.Builder
	builder.WriteString("Beacon(")
	builder.WriteString(fmt.Sprintf("id=%v, ", b.ID))
	builder.WriteString("created_at=")
	builder.WriteString(b.CreatedAt.Format(time.ANSIC))
	builder.WriteString(", ")
	builder.WriteString("last_modified_at=")
	builder.WriteString(b.LastModifiedAt.Format(time.ANSIC))
	builder.WriteString(", ")
	builder.WriteString("name=")
	builder.WriteString(b.Name)
	builder.WriteString(", ")
	builder.WriteString("principal=")
	builder.WriteString(b.Principal)
	builder.WriteString(", ")
	builder.WriteString("identifier=")
	builder.WriteString(b.Identifier)
	builder.WriteString(", ")
	builder.WriteString("agent_identifier=")
	builder.WriteString(b.AgentIdentifier)
	builder.WriteString(", ")
	builder.WriteString("last_seen_at=")
	builder.WriteString(b.LastSeenAt.Format(time.ANSIC))
	builder.WriteString(", ")
	builder.WriteString("interval=")
	builder.WriteString(fmt.Sprintf("%v", b.Interval))
	builder.WriteByte(')')
	return builder.String()
}

// NamedTasks returns the Tasks named value or an error if the edge was not
// loaded in eager-loading with this name.
func (b *Beacon) NamedTasks(name string) ([]*Task, error) {
	if b.Edges.namedTasks == nil {
		return nil, &NotLoadedError{edge: name}
	}
	nodes, ok := b.Edges.namedTasks[name]
	if !ok {
		return nil, &NotLoadedError{edge: name}
	}
	return nodes, nil
}

func (b *Beacon) appendNamedTasks(name string, edges ...*Task) {
	if b.Edges.namedTasks == nil {
		b.Edges.namedTasks = make(map[string][]*Task)
	}
	if len(edges) == 0 {
		b.Edges.namedTasks[name] = []*Task{}
	} else {
		b.Edges.namedTasks[name] = append(b.Edges.namedTasks[name], edges...)
	}
}

// NamedShells returns the Shells named value or an error if the edge was not
// loaded in eager-loading with this name.
func (b *Beacon) NamedShells(name string) ([]*Shell, error) {
	if b.Edges.namedShells == nil {
		return nil, &NotLoadedError{edge: name}
	}
	nodes, ok := b.Edges.namedShells[name]
	if !ok {
		return nil, &NotLoadedError{edge: name}
	}
	return nodes, nil
}

func (b *Beacon) appendNamedShells(name string, edges ...*Shell) {
	if b.Edges.namedShells == nil {
		b.Edges.namedShells = make(map[string][]*Shell)
	}
	if len(edges) == 0 {
		b.Edges.namedShells[name] = []*Shell{}
	} else {
		b.Edges.namedShells[name] = append(b.Edges.namedShells[name], edges...)
	}
}

// Beacons is a parsable slice of Beacon.
type Beacons []*Beacon
