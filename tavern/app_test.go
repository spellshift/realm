package main

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/buildprofile"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestEnsureDefaultBuildProfile(t *testing.T) {
	cases := []struct {
		name         string
		descriptions []string
		want         []string
	}{
		{name: "seeds a blank description", want: []string{""}},
		{name: "preserves operator description", descriptions: []string{"Operator notes"}, want: []string{"Operator notes"}},
		{name: "clears legacy description", descriptions: []string{legacyDefaultBuildProfileDescription}, want: []string{""}},
		{name: "handles duplicate defaults", descriptions: []string{"Operator notes", legacyDefaultBuildProfileDescription, ""}, want: []string{"Operator notes", "", ""}},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			ctx := context.Background()
			client := enttest.OpenTempDB(t)
			t.Cleanup(func() { require.NoError(t, client.Close()) })

			for _, description := range tc.descriptions {
				_, err := client.BuildProfile.Create().
					SetName(builder.DefaultBuildProfileName).
					SetDescription(description).
					Save(ctx)
				require.NoError(t, err)
			}

			for range 2 {
				require.NoError(t, ensureDefaultBuildProfile(ctx, client))
			}

			profiles, err := client.BuildProfile.Query().
				Where(buildprofile.Name(builder.DefaultBuildProfileName)).
				Order(ent.Asc(buildprofile.FieldID)).
				All(ctx)
			require.NoError(t, err)
			require.Len(t, profiles, len(tc.want))
			for i, profile := range profiles {
				require.Equal(t, tc.want[i], profile.Description)
			}
		})
	}
}
