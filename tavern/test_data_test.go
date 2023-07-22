package main

import (
	"context"
	"testing"

	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/stretchr/testify/assert"
)

// TestCreateTestData ensures createTestData runs without error and creates at least one session.
func TestCreateTestData(t *testing.T) {
	var (
		ctx            = context.Background()
		driverName     = "sqlite3"
		dataSourceName = "file:test-create-test-data?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	createTestData(ctx, graph)
	assert.True(t, graph.Session.Query().ExistX(ctx))
}
