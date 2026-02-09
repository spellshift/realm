package builder

import (
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// Config represents the YAML configuration for a builder.
type Config struct {
	SupportedTargets []string `yaml:"supported_targets"`
	MTLS             string   `yaml:"mtls"`
	Upstream         string   `yaml:"upstream"`
}

// ParseConfig reads and parses a builder YAML configuration file.
func ParseConfig(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file %q: %w", path, err)
	}
	return ParseConfigBytes(data)
}

// ParseConfigBytes parses builder YAML configuration from bytes.
func ParseConfigBytes(data []byte) (*Config, error) {
	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	if err := cfg.validate(); err != nil {
		return nil, err
	}

	return &cfg, nil
}

func (cfg *Config) validate() error {
	if len(cfg.SupportedTargets) == 0 {
		return fmt.Errorf("config must specify at least one supported_target")
	}
	for _, target := range cfg.SupportedTargets {
		switch target {
		case "macos", "linux", "windows":
			// valid
		default:
			return fmt.Errorf("unsupported target %q, must be one of: macos, linux, windows", target)
		}
	}
	if cfg.Upstream == "" {
		return fmt.Errorf("config must specify an upstream server address")
	}
	return nil
}
