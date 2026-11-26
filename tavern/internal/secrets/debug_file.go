package secrets

import (
	"errors"
	"fmt"
	"io"
	"os"

	"gopkg.in/yaml.v3"
)

const DEFAULT_PERMS = 0644
const DELIMITER = "="
const MEGABYTES = 1000000
const MAX_FILE_SIZE = 128 * MEGABYTES

type Secret struct {
	Key   string
	Value string
}

type Secrets []Secret

type DebugFileSecrets struct {
	Name string
	Path string
}

func NewDebugFileSecrets(path string) (SecretsManager, error) {
	return DebugFileSecrets{
		Name: "DebugFileSecrets",
		Path: path,
	}, nil
}

func (s DebugFileSecrets) GetName() string {
	return s.Name
}

func (s DebugFileSecrets) SetValue(key string, value []byte) ([]byte, error) {
	path, err := s.ensureSecretsFileExist()
	if err != nil {
		log.Error("failed to create secrets file", "path", path, "key", key, "error", err)
		return []byte{}, err
	}

	secrets, err := s.getYamlStruct(path)
	if err != nil {
		log.Error("failed to load secrets YAML file", "path", path, "key", key, "error", err)
		return []byte{}, err
	}

	var old_value []byte = []byte{}

	// If the value exists update it
	for idx, k := range secrets {
		if k.Key == key {
			secrets[idx].Value = string(value)
			old_value = []byte(k.Value)
		}
	}

	// If the value doesn't exist create it
	if len(old_value) == 0 {
		secrets = append(
			secrets,
			Secret{
				Key:   key,
				Value: string(value),
			},
		)
	}

	err = s.setYamlStruct(path, secrets)
	if err != nil {
		log.Error("failed to update secrets YAML file", "path", path, "key", key, "error", err)
		return []byte{}, err
	}

	return old_value, nil
}

func (s DebugFileSecrets) GetValue(key string) ([]byte, error) {
	path := s.Path

	secrets, err := s.getYamlStruct(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			log.Debug("secrets YAML file does not exist yet", "path", path, "key", key)
			return []byte{}, fmt.Errorf("%w: %w", ErrSecretNotFound, err)
		}
		log.Error("failed to load secrets YAML file", "path", path, "key", key, "error", err)
		return []byte{}, err
	}

	for _, k := range secrets {
		if k.Key == key {
			return []byte(k.Value), nil
		}
	}

	log.Debug("secret not found in secrets YAML file", "path", path, "key", key)
	return []byte{}, ErrSecretNotFound
}

func (s DebugFileSecrets) setYamlStruct(path string, secrets Secrets) error {
	data, err := yaml.Marshal(secrets)
	if err != nil {
		log.Error("failed to marshal secrets YAML", "path", path, "error", err)
		return err
	}

	file, err := os.OpenFile(path, os.O_RDWR, DEFAULT_PERMS)
	if err != nil {
		log.Error("failed to open secrets file", "path", path, "error", err)
		return err
	}
	defer file.Close()

	_, err = file.Write(data)
	if err != nil {
		log.Error("failed to write secrets file", "path", path, "error", err)
		return err
	}

	return nil
}

func (s DebugFileSecrets) getYamlStruct(path string) (Secrets, error) {
	file, err := os.OpenFile(path, os.O_RDWR, DEFAULT_PERMS)
	if err != nil {
		if !errors.Is(err, os.ErrNotExist) {
			log.Error("failed to open secrets file", "path", path, "error", err)
		}
		return Secrets{}, err
	}
	defer file.Close()

	data := make([]byte, MAX_FILE_SIZE)
	n, err := file.Read(data)
	if err != nil && !errors.Is(err, io.EOF) {
		log.Error("failed to read secrets file", "path", path, "error", err)
		return Secrets{}, err
	}
	// An empty (e.g. freshly created or `touch`-ed) secrets file reads as (0, io.EOF), not an
	// error: treat it the same as a file with no secrets in it yet.

	data = data[0:n]

	var secrets Secrets
	err = yaml.Unmarshal(data, &secrets)
	if err != nil {
		log.Error("failed to unmarshal secrets YAML", "path", path, "error", err)
		return Secrets{}, err
	}

	return secrets, nil
}

func (s DebugFileSecrets) ensureSecretsFileExist() (string, error) {
	_, err := os.Stat(s.Path)
	if errors.Is(err, os.ErrNotExist) {
		// Create file
		f, err := os.OpenFile(s.Path, os.O_CREATE, DEFAULT_PERMS)
		if err != nil {
			log.Error("failed to create secrets file", "path", s.Path, "error", err)
			return s.Path, err
		}
		defer f.Close()

		// Write empty struct to file
		err = s.setYamlStruct(s.Path, Secrets{})
		if err != nil {
			log.Error("failed to initialize secrets YAML file", "path", s.Path, "error", err)
			return s.Path, err
		}
	}
	return s.Path, nil
}
