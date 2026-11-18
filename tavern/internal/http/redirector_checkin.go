package http

import (
	"crypto/ed25519"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/redirector"
)

// Redirector check-ins use the same CA and signed timestamp proof as builder RPCs.
// The proof is carried in HTTP headers because redirector transports are not all gRPC.
const redirectorCheckinMaxAge = 5 * time.Minute

func NewRedirectorCheckinHandler(client *ent.Client, ca *x509.Certificate) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		certDER, err := base64.StdEncoding.DecodeString(r.Header.Get("X-Realm-Redirector-Cert"))
		if err != nil || len(certDER) == 0 || ca == nil {
			http.Error(w, "invalid credentials", http.StatusUnauthorized)
			return
		}
		cert, err := x509.ParseCertificate(certDER)
		if err != nil || cert.CheckSignatureFrom(ca) != nil || cert.IsCA || !containsClientAuth(cert.ExtKeyUsage) {
			http.Error(w, "invalid credentials", http.StatusUnauthorized)
			return
		}
		now := time.Now()
		if now.Before(cert.NotBefore) || now.After(cert.NotAfter) || !strings.HasPrefix(cert.Subject.CommonName, "redirector-") {
			http.Error(w, "invalid credentials", http.StatusUnauthorized)
			return
		}
		timestamp := r.Header.Get("X-Realm-Redirector-Timestamp")
		ts, err := time.Parse(time.RFC3339Nano, timestamp)
		if err != nil || now.Sub(ts).Abs() > redirectorCheckinMaxAge {
			http.Error(w, "invalid credentials", http.StatusUnauthorized)
			return
		}
		sig, err := base64.StdEncoding.DecodeString(r.Header.Get("X-Realm-Redirector-Signature"))
		pub, ok := cert.PublicKey.(ed25519.PublicKey)
		if err != nil || !ok || !ed25519.Verify(pub, []byte(timestamp), sig) {
			http.Error(w, "invalid credentials", http.StatusUnauthorized)
			return
		}
		id := strings.TrimPrefix(cert.Subject.CommonName, "redirector-")
		updated, err := client.Redirector.Update().Where(redirector.Identifier(id)).SetLastSeenAt(now).Save(r.Context())
		if err != nil {
			redirectorAuthLog.ErrorContext(r.Context(), "redirector check-in failed", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if updated != 1 {
			http.Error(w, "invalid credentials", http.StatusUnauthorized)
			return
		}
		// Read enabled after the heartbeat write so an admin toggle committed
		// during that write is reflected in this response.
		row, err := client.Redirector.Query().Where(redirector.Identifier(id)).Only(r.Context())
		if err != nil {
			redirectorAuthLog.ErrorContext(r.Context(), "redirector check-in failed", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(struct {
			Enabled bool `json:"enabled"`
		}{row.Enabled})
	}
}

func containsClientAuth(usages []x509.ExtKeyUsage) bool {
	for _, usage := range usages {
		if usage == x509.ExtKeyUsageClientAuth {
			return true
		}
	}
	return false
}
