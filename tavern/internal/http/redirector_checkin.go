package http

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/redirector"
	"realm.pub/tavern/internal/logging"
)

// Redirector check-ins use the same CA and signed timestamp proof as builder RPCs.
// The proof is carried in HTTP headers because redirector transports are not all gRPC.
const (
	redirectorCheckinMaxAge      = 5 * time.Minute
	maxRedirectorCheckinBody     = 4096
	maxRedirectorCertHeader      = 8192
	maxRedirectorTimestampHeader = 64
	maxRedirectorSignatureHeader = 128
)

// NewRedirectorCheckinHandler authenticates a redirector and updates its last_seen_at timestamp.
func NewRedirectorCheckinHandler(client *ent.Client, ca *x509.Certificate) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		if r.Method != http.MethodPost {
			rejectRedirectorAuth(w, ctx, "redirector.checkin_denied", http.StatusMethodNotAllowed, "method not allowed", "method_not_allowed")
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxRedirectorCheckinBody)
		id, now, ok := authenticateRedirectorRequest(w, r, ca, "redirector.checkin_denied")
		if !ok {
			return
		}
		updated, err := client.Redirector.Update().Where(redirector.Identifier(id)).SetLastSeenAt(now).Save(ctx)
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "redirector check-in failed", "error", err, "redirector_identifier", id)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if updated != 1 {
			rejectRedirectorAuth(w, ctx, "redirector.checkin_denied", http.StatusUnauthorized, "invalid credentials", "unknown_redirector", "redirector_identifier", id)
			return
		}
		checkinCtx := logging.WithActor(ctx, logging.Actor{
			Kind:       logging.ActorRedirector,
			Name:       id,
			AuthMethod: "mtls",
		})
		redirectorAuthLog.DebugContext(checkinCtx, "redirector check-in succeeded", "redirector_identifier", id)
		w.WriteHeader(http.StatusNoContent)
	}
}

// NewRedirectorRenewHandler authenticates an existing redirector and issues a renewed certificate for the same identifier.
func NewRedirectorRenewHandler(client *ent.Client, ca *x509.Certificate, caKey ed25519.PrivateKey) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		if r.Method != http.MethodPost {
			rejectRedirectorAuth(w, ctx, "redirector.renew_denied", http.StatusMethodNotAllowed, "method not allowed", "method_not_allowed")
			return
		}
		id, _, ok := authenticateRedirectorRequest(w, r, ca, "redirector.renew_denied")
		if !ok {
			return
		}
		rec, err := client.Redirector.Query().Where(redirector.Identifier(id)).Only(ctx)
		if ent.IsNotFound(err) {
			rejectRedirectorAuth(w, ctx, "redirector.renew_denied", http.StatusUnauthorized, "invalid credentials", "unknown_redirector", "redirector_identifier", id)
			return
		}
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to load redirector for renewal", "error", err, "redirector_identifier", id)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxRedirectorEnrollBody)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var req struct {
			CSR string `json:"csr"`
		}
		if err := decoder.Decode(&req); err != nil {
			rejectRedirectorAuth(w, ctx, "redirector.renew_denied", http.StatusBadRequest, "invalid request body", "invalid_body", "redirector_identifier", id)
			return
		}
		if err := decoder.Decode(new(any)); !errors.Is(err, io.EOF) {
			rejectRedirectorAuth(w, ctx, "redirector.renew_denied", http.StatusBadRequest, "invalid request body", "invalid_body", "redirector_identifier", id)
			return
		}
		if req.CSR == "" {
			rejectRedirectorAuth(w, ctx, "redirector.renew_denied", http.StatusBadRequest, "invalid request body", "missing_csr", "redirector_identifier", id)
			return
		}
		cert, err := builder.SignRedirectorCertificate(ca, caKey, id, req.CSR)
		if errors.Is(err, builder.ErrInvalidRedirectorCSR) {
			rejectRedirectorAuth(w, ctx, "redirector.renew_denied", http.StatusBadRequest, "invalid redirector CSR", "invalid_csr", "redirector_identifier", id)
			return
		}
		if err != nil {
			redirectorAuthLog.ErrorContext(ctx, "failed to sign renewed redirector certificate", "error", err, "redirector_identifier", id)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		renewCtx := logging.WithActor(ctx, logging.Actor{
			Kind:       logging.ActorRedirector,
			ID:         strconv.Itoa(rec.ID),
			Name:       id,
			AuthMethod: "mtls",
		})
		redirectorAuthLog.InfoContext(renewCtx, "redirector certificate renewed", "redirector_id", rec.ID, "redirector_identifier", id)
		logging.Audit(renewCtx, "auth", "redirector.renewed", "renewed redirector certificate", "redirector_id", rec.ID, "redirector_identifier", id)
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(RedirectorEnrollResponse{RedirectorID: id, MTLSCert: string(cert)})
	}
}

func authenticateRedirectorRequest(w http.ResponseWriter, r *http.Request, ca *x509.Certificate, deniedAction string) (id string, now time.Time, ok bool) {
	ctx := r.Context()
	if ca == nil {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "ca_unconfigured")
		return "", time.Time{}, false
	}
	certHeader := r.Header.Get("X-Realm-Redirector-Cert")
	timestamp := r.Header.Get("X-Realm-Redirector-Timestamp")
	sigHeader := r.Header.Get("X-Realm-Redirector-Signature")
	if certHeader == "" || len(certHeader) > maxRedirectorCertHeader {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "missing_or_oversized_cert")
		return "", time.Time{}, false
	}
	if timestamp == "" || len(timestamp) > maxRedirectorTimestampHeader {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "missing_or_oversized_timestamp")
		return "", time.Time{}, false
	}
	if sigHeader == "" || len(sigHeader) > maxRedirectorSignatureHeader {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "missing_or_oversized_signature")
		return "", time.Time{}, false
	}
	certDER, err := base64.StdEncoding.DecodeString(certHeader)
	if err != nil || len(certDER) == 0 {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_cert")
		return "", time.Time{}, false
	}
	cert, err := x509.ParseCertificate(certDER)
	if err != nil {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_cert")
		return "", time.Time{}, false
	}
	if err := cert.CheckSignatureFrom(ca); err != nil {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "untrusted_ca")
		return "", time.Time{}, false
	}
	if cert.IsCA {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "ca_cert_not_allowed")
		return "", time.Time{}, false
	}
	if !containsClientAuth(cert.ExtKeyUsage) {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "missing_client_auth_eku")
		return "", time.Time{}, false
	}
	if cert.KeyUsage&x509.KeyUsageDigitalSignature == 0 {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "missing_digital_signature_ku")
		return "", time.Time{}, false
	}
	now = time.Now()
	if now.Before(cert.NotBefore) || now.After(cert.NotAfter) {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "cert_expired_or_not_yet_valid")
		return "", time.Time{}, false
	}
	if !strings.HasPrefix(cert.Subject.CommonName, "redirector-") {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_cn")
		return "", time.Time{}, false
	}
	id = strings.TrimPrefix(cert.Subject.CommonName, "redirector-")
	if id == "" {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_cn")
		return "", time.Time{}, false
	}
	ts, err := time.Parse(time.RFC3339Nano, timestamp)
	if err != nil {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_timestamp", "redirector_identifier", id)
		return "", time.Time{}, false
	}
	if now.Sub(ts).Abs() > redirectorCheckinMaxAge {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "stale_timestamp", "redirector_identifier", id)
		return "", time.Time{}, false
	}
	sig, err := base64.StdEncoding.DecodeString(sigHeader)
	if err != nil || len(sig) != ed25519.SignatureSize {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_signature", "redirector_identifier", id)
		return "", time.Time{}, false
	}
	pub, ok := cert.PublicKey.(ed25519.PublicKey)
	if !ok || len(pub) != ed25519.PublicKeySize || !ed25519.Verify(pub, []byte(timestamp), sig) {
		rejectRedirectorAuth(w, ctx, deniedAction, http.StatusUnauthorized, "invalid credentials", "invalid_signature", "redirector_identifier", id)
		return "", time.Time{}, false
	}
	return id, now, true
}

func rejectRedirectorAuth(w http.ResponseWriter, ctx context.Context, action string, status int, message, reason string, extra ...any) {
	attrs := append([]any{"reason", reason}, extra...)
	redirectorAuthLog.WarnContext(ctx, message, attrs...)
	logging.AuditDenied(ctx, "auth", action, message, attrs...)
	http.Error(w, message, status)
}

func containsClientAuth(usages []x509.ExtKeyUsage) bool {
	for _, usage := range usages {
		if usage == x509.ExtKeyUsageClientAuth {
			return true
		}
	}
	return false
}
