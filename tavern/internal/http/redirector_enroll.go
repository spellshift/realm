package http

import (
	"crypto/ed25519"
	"crypto/sha256"
	"crypto/x509"
	"encoding/hex"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent"
	entredirector "realm.pub/tavern/internal/ent/redirector"
	"realm.pub/tavern/internal/ent/redirectorenrollmenttoken"
)

const maxRedirectorEnrollBody = 1 << 20

type RedirectorEnrollRequest struct {
	Token     string   `json:"token"`
	CSR       string   `json:"csr"`
	Transport string   `json:"transport"`
	Domains   []string `json:"domains"`
}

type RedirectorEnrollResponse struct {
	RedirectorID string `json:"redirector_id"`
	MTLSCert     string `json:"mtls_cert"`
}

// NewRedirectorEnrollHandler exchanges a one-time token for a redirector identity.
func NewRedirectorEnrollHandler(client *ent.Client, ca *x509.Certificate, caKey ed25519.PrivateKey) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxRedirectorEnrollBody)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var req RedirectorEnrollRequest
		if decoder.Decode(&req) != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if err := decoder.Decode(new(any)); !errors.Is(err, io.EOF) {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if req.Token == "" || len(req.Token) > 4096 || req.CSR == "" || !validRedirectorTransport(req.Transport) {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		domains, ok := sanitizeRedirectorDomains(req.Domains)
		if !ok {
			http.Error(w, "invalid domains", http.StatusBadRequest)
			return
		}
		identifier := uuid.NewString()
		cert, err := builder.SignRedirectorCertificate(ca, caKey, identifier, req.CSR)
		if errors.Is(err, builder.ErrInvalidBuilderCSR) {
			http.Error(w, "invalid redirector CSR", http.StatusBadRequest)
			return
		}
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		hash := sha256.Sum256([]byte(req.Token))
		tokenHash := hex.EncodeToString(hash[:])
		ctx := r.Context()
		tx, err := client.Tx(ctx)
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		defer tx.Rollback()
		n, err := tx.RedirectorEnrollmentToken.Update().Where(redirectorenrollmenttoken.TokenHash(tokenHash), redirectorenrollmenttoken.Used(false)).SetUsed(true).Save(ctx)
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if n != 1 {
			http.Error(w, "invalid or used redirector enrollment token", http.StatusUnauthorized)
			return
		}
		_, err = tx.Redirector.Create().SetIdentifier(identifier).SetTransport(entredirector.Transport(req.Transport)).SetDomains(domains).Save(ctx)
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if err := tx.Commit(); err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(RedirectorEnrollResponse{RedirectorID: identifier, MTLSCert: string(cert)})
	}
}

func validRedirectorTransport(transport string) bool {
	switch transport {
	case "http1", "grpc", "dns", "icmp", "quic":
		return true
	}
	return false
}

func sanitizeRedirectorDomains(input []string) ([]string, bool) {
	if len(input) == 0 || len(input) > 32 {
		return nil, false
	}
	domains := make([]string, 0, len(input))
	seen := make(map[string]bool, len(input))
	for _, raw := range input {
		domain := strings.ToLower(strings.TrimSpace(raw))
		if !validDomain(domain) {
			return nil, false
		}
		if !seen[domain] {
			domains = append(domains, domain)
			seen[domain] = true
		}
	}
	return domains, true
}

func validDomain(domain string) bool {
	if len(domain) == 0 || len(domain) > 253 || !strings.Contains(domain, ".") {
		return false
	}
	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 || len(label) > 63 || label[0] == '-' || label[len(label)-1] == '-' {
			return false
		}
		for _, c := range label {
			if !(c >= 'a' && c <= 'z' || c >= '0' && c <= '9' || c == '-') {
				return false
			}
		}
	}
	return true
}
