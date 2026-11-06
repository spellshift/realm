package graphql

import (
	"context"
	"net/netip"
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
	return client.Host.Create().
		SetIdentifier(name).
		SetPrimaryIP(ip).
		SetSubnet(subnet).
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
}

func TestGroupHostsBySubnet(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	a := subnetHost(t, ctx, client, "a", "10.1.2.10", "10.1.2.0/24")
	b := subnetHost(t, ctx, client, "b", "10.1.2.11", "10.1.2.10/24")
	c := subnetHost(t, ctx, client, "c", "10.2.0.10", "10.2.0.0/16")
	d := subnetHost(t, ctx, client, "d", "10.2.1.10", "10.2.0.0/16")
	subnetHost(t, ctx, client, "singleton", "10.3.0.10", "10.3.0.0/24")
	subnetHost(t, ctx, client, "stale", "10.4.0.10", "10.1.2.0/24")
	subnetHost(t, ctx, client, "invalid", "10.5.0.10", "not-a-cidr")
	subnetHost(t, ctx, client, "loopback-a", "127.0.0.1", "127.0.0.0/8")
	subnetHost(t, ctx, client, "loopback-b", "127.0.0.2", "127.0.0.0/8")

	tags, err := groupHostsBySubnet(ctx, client)
	require.NoError(t, err)
	require.Len(t, tags, 2)
	assert.Equal(t, "Subnet: 10.1.2.0/24", tags[0].Name)
	assert.Equal(t, "Subnet: 10.2.0.0/16", tags[1].Name)
	for _, tg := range tags {
		assert.Equal(t, tag.KindGroup, tg.Kind)
	}
	assert.ElementsMatch(t, []int{a.ID, b.ID}, tags[0].QueryHosts().IDsX(ctx))
	assert.ElementsMatch(t, []int{c.ID, d.ID}, tags[1].QueryHosts().IDsX(ctx))

	// A second run refreshes membership without creating duplicate tags.
	client.Host.UpdateOne(b).SetPrimaryIP("10.2.1.11").SetSubnet("10.2.0.0/16").SaveX(ctx)
	tags, err = groupHostsBySubnet(ctx, client)
	require.NoError(t, err)
	require.Len(t, tags, 1)
	assert.Equal(t, "Subnet: 10.2.0.0/16", tags[0].Name)
	assert.ElementsMatch(t, []int{b.ID, c.ID, d.ID}, tags[0].QueryHosts().IDsX(ctx))
	assert.Equal(t, 2, client.Tag.Query().CountX(ctx))
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

func TestUsableHostSubnet(t *testing.T) {
	tests := []struct {
		name, ip, subnet string
		want             bool
	}{
		{"private IPv4", "10.1.2.10", "10.1.2.0/24", true},
		{"unique-local IPv6", "fd00::10", "fd00::/64", true},
		{"IPv4 catch-all", "10.1.2.10", "0.0.0.0/0", false},
		{"IPv6 catch-all", "fd00::10", "::/0", false},
		{"IPv4 host route", "10.1.2.10", "10.1.2.10/32", false},
		{"IPv6 host route", "fd00::10", "fd00::10/128", false},
		{"multicast", "224.0.1.1", "224.0.1.0/24", false},
		{"broadcast", "255.255.255.255", "255.255.255.0/24", false},
		{"loopback", "127.0.0.1", "127.0.0.0/8", false},
		{"link-local", "169.254.1.1", "169.254.0.0/16", false},
		{"mismatched subnet", "10.1.2.10", "10.2.0.0/16", false},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ip := netip.MustParseAddr(tc.ip)
			prefix := netip.MustParsePrefix(tc.subnet)
			assert.Equal(t, tc.want, usableHostSubnet(ip, prefix))
		})
	}
}
