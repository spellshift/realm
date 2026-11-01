package c2_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tag"
)

func claimRequest(beaconID, hostID, hostName string, platform c2pb.Host_Platform, primaryIP string) *c2pb.ClaimTasksRequest {
	return &c2pb.ClaimTasksRequest{
		Beacon: &c2pb.Beacon{
			Identifier: beaconID,
			Principal:  "root",
			Agent: &c2pb.Agent{
				Identifier: "test-agent",
			},
			Host: &c2pb.Host{
				Identifier: hostID,
				Name:       hostName,
				Platform:   platform,
				PrimaryIp:  primaryIP,
			},
			AvailableTransports: &c2pb.AvailableTransports{
				Transports: []*c2pb.Transport{
					{
						Uri:      "grpc://127.0.0.1:8080",
						Interval: uint64(60),
						Type:     c2pb.Transport_TRANSPORT_GRPC,
					},
				},
				ActiveIndex: 0,
			},
		},
	}
}

func TestClaimTasksAppliesTagRules(t *testing.T) {
	ctx := context.Background()
	client, graph, close, _ := c2test.New(t)
	defer close()

	// Seed a group tag and a rule matching linux hosts.
	linuxGroup := graph.Tag.Create().
		SetKind(tag.KindGroup).
		SetName("auto-linux").
		SaveX(ctx)
	graph.TagRule.Create().
		SetTag(linuxGroup).
		SetPriority(0).
		SetConditions(`[{"field":"platform","predicate":"is","values":["PLATFORM_LINUX"]}]`).
		SaveX(ctx)

	// Matching callback fills the empty group slot.
	_, err := client.ClaimTasks(ctx, claimRequest("beacon-tagrule-1", "host-tagrule-1", "web-1", c2pb.Host_PLATFORM_LINUX, "10.0.0.5"))
	require.NoError(t, err)

	h := graph.Host.Query().Where(host.Identifier("host-tagrule-1")).OnlyX(ctx)
	tags := h.QueryTags().AllX(ctx)
	require.Len(t, tags, 1)
	assert.Equal(t, linuxGroup.ID, tags[0].ID)

	// Second callback is a no-op: re-attaching is idempotent.
	_, err = client.ClaimTasks(ctx, claimRequest("beacon-tagrule-1", "host-tagrule-1", "web-1", c2pb.Host_PLATFORM_LINUX, "10.0.0.5"))
	require.NoError(t, err)
	h = graph.Host.Query().Where(host.Identifier("host-tagrule-1")).OnlyX(ctx)
	assert.Len(t, h.QueryTags().AllX(ctx), 1)

	// Non-matching platform is left alone.
	_, err = client.ClaimTasks(ctx, claimRequest("beacon-tagrule-2", "host-tagrule-2", "win-1", c2pb.Host_PLATFORM_WINDOWS, "10.0.0.6"))
	require.NoError(t, err)
	h2 := graph.Host.Query().Where(host.Identifier("host-tagrule-2")).OnlyX(ctx)
	assert.Empty(t, h2.QueryTags().AllX(ctx))
}

func TestClaimTasksNeverDisplacesExistingTag(t *testing.T) {
	ctx := context.Background()
	client, graph, close, _ := c2test.New(t)
	defer close()

	manual := graph.Tag.Create().
		SetKind(tag.KindGroup).
		SetName("manual-group").
		SaveX(ctx)
	ruleTag := graph.Tag.Create().
		SetKind(tag.KindGroup).
		SetName("rule-group").
		SaveX(ctx)
	graph.TagRule.Create().
		SetTag(ruleTag).
		SetPriority(0).
		SetConditions(`[{"field":"host","predicate":"contains","values":["db"]}]`).
		SaveX(ctx)

	// Pre-attach the manual tag, then callback: the occupied slot is untouched.
	pre := graph.Host.Create().
		SetIdentifier("host-tagrule-3").
		SetName("db-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		AddTagIDs(manual.ID).
		SaveX(ctx)
	_ = pre

	_, err := client.ClaimTasks(ctx, claimRequest("beacon-tagrule-3", "host-tagrule-3", "db-1", c2pb.Host_PLATFORM_LINUX, ""))
	require.NoError(t, err)

	h := graph.Host.Query().Where(host.Identifier("host-tagrule-3")).OnlyX(ctx)
	got := h.QueryTags().AllX(ctx)
	require.Len(t, got, 1)
	assert.Equal(t, manual.ID, got[0].ID)
}
