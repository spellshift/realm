package builder

import (
	"context"
	"crypto/ed25519"
	"crypto/x509"
	"encoding/base64"
	"log/slog"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	"realm.pub/tavern/internal/ent"
	entbuilder "realm.pub/tavern/internal/ent/builder"
)

const (
	// Metadata keys for mTLS authentication.
	mdKeyBuilderCert      = "builder-cert"
	mdKeyBuilderSignature = "builder-signature"
	mdKeyBuilderTimestamp  = "builder-timestamp"

	// Maximum age for a timestamp to be considered valid.
	maxTimestampAge = 5 * time.Minute

	// CN prefix used in builder certificates.
	builderCNPrefix = "builder-"
)

type builderContextKey struct{}

// BuilderFromContext extracts the authenticated builder entity from the context.
func BuilderFromContext(ctx context.Context) (*ent.Builder, bool) {
	b, ok := ctx.Value(builderContextKey{}).(*ent.Builder)
	return b, ok
}

// NewAuthInterceptor creates a gRPC unary server interceptor that validates
// builder mTLS credentials. It verifies:
// 1. The certificate was signed by the provided CA
// 2. The signature proves possession of the corresponding private key
// 3. The timestamp is recent (prevents replay)
// 4. The builder identifier from the CN exists in the database
func NewAuthInterceptor(caCert *x509.Certificate, graph *ent.Client) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return nil, status.Error(codes.Unauthenticated, "missing metadata")
		}

		// Extract metadata values
		certB64 := getMetadataValue(md, mdKeyBuilderCert)
		sigB64 := getMetadataValue(md, mdKeyBuilderSignature)
		timestamp := getMetadataValue(md, mdKeyBuilderTimestamp)

		if certB64 == "" || sigB64 == "" || timestamp == "" {
			return nil, status.Error(codes.Unauthenticated, "missing builder credentials")
		}

		// Parse the certificate
		certDER, err := base64.StdEncoding.DecodeString(certB64)
		if err != nil {
			return nil, status.Error(codes.Unauthenticated, "invalid certificate encoding")
		}

		cert, err := x509.ParseCertificate(certDER)
		if err != nil {
			return nil, status.Error(codes.Unauthenticated, "invalid certificate")
		}

		// Verify certificate was signed by our CA
		if err := cert.CheckSignatureFrom(caCert); err != nil {
			return nil, status.Error(codes.Unauthenticated, "certificate not signed by trusted CA")
		}

		// Verify certificate validity period
		now := time.Now()
		if now.Before(cert.NotBefore) || now.After(cert.NotAfter) {
			return nil, status.Error(codes.Unauthenticated, "certificate expired or not yet valid")
		}

		// Verify timestamp freshness
		ts, err := time.Parse(time.RFC3339Nano, timestamp)
		if err != nil {
			return nil, status.Error(codes.Unauthenticated, "invalid timestamp format")
		}
		if now.Sub(ts).Abs() > maxTimestampAge {
			return nil, status.Error(codes.Unauthenticated, "timestamp too old or too far in the future")
		}

		// Verify signature (proof of private key possession)
		sigBytes, err := base64.StdEncoding.DecodeString(sigB64)
		if err != nil {
			return nil, status.Error(codes.Unauthenticated, "invalid signature encoding")
		}

		pubKey, ok := cert.PublicKey.(ed25519.PublicKey)
		if !ok {
			return nil, status.Error(codes.Unauthenticated, "certificate does not contain ED25519 public key")
		}
		if !ed25519.Verify(pubKey, []byte(timestamp), sigBytes) {
			return nil, status.Error(codes.Unauthenticated, "invalid signature")
		}

		// Extract builder identifier from CN
		cn := cert.Subject.CommonName
		if !strings.HasPrefix(cn, builderCNPrefix) {
			return nil, status.Error(codes.Unauthenticated, "invalid certificate CN format")
		}
		identifier := strings.TrimPrefix(cn, builderCNPrefix)

		// Look up builder in database
		b, err := graph.Builder.Query().Where(entbuilder.IdentifierEQ(identifier)).Only(ctx)
		if err != nil {
			slog.WarnContext(ctx, "builder authentication failed: builder not found", "identifier", identifier, "error", err)
			return nil, status.Error(codes.Unauthenticated, "builder not found")
		}

		slog.InfoContext(ctx, "builder authenticated", "builder_id", b.ID, "identifier", identifier)

		// Store builder in context for downstream handlers
		ctx = context.WithValue(ctx, builderContextKey{}, b)
		return handler(ctx, req)
	}
}

func getMetadataValue(md metadata.MD, key string) string {
	values := md.Get(key)
	if len(values) == 0 {
		return ""
	}
	return values[0]
}
