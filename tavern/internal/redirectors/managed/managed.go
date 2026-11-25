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
	"hash/crc32"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	"cloud.google.com/go/iam/apiv1/iampb"
	secretmanager "cloud.google.com/go/secretmanager/apiv1"
	"cloud.google.com/go/secretmanager/apiv1/secretmanagerpb"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/logging"
)

var (
	log                    = logging.For("redirectors.managed")
	crc32cTable            = crc32.MakeTable(crc32.Castagnoli)
	secretNamePattern      = regexp.MustCompile(`^projects/[^/]+/secrets/[^/]+$`)
	errCheckinUnauthorized = errors.New("redirector check-in unauthorized")
)

const (
	redirectorIdentityWaitTimeout = 2 * time.Minute
	redirectorRenewalWindow       = 30 * 24 * time.Hour
	storeOpTimeout                = 30 * time.Second
	maxIdentityFileBytes          = 64 << 10
	secretAddVersionPermission    = "secretmanager.versions.add"
)

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

type Replacer interface {
	Replace(context.Context, *Identity) error
}

type preflightWriter interface {
	PreflightWrite(context.Context) error
}

type fileStore struct{ path string }

func (s fileStore) Load(_ context.Context) (*Identity, error) {
	file, err := openIdentityFile(s.path)
	if err != nil {
		return nil, err
	}
	defer file.Close()
	info, err := file.Stat()
	if err != nil {
		return nil, err
	}
	if !info.Mode().IsRegular() {
		return nil, fmt.Errorf("redirector identity file must be a regular file")
	}
	if info.Mode().Perm()&0077 != 0 {
		return nil, fmt.Errorf("redirector identity file permissions are too broad")
	}
	if err := verifyFileOwnership(info); err != nil {
		return nil, err
	}
	data, err := io.ReadAll(io.LimitReader(file, maxIdentityFileBytes+1))
	if err != nil {
		return nil, err
	}
	if len(data) > maxIdentityFileBytes {
		return nil, fmt.Errorf("redirector identity file exceeds maximum size of %d bytes", maxIdentityFileBytes)
	}
	var identity Identity
	if err := json.Unmarshal(data, &identity); err != nil {
		return nil, err
	}
	return &identity, nil
}

func writeTempIdentity(dir string, identity *Identity) (string, error) {
	data, err := json.Marshal(identity)
	if err != nil {
		return "", err
	}
	file, err := os.CreateTemp(dir, ".redirector-identity-*")
	if err != nil {
		return "", err
	}
	name := file.Name()
	if err := file.Chmod(0600); err != nil {
		file.Close()
		os.Remove(name)
		return "", err
	}
	if _, err := file.Write(data); err != nil {
		file.Close()
		os.Remove(name)
		return "", err
	}
	if err := file.Sync(); err != nil {
		file.Close()
		os.Remove(name)
		return "", err
	}
	if err := file.Close(); err != nil {
		os.Remove(name)
		return "", err
	}
	return name, nil
}

func (s fileStore) Save(_ context.Context, identity *Identity) error {
	tempPath, err := writeTempIdentity(filepath.Dir(s.path), identity)
	if err != nil {
		return err
	}
	defer os.Remove(tempPath)
	return os.Link(tempPath, s.path)
}

func (s fileStore) Replace(_ context.Context, identity *Identity) error {
	tempPath, err := writeTempIdentity(filepath.Dir(s.path), identity)
	if err != nil {
		return err
	}
	if err := os.Rename(tempPath, s.path); err != nil {
		os.Remove(tempPath)
		return err
	}
	return nil
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
	payload := response.GetPayload()
	if payload == nil {
		return nil, fmt.Errorf("redirector identity secret payload is empty")
	}
	if payload.DataCrc32C != nil {
		expected := int64(crc32.Checksum(payload.Data, crc32cTable))
		if *payload.DataCrc32C != expected {
			return nil, fmt.Errorf("redirector identity secret payload checksum mismatch")
		}
	}
	var identity Identity
	if err := json.Unmarshal(payload.Data, &identity); err != nil {
		return nil, err
	}
	return &identity, nil
}

func (s secretStore) Save(ctx context.Context, identity *Identity) error {
	data, err := json.Marshal(identity)
	if err != nil {
		return err
	}
	checksum := int64(crc32.Checksum(data, crc32cTable))
	_, err = s.client.AddSecretVersion(ctx, &secretmanagerpb.AddSecretVersionRequest{
		Parent: s.name,
		Payload: &secretmanagerpb.SecretPayload{
			Data:       data,
			DataCrc32C: &checksum,
		},
	})
	return err
}

func (s secretStore) Replace(ctx context.Context, identity *Identity) error {
	return s.Save(ctx, identity)
}

func (s secretStore) PreflightWrite(ctx context.Context) error {
	resp, err := s.client.TestIamPermissions(ctx, &iampb.TestIamPermissionsRequest{
		Resource:    s.name,
		Permissions: []string{secretAddVersionPermission},
	})
	if err != nil {
		return fmt.Errorf("verify secret write permission: %w", err)
	}
	for _, perm := range resp.GetPermissions() {
		if perm == secretAddVersionPermission {
			return nil
		}
	}
	return fmt.Errorf("missing %s permission on %s", secretAddVersionPermission, s.name)
}

func OpenStore(ctx context.Context, cfg Config) (Store, func() error, error) {
	if cfg.IdentitySecret != "" && cfg.IdentityFile != "" {
		return nil, nil, fmt.Errorf("managed redirector accepts either --identity-file or --identity-secret, not both")
	}
	if cfg.IdentitySecret != "" {
		if !secretNamePattern.MatchString(cfg.IdentitySecret) {
			return nil, nil, fmt.Errorf("invalid --identity-secret resource name %q: expected projects/<project>/secrets/<name>", cfg.IdentitySecret)
		}
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

func containsClientAuth(usages []x509.ExtKeyUsage) bool {
	for _, usage := range usages {
		if usage == x509.ExtKeyUsageClientAuth {
			return true
		}
	}
	return false
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
	if cert.IsCA {
		return nil, nil, fmt.Errorf("redirector certificate must not be a CA certificate")
	}
	if !containsClientAuth(cert.ExtKeyUsage) {
		return nil, nil, fmt.Errorf("redirector certificate missing client authentication extended key usage")
	}
	if cert.KeyUsage&x509.KeyUsageDigitalSignature == 0 {
		return nil, nil, fmt.Errorf("redirector certificate missing digital signature key usage")
	}
	now := time.Now()
	if now.Before(cert.NotBefore) || now.After(cert.NotAfter) {
		return nil, nil, fmt.Errorf("redirector certificate is expired or not yet valid")
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

func NeedsRenewal(identity *Identity, now time.Time) bool {
	if identity == nil {
		return false
	}
	block, rest := pem.Decode([]byte(identity.Certificate))
	if block == nil || block.Type != "CERTIFICATE" || len(bytes.TrimSpace(rest)) != 0 {
		return false
	}
	cert, err := x509.ParseCertificate(block.Bytes)
	if err != nil {
		return false
	}
	return cert.NotAfter.Sub(now) <= redirectorRenewalWindow
}

var httpClient = &http.Client{Timeout: 30 * time.Second, CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }}

func drainBody(body io.Reader) {
	_, _ = io.Copy(io.Discard, io.LimitReader(body, 4096))
}

func setRedirectorAuthHeaders(req *http.Request, cert *x509.Certificate, key ed25519.PrivateKey) {
	timestamp := time.Now().UTC().Format(time.RFC3339Nano)
	req.Header.Set("X-Realm-Redirector-Cert", base64.StdEncoding.EncodeToString(cert.Raw))
	req.Header.Set("X-Realm-Redirector-Timestamp", timestamp)
	req.Header.Set("X-Realm-Redirector-Signature", base64.StdEncoding.EncodeToString(ed25519.Sign(key, []byte(timestamp))))
}

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
	loadCtx, cancelLoad := context.WithTimeout(ctx, storeOpTimeout)
	identity, err := store.Load(loadCtx)
	cancelLoad()
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
	if pf, ok := store.(preflightWriter); ok {
		pfCtx, cancelPF := context.WithTimeout(ctx, storeOpTimeout)
		err := pf.PreflightWrite(pfCtx)
		cancelPF()
		if err != nil {
			return nil, fmt.Errorf("redirector identity store is not writable: %w", err)
		}
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
		drainBody(resp.Body)
		resp.Body.Close()
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
	saveCtx, cancelSave := context.WithTimeout(ctx, storeOpTimeout)
	err = store.Save(saveCtx, identity)
	cancelSave()
	if err != nil {
		if errors.Is(err, os.ErrExist) {
			winnerCtx, cancelWinner := context.WithTimeout(ctx, storeOpTimeout)
			winner, loadErr := store.Load(winnerCtx)
			cancelWinner()
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

func Renew(ctx context.Context, upstream string, identity *Identity, store Store) (*Identity, error) {
	cert, key, err := validate(identity)
	if err != nil {
		return nil, err
	}
	_, newKey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return nil, err
	}
	csr, err := x509.CreateCertificateRequest(rand.Reader, &x509.CertificateRequest{}, newKey)
	if err != nil {
		return nil, err
	}
	keyDER, err := x509.MarshalPKCS8PrivateKey(newKey)
	if err != nil {
		return nil, err
	}
	csrPEM := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csr}))
	body, err := json.Marshal(map[string]string{"csr": csrPEM})
	if err != nil {
		return nil, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, strings.TrimRight(upstream, "/")+"/auth/redirector/renew", bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	setRedirectorAuthHeaders(req, cert, key)
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		drainBody(resp.Body)
		return nil, fmt.Errorf("redirector renewal failed with HTTP %d", resp.StatusCode)
	}
	var result struct {
		ID          string `json:"redirector_id"`
		Certificate string `json:"mtls_cert"`
	}
	if err := json.NewDecoder(io.LimitReader(resp.Body, 1<<20)).Decode(&result); err != nil {
		return nil, fmt.Errorf("decode redirector renewal response: %w", err)
	}
	if result.ID != identity.ID {
		return nil, fmt.Errorf("renewed redirector identifier mismatch: got %q, want %q", result.ID, identity.ID)
	}
	renewed := &Identity{
		ID:          result.ID,
		Certificate: result.Certificate,
		PrivateKey:  string(pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER})),
	}
	if _, _, err := validate(renewed); err != nil {
		return nil, err
	}
	if store != nil {
		saveCtx, cancel := context.WithTimeout(ctx, storeOpTimeout)
		defer cancel()
		var saveErr error
		if replacer, ok := store.(Replacer); ok {
			saveErr = replacer.Replace(saveCtx, renewed)
		} else {
			saveErr = store.Save(saveCtx, renewed)
		}
		if saveErr != nil {
			return nil, fmt.Errorf("persist renewed redirector identity: %w", saveErr)
		}
	}
	return renewed, nil
}

func identityAbsent(err error) bool {
	return errors.Is(err, os.ErrNotExist) || status.Code(err) == codes.NotFound
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

func Checkin(ctx context.Context, upstream string, identity *Identity) error {
	cert, key, err := validate(identity)
	if err != nil {
		return err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, strings.TrimRight(upstream, "/")+"/auth/redirector/checkin", nil)
	if err != nil {
		return err
	}
	setRedirectorAuthHeaders(req, cert, key)
	resp, err := httpClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusNoContent {
		drainBody(resp.Body)
		if resp.StatusCode == http.StatusUnauthorized {
			return fmt.Errorf("%w: HTTP %d", errCheckinUnauthorized, resp.StatusCode)
		}
		return fmt.Errorf("redirector check-in failed with HTTP %d", resp.StatusCode)
	}
	return nil
}

func sameIdentity(a, b *Identity) bool {
	if a == nil || b == nil {
		return a == b
	}
	return a.ID == b.ID && a.Certificate == b.Certificate && a.PrivateKey == b.PrivateKey
}

func RunCheckins(ctx context.Context, upstream string, identity *Identity) {
	RunCheckinsWithStore(ctx, upstream, identity, nil)
}

func RunCheckinsWithStore(ctx context.Context, upstream string, identity *Identity, store Store) {
	current := identity
	consecutiveAuthFailures := 0

	check := func() {
		if ctx.Err() != nil {
			return
		}
		if store != nil && NeedsRenewal(current, time.Now()) {
			renewed, err := Renew(ctx, upstream, current, store)
			if err != nil {
				log.WarnContext(ctx, "redirector certificate renewal failed", "error", err)
			} else {
				current = renewed
				log.InfoContext(ctx, "redirector certificate renewed", "redirector_id", current.ID)
			}
		}
		err := Checkin(ctx, upstream, current)
		if err == nil {
			consecutiveAuthFailures = 0
			return
		}
		if errors.Is(err, errCheckinUnauthorized) && store != nil {
			loadCtx, cancel := context.WithTimeout(ctx, storeOpTimeout)
			reloaded, loadErr := store.Load(loadCtx)
			cancel()
			if loadErr == nil {
				if _, _, valErr := validate(reloaded); valErr == nil && !sameIdentity(current, reloaded) {
					current = reloaded
					err = Checkin(ctx, upstream, current)
					if err == nil {
						consecutiveAuthFailures = 0
						log.InfoContext(ctx, "reloaded redirector identity after check-in authentication failure", "redirector_id", current.ID)
						return
					}
				}
			}
		}
		if errors.Is(err, errCheckinUnauthorized) {
			consecutiveAuthFailures++
			if consecutiveAuthFailures >= 3 {
				log.ErrorContext(ctx, "redirector check-in persistently unauthorized", "consecutive_failures", consecutiveAuthFailures, "error", err)
				return
			}
			log.WarnContext(ctx, "redirector check-in unauthorized", "consecutive_failures", consecutiveAuthFailures, "error", err)
			return
		}
		log.WarnContext(ctx, "redirector check-in failed", "error", err)
	}

	check()
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
