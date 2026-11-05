package sqliteext

import (
	"database/sql"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRegexpFunctionMatch(t *testing.T) {
	db, err := sql.Open(DriverName, ":memory:")
	require.NoError(t, err)
	defer db.Close()

	var matched int
	require.NoError(t, db.QueryRow(`SELECT 'seafood' REGEXP 'foo.*'`).Scan(&matched))
	assert.Equal(t, 1, matched)

	require.NoError(t, db.QueryRow(`SELECT 'seafood' REGEXP '^bar'`).Scan(&matched))
	assert.Equal(t, 0, matched)
}

func TestRegexpFunctionInvalidPattern(t *testing.T) {
	db, err := sql.Open(DriverName, ":memory:")
	require.NoError(t, err)
	defer db.Close()

	var matched int
	err = db.QueryRow(`SELECT 'seafood' REGEXP '([0-9+'`).Scan(&matched)
	require.Error(t, err)
}
