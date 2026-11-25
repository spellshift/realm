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
const DEFAULT_DIR_PERMS = 0700
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

func validateSecretsFileInfo(path string, info os.FileInfo) error {
	if !info.Mode().IsRegular() {
		return fmt.Errorf("secrets file %q must be a regular file (mode %s)", path, info.Mode())
	}
	if info.Mode().Perm()&0077 != 0 {
		return fmt.Errorf("secrets file %q has insecure permissions %04o (must not be group/world-accessible; expected 0600)", path, info.Mode().Perm())
	}
	return validateSecretsFileOwner(path, info)
}

func validateExistingSecretsFile(path string) error {
	info, err := os.Lstat(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	if err != nil {
		return err
	}
	return validateSecretsFileInfo(path, info)
}

func (s DebugFileSecrets) setYamlStruct(path string, secrets Secrets) error {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, DEFAULT_DIR_PERMS); err != nil {
		log.Error("failed to create secrets parent directory", "path", path, "dir", dir, "error", err)
		return err
	}

	if err := validateExistingSecretsFile(path); err != nil {
		log.Error("existing secrets file failed security check", "path", path, "error", err)
		return err
	}

	data, err := yaml.Marshal(secrets)
	if err != nil {
		log.Error("failed to marshal secrets YAML", "path", path, "error", err)
		return err
	}
	if len(data) > MAX_FILE_SIZE {
		err := fmt.Errorf("marshaled secrets for %q exceed maximum allowed size of %d bytes", path, MAX_FILE_SIZE)
		log.Error("marshaled secrets too large", "path", path, "error", err)
		return err
	}

	tmp, err := os.CreateTemp(dir, ".tavern-secrets-*")
	if err != nil {
		log.Error("failed to create temporary secrets file", "path", path, "error", err)
		return err
	}
	tmpPath := tmp.Name()
	defer os.Remove(tmpPath)

	if err := tmp.Chmod(DEFAULT_PERMS); err != nil {
		_ = tmp.Close()
		log.Error("failed to set permissions on temporary secrets file", "path", path, "error", err)
		return err
	}
	if _, err := tmp.Write(data); err != nil {
		_ = tmp.Close()
		log.Error("failed to write temporary secrets file", "path", path, "error", err)
		return err
	}
	if err := tmp.Sync(); err != nil {
		_ = tmp.Close()
		log.Error("failed to sync temporary secrets file", "path", path, "error", err)
		return err
	}
	if err := tmp.Close(); err != nil {
		log.Error("failed to close temporary secrets file", "path", path, "error", err)
		return err
	}
	if err := os.Rename(tmpPath, path); err != nil {
		log.Error("failed to replace secrets file", "path", path, "error", err)
		return err
	}

	return nil
}

func (s DebugFileSecrets) getYamlStruct(path string) (Secrets, error) {
	file, err := openSecretsFileReadOnly(path)
	if err != nil {
		if !errors.Is(err, os.ErrNotExist) {
			log.Error("failed to open secrets file", "path", path, "error", err)
		}
		return Secrets{}, err
	}
	defer file.Close()

	info, err := file.Stat()
	if err != nil {
		log.Error("failed to stat secrets file", "path", path, "error", err)
		return Secrets{}, err
	}
	if err := validateSecretsFileInfo(path, info); err != nil {
		log.Error("secrets file failed security check", "path", path, "error", err)
		return Secrets{}, err
	}

	// Read at most MAX_FILE_SIZE+1 bytes so oversized files are rejected without
	// unconditionally pre-allocating MAX_FILE_SIZE. An empty file reads as 0 bytes
	// and unmarshals to an empty Secrets slice.
	data, err := io.ReadAll(io.LimitReader(file, MAX_FILE_SIZE+1))
	if err != nil {
		log.Error("failed to read secrets file", "path", path, "error", err)
		return Secrets{}, err
	}
	if len(data) > MAX_FILE_SIZE {
		err := fmt.Errorf("secrets file %q exceeds maximum allowed size of %d bytes", path, MAX_FILE_SIZE)
		log.Error("secrets file too large", "path", path, "error", err)
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
	dir := filepath.Dir(s.Path)
	if err := os.MkdirAll(dir, DEFAULT_DIR_PERMS); err != nil {
		log.Error("failed to create secrets parent directory", "path", s.Path, "dir", dir, "error", err)
		return s.Path, err
	}

	info, err := os.Lstat(s.Path)
	if errors.Is(err, os.ErrNotExist) {
		if err := s.setYamlStruct(s.Path, Secrets{}); err != nil {
			log.Error("failed to initialize secrets YAML file", "path", s.Path, "error", err)
			return s.Path, err
		}
		return s.Path, nil
	}
	if err != nil {
		log.Error("failed to stat secrets file", "path", s.Path, "error", err)
		return s.Path, err
	}
	if err := validateSecretsFileInfo(s.Path, info); err != nil {
		log.Error("secrets file failed security check", "path", s.Path, "error", err)
		return s.Path, err
	}

	return s.Path, nil
}
