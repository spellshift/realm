package secrets_test

import (
	"log"
	"os"
	"path"
	"testing"

	"github.com/stretchr/testify/assert"
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
