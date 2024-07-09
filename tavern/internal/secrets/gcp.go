package secrets

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"strings"

	secretmanager "cloud.google.com/go/secretmanager/apiv1"
	"cloud.google.com/go/secretmanager/apiv1/secretmanagerpb"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/compute/v1"
)

type Gcp struct {
	Name      string
	projectID string
	prefix    string
	client    *secretmanager.Client
	clientctx context.Context
}

// GetName implements SecretsManager.
func (g Gcp) GetName() string {
	return g.Name
}

// GetValue implements SecretsManager.
func (g Gcp) GetValue(key string) ([]byte, error) {
	// name := "projects/my-project/secrets/my-secret"
	name := fmt.Sprintf("projects/%s/secrets/%s_%s/versions/latest", g.projectID, g.prefix, key)

	// Build the request.
	accessRequest := &secretmanagerpb.AccessSecretVersionRequest{
		Name: name,
	}

	// Call the API.
	result, err := g.client.AccessSecretVersion(g.clientctx, accessRequest)
	if err != nil {
		log.Printf("[ERROR] failed to access secret version: %v\n", err)
		return []byte{}, err
	}

	return result.Payload.Data, nil
}

type credentialsJson struct {
	ProjectID string `json:"quota_project_id"`
}

func GetCurrentGcpProject(ctx context.Context) (string, error) {
	respMesg, err := google.FindDefaultCredentials(ctx, compute.ComputeScope)
	if err != nil {
		return "", err
	}

	if respMesg.ProjectID != "" {
		return respMesg.ProjectID, nil
	}

	// respMesg.ProjectID can be empty so instead we grab from the creds JSON file
	credJSON := credentialsJson{}
	err = json.Unmarshal(respMesg.JSON, &credJSON)
	if err != nil {
		return "", err
	}
	ProjectID := credJSON.ProjectID

	if ProjectID == "" {
		return "", errors.New("project id is empty")
	}

	return ProjectID, nil
}

// SetValue implements SecretsManager.
func (g Gcp) SetValue(key string, value []byte) ([]byte, error) {
	// Create the request to create the secret.
	parent := fmt.Sprintf("projects/%s", g.projectID)
	// createSecretReq := secretmanagerpb.CreateSecretRequest{
	// 	Parent:   parent,
	// 	SecretId: fmt.Sprintf("%s_%s", g.prefix, key),
	// 	Secret: &secretmanagerpb.Secret{
	// 		Replication: &secretmanagerpb.Replication{
	// 			Replication: &secretmanagerpb.Replication_Automatic_{
	// 				Automatic: &secretmanagerpb.Replication_Automatic{},
	// 			},
	// 		},
	// 	},
	// }

	// _, err := g.client.CreateSecret(g.clientctx, &createSecretReq)
	// if err != nil {
	// 	if !strings.Contains(err.Error(), "code = AlreadyExists") {
	// 		log.Printf("[ERROR] Failed to create secret: %v\n", err)
	// 		return []byte{}, err
	// 	} else {
	// }

	old_value, err := g.GetValue(key)
	if err != nil && !strings.Contains(err.Error(), "code = NotFound") {
		log.Printf("[ERROR] Failed to get old secret: %v\n", err)
		return []byte{}, err
	}

	// Declare the payload to store.
	path := fmt.Sprintf("%s/secrets/%s_%s", parent, g.prefix, key)
	payload := []byte(value)

	// Build the request.
	addSecretVersionReq := &secretmanagerpb.AddSecretVersionRequest{
		Parent: path,
		Payload: &secretmanagerpb.SecretPayload{
			Data: payload,
		},
	}

	// Call the API.
	_, err = g.client.AddSecretVersion(g.clientctx, addSecretVersionReq)
	if err != nil {
		log.Fatalf("failed to add secret version: %v", err)
	}

	return old_value, nil
}

func NewGcp(projectID string) (SecretsManager, error) {
	// GCP project in which to store secrets in Secret Manager.
	ctx := context.Background()

	// If unset try to figure out the current GCP
	if projectID == "" {
		tmp, err := GetCurrentGcpProject(ctx)
		projectID = tmp
		if err != nil {
			log.Printf("[ERROR] Failed to get current project ID: %v\n", err)
			return nil, err
		}
	}
	log.Printf("[DEBUG] Using projectID: %s\n", projectID)

	// Create the client.
	client, err := secretmanager.NewClient(ctx)
	if err != nil {
		log.Printf("[ERROR] Failed to setup client: %v\n", err)
		return nil, err
	}

	return Gcp{
		Name:      "Gcp",
		projectID: projectID,
		prefix:    "REALM",
		client:    client,
		clientctx: ctx,
	}, nil
}
