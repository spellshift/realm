package secrets

type SecretsManager interface {
	GetName() string
	SetValue(string, string) (string, error)
	GetValue(string) (string, error)
}
