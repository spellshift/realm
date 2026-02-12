package builder

import (
	"fmt"
	"strings"

	yaml "gopkg.in/yaml.v3"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
)

const (
	// RealmRepoURL is the default git repository URL for building realm agents.
	RealmRepoURL = "https://github.com/spellshift/realm.git"

	// DefaultInterval is the default callback interval in seconds for the IMIX agent.
	DefaultInterval = 5

	// DefaultCallbackURI is the default callback URI for the IMIX agent,
	// derived from the IMIX compile-time default (IMIX_CALLBACK_URI).
	DefaultCallbackURI = "http://127.0.0.1:8000"

	// DefaultTransportType is the default transport type for the IMIX agent,
	// derived from the IMIX default behavior for http:// URIs.
	DefaultTransportType = c2pb.Transport_TRANSPORT_GRPC

	// DefaultBuildImage is the default Docker image for building agents.
	DefaultBuildImage = "spellshift/devcontainer:main"

	// DefaultTargetFormat is the default output format for builds.
	DefaultTargetFormat = builderpb.TargetFormat_TARGET_FORMAT_BIN
)

// TargetFormat is an alias for builderpb.TargetFormat.
type TargetFormat = builderpb.TargetFormat

const (
	TargetFormatBin            = builderpb.TargetFormat_TARGET_FORMAT_BIN
	TargetFormatCdylib         = builderpb.TargetFormat_TARGET_FORMAT_CDYLIB
	TargetFormatWindowsService = builderpb.TargetFormat_TARGET_FORMAT_WINDOWS_SERVICE
)

// SupportedFormats maps each target OS to its supported output formats.
var SupportedFormats = map[c2pb.Host_Platform][]TargetFormat{
	c2pb.Host_PLATFORM_LINUX:   {TargetFormatBin},
	c2pb.Host_PLATFORM_WINDOWS: {TargetFormatBin, TargetFormatCdylib, TargetFormatWindowsService},
	c2pb.Host_PLATFORM_MACOS:   {TargetFormatBin},
}

// buildTarget maps target OS to the Rust --target triple.
var buildTarget = map[c2pb.Host_Platform]string{
	c2pb.Host_PLATFORM_LINUX:   "x86_64-unknown-linux-musl",
	c2pb.Host_PLATFORM_WINDOWS: "x86_64-pc-windows-gnu",
	c2pb.Host_PLATFORM_MACOS:   "aarch64-apple-darwin",
}

type buildKey struct {
	OS     c2pb.Host_Platform
	Format TargetFormat
}

// buildCommands maps (target_os, target_format) -> cargo build command.
var buildCommands = map[buildKey]string{
	{c2pb.Host_PLATFORM_LINUX, TargetFormatBin}:              "cargo build --release --bin imix --target=x86_64-unknown-linux-musl",
	{c2pb.Host_PLATFORM_MACOS, TargetFormatBin}:              "cargo zigbuild --release --target aarch64-apple-darwin",
	{c2pb.Host_PLATFORM_WINDOWS, TargetFormatBin}:            "cargo build --release --target=x86_64-pc-windows-gnu",
	{c2pb.Host_PLATFORM_WINDOWS, TargetFormatWindowsService}: "cargo build --release --features win_service --target=x86_64-pc-windows-gnu",
	{c2pb.Host_PLATFORM_WINDOWS, TargetFormatCdylib}:         "cargo build --release --lib --target=x86_64-pc-windows-gnu",
}

// ValidateTargetFormat checks whether the given format is supported for the given OS.
func ValidateTargetFormat(os c2pb.Host_Platform, format TargetFormat) error {
	formats, ok := SupportedFormats[os]
	if !ok {
		return fmt.Errorf("unsupported target OS: %s", os.String())
	}
	for _, f := range formats {
		if f == format {
			return nil
		}
	}
	supported := make([]string, len(formats))
	for i, f := range formats {
		supported[i] = f.String()
	}
	return fmt.Errorf("target format %q is not supported for %s (supported: %s)", format.String(), os.String(), strings.Join(supported, ", "))
}

// BuildCommand returns the cargo build command for the given OS and format.
func BuildCommand(os c2pb.Host_Platform, format TargetFormat) (string, error) {
	cmd, ok := buildCommands[buildKey{os, format}]
	if !ok {
		return "", fmt.Errorf("no build command for %s + %s", os.String(), format.String())
	}
	return cmd, nil
}

// TransportTypeToString converts a c2pb.Transport_Type to the lowercase string
// used in the IMIX agent configuration YAML.
func TransportTypeToString(t c2pb.Transport_Type) string {
	switch t {
	case c2pb.Transport_TRANSPORT_GRPC:
		return "grpc"
	case c2pb.Transport_TRANSPORT_HTTP1:
		return "http"
	case c2pb.Transport_TRANSPORT_DNS:
		return "dns"
	default:
		return "unspecified"
	}
}

// ImixTransportConfig represents the transport section of the IMIX configuration.
type ImixTransportConfig struct {
	URI string `yaml:"URI"`
	Interval    int    `yaml:"interval"`
	Type        string `yaml:"type"`
	Extra       string `yaml:"extra"`
}

// ImixConfig represents the IMIX agent configuration YAML.
type ImixConfig struct {
	ServerPubkey string                `yaml:"server_pubkey"`
	Transports   []ImixTransportConfig `yaml:"transports"`
}

// DeriveArtifactPath returns the artifact path inside the build container
// based on the target OS.
func DeriveArtifactPath(os c2pb.Host_Platform) string {
	target, ok := buildTarget[os]
	if !ok {
		return "/home/vscode/realm/implants/target/release/imix"
	}
	return fmt.Sprintf("/home/vscode/realm/implants/target/%s/release/imix", target)
}

// MarshalImixConfig serializes the ImixConfig to a YAML string suitable
// for passing as the IMIX_CONFIG environment variable.
func MarshalImixConfig(cfg ImixConfig) (string, error) {
	cfgBytes, err := yaml.Marshal(cfg)
	if err != nil {
		return "", fmt.Errorf("failed to marshal IMIX config: %w", err)
	}
	return string(cfgBytes), nil
}

// GenerateBuildScript generates the full build script from the build configuration.
// It clones the repository and runs the build command. The IMIX configuration
// is passed via the IMIX_CONFIG environment variable rather than being written
// to a file in the build script.
func GenerateBuildScript(os c2pb.Host_Platform, format TargetFormat) (string, error) {
	buildCmd, err := BuildCommand(os, format)
	if err != nil {
		return "", err
	}

	script := fmt.Sprintf(
		`cd /home/vscode && git clone %s realm && cd realm/implants/imix && %s`,
		RealmRepoURL,
		buildCmd,
	)

	return script, nil
}
