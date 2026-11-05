// Package sqliteext provides a SQLite driver that extends mattn/go-sqlite3
// with a REGEXP function, so Ent predicates using REGEXP work on both
// SQLite (local dev and tests) and MySQL (prod).
package sqliteext

import (
	"database/sql"
	"database/sql/driver"
	"regexp"

	sqlite3 "github.com/mattn/go-sqlite3"
)

// DriverName is the sql driver name for SQLite with REGEXP support.
// Use it anywhere sql.Open would use "sqlite3".
const DriverName = "sqlite3-regexp"

func init() {
	sql.Register(DriverName, &regexpDriver{})
}

type regexpDriver struct{}

func (d *regexpDriver) Open(dsn string) (driver.Conn, error) {
	conn, err := (&sqlite3.SQLiteDriver{}).Open(dsn)
	if err != nil {
		return nil, err
	}
	sc, ok := conn.(*sqlite3.SQLiteConn)
	if !ok {
		return conn, nil
	}
	if err := sc.RegisterFunc("regexp", matchRegexp, true); err != nil {
		conn.Close()
		return nil, err
	}
	return conn, nil
}

// matchRegexp backs the `value REGEXP pattern` operator. SQLite invokes it
// as regexp(pattern, value).
func matchRegexp(pattern, value string) (bool, error) {
	return regexp.MatchString(pattern, value)
}
