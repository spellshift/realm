package secrets

import (
	"errors"

	"realm.pub/tavern/internal/logging"
)

var log = logging.For("storage.secrets")

// ErrSecretNotFound is returned (possibly wrapped) by GetValue when the requested
// key does not exist in the secrets store. Check it with errors.Is.
var ErrSecretNotFound = errors.New("secret not found")

type SecretsManager interface {
	GetName() string
	SetValue(string, []byte) ([]byte, error)
	GetValue(string) ([]byte, error)
}
