package secrets

import "realm.pub/tavern/internal/logging"

var log = logging.For("storage.secrets")

type SecretsManager interface {
	GetName() string
	SetValue(string, []byte) ([]byte, error)
	GetValue(string) ([]byte, error)
}
