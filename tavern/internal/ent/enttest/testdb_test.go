package enttest

import (
	"context"
	"database/sql"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSQLiteDSN(t *testing.T) {
	tests := []struct {
		name         string
		path         string
		wantPathPart string
	}{
		{
			name:         "UnixPath",
			path:         "/tmp/tavern-test/tavern.db",
			wantPathPart: "file:///tmp/tavern-test/tavern.db",
		},
		{
			name:         "SpecialURIChars",
			path:         "/tmp/dir?a#b%20c/tavern.db",
			wantPathPart: "file:///tmp/dir%3Fa%23b%2520c/tavern.db",
		},
		{
			name:         "DrivePrefixPath",
			path:         "C:/Users/tester/tavern.db",
			wantPathPart: "file:///C:/Users/tester/tavern.db",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			dsn := sqliteDSN(tc.path)
			pathPart, rawQuery, found := strings.Cut(dsn, "?")
			require.True(t, found, "expected query string in DSN %q", dsn)
			assert.Equal(t, tc.wantPathPart, pathPart)

			params, err := url.ParseQuery(rawQuery)
			require.NoError(t, err)
			assert.Equal(t, "WAL", params.Get("_journal"))
			assert.Equal(t, "10000", params.Get("_busy_timeout"))
			assert.Equal(t, "1", params.Get("_fk"))
			assert.Equal(t, "NORMAL", params.Get("_sync"))
			assert.Equal(t, "immediate", params.Get("_txlock"))
		})
	}
}

func TestSQLiteDSN_OpensPathWithSpecialChars(t *testing.T) {
	dir := filepath.Join(t.TempDir(), "dir?with#special%chars and spaces")
	require.NoError(t, os.MkdirAll(dir, 0o700))

	dbPath := filepath.Join(dir, "tavern.db")
	db, err := sql.Open("sqlite3", sqliteDSN(dbPath))
	require.NoError(t, err)
	t.Cleanup(func() {
		_ = db.Close()
	})

	_, err = db.Exec(`CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)`)
	require.NoError(t, err)
	_, err = db.Exec(`INSERT INTO items (name) VALUES ('ok')`)
	require.NoError(t, err)

	var name string
	require.NoError(t, db.QueryRow(`SELECT name FROM items LIMIT 1`).Scan(&name))
	assert.Equal(t, "ok", name)

	_, err = os.Stat(dbPath)
	require.NoError(t, err, "expected database file at %q", dbPath)
}

func TestNewDB_PragmasAndIsolation(t *testing.T) {
	db1 := NewDB(t)
	db2 := NewDB(t)

	var journalMode string
	require.NoError(t, db1.QueryRow(`PRAGMA journal_mode`).Scan(&journalMode))
	assert.Equal(t, "wal", strings.ToLower(journalMode))

	var foreignKeys int
	require.NoError(t, db1.QueryRow(`PRAGMA foreign_keys`).Scan(&foreignKeys))
	assert.Equal(t, 1, foreignKeys)

	var busyTimeout int
	require.NoError(t, db1.QueryRow(`PRAGMA busy_timeout`).Scan(&busyTimeout))
	assert.Equal(t, 10000, busyTimeout)

	var syncMode int
	require.NoError(t, db1.QueryRow(`PRAGMA synchronous`).Scan(&syncMode))
	assert.Equal(t, 1, syncMode) // NORMAL

	// Verify db1 and db2 are backed by distinct files in separate t.TempDir() subdirectories.
	_, err := db1.Exec(`CREATE TABLE marker (id INTEGER PRIMARY KEY)`)
	require.NoError(t, err)

	var count int
	err = db2.QueryRow(`SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='marker'`).Scan(&count)
	require.NoError(t, err)
	assert.Equal(t, 0, count)
}

func TestNewDB_ConcurrentReadToWriteTxUpgrade(t *testing.T) {
	db := NewDB(t)
	ctx := context.Background()

	_, err := db.ExecContext(ctx, `CREATE TABLE counter (id INTEGER PRIMARY KEY, val INTEGER NOT NULL)`)
	require.NoError(t, err)
	_, err = db.ExecContext(ctx, `INSERT INTO counter (id, val) VALUES (1, 0)`)
	require.NoError(t, err)

	const workers = 10
	const iterations = 10

	var wg sync.WaitGroup
	errCh := make(chan error, workers*iterations)

	for range workers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for range iterations {
				tx, err := db.BeginTx(ctx, nil)
				if err != nil {
					errCh <- err
					return
				}

				var current int
				if err := tx.QueryRowContext(ctx, `SELECT val FROM counter WHERE id = 1`).Scan(&current); err != nil {
					_ = tx.Rollback()
					errCh <- err
					return
				}
				if _, err := tx.ExecContext(ctx, `UPDATE counter SET val = ? WHERE id = 1`, current+1); err != nil {
					_ = tx.Rollback()
					errCh <- err
					return
				}
				if err := tx.Commit(); err != nil {
					errCh <- err
					return
				}
			}
		}()
	}

	wg.Wait()
	close(errCh)

	for err := range errCh {
		require.NoError(t, err)
	}

	var total int
	require.NoError(t, db.QueryRowContext(ctx, `SELECT val FROM counter WHERE id = 1`).Scan(&total))
	assert.Equal(t, workers*iterations, total)
}

func TestOpenTempDBWithDB(t *testing.T) {
	client, db := OpenTempDBWithDB(t)
	require.NotNil(t, client)
	require.NotNil(t, db)

	ctx := context.Background()
	u, err := client.User.Create().
		SetName("tester").
		SetOauthID("oauth-tester").
		SetPhotoURL("https://example.com/p.png").
		Save(ctx)
	require.NoError(t, err)

	var count int
	require.NoError(t, db.QueryRowContext(ctx, `SELECT COUNT(*) FROM users WHERE id = ?`, u.ID).Scan(&count))
	assert.Equal(t, 1, count)
}
