package secrets

import (
	"errors"

	"realm.pub/tavern/internal/logging"
)

var log = logging.For("storage.secrets")

// ErrSecretNotFound is returned by GetValue when the secrets store is readable
// but does not contain the requested key.
var ErrSecretNotFound = errors.New("secret not found")

type SecretsManager interface {
	GetName() string
	SetValue(string, []byte) ([]byte, error)
	GetValue(string) ([]byte, error)
}
