package dbutils

import (
	"fmt"

	"realm.pub/tavern/internal/ent"
)

// Rollback rolls back the transaction and wraps the error if the rollback fails.
func Rollback(tx *ent.Tx, err error) error {
	if rerr := tx.Rollback(); rerr != nil {
		return fmt.Errorf("%w: %v", err, rerr)
	}
	return err
}
