package c2

import (
	"errors"
	"fmt"
	"testing"

	"github.com/go-sql-driver/mysql"
	"github.com/mattn/go-sqlite3"
)

func TestIsRetryableAutomatedTaskError(t *testing.T) {
	sqliteUnique := sqlite3.Error{Code: sqlite3.ErrConstraint, ExtendedCode: sqlite3.ErrConstraintUnique}
	sqliteForeignKey := sqlite3.Error{Code: sqlite3.ErrConstraint, ExtendedCode: sqlite3.ErrConstraintForeignKey}
	tests := []struct {
		name string
		err  error
		want bool
	}{
		{"sqlite occurrence race", fmt.Errorf("UNIQUE constraint failed: quests.scheduled_occurrence_key: %w", sqliteUnique), true},
		{"sqlite task race", fmt.Errorf("UNIQUE constraint failed: tasks.scheduled_dedup_key: %w", sqliteUnique), true},
		{"sqlite unrelated unique", fmt.Errorf("UNIQUE constraint failed: assets.name: %w", sqliteUnique), false},
		{"sqlite foreign key", fmt.Errorf("scheduled_occurrence_key: %w", sqliteForeignKey), false},
		{"sqlite busy", sqlite3.Error{Code: sqlite3.ErrBusy}, true},
		{"sqlite locked", sqlite3.Error{Code: sqlite3.ErrLocked}, true},
		{"mysql occurrence race", &mysql.MySQLError{Number: 1062, Message: "Duplicate entry '1:2' for key 'quests.scheduled_occurrence_key'"}, true},
		{"mysql task race", &mysql.MySQLError{Number: 1062, Message: "Duplicate entry '3:4' for key 'scheduled_dedup_key'"}, true},
		{"mysql unrelated unique", &mysql.MySQLError{Number: 1062, Message: "Duplicate entry 'x' for key 'assets.name'"}, false},
		{"mysql foreign key", &mysql.MySQLError{Number: 1452, Message: "scheduled_occurrence_key"}, false},
		{"other error", errors.New("scheduled_occurrence_key"), false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := isRetryableAutomatedTaskError(tt.err); got != tt.want {
				t.Errorf("isRetryableAutomatedTaskError(%v) = %v, want %v", tt.err, got, tt.want)
			}
		})
	}
}
