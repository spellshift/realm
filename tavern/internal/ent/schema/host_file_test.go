package schema_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestHostFileHooks(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	var (
		expectedPath    = "TestFile"
		expectedContent = []byte("ABunchOfBytes")
		expectedSize    = uint64(13)
		expectedHash    = "adaf38cc9a3d8d810f051a0098cb8737001394ae9b85d9f6fa56dbc2bcc08db6"
	)

	testFile := newHostFile(graph, expectedPath, expectedContent)
	assert.NotNil(t, testFile)
	assert.NotZero(t, testFile.ID)
	assert.Equal(t, expectedPath, testFile.Path)
	assert.Equal(t, string(expectedContent), string(testFile.Content))
	assert.Equal(t, expectedSize, testFile.Size)
	assert.Equal(t, expectedHash, testFile.Hash)
	assert.NotZero(t, testFile.CreatedAt)
	assert.WithinRange(t, testFile.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
	assert.WithinRange(t, testFile.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
	assert.NotZero(t, testFile.LastModifiedAt)
}

// newHostFile is a helper to create files directly via ent
func newHostFile(graph *ent.Client, path string, content []byte) *ent.HostFile {
	ctx := context.Background()
	host := graph.Host.Create().
		SetIdentifier("AAAA-BBBB-CCCC").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)
	beacon := graph.Beacon.Create().
		SetHost(host).
		SetIdentifier("ABCDEFG").
		SetTransport(c2pb.Beacon_TRANSPORT_UNSPECIFIED).
		SaveX(ctx)
	tome := graph.Tome.Create().
		SetName("Wowza").
		SetDescription("Why did we require this?").
		SetAuthor("kcarretto").
		SetEldritch("blah").
		SaveX(ctx)
	quest := graph.Quest.Create().
		SetName("HelloWorld").
		SetTome(tome).
		SaveX(ctx)
	task := graph.Task.Create().
		SetBeacon(beacon).
		SetQuest(quest).
		SaveX(ctx)
	return graph.HostFile.Create().
		SetTask(task).
		SetHost(host).
		SetPath(path).
		SetContent(content).
		SaveX(ctx)
}
