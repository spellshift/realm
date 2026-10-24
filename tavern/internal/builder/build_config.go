package builder

import (
	"fmt"
	"strings"

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
)

const (
	// DefaultRealmRepoURL is the default git repository URL for building realm agents.
	DefaultRealmRepoURL = "https://github.com/spellshift/realm.git"

	// DefaultBuildImage is the default Docker image for building agents.
	DefaultBuildImage = "spellshift/devcontainer:main"

	// DefaultTargetFormat is the default output format for builds.
	DefaultTargetFormat = builderpb.TargetFormat_TARGET_FORMAT_BIN

	// DefaultBuildProfileName is the name of the build profile Tavern seeds
	// on startup (see ensureDefaultBuildProfile in tavern/app.go). It can't
	// be edited or deleted through the API, since it's recreated under this
	// same name on every restart if missing.
	DefaultBuildProfileName = "Default"
)

// DefaultTransports is the default transport configuration for a build task.
var DefaultTransports = []builderpb.BuildProfileTransport{{
	URI:      "http://127.0.0.1:8000",
	Interval: 5,
	Type:     c2pb.Transport_TRANSPORT_GRPC,
}}

// TargetFormat is an alias for builderpb.TargetFormat.
type TargetFormat = builderpb.TargetFormat

// SupportedFormats maps each target OS to its supported output formats.
var SupportedFormats = map[c2pb.Host_Platform][]TargetFormat{
	c2pb.Host_PLATFORM_LINUX:   {builderpb.TargetFormat_TARGET_FORMAT_BIN},
	c2pb.Host_PLATFORM_WINDOWS: {builderpb.TargetFormat_TARGET_FORMAT_BIN, builderpb.TargetFormat_TARGET_FORMAT_CDYLIB, builderpb.TargetFormat_TARGET_FORMAT_WINDOWS_SERVICE},
	c2pb.Host_PLATFORM_MACOS:   {builderpb.TargetFormat_TARGET_FORMAT_BIN},
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
	{c2pb.Host_PLATFORM_LINUX, builderpb.TargetFormat_TARGET_FORMAT_BIN}:               "cargo build --release --bin imix --target=x86_64-unknown-linux-musl",
	{c2pb.Host_PLATFORM_MACOS, builderpb.TargetFormat_TARGET_FORMAT_BIN}:               "cargo zigbuild --release --target aarch64-apple-darwin",
	{c2pb.Host_PLATFORM_WINDOWS, builderpb.TargetFormat_TARGET_FORMAT_BIN}:             "cargo build --release --target=x86_64-pc-windows-gnu",
	{c2pb.Host_PLATFORM_WINDOWS, builderpb.TargetFormat_TARGET_FORMAT_WINDOWS_SERVICE}: "cargo build --release --features win_service --target=x86_64-pc-windows-gnu",
	{c2pb.Host_PLATFORM_WINDOWS, builderpb.TargetFormat_TARGET_FORMAT_CDYLIB}:          "cargo build --release --lib --target=x86_64-pc-windows-gnu",
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
	case c2pb.Transport_TRANSPORT_ICMP:
		return "icmp"
	default:
		return "unspecified"
	}
}

// ImixTransportConfig represents the transport section of the IMIX configuration.
type ImixTransportConfig struct {
	URI      string `yaml:"URI"`
	Interval int    `yaml:"interval"`
	Type     string `yaml:"type"`
	Extra    string `yaml:"extra"`
}

// ImixConfig represents the IMIX agent configuration YAML.
type ImixConfig struct {
	ServerPubkey string                `yaml:"server_pubkey"`
	Transports   []ImixTransportConfig `yaml:"transports"`
}

// NewBuildDir generates a unique build directory path for a single build
// task. It's templated into the setup/build/artifact-path recipes (as
// {{.BuildDir}}) so they all agree on exactly where the setup script clones
// the repo, regardless of how that script gets there (no more hardcoding a
// fixed path like /home/vscode that only matched one specific setup script).
func NewBuildDir() string {
	return fmt.Sprintf("/tmp/realm-build-%s", uuid.NewString())
}

// DeriveArtifactPath returns the artifact path inside the build container
// based on the target OS, rooted at buildDir (see NewBuildDir).
func DeriveArtifactPath(os c2pb.Host_Platform, buildDir string) string {
	target, ok := buildTarget[os]
	if !ok {
		return fmt.Sprintf("%s/realm/implants/target/release/imix", buildDir)
	}
	return fmt.Sprintf("%s/realm/implants/target/%s/release/imix", buildDir, target)
}
