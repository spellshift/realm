package executor_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/builder/executor"
)

// buildTarGz packages name -> contents pairs into a tar.gz archive, writing
// entries with the header names exactly as given (no path sanitization), so
// tests can construct archives containing path traversal sequences.
func buildTarGz(t *testing.T, files map[string]string) []byte {
	t.Helper()

	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)

	for name, contents := range files {
		hdr := &tar.Header{
			Name:     name,
			Typeflag: tar.TypeReg,
			Mode:     0o644,
			Size:     int64(len(contents)),
		}
		require.NoError(t, tw.WriteHeader(hdr))
		_, err := tw.Write([]byte(contents))
		require.NoError(t, err)
	}

	require.NoError(t, tw.Close())
	require.NoError(t, gw.Close())
	return buf.Bytes()
}

// TestLocalExecutor_Build_TomeArchive_PathTraversal verifies that a tome
// archive containing a path-traversal entry (e.g. "../../../etc/cron.d/pwn")
// is rejected with an error instead of being extracted outside the per-tome
// workspace directory.
func TestLocalExecutor_Build_TomeArchive_PathTraversal(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	maliciousArchive := buildTarGz(t, map[string]string{
		"../../../../tmp/realm-tome-traversal-poc": "pwned",
	})

	spec := executor.BuildSpec{
		TaskID:      1,
		TargetOS:    "linux",
		BuildScript: "true",
		Tomes: []executor.TomeData{
			{ID: 1, Name: "evil", Contents: maliciousArchive},
		},
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	_, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "illegal path traversal")

	// Confirm the archive entry was never written outside the workspace.
	_, statErr := os.Stat("/tmp/realm-tome-traversal-poc")
	assert.True(t, os.IsNotExist(statErr), "path traversal entry must not be written to the host filesystem")
	t.Cleanup(func() { os.Remove("/tmp/realm-tome-traversal-poc") })
}

// TestLocalExecutor_Build_TomeArchive_NestedPathsExtractNormally verifies that
// legitimate nested asset paths within a tome archive still extract correctly
// after path-traversal validation was added.
func TestLocalExecutor_Build_TomeArchive_NestedPathsExtractNormally(t *testing.T) {
	exec := executor.NewLocalExecutor()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	archive := buildTarGz(t, map[string]string{
		"main.eldritch":         "print('hi')",
		"assets/nested/foo.txt": "foo contents",
	})

	spec := executor.BuildSpec{
		TaskID:      2,
		TargetOS:    "linux",
		BuildScript: `test -f "$REALM_TOMES_DIR/1/main.eldritch" && test -f "$REALM_TOMES_DIR/1/assets/nested/foo.txt"`,
		Tomes: []executor.TomeData{
			{ID: 1, Name: "good", Contents: archive},
		},
	}

	outputCh := make(chan string, 100)
	errorCh := make(chan string, 100)

	result, err := exec.Build(ctx, spec, outputCh, errorCh)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, int64(0), result.ExitCode)
}
