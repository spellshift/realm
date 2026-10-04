package builder

import (
	"fmt"
	"os"
	"time"

	"gopkg.in/yaml.v3"
)

// DefaultPollInterval is the builder polling interval in seconds for legacy configurations.
const DefaultPollInterval = 5

// BuilderHealthy reports whether a builder has polled within three polling intervals.
func BuilderHealthy(lastSeen time.Time, interval int, now time.Time) bool {
	return interval > 0 && interval <= 86400 && !lastSeen.Before(now.Add(-3*time.Duration(interval)*time.Second))
}

// Config represents the YAML configuration for a builder.
type Config struct {
	PollInterval     int      `yaml:"poll_interval"`
	ID               string   `yaml:"id"`
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
	cfg := Config{PollInterval: DefaultPollInterval}
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	if err := cfg.validate(); err != nil {
		return nil, err
	}

	return &cfg, nil
}

func (cfg *Config) validate() error {
	if cfg.PollInterval < 1 || cfg.PollInterval > 86400 {
		return fmt.Errorf("poll_interval must be between 1 and 86400 seconds")
	}
	if cfg.ID == "" {
		return fmt.Errorf("config must specify a builder id")
	}
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
