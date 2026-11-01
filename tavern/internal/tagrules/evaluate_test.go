package tagrules

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

func newTestClient(t *testing.T) (*ent.Client, context.Context) {
	t.Helper()
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	return client, ctx
}

func mustRule(t *testing.T, ctx context.Context, client *ent.Client, tagID int, priority int, conds []Condition) *ent.TagRule {
	t.Helper()
	raw, err := MarshalConditions(conds)
	require.NoError(t, err)
	return client.TagRule.Create().
		SetTagID(tagID).
		SetPriority(priority).
		SetConditions(raw).
		SaveX(ctx)
}

func hostTagIDs(t *testing.T, ctx context.Context, client *ent.Client, hostID int) []int {
	t.Helper()
	h := client.Host.GetX(ctx, hostID)
	edges := h.QueryTags().AllX(ctx)
	ids := make([]int, 0, len(edges))
	for _, tg := range edges {
		ids = append(ids, tg.ID)
	}
	return ids
}

func TestEvaluateHostForCallbackFillsEmptySlot(t *testing.T) {
	client, ctx := newTestClient(t)
	defer client.Close()

	groupA := client.Tag.Create().SetKind(tag.KindGroup).SetName("group-a").SaveX(ctx)
	groupB := client.Tag.Create().SetKind(tag.KindGroup).SetName("group-b").SaveX(ctx)

	mustRule(t, ctx, client, groupA.ID, 0, []Condition{
		{Field: "platform", Predicate: "is", Values: []string{"PLATFORM_LINUX"}},
	})
	mustRule(t, ctx, client, groupB.ID, 10, []Condition{
		{Field: "platform", Predicate: "is", Values: []string{"PLATFORM_LINUX"}},
	})

	h := client.Host.Create().
		SetIdentifier("cb-host-1").
		SetName("web-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	require.NoError(t, EvaluateHostForCallback(ctx, client, h.ID))
	assert.ElementsMatch(t, []int{groupA.ID}, hostTagIDs(t, ctx, client, h.ID))

	// Second callback is a no-op: first matching rule already filled the slot.
	require.NoError(t, EvaluateHostForCallback(ctx, client, h.ID))
	assert.ElementsMatch(t, []int{groupA.ID}, hostTagIDs(t, ctx, client, h.ID))
}

func TestEvaluateHostForCallbackNeverDisplaces(t *testing.T) {
	client, ctx := newTestClient(t)
	defer client.Close()

	manual := client.Tag.Create().SetKind(tag.KindGroup).SetName("manual-group").SaveX(ctx)
	ruleTag := client.Tag.Create().SetKind(tag.KindGroup).SetName("rule-group").SaveX(ctx)
	mustRule(t, ctx, client, ruleTag.ID, 0, []Condition{
		{Field: "host", Predicate: "contains", Values: []string{"db"}},
	})

	h := client.Host.Create().
		SetIdentifier("cb-host-2").
		SetName("db-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		AddTagIDs(manual.ID).
		SaveX(ctx)

	require.NoError(t, EvaluateHostForCallback(ctx, client, h.ID))
	assert.ElementsMatch(t, []int{manual.ID}, hostTagIDs(t, ctx, client, h.ID))
}

func TestEvaluateHostForCallbackPerKindSlots(t *testing.T) {
	client, ctx := newTestClient(t)
	defer client.Close()

	groupTag := client.Tag.Create().SetKind(tag.KindGroup).SetName("g").SaveX(ctx)
	svcTag := client.Tag.Create().SetKind(tag.KindService).SetName("s").SaveX(ctx)
	mustRule(t, ctx, client, groupTag.ID, 0, []Condition{
		{Field: "primaryIP", Predicate: "contains", Values: []string{"10.0.0"}},
	})
	mustRule(t, ctx, client, svcTag.ID, 0, []Condition{
		{Field: "primaryIP", Predicate: "contains", Values: []string{"10.0.0"}},
	})

	h := client.Host.Create().
		SetIdentifier("cb-host-3").
		SetName("app-1").
		SetPlatform(c2pb.Host_PLATFORM_WINDOWS).
		SetPrimaryIP("10.0.0.9").
		SaveX(ctx)

	require.NoError(t, EvaluateHostForCallback(ctx, client, h.ID))
	assert.ElementsMatch(t, []int{groupTag.ID, svcTag.ID}, hostTagIDs(t, ctx, client, h.ID))
}

func TestSweepAllFillEmptyVsOverwrite(t *testing.T) {
	newFixture := func(t *testing.T) (*ent.Client, context.Context, int, int, int) {
		client, ctx := newTestClient(t)
		oldGroup := client.Tag.Create().SetKind(tag.KindGroup).SetName("old-group").SaveX(ctx)
		newGroup := client.Tag.Create().SetKind(tag.KindGroup).SetName("new-group").SaveX(ctx)
		mustRule(t, ctx, client, newGroup.ID, 0, []Condition{
			{Field: "host", Predicate: "contains", Values: []string{"web"}},
		})
		h := client.Host.Create().
			SetIdentifier("sweep-host").
			SetName("web-1").
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			AddTagIDs(oldGroup.ID).
			SaveX(ctx)
		return client, ctx, h.ID, oldGroup.ID, newGroup.ID
	}

	t.Run("fill-empty leaves occupied slot", func(t *testing.T) {
		client, ctx, hostID, oldGroupID, _ := newFixture(t)
		defer client.Close()
		results, err := SweepAll(ctx, client, SweepFillEmpty)
		require.NoError(t, err)
		require.Len(t, results, 1)
		assert.Equal(t, 1, results[0].MatchedHosts)
		assert.Equal(t, 0, results[0].TaggedHosts)
		assert.ElementsMatch(t, []int{oldGroupID}, hostTagIDs(t, ctx, client, hostID))
	})

	t.Run("overwrite replaces slot winner", func(t *testing.T) {
		client, ctx, hostID, _, newGroupID := newFixture(t)
		defer client.Close()
		results, err := SweepAll(ctx, client, SweepOverwrite)
		require.NoError(t, err)
		require.Len(t, results, 1)
		assert.Equal(t, 1, results[0].MatchedHosts)
		assert.Equal(t, 1, results[0].TaggedHosts)
		assert.ElementsMatch(t, []int{newGroupID}, hostTagIDs(t, ctx, client, hostID))
	})
}

func TestSweepAllLeavesUnmatchedUntouched(t *testing.T) {
	client, ctx := newTestClient(t)
	defer client.Close()

	other := client.Tag.Create().SetKind(tag.KindService).SetName("other").SaveX(ctx)
	ruleTag := client.Tag.Create().SetKind(tag.KindService).SetName("rule-svc").SaveX(ctx)
	mustRule(t, ctx, client, ruleTag.ID, 0, []Condition{
		{Field: "host", Predicate: "is", Values: []string{"no-such-host"}},
	})

	h := client.Host.Create().
		SetIdentifier("untouched-host").
		SetName("app-9").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		AddTagIDs(other.ID).
		SaveX(ctx)

	for _, mode := range []SweepMode{SweepFillEmpty, SweepOverwrite} {
		results, err := SweepAll(ctx, client, mode)
		require.NoError(t, err)
		require.Len(t, results, 1)
		assert.Equal(t, 0, results[0].MatchedHosts)
		assert.Equal(t, 0, results[0].TaggedHosts)
	}
	assert.ElementsMatch(t, []int{other.ID}, hostTagIDs(t, ctx, client, h.ID))
}
