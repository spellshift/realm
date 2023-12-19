package c2

import (
	"fmt"

	"github.com/kcarretto/realm/tavern/internal/ent"
)

func rollback(tx *ent.Tx, err error) error {
	if rerr := tx.Rollback(); rerr != nil {
		err = fmt.Errorf("%w: %v", err, rerr)
	}
	return err
}
