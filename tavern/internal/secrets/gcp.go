package secrets

import (
	"context"
	"fmt"
	"log"

	secretmanager "cloud.google.com/go/secretmanager/apiv1"
	"cloud.google.com/go/secretmanager/apiv1/secretmanagerpb"
)

type Gcp struct {
	Name      string
	projectID string
}

// GetName implements SecretsManager.
func (g Gcp) GetName() string {
	return g.Name
}

// GetValue implements SecretsManager.
func (g Gcp) GetValue(string) (string, error) {
	panic("unimplemented")
}

// SetValue implements SecretsManager.
func (g Gcp) SetValue(key string, value string) (string, error) {
	// GCP project in which to store secrets in Secret Manager.
	projectID := "your-project-id"

	// Create the client.
	ctx := context.Background()
	client, err := secretmanager.NewClient(ctx)
	if err != nil {
		log.Fatalf("failed to setup client: %v", err)
	}
	defer client.Close()

	// Create the request to create the secret.
	createSecretReq := &secretmanagerpb.CreateSecretRequest{
		Parent:   fmt.Sprintf("projects/%s", projectID),
		SecretId: "my-secret",
		Secret: &secretmanagerpb.Secret{
			Replication: &secretmanagerpb.Replication{
				Replication: &secretmanagerpb.Replication_Automatic_{
					Automatic: &secretmanagerpb.Replication_Automatic{},
				},
			},
		},
	}

	secret, err := client.CreateSecret(ctx, createSecretReq)
	if err != nil {
		log.Fatalf("failed to create secret: %v", err)
	}

	// Declare the payload to store.
	payload := []byte("my super secret data")

	// Build the request.
	addSecretVersionReq := &secretmanagerpb.AddSecretVersionRequest{
		Parent: secret.Name,
		Payload: &secretmanagerpb.SecretPayload{
			Data: payload,
		},
	}

	// Call the API.
	version, err := client.AddSecretVersion(ctx, addSecretVersionReq)
	if err != nil {
		log.Fatalf("failed to add secret version: %v", err)
	}

}

func NewGcp() SecretsManager {
	return Gcp{
		Name: "Gcp",
	}
}
