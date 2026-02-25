package schema_test

import (
	"context"
	"encoding/base64"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/schema/hostfilepreviewtype"
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

func TestHostFileHooks_TextPreview(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	content := []byte("Hello, world! This is a text file.")
	testFile := newHostFile(graph, "/test/text.txt", content)
	assert.Equal(t, hostfilepreviewtype.Text, testFile.PreviewType)
	assert.Equal(t, string(content), testFile.Preview)
}

func TestHostFileHooks_ImagePreview(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// PNG magic bytes + small payload
	content := append([]byte{0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A}, []byte("fake png data")...)
	testFile := newHostFile(graph, "/test/image.png", content)
	assert.Equal(t, hostfilepreviewtype.Image, testFile.PreviewType)
	assert.Equal(t, base64.StdEncoding.EncodeToString(content), testFile.Preview)
}

func TestHostFileHooks_ImageTooLarge(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// PNG magic bytes + payload > 512KB
	content := make([]byte, 524289) // 512KB + 1
	copy(content, []byte{0x89, 0x50, 0x4E, 0x47})
	testFile := newHostFile(graph, "/test/large_image.png", content)
	assert.Equal(t, hostfilepreviewtype.None, testFile.PreviewType)
	assert.Empty(t, testFile.Preview)
}

func TestHostFileHooks_BinaryNoPreview(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	content := []byte{0x00, 0x01, 0x02, 0x03, 0x04, 0x05}
	testFile := newHostFile(graph, "/test/binary.bin", content)
	assert.Equal(t, hostfilepreviewtype.None, testFile.PreviewType)
	assert.Empty(t, testFile.Preview)
}

func TestHostFileHooks_EmptyContent(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	testFile := newHostFile(graph, "/test/empty.txt", nil)
	assert.Equal(t, hostfilepreviewtype.None, testFile.PreviewType)
	assert.Empty(t, testFile.Preview)
}

func TestHostFileHooks_TextTruncation(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Create text content larger than 512KB
	content := []byte(strings.Repeat("a", 524288+500))
	testFile := newHostFile(graph, "/test/large_text.txt", content)
	assert.Equal(t, hostfilepreviewtype.Text, testFile.PreviewType)
	assert.Len(t, testFile.Preview, 524288) // Truncated to 512KB
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
