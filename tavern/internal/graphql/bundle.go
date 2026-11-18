package graphql

import (
	"context"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/tomebundle"
)

func createBundle(ctx context.Context, client *ent.Client, assets []*ent.Asset) (*ent.Asset, error) {
	return tomebundle.Create(ctx, client, assets)
}
