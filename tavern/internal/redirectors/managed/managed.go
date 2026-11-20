package managed

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"

	secretmanager "cloud.google.com/go/secretmanager/apiv1"
	"cloud.google.com/go/secretmanager/apiv1/secretmanagerpb"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/redirectors"
)

var log = logging.For("redirectors.managed")

const redirectorIdentityWaitTimeout = 2 * time.Minute

type Config struct {
	Upstream, Transport, Token, IdentityFile, IdentitySecret string
	Domains                                                  []string
}

type Identity struct {
	ID          string `json:"id"`
	Certificate string `json:"certificate"`
	PrivateKey  string `json:"private_key"`
}

type Store interface {
	Load(context.Context) (*Identity, error)
	Save(context.Context, *Identity) error
}

type fileStore struct{ path string }

func (s fileStore) Load(_ context.Context) (*Identity, error) {
	data, err := os.ReadFile(s.path)
	if err != nil {
		return nil, err
	}
	info, err := os.Stat(s.path)
	if err != nil {
		return nil, err
	}
	if info.Mode().Perm()&0077 != 0 {
		return nil, fmt.Errorf("redirector identity file permissions are too broad")
	}
	var identity Identity
	if err := json.Unmarshal(data, &identity); err != nil {
		return nil, err
	}
	return &identity, nil
}

func (s fileStore) Save(_ context.Context, identity *Identity) error {
	data, err := json.Marshal(identity)
	if err != nil {
		return err
	}
	file, err := os.CreateTemp(filepath.Dir(s.path), ".redirector-identity-*")
	if err != nil {
		return err
	}
	defer os.Remove(file.Name())
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
	return os.Link(file.Name(), s.path)
}

type secretStore struct {
	client *secretmanager.Client
	name   string
}

func (s secretStore) Load(ctx context.Context) (*Identity, error) {
	response, err := s.client.AccessSecretVersion(ctx, &secretmanagerpb.AccessSecretVersionRequest{Name: s.name + "/versions/latest"})
	if err != nil {
		return nil, err
	}
	var identity Identity
	if err := json.Unmarshal(response.Payload.Data, &identity); err != nil {
		return nil, err
	}
	return &identity, nil
}

func (s secretStore) Save(ctx context.Context, identity *Identity) error {
	data, err := json.Marshal(identity)
	if err != nil {
		return err
	}
	_, err = s.client.AddSecretVersion(ctx, &secretmanagerpb.AddSecretVersionRequest{Parent: s.name, Payload: &secretmanagerpb.SecretPayload{Data: data}})
	return err
}

func OpenStore(ctx context.Context, cfg Config) (Store, func() error, error) {
	if cfg.IdentitySecret != "" {
		client, err := secretmanager.NewClient(ctx)
		if err != nil {
			return nil, nil, err
		}
		return secretStore{client: client, name: cfg.IdentitySecret}, client.Close, nil
	}
	if cfg.IdentityFile == "" {
		return nil, nil, fmt.Errorf("managed redirector requires --identity-file or --identity-secret")
	}
	if err := os.MkdirAll(filepath.Dir(cfg.IdentityFile), 0700); err != nil {
		return nil, nil, fmt.Errorf("redirector identity directory: %w", err)
	}
	probe, err := os.CreateTemp(filepath.Dir(cfg.IdentityFile), ".redirector-identity-probe-*")
	if err != nil {
		return nil, nil, fmt.Errorf("redirector identity directory is not writable: %w", err)
	}
	probe.Close()
	os.Remove(probe.Name())
	return fileStore{path: cfg.IdentityFile}, func() error { return nil }, nil
}

func validate(identity *Identity) (*x509.Certificate, ed25519.PrivateKey, error) {
	if identity == nil || identity.ID == "" {
		return nil, nil, fmt.Errorf("missing redirector identity")
	}
	block, rest := pem.Decode([]byte(identity.Certificate))
	if block == nil || block.Type != "CERTIFICATE" || len(bytes.TrimSpace(rest)) != 0 {
		return nil, nil, fmt.Errorf("invalid redirector certificate")
	}
	cert, err := x509.ParseCertificate(block.Bytes)
	if err != nil {
		return nil, nil, err
	}
	block, rest = pem.Decode([]byte(identity.PrivateKey))
	if block == nil || block.Type != "PRIVATE KEY" || len(bytes.TrimSpace(rest)) != 0 {
		return nil, nil, fmt.Errorf("invalid redirector private key")
	}
	keyAny, err := x509.ParsePKCS8PrivateKey(block.Bytes)
	if err != nil {
		return nil, nil, err
	}
	key, ok := keyAny.(ed25519.PrivateKey)
	pub, pubOK := cert.PublicKey.(ed25519.PublicKey)
	if !ok || !pubOK || !pub.Equal(key.Public()) || cert.Subject.CommonName != "redirector-"+identity.ID {
		return nil, nil, fmt.Errorf("redirector identity key or identifier mismatch")
	}
	return cert, key, nil
}

var httpClient = &http.Client{Timeout: 30 * time.Second, CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }}

// ManagementURL accepts an explicit Tavern URL or an HTTP(S) upstream URL.
// The traffic upstream may instead be a bare gRPC host:port, which cannot be
// used to construct enrollment and check-in HTTP requests.
func ManagementURL(explicit, upstream string) (string, error) {
	raw := explicit
	if raw == "" {
		raw = upstream
	}
	parsed, err := url.Parse(raw)
	if err != nil || (parsed.Scheme != "http" && parsed.Scheme != "https") || parsed.Hostname() == "" || parsed.User != nil || parsed.RawQuery != "" || parsed.Fragment != "" || (parsed.Path != "" && parsed.Path != "/") {
		return "", fmt.Errorf("managed redirector requires --management-url with a Tavern http:// or https:// URL when upstream is not an HTTP URL")
	}
	return strings.TrimRight(parsed.String(), "/"), nil
}

func EnsureIdentity(ctx context.Context, cfg Config, store Store) (*Identity, error) {
	identity, err := store.Load(ctx)
	if err == nil {
		_, _, err = validate(identity)
		return identity, err
	}
	if !identityAbsent(err) {
		return nil, fmt.Errorf("load redirector identity: %w", err)
	}
	if cfg.Token == "" {
		return nil, fmt.Errorf("managed redirector enrollment token is required when identity is absent")
	}
	_, key, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return nil, err
	}
	csr, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, key)
	if err != nil {
		return nil, err
	}
	keyDER, err := x509.MarshalPKCS8PrivateKey(key)
	if err != nil {
		return nil, err
	}
	body, err := json.Marshal(map[string]any{"token": cfg.Token, "csr": string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csr})), "transport": cfg.Transport, "domains": cfg.Domains})
	if err != nil {
		return nil, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, strings.TrimRight(cfg.Upstream, "/")+"/auth/redirector/enroll", bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("redirector enrollment response unavailable; token may have been consumed: %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		// Another replica may have redeemed the deployment token. It must publish
		// the shared identity before this instance starts serving traffic.
		if resp.StatusCode == http.StatusUnauthorized {
			log.WarnContext(ctx, "redirector enrollment token rejected; waiting for shared identity")
			return waitForIdentity(ctx, store, redirectorIdentityWaitTimeout)
		}
		return nil, fmt.Errorf("redirector enrollment failed with HTTP %d", resp.StatusCode)
	}
	var result struct {
		ID          string `json:"redirector_id"`
		Certificate string `json:"mtls_cert"`
	}
	if err := json.NewDecoder(io.LimitReader(resp.Body, 1<<20)).Decode(&result); err != nil {
		return nil, fmt.Errorf("enrollment succeeded but response could not be read; a fresh token is needed: %w", err)
	}
	identity = &Identity{ID: result.ID, Certificate: result.Certificate, PrivateKey: string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER}))}
	if _, _, err := validate(identity); err != nil {
		return nil, err
	}
	if err := store.Save(ctx, identity); err != nil {
		if errors.Is(err, os.ErrExist) {
			winner, loadErr := store.Load(ctx)
			if loadErr == nil {
				_, _, loadErr = validate(winner)
			}
			if loadErr != nil {
				return nil, fmt.Errorf("redirector identity was published concurrently but cannot be loaded: %w", loadErr)
			}
			return winner, nil
		}
		return nil, fmt.Errorf("enrollment succeeded but identity could not be saved: %w", err)
	}
	return identity, nil
}

func identityAbsent(err error) bool {
	return errors.Is(err, os.ErrNotExist) || status.Code(err) == codes.NotFound || status.Code(err) == codes.FailedPrecondition
}

func waitForIdentity(ctx context.Context, store Store, timeout time.Duration) (*Identity, error) {
	waitCtx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()
	attempts := 0
	for {
		identity, err := store.Load(waitCtx)
		if waitCtx.Err() != nil {
			return nil, identityWaitError(ctx, timeout)
		}
		if err == nil {
			_, _, err = validate(identity)
			if err != nil {
				return nil, fmt.Errorf("published redirector identity is invalid: %w", err)
			}
			return identity, nil
		}
		if !identityAbsent(err) {
			return nil, fmt.Errorf("load shared redirector identity: %w", err)
		}
		attempts++
		if attempts%60 == 0 {
			log.WarnContext(ctx, "waiting for redirector identity after enrollment token was rejected; verify the token and shared store if this persists")
		}
		select {
		case <-waitCtx.Done():
			return nil, identityWaitError(ctx, timeout)
		case <-ticker.C:
		}
	}
}

func identityWaitError(ctx context.Context, timeout time.Duration) error {
	if err := ctx.Err(); err != nil {
		return err
	}
	return fmt.Errorf("redirector enrollment rejected and shared identity did not appear within %s; verify the enrollment token and identity store: %w", timeout, context.DeadlineExceeded)
}

// CheckinEnabled returns the persisted control state after an authenticated check-in.
func CheckinEnabled(ctx context.Context, upstream string, identity *Identity) (bool, error) {
	cert, key, err := validate(identity)
	if err != nil {
		return false, err
	}
	timestamp := time.Now().UTC().Format(time.RFC3339Nano)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, strings.TrimRight(upstream, "/")+"/auth/redirector/checkin", nil)
	if err != nil {
		return false, err
	}
	req.Header.Set("X-Realm-Redirector-Cert", base64.StdEncoding.EncodeToString(cert.Raw))
	req.Header.Set("X-Realm-Redirector-Timestamp", timestamp)
	req.Header.Set("X-Realm-Redirector-Signature", base64.StdEncoding.EncodeToString(ed25519.Sign(key, []byte(timestamp))))
	resp, err := httpClient.Do(req)
	if err != nil {
		return false, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return false, fmt.Errorf("redirector check-in failed with HTTP %d", resp.StatusCode)
	}
	var state struct {
		Enabled *bool `json:"enabled"`
	}
	if err := json.NewDecoder(io.LimitReader(resp.Body, 1<<20)).Decode(&state); err != nil {
		return false, fmt.Errorf("redirector check-in state: %w", err)
	}
	if state.Enabled == nil {
		return false, fmt.Errorf("redirector check-in state missing enabled field")
	}
	return *state.Enabled, nil
}

// WaitForInitialCheckin prevents a restarted managed redirector from serving
// traffic before it has learned Tavern's persisted enabled state.
func WaitForInitialCheckin(ctx context.Context, upstream string, identity *Identity, gate *redirectors.Gate) error {
	return waitForInitialCheckin(ctx, 5*time.Second, func() error {
		return RefreshGate(ctx, upstream, identity, gate)
	})
}

func waitForInitialCheckin(ctx context.Context, retry time.Duration, check func() error) error {
	for {
		if err := ctx.Err(); err != nil {
			return err
		}
		if err := check(); err == nil {
			return nil
		} else {
			log.WarnContext(ctx, "initial redirector check-in failed; waiting before opening ingress", "error", err)
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(retry):
		}
	}
}

func RunCheckins(ctx context.Context, upstream string, identity *Identity, gate *redirectors.Gate) {
	check := func() {
		if err := RefreshGate(ctx, upstream, identity, gate); err != nil {
			log.WarnContext(ctx, "redirector check-in failed", "error", err)
		}
	}
	ticker := time.NewTicker(time.Minute)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			check()
		}
	}
}

// RefreshGate changes ingress state only after a successful check-in.
func RefreshGate(ctx context.Context, upstream string, identity *Identity, gate *redirectors.Gate) error {
	enabled, err := CheckinEnabled(ctx, upstream, identity)
	if err != nil {
		return err
	}
	gate.SetEnabled(enabled)
	return nil
}
