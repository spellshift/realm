// Code generated by ent, DO NOT EDIT.

package ent

import (
	"context"

	"entgo.io/ent/dialect/sql"
	"entgo.io/ent/dialect/sql/sqlgraph"
	"entgo.io/ent/schema/field"
	"realm.pub/tavern/internal/ent/hostcredential"
	"realm.pub/tavern/internal/ent/predicate"
)

// HostCredentialDelete is the builder for deleting a HostCredential entity.
type HostCredentialDelete struct {
	config
	hooks    []Hook
	mutation *HostCredentialMutation
}

// Where appends a list predicates to the HostCredentialDelete builder.
func (hcd *HostCredentialDelete) Where(ps ...predicate.HostCredential) *HostCredentialDelete {
	hcd.mutation.Where(ps...)
	return hcd
}

// Exec executes the deletion query and returns how many vertices were deleted.
func (hcd *HostCredentialDelete) Exec(ctx context.Context) (int, error) {
	return withHooks(ctx, hcd.sqlExec, hcd.mutation, hcd.hooks)
}

// ExecX is like Exec, but panics if an error occurs.
func (hcd *HostCredentialDelete) ExecX(ctx context.Context) int {
	n, err := hcd.Exec(ctx)
	if err != nil {
		panic(err)
	}
	return n
}

func (hcd *HostCredentialDelete) sqlExec(ctx context.Context) (int, error) {
	_spec := sqlgraph.NewDeleteSpec(hostcredential.Table, sqlgraph.NewFieldSpec(hostcredential.FieldID, field.TypeInt))
	if ps := hcd.mutation.predicates; len(ps) > 0 {
		_spec.Predicate = func(selector *sql.Selector) {
			for i := range ps {
				ps[i](selector)
			}
		}
	}
	affected, err := sqlgraph.DeleteNodes(ctx, hcd.driver, _spec)
	if err != nil && sqlgraph.IsConstraintError(err) {
		err = &ConstraintError{msg: err.Error(), wrap: err}
	}
	hcd.mutation.done = true
	return affected, err
}

// HostCredentialDeleteOne is the builder for deleting a single HostCredential entity.
type HostCredentialDeleteOne struct {
	hcd *HostCredentialDelete
}

// Where appends a list predicates to the HostCredentialDelete builder.
func (hcdo *HostCredentialDeleteOne) Where(ps ...predicate.HostCredential) *HostCredentialDeleteOne {
	hcdo.hcd.mutation.Where(ps...)
	return hcdo
}

// Exec executes the deletion query.
func (hcdo *HostCredentialDeleteOne) Exec(ctx context.Context) error {
	n, err := hcdo.hcd.Exec(ctx)
	switch {
	case err != nil:
		return err
	case n == 0:
		return &NotFoundError{hostcredential.Label}
	default:
		return nil
	}
}

// ExecX is like Exec, but panics if an error occurs.
func (hcdo *HostCredentialDeleteOne) ExecX(ctx context.Context) {
	if err := hcdo.Exec(ctx); err != nil {
		panic(err)
	}
}
