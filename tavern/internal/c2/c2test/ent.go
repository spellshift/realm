package c2test

import (
	"context"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/file"
	"realm.pub/tavern/internal/namegen"
)

// NewRandomBeacon creates and returns a new randomized beacon.
func NewRandomBeacon(ctx context.Context, graph *ent.Client) *ent.Beacon {
	host := graph.Host.Create().
		SetIdentifier(namegen.NewComplex()).
		SaveX(ctx)

	return graph.Beacon.Create().
		SetIdentifier(namegen.NewComplex()).
		SetHost(host).
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

	var fileNames []string
	files := task.
		QueryQuest().
		QueryTome().
		QueryFiles().
		Order(file.ByID()).
		AllX(ctx)
	for _, f := range files {
		fileNames = append(fileNames, f.Name)
	}

	return &c2pb.Task{
		Id: int64(task.ID),
		Eldritch: task.
			QueryQuest().
			QueryTome().
			OnlyX(ctx).
			Eldritch,
		Parameters: params,
		FileNames:  fileNames,
	}
}

// NewRandomAssignedTask generates a random task and assigns it to the provided beacon identifier.
func NewRandomAssignedTask(ctx context.Context, graph *ent.Client, beaconIdentifier string) *ent.Task {
	beacon := graph.Beacon.Query().
		Where(
			beacon.Identifier(beaconIdentifier),
		).OnlyX(ctx)
	bundle := graph.File.Create().
		SetName(namegen.NewComplex()).
		SetContent(newRandomBytes(1024)).
		SaveX(ctx)
	files := []*ent.File{
		graph.File.Create().
			SetName(namegen.NewComplex()).
			SetContent(newRandomBytes(1024)).
			SaveX(ctx),
		graph.File.Create().
			SetName(namegen.NewComplex()).
			SetContent(newRandomBytes(1024)).
			SaveX(ctx),
	}
	tome := graph.Tome.Create().
		SetName(namegen.NewComplex()).
		SetEldritch(fmt.Sprintf(`print("%s")`, namegen.NewComplex())).
		SetDescription(string(newRandomBytes(120))).
		SetParamDefs(`{"test":"string"}`).
		AddFiles(files...).
		SaveX(ctx)
	quest := graph.Quest.Create().
		SetName(namegen.NewComplex()).
		SetBundle(bundle).
		SetTome(tome).
		SetParameters(fmt.Sprintf(`{"test":"%v"}`, namegen.NewComplex())).
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
