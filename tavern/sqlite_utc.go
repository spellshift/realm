package main

import (
	"context"
	"time"

	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/sql"
)

// utcDriver wraps an Ent SQL driver for SQLite so all time.Time arguments are
// converted to UTC before binding.
//
// mattn/go-sqlite3 serializes time.Time as TEXT with its original timezone
// offset, and SQLite compares TEXT values lexicographically. Mixing local
// time.Now() writes with UTC GraphQL filters (e.g. nextSeenAtGTE) causes range
// comparisons to fail when running in non-UTC timezones.
type utcDriver struct {
	*sql.Driver
}

var _ dialect.Driver = (*utcDriver)(nil)

func (d *utcDriver) Exec(ctx context.Context, query string, args, v any) error {
	return d.Driver.Exec(ctx, query, normalizeTimeArgsUTC(args), v)
}

func (d *utcDriver) Query(ctx context.Context, query string, args, v any) error {
	return d.Driver.Query(ctx, query, normalizeTimeArgsUTC(args), v)
}

func (d *utcDriver) Tx(ctx context.Context) (dialect.Tx, error) {
	return d.BeginTx(ctx, nil)
}

func (d *utcDriver) BeginTx(ctx context.Context, opts *sql.TxOptions) (dialect.Tx, error) {
	tx, err := d.Driver.BeginTx(ctx, opts)
	if err != nil {
		return nil, err
	}
	return &utcTx{Tx: tx}, nil
}

type utcTx struct {
	dialect.Tx
}

func (t *utcTx) Exec(ctx context.Context, query string, args, v any) error {
	return t.Tx.Exec(ctx, query, normalizeTimeArgsUTC(args), v)
}

func (t *utcTx) Query(ctx context.Context, query string, args, v any) error {
	return t.Tx.Query(ctx, query, normalizeTimeArgsUTC(args), v)
}

func normalizeTimeArgsUTC(args any) any {
	argv, ok := args.([]any)
	if !ok {
		return args
	}

	var out []any
	for i, arg := range argv {
		normalized, changed := toUTCArg(arg)
		if !changed {
			continue
		}
		if out == nil {
			out = make([]any, len(argv))
			copy(out, argv)
		}
		out[i] = normalized
	}
	if out == nil {
		return args
	}
	return out
}

func toUTCArg(arg any) (any, bool) {
	switch v := arg.(type) {
	case time.Time:
		if v.Location() == time.UTC {
			return arg, false
		}
		return v.UTC(), true
	case *time.Time:
		if v == nil || v.Location() == time.UTC {
			return arg, false
		}
		u := v.UTC()
		return &u, true
	case sql.NullTime:
		if !v.Valid || v.Time.Location() == time.UTC {
			return arg, false
		}
		v.Time = v.Time.UTC()
		return v, true
	case *sql.NullTime:
		if v == nil || !v.Valid || v.Time.Location() == time.UTC {
			return arg, false
		}
		u := *v
		u.Time = u.Time.UTC()
		return &u, true
	default:
		return arg, false
	}
}
