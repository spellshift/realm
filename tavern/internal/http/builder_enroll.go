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

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/enrollment"
	"realm.pub/tavern/internal/ent"
	entbuilder "realm.pub/tavern/internal/ent/builder"
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
			rejectBuilderEnrollment(w, ctx, http.StatusMethodNotAllowed, "method not allowed", "method_not_allowed")
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxBuilderEnrollBody)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var req BuilderEnrollRequest
		if err := decoder.Decode(&req); err != nil {
			rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "invalid_body")
			return
		}
		if err := decoder.Decode(new(any)); !errors.Is(err, io.EOF) {
			rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "invalid_body")
			return
		}
		if req.Token == "" {
			rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "missing_token")
			return
		}
		if len(req.Token) > 4096 {
			rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "token_too_long")
			return
		}
		if req.CSR == "" {
			rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid request body", "missing_csr")
			return
		}
		valid, err := enrollment.Available(ctx, client, enrollment.Builder, req.Token)
		if err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to verify builder enrollment token", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if !valid {
			rejectBuilderEnrollment(w, ctx, http.StatusUnauthorized, "invalid or used builder enrollment token", "invalid_or_used_token")
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
				rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid builder settings", "invalid_target")
				return
			}
		}
		pollInterval := builder.DefaultPollInterval
		if req.PollInterval != nil {
			pollInterval = *req.PollInterval
		}
		cfg := builder.Config{SupportedTargets: req.SupportedTargets, Upstream: req.Upstream, PollInterval: pollInterval, Executor: req.Executor, IdentityFile: "builder.identity"}
		if err := cfg.Validate(); err != nil {
			rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid builder settings", "invalid_config")
			return
		}
		identifier := uuid.NewString()
		cert, err := builder.SignBuilderCertificate(ca, caKey, identifier, req.CSR)
		if err != nil {
			if errors.Is(err, builder.ErrInvalidBuilderCSR) {
				rejectBuilderEnrollment(w, ctx, http.StatusBadRequest, "invalid builder CSR", "invalid_csr")
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
		consumed, err := enrollment.Consume(ctx, tx, enrollment.Builder, req.Token)
		if err != nil {
			builderAuthLog.ErrorContext(ctx, "failed to mark builder enrollment token used", "error", err)
			http.Error(w, "internal server error", http.StatusInternalServerError)
			return
		}
		if !consumed {
			rejectBuilderEnrollment(w, ctx, http.StatusUnauthorized, "invalid or used builder enrollment token", "invalid_or_used_token")
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

func rejectBuilderEnrollment(w http.ResponseWriter, ctx context.Context, status int, message, reason string) {
	builderAuthLog.WarnContext(ctx, "builder enrollment rejected", "reason", reason)
	logging.AuditDenied(ctx, "auth", "builder.enroll_denied", "builder enrollment rejected", "reason", reason)
	http.Error(w, message, status)
}
