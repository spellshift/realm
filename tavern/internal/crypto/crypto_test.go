package crypto_test

import (
	"crypto/ed25519"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/crypto"
	"realm.pub/tavern/internal/secrets"
)

func TestGetPrivKeyED25519(t *testing.T) {
	// newSecretsFile returns a path to a secrets file in a temp dir. If content is nil the file
	// is not created.
	newSecretsFile := func(t *testing.T, content []byte) string {
		t.Helper()
		path := filepath.Join(t.TempDir(), "secrets.yaml")
		if content != nil {
			require.NoError(t, os.WriteFile(path, content, 0644))
		}
		return path
	}

	// assertGeneratesAndPersistsKey checks that a valid key is generated for the given secrets
	// file, and that the same key is returned on the next call (i.e. it was stored).
	assertGeneratesAndPersistsKey := func(path string) func(*testing.T) {
		return func(t *testing.T) {
			sm, err := secrets.NewDebugFileSecrets(path)
			require.NoError(t, err)

			first, err := crypto.GetPrivKeyED25519(sm)
			require.NoError(t, err)
			assert.Len(t, first, ed25519.PrivateKeySize)

			second, err := crypto.GetPrivKeyED25519(sm)
			require.NoError(t, err)
			assert.Equal(t, first, second)

			stored, err := sm.GetValue("tavern_ed25519_private_key")
			require.NoError(t, err)
			assert.Equal(t, []byte(first), stored)
		}
	}

	t.Run("NoFile", assertGeneratesAndPersistsKey(newSecretsFile(t, nil)))
	t.Run("EmptyFile", assertGeneratesAndPersistsKey(newSecretsFile(t, []byte{})))
	t.Run("FileWithOtherKeysOnly", assertGeneratesAndPersistsKey(newSecretsFile(t, []byte(`
- key: super_secret_test_data
  value: hello world!
`))))

	t.Run("FileWithOtherKeysOnly/PreservesOtherKeys", func(t *testing.T) {
		path := newSecretsFile(t, []byte(`
- key: super_secret_test_data
  value: hello world!
`))
		sm, err := secrets.NewDebugFileSecrets(path)
		require.NoError(t, err)

		_, err = crypto.GetPrivKeyED25519(sm)
		require.NoError(t, err)

		val, err := sm.GetValue("super_secret_test_data")
		require.NoError(t, err)
		assert.Equal(t, []byte("hello world!"), val)
	})

	t.Run("ExistingKeyIsReused", func(t *testing.T) {
		_, want, err := ed25519.GenerateKey(nil)
		require.NoError(t, err)

		sm, err := secrets.NewDebugFileSecrets(newSecretsFile(t, nil))
		require.NoError(t, err)
		_, err = sm.SetValue("tavern_ed25519_private_key", want)
		require.NoError(t, err)

		got, err := crypto.GetPrivKeyED25519(sm)
		require.NoError(t, err)
		assert.Equal(t, want, got)
	})
}
