package secrets

import (
	"errors"
	"fmt"
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

type Secrets struct {
	Secrets []Secret
}

type DebugFileSecrets struct {
	Name string
	Path string
}

func NewDebugFileSecrets(path string) SecretsManager {
	return DebugFileSecrets{
		Name: "DebugFileSecrets",
		Path: path,
	}
}

func (s DebugFileSecrets) GetName() string {
	return s.Name
}

func (s DebugFileSecrets) SetValue(key string, value string) (string, error) {
	path, err := s.createSecretsFile()
	if err != nil {
		log.Printf("[ERROR] Failed to create secrets file %s: %v", path, err)
		return "", err
	}

	secrets, err := s.getYamlStruct(path)
	if err != nil {
		log.Printf("[ERROR] Failed to parse YAML file %s: %v", path, err)
		return "", err
	}

	var old_value string = ""

	// If the value exists update it
	for idx, k := range secrets.Secrets {
		if k.Key == key {
			secrets.Secrets[idx].Value = value
			old_value = k.Value
		}
	}

	// If the value doesn't exist create it
	if old_value == "" {
		secrets.Secrets = append(
			secrets.Secrets,
			Secret{
				Key:   key,
				Value: value,
			},
		)
	}

	err = s.setYamlStruct(path, secrets)
	if err != nil {
		log.Printf("[ERROR] Failed to update YAML file %s: %v", path, err)
		return "", err
	}

	return old_value, nil
}

func (s DebugFileSecrets) GetValue(key string) (string, error) {
	path := s.Path

	secrets, err := s.getYamlStruct(path)
	if err != nil {
		log.Printf("[ERROR] Failed to parse YAML file %s: %v", path, err)
		return "", err
	}

	for _, k := range secrets.Secrets {
		if k.Key == key {
			return k.Value, nil
		}
	}

	return "", nil
}

func (s DebugFileSecrets) setYamlStruct(path string, secrets Secrets) error {
	data, err := yaml.Marshal(secrets)
	if err != nil {
		fmt.Printf("[ERROR] Failed to parse file YAML %s: %v", path, err)
		return err
	}

	file, err := os.OpenFile(path, os.O_RDWR, DEFAULT_PERMS)
	if err != nil {
		log.Printf("[ERROR] Failed to open secrets file %s: %v", path, err)
		return err
	}

	log.Printf("data: %s\n", data)

	_, err = file.Write(data)
	if err != nil {
		log.Printf("[ERROR] Failed to read file %s: %v", path, err)
		return err
	}

	return nil
}

func (s DebugFileSecrets) getYamlStruct(path string) (Secrets, error) {
	file, err := os.OpenFile(path, os.O_RDWR, DEFAULT_PERMS)
	if err != nil {
		log.Printf("[ERROR] Failed to open secrets file %s: %v", path, err)
		return Secrets{}, err
	}

	data := make([]byte, MAX_FILE_SIZE)
	n, err := file.Read(data)
	if err != nil {
		log.Printf("[ERROR] Failed to read file %s: %v", path, err)
		return Secrets{}, err
	}

	data = data[0:n]

	var secrets Secrets
	err = yaml.Unmarshal(data, &secrets)
	if err != nil {
		fmt.Printf("[ERROR] Failed to parse file YAML %s: %v", path, err)
		return Secrets{}, err
	}

	return secrets, nil
}

func (s DebugFileSecrets) createSecretsFile() (string, error) {
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
		err = s.setYamlStruct(s.Path, Secrets{})
		if err != nil {
			log.Printf("[ERROR] Failed to set yaml struct")
			return s.Path, err
		}
	}
	return s.Path, nil
}

// func checkFileExists(filePath string) bool {
// 	_, error := os.Stat(filePath)
// 	//return !os.IsNotExist(err)
// 	return !errors.Is(error, os.ErrNotExist)
// }
