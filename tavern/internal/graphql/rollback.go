package graphql

import (
	"realm.pub/tavern/internal/dbutils"
	"realm.pub/tavern/internal/ent"
)

func rollback(tx *ent.Tx, err error) error {
	return dbutils.Rollback(tx, err)
}
