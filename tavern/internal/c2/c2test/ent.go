package c2test

import (
	"context"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/namegen"
)

// NewRandomBeacon creates and returns a new randomized beacon.
func NewRandomBeacon(ctx context.Context, graph *ent.Client) *ent.Beacon {
	host := graph.Host.Create().
		SetPlatform(c2pb.Host_PLATFORM_UNSPECIFIED).
		SetIdentifier(namegen.NewComplex()).
		SaveX(ctx)

	return graph.Beacon.Create().
		SetIdentifier(namegen.NewComplex()).
		SetHost(host).
		SetTransport(c2pb.ActiveTransport_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)
}

// ConvertTaskToC2PB converts an *ent.Task to it's corresponding *c2pb.Task.
func ConvertTaskToC2PB(t *testing.T, ctx context.Context, task *ent.Task) *c2pb.Task {
	t.Helper()
	if task == nil {
		return nil
	}

	var params map[string]string
	require.NoError(t,
		json.Unmarshal(
			[]byte(
				task.
					QueryQuest().
					OnlyX(ctx).
					Parameters,
			),
			&params,
		),
	)

	var assetNames []string
	assets := task.
		QueryQuest().
		QueryTome().
		QueryAssets().
		Order(asset.ByID()).
		AllX(ctx)
	for _, a := range assets {
		assetNames = append(assetNames, a.Name)
	}

	return &c2pb.Task{
		Id: int64(task.ID),
		Tome: &epb.Tome{
			Eldritch: task.
				QueryQuest().
				QueryTome().
				OnlyX(ctx).
				Eldritch,
			Parameters: params,
			FileNames:  assetNames,
		},
		QuestName: task.
			QueryQuest().
			OnlyX(ctx).
			Name,
	}
}

// NewRandomAssignedTask generates a random task and assigns it to the provided beacon identifier.
func NewRandomAssignedTask(ctx context.Context, graph *ent.Client, beaconIdentifier string) *ent.Task {
	beacon := graph.Beacon.Query().
		Where(
			beacon.Identifier(beaconIdentifier),
		).OnlyX(ctx)
	bundle := graph.Asset.Create().
		SetName(namegen.NewComplex()).
		SetContent(newRandomBytes(1024)).
		SaveX(ctx)
	assets := []*ent.Asset{
		graph.Asset.Create().
			SetName(namegen.NewComplex()).
			SetContent(newRandomBytes(1024)).
			SaveX(ctx),
		graph.Asset.Create().
			SetName(namegen.NewComplex()).
			SetContent(newRandomBytes(1024)).
			SaveX(ctx),
	}
	tome := graph.Tome.Create().
		SetName(namegen.NewComplex()).
		SetEldritch(fmt.Sprintf(`print("%s")`, namegen.NewComplex())).
		SetDescription(string(newRandomBytes(120))).
		SetAuthor("kcarretto").
		SetParamDefs(`[{"name":"test-param","label":"Test","type":"string","placeholder":"Enter text..."}]`).
		AddAssets(assets...).
		SaveX(ctx)
	quest := graph.Quest.Create().
		SetName(namegen.NewComplex()).
		SetBundle(bundle).
		SetTome(tome).
		SetParameters(fmt.Sprintf(`{"test-param":"%v"}`, namegen.NewComplex())).
		SaveX(ctx)

	return graph.Task.Create().
		SetBeacon(beacon).
		SetQuest(quest).
		SaveX(ctx)
}

func newRandomBytes(length int) []byte {
	data := make([]byte, length)
	rand.Read(data)
	return data
}
