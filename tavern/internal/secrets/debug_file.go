package secrets

import (
	"errors"
	"fmt"
	"io"
	"log"
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
		log.Printf("[ERROR] Failed to create secrets file %s: %v", path, err)
		return []byte{}, err
	}

	secrets, err := s.getYamlStruct(path)
	if err != nil {
		log.Printf("[ERROR] Failed to parse YAML file %s: %v", path, err)
		return []byte{}, err
	}

	var old_value []byte
	var found bool

	// If the value exists update it
	for i, s := range *secrets {
		if s.Key == key {
			old_value = []byte(s.Value)
			(*secrets)[i].Value = string(value)
			found = true
			break
		}
	}

	// If the value doesn't exist create it
	if !found {
		*secrets = append(*secrets, Secret{Key: key, Value: string(value)})
	}

	err = s.setYamlStruct(path, secrets)
	if err != nil {
		log.Printf("[ERROR] Failed to update YAML file %s: %v", path, err)
		return []byte{}, err
	}

	if old_value == nil {
		return []byte{}, nil
	}
	return old_value, nil
}

func (s DebugFileSecrets) GetValue(key string) ([]byte, error) {
	path := s.Path

	secrets, err := s.getYamlStruct(path)
	if err != nil {
		log.Printf("[ERROR] Failed to parse YAML file %s: %v", path, err)
		return []byte{}, err
	}

	for _, k := range *secrets {
		if k.Key == key {
			return []byte(k.Value), nil
		}
	}

	return []byte{}, nil
}

func (s DebugFileSecrets) setYamlStruct(path string, secrets *Secrets) error {
	data, err := yaml.Marshal(secrets)
	if err != nil {
		fmt.Printf("[ERROR] Failed to parse file YAML %s: %v", path, err)
		return err
	}

	file, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE|os.O_TRUNC, DEFAULT_PERMS)
	if err != nil {
		log.Printf("[ERROR] Failed to open secrets file %s: %v", path, err)
		return err
	}
	defer file.Close()

	_, err = file.Write(data)
	if err != nil {
		log.Printf("[ERROR] Failed to read file %s: %v", path, err)
		return err
	}

	return nil
}

func (s DebugFileSecrets) getYamlStruct(path string) (*Secrets, error) {
	file, err := os.OpenFile(path, os.O_RDWR, DEFAULT_PERMS)
	if err != nil {
		log.Printf("[ERROR] Failed to open secrets file %s: %v", path, err)
		return nil, err
	}
	defer file.Close()

	data, err := io.ReadAll(file)
	if err != nil {
		log.Printf("[ERROR] Failed to read file %s: %v", path, err)
		return nil, err
	}

	var secrets Secrets
	err = yaml.Unmarshal(data, &secrets)
	if err != nil {
		fmt.Printf("[ERROR] Failed to parse file YAML %s: %v", path, err)
		return nil, err
	}

	return &secrets, nil
}

func (s DebugFileSecrets) ensureSecretsFileExist() (string, error) {
	_, err := os.Stat(s.Path)
	if errors.Is(err, os.ErrNotExist) {
		// Create file
		f, err := os.OpenFile(s.Path, os.O_CREATE, DEFAULT_PERMS)
		if err != nil {
			log.Printf("[ERROR] Failed to create file %s\n", s.Path)
			return s.Path, err
		}
		defer f.Close()

		// Write empty struct to file
		err = s.setYamlStruct(s.Path, &Secrets{})
		if err != nil {
			log.Printf("[ERROR] Failed to set yaml struct")
			return s.Path, err
		}
	}
	return s.Path, nil
}
