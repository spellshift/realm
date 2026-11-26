package secrets_test

import (
	"log"
	"os"
	"path"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/secrets"
)

func createTestSecrets(t *testing.T) string {
	// TODO: How do I clean up this test file 🫠
	tmpDir := t.TempDir()
	secretsPath := path.Join(tmpDir, "secrets.yaml")
	data := []byte(`
- key: TAVERN_PRIVATE_KEY
  value: !!binary Fe8E/SVf7MLgCHzBWvjdnavejsJijJZD9mPvLQbgybE=
- key: super_secret_test_data
  value: hello world!
`)
	err := os.WriteFile(secretsPath, data, 0644)
	if err != nil {
		log.Fatalf("Failed to write test file %s: %v", secretsPath, err)
	}

	return secretsPath
}

func TestGetSecretsFile(t *testing.T) {
	path := createTestSecrets(t)

	secretsManager, err := secrets.NewDebugFileSecrets(path)
	assert.Nil(t, err)
	res, err := secretsManager.GetValue("super_secret_test_data")
	assert.Nil(t, err)
	assert.Equal(t, []byte("hello world!"), res)

	res, err = secretsManager.GetValue("TAVERN_PRIVATE_KEY")
	assert.Nil(t, err)
	assert.Equal(t, []byte{0x15, 0xef, 0x04, 0xfd, 0x25, 0x5f, 0xec, 0xc2, 0xe0, 0x08, 0x7c, 0xc1, 0x5a, 0xf8, 0xdd, 0x9d, 0xab, 0xde, 0x8e, 0xc2, 0x62, 0x8c, 0x96, 0x43, 0xf6, 0x63, 0xef, 0x2d, 0x06, 0xe0, 0xc9, 0xb1}, res)
}

func TestSetSecretsAddFile(t *testing.T) {
	path := createTestSecrets(t)

	secretsManager, err := secrets.NewDebugFileSecrets(path)
	assert.Nil(t, err)
	res, err := secretsManager.SetValue("WOAH!", []byte("This works!"))
	assert.Nil(t, err)
	assert.Equal(t, []byte{}, res)

	res, err = secretsManager.GetValue("WOAH!")
	assert.Nil(t, err)
	assert.Equal(t, []byte("This works!"), res)

	// ---

	res, err = secretsManager.SetValue("WOAH!", []byte("This works too!"))
	assert.Nil(t, err)
	assert.Equal(t, []byte("This works!"), res)

	res, err = secretsManager.GetValue("WOAH!")
	assert.Nil(t, err)
	assert.Equal(t, []byte("This works too!"), res)

	// ---

	res, err = secretsManager.GetValue("TAVERN_PRIVATE_KEY")
	assert.Nil(t, err)
	assert.Equal(t, []byte{0x15, 0xef, 0x04, 0xfd, 0x25, 0x5f, 0xec, 0xc2, 0xe0, 0x08, 0x7c, 0xc1, 0x5a, 0xf8, 0xdd, 0x9d, 0xab, 0xde, 0x8e, 0xc2, 0x62, 0x8c, 0x96, 0x43, 0xf6, 0x63, 0xef, 0x2d, 0x06, 0xe0, 0xc9, 0xb1}, res)
}

func TestSetSecretsNewFile(t *testing.T) {
	tmpDir := t.TempDir()
	path := path.Join(tmpDir, "secrets.yaml")

	secretsManager, err := secrets.NewDebugFileSecrets(path)
	assert.Nil(t, err)
	res, err := secretsManager.GetValue("super_secret_test_data")
	// Make sure this fails
	assert.NotNil(t, err)
	assert.Equal(t, []byte{}, res)

	res, err = secretsManager.SetValue("super_secret_test_data", []byte{0x99, 0x99})
	assert.Nil(t, err)
	assert.Equal(t, []byte{}, res)

	res, err = secretsManager.GetValue("super_secret_test_data")
	assert.Nil(t, err)
	assert.Equal(t, []byte{0x99, 0x99}, res)
}

func TestGetSecretsMissingKey(t *testing.T) {
	path := createTestSecrets(t)

	secretsManager, err := secrets.NewDebugFileSecrets(path)
	require.NoError(t, err)

	res, err := secretsManager.GetValue("does_not_exist")
	require.Error(t, err)
	assert.ErrorIs(t, err, secrets.ErrSecretNotFound)
	assert.Equal(t, []byte{}, res)
}

func TestGetSecretsEmptyFile(t *testing.T) {
	tmpDir := t.TempDir()
	path := path.Join(tmpDir, "secrets.yaml")
	require.NoError(t, os.WriteFile(path, []byte{}, 0644))

	secretsManager, err := secrets.NewDebugFileSecrets(path)
	require.NoError(t, err)

	res, err := secretsManager.GetValue("super_secret_test_data")
	require.Error(t, err)
	assert.ErrorIs(t, err, secrets.ErrSecretNotFound)
	assert.Equal(t, []byte{}, res)

	old, err := secretsManager.SetValue("super_secret_test_data", []byte("hello world!"))
	require.NoError(t, err)
	assert.Equal(t, []byte{}, old)

	res, err = secretsManager.GetValue("super_secret_test_data")
	require.NoError(t, err)
	assert.Equal(t, []byte("hello world!"), res)
}

func TestGetSecretsNoFile(t *testing.T) {
	path := path.Join(t.TempDir(), "secrets.yaml")

	secretsManager, err := secrets.NewDebugFileSecrets(path)
	require.NoError(t, err)

	res, err := secretsManager.GetValue("super_secret_test_data")
	require.Error(t, err)
	assert.ErrorIs(t, err, secrets.ErrSecretNotFound)
	assert.ErrorIs(t, err, os.ErrNotExist)
	assert.Equal(t, []byte{}, res)
}

func TestDebugFileSecrets_Permissions0600(t *testing.T) {
	t.Run("new file created with 0600", func(t *testing.T) {
		filePath := path.Join(t.TempDir(), "secrets.yaml")
		sm, err := secrets.NewDebugFileSecrets(filePath)
		require.NoError(t, err)

		_, err = sm.SetValue("privkey", []byte("topsecret"))
		require.NoError(t, err)

		info, err := os.Stat(filePath)
		require.NoError(t, err)
		assert.Equal(t, os.FileMode(0600), info.Mode().Perm())
	})

	t.Run("existing loose permissions tightened to 0600 on write", func(t *testing.T) {
		filePath := createTestSecrets(t) // created with 0644
		sm, err := secrets.NewDebugFileSecrets(filePath)
		require.NoError(t, err)

		_, err = sm.SetValue("super_secret_test_data", []byte("updated"))
		require.NoError(t, err)

		info, err := os.Stat(filePath)
		require.NoError(t, err)
		assert.Equal(t, os.FileMode(0600), info.Mode().Perm())
	})
}

func TestDebugFileSecrets_ShrinkDoesNotCorruptYAML(t *testing.T) {
	filePath := path.Join(t.TempDir(), "secrets.yaml")
	sm, err := secrets.NewDebugFileSecrets(filePath)
	require.NoError(t, err)

	longVal := make([]byte, 2048)
	for i := range longVal {
		longVal[i] = 'A'
	}

	_, err = sm.SetValue("k1", longVal)
	require.NoError(t, err)
	_, err = sm.SetValue("k2", []byte("keep-me"))
	require.NoError(t, err)

	// Shrink k1 dramatically; previously missing O_TRUNC left trailing YAML garbage.
	oldVal, err := sm.SetValue("k1", []byte("short"))
	require.NoError(t, err)
	assert.Equal(t, longVal, oldVal)

	v1, err := sm.GetValue("k1")
	require.NoError(t, err)
	assert.Equal(t, []byte("short"), v1)

	v2, err := sm.GetValue("k2")
	require.NoError(t, err)
	assert.Equal(t, []byte("keep-me"), v2)
}

func TestDebugFileSecrets_UpdateEmptyValueNoDuplicate(t *testing.T) {
	filePath := path.Join(t.TempDir(), "secrets.yaml")
	sm, err := secrets.NewDebugFileSecrets(filePath)
	require.NoError(t, err)

	_, err = sm.SetValue("empty_key", []byte{})
	require.NoError(t, err)

	_, err = sm.SetValue("empty_key", []byte("now-populated"))
	require.NoError(t, err)

	val, err := sm.GetValue("empty_key")
	require.NoError(t, err)
	assert.Equal(t, []byte("now-populated"), val)

	raw, err := os.ReadFile(filePath)
	require.NoError(t, err)
	assert.Equal(t, 1, countSubstring(string(raw), "empty_key"))
}

func TestDebugFileSecrets_ReadOnlyFile(t *testing.T) {
	filePath := createTestSecrets(t)
	require.NoError(t, os.Chmod(filePath, 0400))

	sm, err := secrets.NewDebugFileSecrets(filePath)
	require.NoError(t, err)

	val, err := sm.GetValue("super_secret_test_data")
	require.NoError(t, err)
	assert.Equal(t, []byte("hello world!"), val)
}

func countSubstring(s, sub string) int {
	count := 0
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			count++
		}
	}
	return count
}
