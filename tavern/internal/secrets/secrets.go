package secrets

type SecretsManager interface {
	GetName() string
	SetValue(string, []byte) ([]byte, error)
	GetValue(string) ([]byte, error)
}
