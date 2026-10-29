package builder

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	"gopkg.in/yaml.v3"
)

// HTTPClient performs the enrollment token exchange request. It is exported
// so tests can substitute a client that trusts a test CA; callers should not
// normally need to change it. Redirects are not followed, since the request
// carries the one-time token and a 307/308 would replay it to another host.
var HTTPClient = &http.Client{
	Timeout: 30 * time.Second,
	CheckRedirect: func(_ *http.Request, _ []*http.Request) error {
		return http.ErrUseLastResponse
	},
}

// Identity holds the builder's persisted enrollment materials: the identifier
// assigned by Tavern, the CA-signed certificate, and the builder's own
// private key. It is written to disk once as a single file so a builder
// never needs to re-enroll after a restart.
type Identity struct {
	ID          string `yaml:"id"`
	Certificate string `yaml:"certificate"`
	PrivateKey  string `yaml:"private_key"`
}

// LoadIdentity reads and validates a builder identity file. The certificate
// and private key are verified to belong together and to the identifier.
func LoadIdentity(path string) (*Identity, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()
	info, err := file.Stat()
	if err != nil {
		return nil, err
	}
	if info.Mode().Perm()&0077 != 0 {
		return nil, fmt.Errorf("builder identity file permissions are too broad: %v", info.Mode().Perm())
	}
	data, err := io.ReadAll(file)
	if err != nil {
		return nil, err
	}
	var identity Identity
	if err := yaml.Unmarshal(data, &identity); err != nil {
		return nil, fmt.Errorf("invalid builder identity: %w", err)
	}
	if identity.ID == "" {
		return nil, fmt.Errorf("invalid builder identity: missing id")
	}
	creds, err := parseMTLSCredentials(identity.Certificate, []byte(identity.PrivateKey))
	if err != nil {
		return nil, fmt.Errorf("invalid builder identity: %w", err)
	}
	cert, err := x509.ParseCertificate(creds.certDER)
	if err != nil || cert.Subject.CommonName != "builder-"+identity.ID {
		return nil, fmt.Errorf("invalid builder identity: certificate id mismatch")
	}
	return &identity, nil
}

// SaveIdentity writes an identity file atomically with mode 0600, refusing to
// overwrite an existing file. The no-clobber guarantee is enforced with
// os.Link rather than a stat-then-rename check, since the latter leaves a
// window where a concurrently written file could be overwritten.
func SaveIdentity(path string, identity *Identity) error {
	data, err := yaml.Marshal(identity)
	if err != nil {
		return err
	}
	file, err := os.CreateTemp(filepath.Dir(path), ".builder-identity-*")
	if err != nil {
		return err
	}
	tmpPath := file.Name()
	defer os.Remove(tmpPath)
	if err := file.Chmod(0600); err != nil {
		file.Close()
		return err
	}
	if _, err := file.Write(data); err != nil {
		file.Close()
		return err
	}
	if err := file.Sync(); err != nil {
		file.Close()
		return err
	}
	if err := file.Close(); err != nil {
		return err
	}
	if err := os.Link(tmpPath, path); err != nil {
		if os.IsExist(err) {
			return fmt.Errorf("identity file already exists")
		}
		return err
	}
	return nil
}

// EnsureIdentity returns the builder's identity, loading it from disk if
// present. If absent, it generates a local key/CSR, exchanges a one-time
// enrollment token for a CA-signed certificate via GraphQL, persists the
// resulting identity, and returns it.
func EnsureIdentity(ctx context.Context, cfg *Config) (*Identity, error) {
	identity, err := LoadIdentity(cfg.IdentityFile)
	if err == nil {
		return identity, nil
	}
	if !os.IsNotExist(err) && !errors.Is(err, syscall.ENOTDIR) {
		// No exchange has happened yet, so the existing token (if any) is
		// untouched; the operator needs to fix or remove the identity file,
		// not obtain a fresh token.
		return nil, fmt.Errorf("existing builder identity is invalid, remove or repair %q: %w", cfg.IdentityFile, err)
	}

	token := os.Getenv("REALM_BUILDER_ENROLLMENT_TOKEN")
	if token == "" {
		return nil, fmt.Errorf("REALM_BUILDER_ENROLLMENT_TOKEN is required when identity_file is absent")
	}

	// Preflight the identity destination before spending the one-time
	// token: an unwritable directory would otherwise consume the token and
	// then fail to persist the resulting identity.
	if err := ensureIdentityDirWritable(cfg.IdentityFile); err != nil {
		return nil, fmt.Errorf("builder identity cannot be saved: %w", err)
	}

	_, key, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return nil, err
	}
	keyDER, err := x509.MarshalPKCS8PrivateKey(key)
	if err != nil {
		return nil, err
	}
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	if err != nil {
		return nil, err
	}
	keyPEM := string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER}))
	csrPEM := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrDER}))

	builderID, certPEM, err := exchangeEnrollmentToken(ctx, cfg, token, csrPEM)
	if err != nil {
		return nil, err
	}

	identity = &Identity{ID: builderID, Certificate: certPEM, PrivateKey: keyPEM}
	if _, err := parseMTLSCredentials(identity.Certificate, []byte(identity.PrivateKey)); err != nil {
		return nil, fmt.Errorf("invalid enrollment certificate: %w", err)
	}
	if err := SaveIdentity(cfg.IdentityFile, identity); err != nil {
		return nil, fmt.Errorf("builder enrollment succeeded but identity could not be saved, a fresh token is needed: %w", err)
	}
	return identity, nil
}

// ensureIdentityDirWritable verifies that path's parent directory exists
// (creating it if needed) and is writable, without leaving any trace behind.
func ensureIdentityDirWritable(path string) error {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0700); err != nil {
		return fmt.Errorf("directory %q is not usable: %w", dir, err)
	}
	probe, err := os.CreateTemp(dir, ".builder-identity-probe-*")
	if err != nil {
		return fmt.Errorf("directory %q is not writable: %w", dir, err)
	}
	probePath := probe.Name()
	probe.Close()
	os.Remove(probePath)
	return nil
}

// enrollmentExchangeRequest is the GraphQL request body sent to exchange a
// one-time enrollment token for a signed builder certificate.
type enrollmentExchangeRequest struct {
	Query     string                        `json:"query"`
	Variables enrollmentExchangeRequestVars `json:"variables"`
}

type enrollmentExchangeRequestVars struct {
	Token string                     `json:"token"`
	CSR   string                     `json:"csr"`
	Input enrollmentExchangeInputVar `json:"input"`
}

type enrollmentExchangeInputVar struct {
	SupportedTargets []string `json:"supportedTargets"`
	Upstream         string   `json:"upstream"`
	PollInterval     int      `json:"pollInterval"`
	Executor         string   `json:"executor"`
}

type enrollmentExchangeResponse struct {
	Data struct {
		Exchange struct {
			BuilderID string `json:"builderID"`
			MtlsCert  string `json:"mtlsCert"`
		} `json:"exchangeBuilderEnrollmentToken"`
	} `json:"data"`
	Errors []struct {
		Message string `json:"message"`
	} `json:"errors"`
}

const enrollmentExchangeMutation = `mutation($token: String!, $input: CreateBuilderInput!, $csr: String!) {
	exchangeBuilderEnrollmentToken(token: $token, input: $input, csr: $csr) { builderID mtlsCert }
}`

// exchangeEnrollmentToken calls the exchangeBuilderEnrollmentToken GraphQL
// mutation at cfg.Upstream, redeeming the one-time token for a builder ID and
// CA-signed certificate.
func exchangeEnrollmentToken(ctx context.Context, cfg *Config, token, csrPEM string) (builderID, certPEM string, err error) {
	targets := make([]string, len(cfg.SupportedTargets))
	for i, target := range cfg.SupportedTargets {
		targets[i] = "PLATFORM_" + strings.ToUpper(target)
	}

	body, err := json.Marshal(enrollmentExchangeRequest{
		Query: enrollmentExchangeMutation,
		Variables: enrollmentExchangeRequestVars{
			Token: token,
			CSR:   csrPEM,
			Input: enrollmentExchangeInputVar{
				SupportedTargets: targets,
				Upstream:         cfg.Upstream,
				PollInterval:     cfg.PollInterval,
				Executor:         cfg.Executor,
			},
		},
	})
	if err != nil {
		return "", "", err
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, strings.TrimRight(cfg.Upstream, "/")+"/graphql", bytes.NewReader(body))
	if err != nil {
		return "", "", err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := HTTPClient.Do(req)
	if err != nil {
		return "", "", fmt.Errorf("builder enrollment exchange failed before a response was received, the token may or may not have been consumed: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	if err != nil {
		return "", "", fmt.Errorf("failed to read enrollment response, the token may or may not have been consumed: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return "", "", fmt.Errorf("builder enrollment exchange failed with HTTP status %d", resp.StatusCode)
	}

	var result enrollmentExchangeResponse
	if err := json.Unmarshal(respBody, &result); err != nil {
		return "", "", fmt.Errorf("invalid enrollment response, the token may or may not have been consumed: %w", err)
	}
	if len(result.Errors) > 0 {
		return "", "", fmt.Errorf("builder enrollment exchange rejected: %s", truncate(result.Errors[0].Message, 200))
	}
	if result.Data.Exchange.BuilderID == "" || result.Data.Exchange.MtlsCert == "" {
		return "", "", fmt.Errorf("builder enrollment exchange returned an incomplete response, the token may or may not have been consumed")
	}

	return result.Data.Exchange.BuilderID, result.Data.Exchange.MtlsCert, nil
}

func truncate(s string, max int) string {
	if len(s) <= max {
		return s
	}
	return s[:max] + "..."
}
