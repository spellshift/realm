package crypto_test

import (
	"crypto/ed25519"
	"errors"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/crypto"
	"realm.pub/tavern/internal/secrets"
)

func TestGetPrivKeyED25519(t *testing.T) {
	// newSecretsManager returns a file secrets manager backed by a secrets file in a temp dir.
	// If content is nil the file is not created.
	newSecretsManager := func(t *testing.T, content []byte) secrets.SecretsManager {
		t.Helper()
		path := filepath.Join(t.TempDir(), "secrets.yaml")
		if content != nil {
			require.NoError(t, os.WriteFile(path, content, 0644))
		}
		sm, err := secrets.NewDebugFileSecrets(path)
		require.NoError(t, err)
		return sm
	}

	// assertGeneratesAndPersistsKey checks that a valid key is generated, and that the same key
	// is returned on the next call (i.e. it was stored).
	assertGeneratesAndPersistsKey := func(t *testing.T, sm secrets.SecretsManager) {
		t.Helper()
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

	t.Run("NoFile", func(t *testing.T) {
		assertGeneratesAndPersistsKey(t, newSecretsManager(t, nil))
	})

	t.Run("EmptyFile", func(t *testing.T) {
		assertGeneratesAndPersistsKey(t, newSecretsManager(t, []byte{}))
	})

	t.Run("FileWithOtherKeysOnly", func(t *testing.T) {
		sm := newSecretsManager(t, []byte(`
- key: super_secret_test_data
  value: hello world!
`))
		assertGeneratesAndPersistsKey(t, sm)

		// The other keys in the file are kept.
		val, err := sm.GetValue("super_secret_test_data")
		require.NoError(t, err)
		assert.Equal(t, []byte("hello world!"), val)
	})

	t.Run("OtherErrorDoesNotGenerateKey", func(t *testing.T) {
		backendErr := errors.New("backend unavailable")
		sm := &failingSecretsManager{err: backendErr}

		key, err := crypto.GetPrivKeyED25519(sm)
		require.Error(t, err)
		assert.ErrorIs(t, err, backendErr)
		assert.Nil(t, key)
		assert.Zero(t, sm.setCalls, "SetValue must not be called, a new key would replace the server key")
	})

	t.Run("ExistingKeyIsReused", func(t *testing.T) {
		_, want, err := ed25519.GenerateKey(nil)
		require.NoError(t, err)

		sm := newSecretsManager(t, nil)
		_, err = sm.SetValue("tavern_ed25519_private_key", want)
		require.NoError(t, err)

		got, err := crypto.GetPrivKeyED25519(sm)
		require.NoError(t, err)
		assert.Equal(t, want, got)
	})
}

// failingSecretsManager returns err from GetValue and counts SetValue calls.
type failingSecretsManager struct {
	err      error
	setCalls int
}

func (f *failingSecretsManager) GetName() string { return "failing" }

func (f *failingSecretsManager) GetValue(string) ([]byte, error) { return []byte{}, f.err }

func (f *failingSecretsManager) SetValue(string, []byte) ([]byte, error) {
	f.setCalls++
	return []byte{}, nil
}
