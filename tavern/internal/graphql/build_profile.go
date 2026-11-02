package graphql

import (
	"context"
	"fmt"

	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/buildprofile"
)

// The first profile named Default is the one adopted or seeded at startup.
func (r *mutationResolver) isSeededDefaultBuildProfile(ctx context.Context, profile *ent.BuildProfile) (bool, error) {
	if profile.Name != builder.DefaultBuildProfileName {
		return false, nil
	}
	firstID, err := r.client.BuildProfile.Query().
		Where(buildprofile.Name(builder.DefaultBuildProfileName)).
		Order(ent.Asc(buildprofile.FieldID)).
		FirstID(ctx)
	if err != nil {
		return false, fmt.Errorf("failed to identify default build profile: %w", err)
	}
	return profile.ID == firstID, nil
}
