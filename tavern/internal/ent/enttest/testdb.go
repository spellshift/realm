package enttest

import (
	"database/sql"
	"os"
	"testing"

	"realm.pub/tavern/internal/ent"
	entsql "entgo.io/ent/dialect/sql"

	// The mattn driver
	_ "github.com/mattn/go-sqlite3"
)

// NewDB creates a new on-disk temporary SQLite database for testing.
// It enables WAL mode, sets a busy timeout, and tunes the connection pool.
// It also registers a cleanup function to remove the database files after the test.
func NewDB(t testing.TB) *sql.DB {
	// 1. Create the RAM-backed temp file
	file, err := os.CreateTemp("", "tavern-*.db")
	if err != nil {
		t.Fatalf("failed to create temp db: %v", err)
	}
	file.Close()

	// 2. The Magic DSN for mattn
	// _journal=WAL: Enables concurrent readers/writer
	// _busy_timeout=10000: Forces SQLite to wait 10 seconds for a write lock before failing
	// _fk=1: Enables foreign keys (often needed for Realm)
	// _sync=NORMAL: Speeds up WAL mode safely
	dsn := "file:" + file.Name() + "?_journal=WAL&_busy_timeout=10000&_fk=1&_sync=NORMAL"

	db, err := sql.Open("sqlite3", dsn)
	if err != nil {
		t.Fatalf("failed to open db: %v", err)
	}

	// Clean up the main db, WAL file, and Shared Memory file
	t.Cleanup(func() {
		db.Close()
		os.Remove(file.Name())
		os.Remove(file.Name() + "-wal")
		os.Remove(file.Name() + "-shm")
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
	client, _ := OpenTempDBWithDB(t, opts...)
	return client
}

// OpenTempDBWithDB creates a new ent.Client and returns it along with the underlying sql.DB.
func OpenTempDBWithDB(t testing.TB, opts ...Option) (*ent.Client, *sql.DB) {
	db := NewDB(t)
	drv := entsql.OpenDB("sqlite3", db)

	o := newOptions(opts)
	client := ent.NewClient(append(o.opts, ent.Driver(drv))...)

	migrateSchema(t, client, o)

	return client, db
}
