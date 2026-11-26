package secrets

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

const DEFAULT_PERMS = 0600
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
	found := false

	// If the value exists update it
	for idx, k := range secrets {
		if k.Key == key {
			secrets[idx].Value = string(value)
			old_value = []byte(k.Value)
			found = true
			break
		}
	}

	// If the value doesn't exist create it
	if !found {
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

	// Write atomically via a 0600 temp file and rename so shrinking never leaves trailing bytes
	// and permissions are always restricted to the owner.
	dir := filepath.Dir(path)
	tmpFile, err := os.CreateTemp(dir, ".secrets-*.tmp")
	if err != nil {
		log.Error("failed to create temp secrets file", "path", path, "error", err)
		return err
	}
	tmpPath := tmpFile.Name()
	cleanup := true
	defer func() {
		if cleanup {
			_ = os.Remove(tmpPath)
		}
	}()

	if err := tmpFile.Chmod(DEFAULT_PERMS); err != nil {
		_ = tmpFile.Close()
		log.Error("failed to set secrets file permissions", "path", path, "error", err)
		return err
	}

	if _, err := tmpFile.Write(data); err != nil {
		_ = tmpFile.Close()
		log.Error("failed to write secrets file", "path", path, "error", err)
		return err
	}

	if err := tmpFile.Close(); err != nil {
		log.Error("failed to close temp secrets file", "path", path, "error", err)
		return err
	}

	if err := os.Rename(tmpPath, path); err != nil {
		log.Error("failed to replace secrets file", "path", path, "error", err)
		return err
	}

	cleanup = false
	return nil
}

func (s DebugFileSecrets) getYamlStruct(path string) (Secrets, error) {
	file, err := os.Open(path)
	if err != nil {
		if !errors.Is(err, os.ErrNotExist) {
			log.Error("failed to open secrets file", "path", path, "error", err)
		}
		return Secrets{}, err
	}
	defer file.Close()

	data, err := io.ReadAll(io.LimitReader(file, MAX_FILE_SIZE+1))
	if err != nil {
		log.Error("failed to read secrets file", "path", path, "error", err)
		return Secrets{}, err
	}
	if len(data) > MAX_FILE_SIZE {
		err := fmt.Errorf("secrets file exceeds maximum size of %d bytes", MAX_FILE_SIZE)
		log.Error("secrets file exceeds maximum size", "path", path, "max_size", MAX_FILE_SIZE)
		return Secrets{}, err
	}

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
	if err == nil {
		return s.Path, nil
	}
	if !errors.Is(err, os.ErrNotExist) {
		log.Error("failed to stat secrets file", "path", s.Path, "error", err)
		return s.Path, err
	}

	if err := s.setYamlStruct(s.Path, Secrets{}); err != nil {
		log.Error("failed to initialize secrets YAML file", "path", s.Path, "error", err)
		return s.Path, err
	}
	return s.Path, nil
}
