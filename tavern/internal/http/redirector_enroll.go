package http

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strconv"
	"strings"

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/enrollment"
	"realm.pub/tavern/internal/ent"
	entredirector "realm.pub/tavern/internal/ent/redirector"
	"realm.pub/tavern/internal/logging"
)

const maxRedirectorEnrollBody = 1 << 20

var redirectorAuthLog = logging.For("redirector.auth")

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
		ctx := r.Context()
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxRedirectorEnrollBody)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var req RedirectorEnrollRequest
		if err := decoder.Decode(&req); err != nil {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "invalid_body")
			return
		}
		if err := decoder.Decode(new(any)); !errors.Is(err, io.EOF) {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "invalid_body")
			return
		}
		if req.Token == "" || len(req.Token) > 4096 {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "invalid_token")
			return
		}
		if req.CSR == "" {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "missing_csr")
			return
		}
		if !validRedirectorTransport(req.Transport) {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid transport", "invalid_transport")
			return
		}
		domains, ok := sanitizeRedirectorDomains(req.Domains)
		if !ok {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid domains", "invalid_domains")
			return
		}
		valid, err := enrollment.Available(ctx, client, enrollment.Redirector, req.Token)
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to verify redirector enrollment token", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if !valid {
			rejectRedirectorEnrollment(w, ctx, http.StatusUnauthorized, "invalid or used redirector enrollment token", "invalid_or_used_token")
			return
		}
		identifier := uuid.NewString()
		cert, err := builder.SignRedirectorCertificate(ca, caKey, identifier, req.CSR)
		if errors.Is(err, builder.ErrInvalidRedirectorCSR) {
			rejectRedirectorEnrollment(w, ctx, http.StatusBadRequest, "invalid redirector CSR", "invalid_csr")
			return
		}
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to sign redirector certificate", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		tx, err := client.Tx(ctx)
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to start redirector enrollment transaction", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		defer tx.Rollback()
		consumed, err := enrollment.Consume(ctx, tx, enrollment.Redirector, req.Token)
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to mark redirector enrollment token used", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if !consumed {
			rejectRedirectorEnrollment(w, ctx, http.StatusUnauthorized, "invalid or used redirector enrollment token", "invalid_or_used_token")
			return
		}
		created, err := tx.Redirector.Create().SetIdentifier(identifier).SetTransport(entredirector.Transport(req.Transport)).SetDomains(domains).Save(ctx)
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to create redirector record", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if err := tx.Commit(); err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to commit redirector enrollment transaction", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		enrollCtx := logging.WithActor(ctx, logging.Actor{
			Kind: logging.ActorRedirector, ID: strconv.Itoa(created.ID),
			Name: identifier, AuthMethod: "enrollment_token",
		})
		redirectorAuthLog.InfoContext(enrollCtx, "redirector enrolled successfully", "redirector_id", created.ID, "redirector_identifier", identifier)
		logging.Audit(enrollCtx, "auth", "redirector.enrolled", "enrolled new redirector",
			"redirector_id", created.ID, "redirector_identifier", identifier,
			"transport", req.Transport, "domains", domains)
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(RedirectorEnrollResponse{RedirectorID: identifier, MTLSCert: string(cert)})
	}
}

func rejectRedirectorEnrollment(w http.ResponseWriter, ctx context.Context, status int, message, reason string) {
	redirectorAuthLog.WarnContext(ctx, "redirector enrollment rejected", "reason", reason)
	logging.AuditDenied(ctx, "auth", "redirector.enroll_denied", "redirector enrollment rejected", "reason", reason)
	http.Error(w, message, status)
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
	hasLetter := false
	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 || len(label) > 63 || label[0] == '-' || label[len(label)-1] == '-' {
			return false
		}
		for _, c := range label {
			if !(c >= 'a' && c <= 'z' || c >= '0' && c <= '9' || c == '-') {
				return false
			}
			hasLetter = hasLetter || c >= 'a' && c <= 'z'
		}
	}
	return hasLetter
}
