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
	"strconv"

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	entbuilder "realm.pub/tavern/internal/ent/builder"
	"realm.pub/tavern/internal/ent/builderenrollmenttoken"
	"realm.pub/tavern/internal/logging"
)

var builderAuthLog = logging.For("builder.auth")

const maxBuilderEnrollBody = 1 << 20

type BuilderEnrollRequest struct {
	Token            string   `json:"token"`
	CSR              string   `json:"csr"`
	SupportedTargets []string `json:"supported_targets"`
	Upstream         string   `json:"upstream"`
	PollInterval     *int     `json:"poll_interval"`
	Executor         string   `json:"executor"`
}

type BuilderEnrollResponse struct {
	BuilderID string `json:"builder_id"`
	MTLSCert  string `json:"mtls_cert"`
}

func NewBuilderEnrollHandler(client *ent.Client, ca *x509.Certificate, caKey ed25519.PrivateKey) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxBuilderEnrollBody)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var req BuilderEnrollRequest
		if err := decoder.Decode(&req); err != nil {
			builderAuthLog.DebugContext(ctx, "invalid builder enrollment json payload", "error", err)
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if err := decoder.Decode(new(any)); !errors.Is(err, io.EOF) {
			builderAuthLog.DebugContext(ctx, "trailing data in builder enrollment payload")
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if req.Token == "" {
			builderAuthLog.WarnContext(ctx, "builder enrollment rejected: missing token")
			logging.AuditDenied(ctx, "auth", "builder.enroll_denied", "builder enrollment missing token", "reason", "missing_token")
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		if req.CSR == "" {
			builderAuthLog.WarnContext(ctx, "builder enrollment rejected: missing csr")
			logging.AuditDenied(ctx, "auth", "builder.enroll_denied", "builder enrollment missing csr", "reason", "missing_csr")
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}
		hash := sha256.Sum256([]byte(req.Token))
		tokenHash := hex.EncodeToString(hash[:])
		valid, err := client.BuilderEnrollmentToken.Query().Where(builderenrollmenttoken.TokenHash(tokenHash), builderenrollmenttoken.Used(false)).Exist(ctx)
		if err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to verify builder enrollment token", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if !valid {
			builderAuthLog.WarnContext(ctx, "builder enrollment rejected: invalid or used enrollment token")
			logging.AuditDenied(ctx, "auth", "builder.enroll_denied", "invalid or used builder enrollment token", "reason", "invalid_or_used_token")
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
				builderAuthLog.WarnContext(ctx, "builder enrollment rejected: unsupported target platform", "target", target)
				http.Error(w, "invalid builder settings", http.StatusBadRequest)
				return
			}
		}
		pollInterval := builder.DefaultPollInterval
		if req.PollInterval != nil {
			pollInterval = *req.PollInterval
		}
		cfg := builder.Config{SupportedTargets: req.SupportedTargets, Upstream: req.Upstream, PollInterval: pollInterval, Executor: req.Executor, IdentityFile: "builder.identity"}
		if err := cfg.Validate(); err != nil {
			builderAuthLog.WarnContext(ctx, "builder enrollment rejected: invalid builder config", "error", err)
			http.Error(w, "invalid builder settings", http.StatusBadRequest)
			return
		}
		identifier := uuid.NewString()
		cert, err := builder.SignBuilderCertificate(ca, caKey, identifier, req.CSR)
		if err != nil {
			if errors.Is(err, builder.ErrInvalidBuilderCSR) {
				builderAuthLog.WarnContext(ctx, "builder enrollment rejected: invalid csr")
				http.Error(w, "invalid builder CSR", http.StatusBadRequest)
				return
			}
			builderAuthLog.ErrorContext(ctx, "failed to sign builder certificate", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		tx, err := client.Tx(ctx)
		if err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to start builder enrollment transaction", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		defer tx.Rollback()
		n, err := tx.BuilderEnrollmentToken.Update().Where(builderenrollmenttoken.TokenHash(tokenHash), builderenrollmenttoken.Used(false)).SetUsed(true).Save(ctx)
		if err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to mark builder enrollment token used", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if n != 1 {
			builderAuthLog.WarnContext(ctx, "builder enrollment rejected: enrollment token concurrently consumed")
			logging.AuditDenied(ctx, "auth", "builder.enroll_denied", "invalid or used builder enrollment token", "reason", "invalid_or_used_token")
			http.Error(w, "invalid or used builder enrollment token", http.StatusUnauthorized)
			return
		}
		createdBuilder, err := tx.Builder.Create().SetIdentifier(identifier).SetSupportedTargets(targets).SetUpstream(req.Upstream).SetPollInterval(pollInterval).SetExecutor(entbuilder.Executor(cfg.Executor)).Save(ctx)
		if err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to create builder record", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if err := tx.Commit(); err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to commit builder enrollment transaction", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}

		enrollCtx := logging.WithActor(ctx, logging.Actor{
			Kind:       logging.ActorBuilder,
			ID:         strconv.Itoa(createdBuilder.ID),
			Name:       identifier,
			AuthMethod: "enrollment_token",
		})
		builderAuthLog.InfoContext(enrollCtx, "builder enrolled successfully",
			"builder_id", createdBuilder.ID,
			"builder_identifier", identifier,
			"supported_targets", req.SupportedTargets,
			"executor", cfg.Executor,
		)
		logging.Audit(enrollCtx, "auth", "builder.enrolled", "enrolled new builder",
			"builder_id", createdBuilder.ID,
			"builder_identifier", identifier,
			"supported_targets", req.SupportedTargets,
			"executor", cfg.Executor,
		)

		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(BuilderEnrollResponse{BuilderID: identifier, MTLSCert: string(cert)})
	}
}
