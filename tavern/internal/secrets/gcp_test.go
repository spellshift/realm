package secrets_test

// No way to test this in CI but worked in dev

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
// 	assert.Equal(t, "", res)

// 	// Create a value
// 	res, err = secretsManager.SetValue(test_key, "This should work")
// 	assert.Nil(t, err)
// 	assert.Equal(t, "", res)

// 	// Update the value
// 	res, err = secretsManager.SetValue(test_key, "This should work too")
// 	assert.Nil(t, err)
// 	assert.Equal(t, "This should work", res)

// 	// Get the value
// 	res, err = secretsManager.GetValue(test_key)
// 	assert.Nil(t, err)
// 	assert.Equal(t, "This should work too", res)
// }
