package graphql

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tag"
)

func subnetHost(t *testing.T, ctx context.Context, client *ent.Client, name, ip, subnet string) *ent.Host {
	t.Helper()
	create := client.Host.Create().
		SetIdentifier(name).
		SetPrimaryIP(ip).
		SetPlatform(c2pb.Host_PLATFORM_LINUX)
	if subnet != "" {
		create.SetSubnet(subnet)
	}
	return create.SaveX(ctx)
}

func TestGroupHostsBySubnet(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	a := subnetHost(t, ctx, client, "a", "10.1.2.10", "10.1.2.0/24")
	b := subnetHost(t, ctx, client, "b", "10.1.2.11", "10.1.2.0/24")
	c := subnetHost(t, ctx, client, "c", "10.2.0.10", "10.2.0.0/16")
	d := subnetHost(t, ctx, client, "d", "10.2.1.10", "10.2.0.0/16")
	singleton := subnetHost(t, ctx, client, "singleton", "10.3.0.10", "10.3.0.0/24")
	unmatchedIP := subnetHost(t, ctx, client, "unmatched", "10.4.0.10", "10.1.2.0/24")
	subnetHost(t, ctx, client, "unset", "10.5.0.10", "")

	tags, err := groupHostsBySubnet(ctx, client)
	require.NoError(t, err)
	require.Len(t, tags, 3)
	assert.Equal(t, "Subnet: 10.1.2.0/24", tags[0].Name)
	assert.Equal(t, "Subnet: 10.2.0.0/16", tags[1].Name)
	assert.Equal(t, "Subnet: 10.3.0.0/24", tags[2].Name)
	for _, tg := range tags {
		assert.Equal(t, tag.KindGroup, tg.Kind)
	}
	assert.ElementsMatch(t, []int{a.ID, b.ID, unmatchedIP.ID}, tags[0].QueryHosts().IDsX(ctx))
	assert.ElementsMatch(t, []int{c.ID, d.ID}, tags[1].QueryHosts().IDsX(ctx))
	assert.Equal(t, []int{singleton.ID}, tags[2].QueryHosts().IDsX(ctx))

	// A second run refreshes membership without creating duplicate tags.
	client.Host.UpdateOne(b).SetSubnet("10.2.0.0/16").SaveX(ctx)
	tags, err = groupHostsBySubnet(ctx, client)
	require.NoError(t, err)
	require.Len(t, tags, 3)
	assert.ElementsMatch(t, []int{a.ID, unmatchedIP.ID}, tags[0].QueryHosts().IDsX(ctx))
	assert.ElementsMatch(t, []int{b.ID, c.ID, d.ID}, tags[1].QueryHosts().IDsX(ctx))
	assert.Equal(t, 3, client.Tag.Query().CountX(ctx))
}

func TestGroupHostsBySubnetRejectsServiceNameCollision(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	subnetHost(t, ctx, client, "a", "10.1.2.10", "10.1.2.0/24")
	subnetHost(t, ctx, client, "b", "10.1.2.11", "10.1.2.0/24")
	client.Tag.Create().SetName("Subnet: 10.1.2.0/24").SetKind(tag.KindService).SaveX(ctx)

	_, err := groupHostsBySubnet(ctx, client)
	require.ErrorContains(t, err, "already exists")
	assert.Equal(t, 1, client.Tag.Query().CountX(ctx))
}
