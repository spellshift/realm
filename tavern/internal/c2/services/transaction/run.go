package transaction

import (
	"context"
	"fmt"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/ent"
)

// Run executes the given function within a transaction.
// It handles panic recovery and transaction rollback/commit.
func Run(ctx context.Context, client *ent.Client, fn func(tx *ent.Tx) error) error {
	tx, err := client.Tx(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to initialize transaction: %v", err)
	}

	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	if err := fn(tx); err != nil {
		if rerr := tx.Rollback(); rerr != nil {
			err = fmt.Errorf("%w: %v", err, rerr)
		}
		// Check if the error is already a gRPC status error, if not wrap it?
		// The original code wrapped it in `rollback` function which returned status.Errorf(Internal...)
		// We want to preserve the original logic.
		// If `fn` returns a status error, we should probably return it as is or wrap it?
		// Original `rollback` func: return status.Errorf(codes.Internal, "%v", err)
		// So it swallows the code and makes it Internal.
		return status.Errorf(codes.Internal, "%v", err)
	}

	if err := tx.Commit(); err != nil {
		if rerr := tx.Rollback(); rerr != nil {
			err = fmt.Errorf("%w: %v", err, rerr)
		}
		return status.Errorf(codes.Internal, "failed to commit transaction: %v", err)
	}

	return nil
}
