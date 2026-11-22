package enttest

import (
	"database/sql"
	"net/url"
	"path/filepath"
	"strings"
	"testing"

	entsql "entgo.io/ent/dialect/sql"
	"realm.pub/tavern/internal/ent"

	// The mattn driver
	_ "github.com/mattn/go-sqlite3"
)

func sqliteDSN(path string) string {
	slashPath := filepath.ToSlash(path)
	if !strings.HasPrefix(slashPath, "/") {
		slashPath = "/" + slashPath
	}

	// _journal=WAL: Enables concurrent readers and a writer
	// _busy_timeout=10000: Forces SQLite to wait 10 seconds for a write lock before failing
	// _fk=1: Enables foreign keys (often needed for Realm)
	// _sync=NORMAL: Speeds up WAL mode safely
	// _txlock=immediate: Acquires write locks on BEGIN so read->write tx upgrades respect _busy_timeout
	q := url.Values{}
	q.Set("_journal", "WAL")
	q.Set("_busy_timeout", "10000")
	q.Set("_fk", "1")
	q.Set("_sync", "NORMAL")
	q.Set("_txlock", "immediate")

	u := url.URL{
		Scheme:   "file",
		Path:     slashPath,
		RawQuery: q.Encode(),
	}
	return u.String()
}

// NewDB creates a new on-disk temporary SQLite database for testing.
// It enables WAL mode, sets a busy timeout with immediate transaction locks, and tunes the connection pool.
// It uses t.TempDir() for isolated per-test storage and automatic cleanup after closing the DB.
func NewDB(t testing.TB) *sql.DB {
	t.Helper()

	// 1. Create an isolated per-test temp directory (automatically cleaned up by testing.TB)
	dbPath := filepath.Join(t.TempDir(), "tavern.db")

	// 2. Open SQLite with WAL mode, busy timeout, foreign keys, and immediate tx locking
	db, err := sql.Open("sqlite3", sqliteDSN(dbPath))
	if err != nil {
		t.Fatalf("failed to open db: %v", err)
	}

	// Register db.Close() after t.TempDir() so LIFO cleanup closes SQLite before removing the directory
	t.Cleanup(func() {
		_ = db.Close()
	})

	// 3. CRITICAL: Tune the Go Connection Pool
	// Limit open connections to prevent Go from overwhelming SQLite's queue
	db.SetMaxOpenConns(10)
	// Keep idle connections low to prevent stale CGO threads
	db.SetMaxIdleConns(2)

	return db
}

// OpenTempDB creates a new ent.Client attached to an on-disk temporary SQLite database.
func OpenTempDB(t testing.TB, opts ...Option) *ent.Client {
	t.Helper()

	client, _ := OpenTempDBWithDB(t, opts...)
	return client
}

// OpenTempDBWithDB creates a new ent.Client and returns it along with the underlying sql.DB.
func OpenTempDBWithDB(t testing.TB, opts ...Option) (*ent.Client, *sql.DB) {
	t.Helper()

	db := NewDB(t)
	drv := entsql.OpenDB("sqlite3", db)

	o := newOptions(opts)
	client := ent.NewClient(append(o.opts, ent.Driver(drv))...)

	migrateSchema(t, client, o)

	return client, db
}
