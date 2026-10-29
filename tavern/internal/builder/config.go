package builder

import (
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"time"

	"gopkg.in/yaml.v3"
)

// DefaultPollInterval is the builder polling interval in seconds.
const DefaultPollInterval = 5

// MaxPollInterval is the largest poll_interval (in seconds) ParseConfig accepts.
const MaxPollInterval = 86400

// MaxStaleAge is the longest a builder can go without checking in and still be
// considered healthy, even at MaxPollInterval. Callers that need to bound a
// freshness query at the database layer (before BuilderHealthy's per-builder
// check, which needs each builder's own interval) can use this as a
// conservative upper bound.
const MaxStaleAge = 3 * MaxPollInterval * time.Second

// BuilderHealthy reports whether a builder has polled within three polling intervals.
func BuilderHealthy(lastSeen time.Time, interval int, now time.Time) bool {
	return interval > 0 && interval <= MaxPollInterval && !lastSeen.Before(now.Add(-3*time.Duration(interval)*time.Second))
}

// Config represents the YAML configuration for a builder.
type Config struct {
	PollInterval     int      `yaml:"poll_interval"`
	SupportedTargets []string `yaml:"supported_targets"`
	IdentityFile     string   `yaml:"identity_file"`
	Upstream         string   `yaml:"upstream"`
	Executor         string   `yaml:"executor"`
}

// DefaultConfig returns the settings used when no builder configuration is provided.
func DefaultConfig() *Config {
	return &Config{
		PollInterval:     DefaultPollInterval,
		SupportedTargets: []string{"linux", "windows"},
		IdentityFile:     "builder.identity",
		Upstream:         "http://127.0.0.1:8000",
		Executor:         "docker",
	}
}

// ParseConfig reads and parses a builder YAML configuration file.
func ParseConfig(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file %q: %w", path, err)
	}
	cfg, err := ParseConfigBytes(data)
	if err != nil {
		return nil, err
	}
	if !filepath.IsAbs(cfg.IdentityFile) {
		cfg.IdentityFile = filepath.Join(filepath.Dir(path), cfg.IdentityFile)
	}
	return cfg, nil
}

// ParseConfigBytes parses builder YAML configuration from bytes.
func ParseConfigBytes(data []byte) (*Config, error) {
	var fields map[string]interface{}
	if err := yaml.Unmarshal(data, &fields); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}
	for _, name := range []string{"identity_file", "supported_targets", "upstream", "executor", "poll_interval"} {
		if value, present := fields[name]; present && value == nil {
			return nil, fmt.Errorf("%s must not be null", name)
		}
	}
	if fields["executor"] == "" {
		return nil, fmt.Errorf("unsupported executor %q, must be one of: docker, local", "")
	}
	cfg := DefaultConfig()
	if err := yaml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	if err := cfg.validate(); err != nil {
		return nil, err
	}

	return cfg, nil
}

func (cfg *Config) validate() error {
	if cfg.PollInterval < 1 || cfg.PollInterval > 86400 {
		return fmt.Errorf("poll_interval must be between 1 and 86400 seconds")
	}
	if cfg.IdentityFile == "" {
		return fmt.Errorf("config must specify identity_file")
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
	if err := validateUpstream(cfg.Upstream); err != nil {
		return err
	}
	if cfg.Executor == "" {
		cfg.Executor = "docker"
	}
	switch cfg.Executor {
	case "docker", "local":
		// valid
	default:
		return fmt.Errorf("unsupported executor %q, must be one of: docker, local", cfg.Executor)
	}
	return nil
}

// Validate exposes validate for callers outside this package, such as the
// enrollment handler that validates builder-supplied settings before
// consuming an enrollment token.
func (cfg *Config) Validate() error { return cfg.validate() }

// validateUpstream checks that upstream is an http or https URL with a host.
func validateUpstream(upstream string) error {
	u, err := url.Parse(upstream)
	if err != nil {
		return fmt.Errorf("invalid upstream server address %q: %w", upstream, err)
	}
	if u.Scheme != "http" && u.Scheme != "https" {
		return fmt.Errorf("upstream server address %q must use http or https", upstream)
	}
	if u.Host == "" {
		return fmt.Errorf("upstream server address %q must include a host", upstream)
	}
	return nil
}
