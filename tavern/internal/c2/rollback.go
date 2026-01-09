package c2

import (
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/dbutils"
	"realm.pub/tavern/internal/ent"
)

func rollback(tx *ent.Tx, err error) error {
	return status.Errorf(codes.Internal, "%v", dbutils.Rollback(tx, err))
}
