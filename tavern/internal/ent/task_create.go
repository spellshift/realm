// Code generated by ent, DO NOT EDIT.

package ent

import (
	"context"
	"errors"
	"fmt"
	"time"

	"entgo.io/ent/dialect/sql/sqlgraph"
	"entgo.io/ent/schema/field"
	"github.com/kcarretto/realm/tavern/internal/ent/beacon"
	"github.com/kcarretto/realm/tavern/internal/ent/quest"
	"github.com/kcarretto/realm/tavern/internal/ent/task"
)

// TaskCreate is the builder for creating a Task entity.
type TaskCreate struct {
	config
	mutation *TaskMutation
	hooks    []Hook
}

// SetCreatedAt sets the "created_at" field.
func (tc *TaskCreate) SetCreatedAt(t time.Time) *TaskCreate {
	tc.mutation.SetCreatedAt(t)
	return tc
}

// SetNillableCreatedAt sets the "created_at" field if the given value is not nil.
func (tc *TaskCreate) SetNillableCreatedAt(t *time.Time) *TaskCreate {
	if t != nil {
		tc.SetCreatedAt(*t)
	}
	return tc
}

// SetLastModifiedAt sets the "last_modified_at" field.
func (tc *TaskCreate) SetLastModifiedAt(t time.Time) *TaskCreate {
	tc.mutation.SetLastModifiedAt(t)
	return tc
}

// SetNillableLastModifiedAt sets the "last_modified_at" field if the given value is not nil.
func (tc *TaskCreate) SetNillableLastModifiedAt(t *time.Time) *TaskCreate {
	if t != nil {
		tc.SetLastModifiedAt(*t)
	}
	return tc
}

// SetClaimedAt sets the "claimed_at" field.
func (tc *TaskCreate) SetClaimedAt(t time.Time) *TaskCreate {
	tc.mutation.SetClaimedAt(t)
	return tc
}

// SetNillableClaimedAt sets the "claimed_at" field if the given value is not nil.
func (tc *TaskCreate) SetNillableClaimedAt(t *time.Time) *TaskCreate {
	if t != nil {
		tc.SetClaimedAt(*t)
	}
	return tc
}

// SetExecStartedAt sets the "exec_started_at" field.
func (tc *TaskCreate) SetExecStartedAt(t time.Time) *TaskCreate {
	tc.mutation.SetExecStartedAt(t)
	return tc
}

// SetNillableExecStartedAt sets the "exec_started_at" field if the given value is not nil.
func (tc *TaskCreate) SetNillableExecStartedAt(t *time.Time) *TaskCreate {
	if t != nil {
		tc.SetExecStartedAt(*t)
	}
	return tc
}

// SetExecFinishedAt sets the "exec_finished_at" field.
func (tc *TaskCreate) SetExecFinishedAt(t time.Time) *TaskCreate {
	tc.mutation.SetExecFinishedAt(t)
	return tc
}

// SetNillableExecFinishedAt sets the "exec_finished_at" field if the given value is not nil.
func (tc *TaskCreate) SetNillableExecFinishedAt(t *time.Time) *TaskCreate {
	if t != nil {
		tc.SetExecFinishedAt(*t)
	}
	return tc
}

// SetOutput sets the "output" field.
func (tc *TaskCreate) SetOutput(s string) *TaskCreate {
	tc.mutation.SetOutput(s)
	return tc
}

// SetNillableOutput sets the "output" field if the given value is not nil.
func (tc *TaskCreate) SetNillableOutput(s *string) *TaskCreate {
	if s != nil {
		tc.SetOutput(*s)
	}
	return tc
}

// SetError sets the "error" field.
func (tc *TaskCreate) SetError(s string) *TaskCreate {
	tc.mutation.SetError(s)
	return tc
}

// SetNillableError sets the "error" field if the given value is not nil.
func (tc *TaskCreate) SetNillableError(s *string) *TaskCreate {
	if s != nil {
		tc.SetError(*s)
	}
	return tc
}

// SetQuestID sets the "quest" edge to the Quest entity by ID.
func (tc *TaskCreate) SetQuestID(id int) *TaskCreate {
	tc.mutation.SetQuestID(id)
	return tc
}

// SetQuest sets the "quest" edge to the Quest entity.
func (tc *TaskCreate) SetQuest(q *Quest) *TaskCreate {
	return tc.SetQuestID(q.ID)
}

// SetBeaconID sets the "beacon" edge to the Beacon entity by ID.
func (tc *TaskCreate) SetBeaconID(id int) *TaskCreate {
	tc.mutation.SetBeaconID(id)
	return tc
}

// SetBeacon sets the "beacon" edge to the Beacon entity.
func (tc *TaskCreate) SetBeacon(b *Beacon) *TaskCreate {
	return tc.SetBeaconID(b.ID)
}

// Mutation returns the TaskMutation object of the builder.
func (tc *TaskCreate) Mutation() *TaskMutation {
	return tc.mutation
}

// Save creates the Task in the database.
func (tc *TaskCreate) Save(ctx context.Context) (*Task, error) {
	tc.defaults()
	return withHooks[*Task, TaskMutation](ctx, tc.sqlSave, tc.mutation, tc.hooks)
}

// SaveX calls Save and panics if Save returns an error.
func (tc *TaskCreate) SaveX(ctx context.Context) *Task {
	v, err := tc.Save(ctx)
	if err != nil {
		panic(err)
	}
	return v
}

// Exec executes the query.
func (tc *TaskCreate) Exec(ctx context.Context) error {
	_, err := tc.Save(ctx)
	return err
}

// ExecX is like Exec, but panics if an error occurs.
func (tc *TaskCreate) ExecX(ctx context.Context) {
	if err := tc.Exec(ctx); err != nil {
		panic(err)
	}
}

// defaults sets the default values of the builder before save.
func (tc *TaskCreate) defaults() {
	if _, ok := tc.mutation.CreatedAt(); !ok {
		v := task.DefaultCreatedAt()
		tc.mutation.SetCreatedAt(v)
	}
	if _, ok := tc.mutation.LastModifiedAt(); !ok {
		v := task.DefaultLastModifiedAt()
		tc.mutation.SetLastModifiedAt(v)
	}
}

// check runs all checks and user-defined validators on the builder.
func (tc *TaskCreate) check() error {
	if _, ok := tc.mutation.CreatedAt(); !ok {
		return &ValidationError{Name: "created_at", err: errors.New(`ent: missing required field "Task.created_at"`)}
	}
	if _, ok := tc.mutation.LastModifiedAt(); !ok {
		return &ValidationError{Name: "last_modified_at", err: errors.New(`ent: missing required field "Task.last_modified_at"`)}
	}
	if _, ok := tc.mutation.QuestID(); !ok {
		return &ValidationError{Name: "quest", err: errors.New(`ent: missing required edge "Task.quest"`)}
	}
	if _, ok := tc.mutation.BeaconID(); !ok {
		return &ValidationError{Name: "beacon", err: errors.New(`ent: missing required edge "Task.beacon"`)}
	}
	return nil
}

func (tc *TaskCreate) sqlSave(ctx context.Context) (*Task, error) {
	if err := tc.check(); err != nil {
		return nil, err
	}
	_node, _spec := tc.createSpec()
	if err := sqlgraph.CreateNode(ctx, tc.driver, _spec); err != nil {
		if sqlgraph.IsConstraintError(err) {
			err = &ConstraintError{msg: err.Error(), wrap: err}
		}
		return nil, err
	}
	id := _spec.ID.Value.(int64)
	_node.ID = int(id)
	tc.mutation.id = &_node.ID
	tc.mutation.done = true
	return _node, nil
}

func (tc *TaskCreate) createSpec() (*Task, *sqlgraph.CreateSpec) {
	var (
		_node = &Task{config: tc.config}
		_spec = sqlgraph.NewCreateSpec(task.Table, sqlgraph.NewFieldSpec(task.FieldID, field.TypeInt))
	)
	if value, ok := tc.mutation.CreatedAt(); ok {
		_spec.SetField(task.FieldCreatedAt, field.TypeTime, value)
		_node.CreatedAt = value
	}
	if value, ok := tc.mutation.LastModifiedAt(); ok {
		_spec.SetField(task.FieldLastModifiedAt, field.TypeTime, value)
		_node.LastModifiedAt = value
	}
	if value, ok := tc.mutation.ClaimedAt(); ok {
		_spec.SetField(task.FieldClaimedAt, field.TypeTime, value)
		_node.ClaimedAt = value
	}
	if value, ok := tc.mutation.ExecStartedAt(); ok {
		_spec.SetField(task.FieldExecStartedAt, field.TypeTime, value)
		_node.ExecStartedAt = value
	}
	if value, ok := tc.mutation.ExecFinishedAt(); ok {
		_spec.SetField(task.FieldExecFinishedAt, field.TypeTime, value)
		_node.ExecFinishedAt = value
	}
	if value, ok := tc.mutation.Output(); ok {
		_spec.SetField(task.FieldOutput, field.TypeString, value)
		_node.Output = value
	}
	if value, ok := tc.mutation.Error(); ok {
		_spec.SetField(task.FieldError, field.TypeString, value)
		_node.Error = value
	}
	if nodes := tc.mutation.QuestIDs(); len(nodes) > 0 {
		edge := &sqlgraph.EdgeSpec{
			Rel:     sqlgraph.M2O,
			Inverse: true,
			Table:   task.QuestTable,
			Columns: []string{task.QuestColumn},
			Bidi:    false,
			Target: &sqlgraph.EdgeTarget{
				IDSpec: &sqlgraph.FieldSpec{
					Type:   field.TypeInt,
					Column: quest.FieldID,
				},
			},
		}
		for _, k := range nodes {
			edge.Target.Nodes = append(edge.Target.Nodes, k)
		}
		_node.quest_tasks = &nodes[0]
		_spec.Edges = append(_spec.Edges, edge)
	}
	if nodes := tc.mutation.BeaconIDs(); len(nodes) > 0 {
		edge := &sqlgraph.EdgeSpec{
			Rel:     sqlgraph.M2O,
			Inverse: false,
			Table:   task.BeaconTable,
			Columns: []string{task.BeaconColumn},
			Bidi:    false,
			Target: &sqlgraph.EdgeTarget{
				IDSpec: &sqlgraph.FieldSpec{
					Type:   field.TypeInt,
					Column: beacon.FieldID,
				},
			},
		}
		for _, k := range nodes {
			edge.Target.Nodes = append(edge.Target.Nodes, k)
		}
		_node.task_beacon = &nodes[0]
		_spec.Edges = append(_spec.Edges, edge)
	}
	return _node, _spec
}

// TaskCreateBulk is the builder for creating many Task entities in bulk.
type TaskCreateBulk struct {
	config
	builders []*TaskCreate
}

// Save creates the Task entities in the database.
func (tcb *TaskCreateBulk) Save(ctx context.Context) ([]*Task, error) {
	specs := make([]*sqlgraph.CreateSpec, len(tcb.builders))
	nodes := make([]*Task, len(tcb.builders))
	mutators := make([]Mutator, len(tcb.builders))
	for i := range tcb.builders {
		func(i int, root context.Context) {
			builder := tcb.builders[i]
			builder.defaults()
			var mut Mutator = MutateFunc(func(ctx context.Context, m Mutation) (Value, error) {
				mutation, ok := m.(*TaskMutation)
				if !ok {
					return nil, fmt.Errorf("unexpected mutation type %T", m)
				}
				if err := builder.check(); err != nil {
					return nil, err
				}
				builder.mutation = mutation
				nodes[i], specs[i] = builder.createSpec()
				var err error
				if i < len(mutators)-1 {
					_, err = mutators[i+1].Mutate(root, tcb.builders[i+1].mutation)
				} else {
					spec := &sqlgraph.BatchCreateSpec{Nodes: specs}
					// Invoke the actual operation on the latest mutation in the chain.
					if err = sqlgraph.BatchCreate(ctx, tcb.driver, spec); err != nil {
						if sqlgraph.IsConstraintError(err) {
							err = &ConstraintError{msg: err.Error(), wrap: err}
						}
					}
				}
				if err != nil {
					return nil, err
				}
				mutation.id = &nodes[i].ID
				if specs[i].ID.Value != nil {
					id := specs[i].ID.Value.(int64)
					nodes[i].ID = int(id)
				}
				mutation.done = true
				return nodes[i], nil
			})
			for i := len(builder.hooks) - 1; i >= 0; i-- {
				mut = builder.hooks[i](mut)
			}
			mutators[i] = mut
		}(i, ctx)
	}
	if len(mutators) > 0 {
		if _, err := mutators[0].Mutate(ctx, tcb.builders[0].mutation); err != nil {
			return nil, err
		}
	}
	return nodes, nil
}

// SaveX is like Save, but panics if an error occurs.
func (tcb *TaskCreateBulk) SaveX(ctx context.Context) []*Task {
	v, err := tcb.Save(ctx)
	if err != nil {
		panic(err)
	}
	return v
}

// Exec executes the query.
func (tcb *TaskCreateBulk) Exec(ctx context.Context) error {
	_, err := tcb.Save(ctx)
	return err
}

// ExecX is like Exec, but panics if an error occurs.
func (tcb *TaskCreateBulk) ExecX(ctx context.Context) {
	if err := tcb.Exec(ctx); err != nil {
		panic(err)
	}
}