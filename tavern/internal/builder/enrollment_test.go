package builder_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
)

func testEnrollment(t *testing.T) (string, string) {
	t.Helper()
	dir := t.TempDir()
	require.NoError(t, builder.GenerateCSR(dir))
	csr, err := os.ReadFile(filepath.Join(dir, "builder.csr"))
	require.NoError(t, err)
	return string(csr), filepath.Join(dir, "builder.key")
}
