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

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	entbuilder "realm.pub/tavern/internal/ent/builder"
	"realm.pub/tavern/internal/ent/builderenrollmenttoken"
)

const maxBuilderEnrollBody = 1 << 20

type BuilderEnrollRequest struct {
	Token            string   `json:"token"`
	CSR              string   `json:"csr"`
	SupportedTargets []string `json:"supported_targets"`
	Upstream         string   `json:"upstream"`
	PollInterval     int      `json:"poll_interval"`
	Executor         string   `json:"executor"`
}

type BuilderEnrollResponse struct {
	BuilderID string `json:"builder_id"`
	MTLSCert  string `json:"mtls_cert"`
}

func NewBuilderEnrollHandler(client *ent.Client, ca *x509.Certificate, caKey ed25519.PrivateKey) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxBuilderEnrollBody)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var req BuilderEnrollRequest
		if err := decoder.Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if err := decoder.Decode(new(any)); !errors.Is(err, io.EOF) {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if req.Token == "" || req.CSR == "" {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		hash := sha256.Sum256([]byte(req.Token))
		tokenHash := hex.EncodeToString(hash[:])
		valid, err := client.BuilderEnrollmentToken.Query().Where(builderenrollmenttoken.TokenHash(tokenHash), builderenrollmenttoken.Used(false)).Exist(r.Context())
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if !valid {
			http.Error(w, "invalid or used builder enrollment token", http.StatusUnauthorized)
			return
		}
		targets := make([]c2pb.Host_Platform, len(req.SupportedTargets))
		for i, target := range req.SupportedTargets {
			switch target {
			case "linux":
				targets[i] = c2pb.Host_PLATFORM_LINUX
			case "macos":
				targets[i] = c2pb.Host_PLATFORM_MACOS
			case "windows":
				targets[i] = c2pb.Host_PLATFORM_WINDOWS
			default:
				http.Error(w, "invalid builder settings", http.StatusBadRequest)
				return
			}
		}
		cfg := builder.Config{SupportedTargets: req.SupportedTargets, Upstream: req.Upstream, PollInterval: req.PollInterval, Executor: req.Executor, IdentityFile: "builder.identity"}
		if err := cfg.Validate(); err != nil {
			http.Error(w, "invalid builder settings", http.StatusBadRequest)
			return
		}
		identifier := uuid.NewString()
		cert, err := builder.SignBuilderCertificate(ca, caKey, identifier, req.CSR)
		if err != nil {
			if ca == nil || len(caKey) != ed25519.PrivateKeySize {
				http.Error(w, "internal server error", http.StatusInternalServerError)
				return
			}
			http.Error(w, "invalid builder CSR", http.StatusBadRequest)
			return
		}
		tx, err := client.Tx(r.Context())
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		defer tx.Rollback()
		n, err := tx.BuilderEnrollmentToken.Update().Where(builderenrollmenttoken.TokenHash(tokenHash), builderenrollmenttoken.Used(false)).SetUsed(true).Save(r.Context())
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if n != 1 {
			http.Error(w, "invalid or used builder enrollment token", http.StatusUnauthorized)
			return
		}
		_, err = tx.Builder.Create().SetIdentifier(identifier).SetSupportedTargets(targets).SetUpstream(req.Upstream).SetPollInterval(req.PollInterval).SetExecutor(entbuilder.Executor(cfg.Executor)).Save(r.Context())
		if err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if err := tx.Commit(); err != nil {
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(BuilderEnrollResponse{BuilderID: identifier, MTLSCert: string(cert)})
	}
}
