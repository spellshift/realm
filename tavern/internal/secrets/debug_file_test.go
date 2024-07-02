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
secrets:
  - key: TAVERN_PRIVATE_KEY
    value: Fe8E/SVf7MLgCHzBWvjdnavejsJijJZD9mPvLQbgybE=
  - key: super_secret_test_data
    value: hello world!
`)
	err := os.WriteFile(secretsPath, data, 0644)
	if err != nil {
		log.Fatalf("Failed to write test file %s: %v", secretsPath, err)
	}

	return secretsPath
}

func TestGetSecrets(t *testing.T) {
	path := createTestSecrets(t)
	defer os.Remove(path)

	secretsManager := secrets.NewDebugFileSecrets(path)
	res, err := secretsManager.GetValue("super_secret_test_data")
	assert.Nil(t, err)
	assert.Equal(t, "hello world!", res)

	res, err = secretsManager.GetValue("TAVERN_PRIVATE_KEY")
	assert.Nil(t, err)
	assert.Equal(t, "Fe8E/SVf7MLgCHzBWvjdnavejsJijJZD9mPvLQbgybE=", res)
}

func TestSetSecretsAdd(t *testing.T) {
	path := createTestSecrets(t)
	defer os.Remove(path)

	secretsManager := secrets.NewDebugFileSecrets(path)
	res, err := secretsManager.SetValue("WOAH!", "This works!")
	assert.Nil(t, err)
	assert.Equal(t, "", res)

	res, err = secretsManager.GetValue("WOAH!")
	assert.Nil(t, err)
	assert.Equal(t, "This works!", res)

	// ---

	res, err = secretsManager.SetValue("WOAH!", "This works too!")
	assert.Nil(t, err)
	assert.Equal(t, "This works!", res)

	res, err = secretsManager.GetValue("WOAH!")
	assert.Nil(t, err)
	assert.Equal(t, "This works too!", res)

	// ---

	res, err = secretsManager.GetValue("TAVERN_PRIVATE_KEY")
	assert.Nil(t, err)
	assert.Equal(t, "Fe8E/SVf7MLgCHzBWvjdnavejsJijJZD9mPvLQbgybE=", res)
}

func TestSetSecretsNew(t *testing.T) {
	tmpDir := t.TempDir()
	path := path.Join(tmpDir, "secrets.yaml")
	defer os.Remove(path)

	secretsManager := secrets.NewDebugFileSecrets(path)
	res, err := secretsManager.GetValue("super_secret_test_data")
	// Make sure this fails
	assert.NotNil(t, err)

	res, err = secretsManager.SetValue("super_secret_test_data", "now this will work")
	assert.Nil(t, err)
	assert.Equal(t, "", res)

	res, err = secretsManager.GetValue("super_secret_test_data")
	assert.Nil(t, err)
	assert.Equal(t, "now this will work", res)
}
