package secrets_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/secrets"
)

func TestFormatSecretPrefix(t *testing.T) {
	assert.Equal(t, "tavern-REALM", secrets.FormatSecretPrefix("tavern"))
	assert.Equal(t, "abcxyz-REALM", secrets.FormatSecretPrefix("abcxyz"))
}

// No way to test GCP Secret Manager API in CI without credentials, but worked in dev

// import (
// 	"testing"

// 	"github.com/stretchr/testify/assert"
// 	"realm.pub/tavern/internal/secrets"
// )

// func TestSetSecretsGcp(t *testing.T) {
// 	test_key := "super_secret_test_data"
// 	// Create manager
// 	secretsManager, err := secrets.NewGcp("")
// 	assert.NotNil(t, secretsManager)
// 	assert.Nil(t, err)
// 	// Get a non existent value
// 	res, err := secretsManager.GetValue(test_key)
// 	assert.NotNil(t, err)
// 	assert.Equal(t, []byte(""), res)

// 	// Create a value
// 	res, err = secretsManager.SetValue(test_key, []byte("This should work"))
// 	assert.Nil(t, err)
// 	assert.Equal(t, []byte(""), res)

// 	// Update the value
// 	res, err = secretsManager.SetValue(test_key, []byte{0x99, 0x99})
// 	assert.Nil(t, err)
// 	assert.Equal(t, []byte("This should work"), res)

// 	// Get the value
// 	res, err = secretsManager.GetValue(test_key)
// 	assert.Nil(t, err)
// 	assert.Equal(t, []byte{0x99, 0x99}, res)
// }
