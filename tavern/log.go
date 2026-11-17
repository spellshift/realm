package main

import (
	"context"
	"errors"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/logging"
)

// DefaultLogSpec sets the compiled-in logging preset when neither `--log` nor `TAVERN_LOG` is provided.
// Release binaries and production containers override this at link time via `-ldflags '-X main.DefaultLogSpec=prod'`.
var DefaultLogSpec = "dev"

var (
	startupLog     = logging.For("app.startup")
	grpcAccessLog  = logging.For("grpc.access")
	gqlQueryLog    = logging.For("graphql.query")
	gqlMutationLog = logging.For("graphql.mutation")
)

func configureLogging(cliSpec, cliFormat string) error {
	spec := strings.TrimSpace(cliSpec)
	if spec == "" {
		if envSpec := strings.TrimSpace(EnvLogSpec.String()); envSpec != "" {
			spec = envSpec
		} else if EnvDebugLogging.IsSet() {
			spec = "*=debug"
		} else {
			spec = DefaultLogSpec
		}
	}

	rawFormat := strings.TrimSpace(cliFormat)
	if rawFormat == "" {
		rawFormat = strings.TrimSpace(EnvLogFormat.String())
	}
	format, err := logging.ParseFormat(rawFormat)
	if err != nil {
		return err
	}
	if format == logging.FormatAuto && EnvJSONLogging.IsSet() {
		format = logging.FormatJSON
	}

	var includeInstanceID *bool
	if EnvLogInstanceID.IsSet() {
		v := true
		includeInstanceID = &v
	}

	var logGraphQLRawQuery *bool
	if EnvLogGraphQLRawQuery.IsSet() {
		v := true
		logGraphQLRawQuery = &v
	}

	return logging.Configure(logging.Options{
		Spec:               spec,
		Format:             format,
		AuditFilePath:      EnvAuditFile.String(),
		InstanceID:         FormatResourceName(GlobalInstanceID),
		IncludeInstanceID:  includeInstanceID,
		LogGraphQLRawQuery: logGraphQLRawQuery,
		GCPProjectID:       EnvGCPProjectID.String(),
	})
}

type wrappedServerStream struct {
	grpc.ServerStream
	ctx context.Context
}

func (w *wrappedServerStream) Context() context.Context {
	return w.ctx
}

func enrichGRPCContext(ctx context.Context) (context.Context, string, string) {
	ctx, traceID, reqID := logging.ExtractGRPCTrace(ctx)
	ctx = logging.WithEntrypoint(ctx, "grpc")
	if logging.ClientIPFromContext(ctx) == "" {
		if p, ok := peer.FromContext(ctx); ok && p != nil && p.Addr != nil {
			ctx = logging.WithClientIP(ctx, logging.NormalizeClientIP(p.Addr.String(), "", ""))
		}
	}
	return ctx, traceID, reqID
}

func logCompletedGRPC(ctx context.Context, fullMethod string, duration time.Duration, err error) {
	code := status.Code(err)
	if err != nil && code == codes.Unknown && errors.Is(err, context.Canceled) {
		code = codes.Canceled
	}

	attrs := []any{
		"rpc_method", fullMethod,
		"grpc_code", code.String(),
		"duration_ms", duration.Milliseconds(),
	}
	if err != nil && code != codes.Canceled {
		attrs = append(attrs, "error", err)
	}

	switch code {
	case codes.OK, codes.Canceled:
		grpcAccessLog.InfoContext(ctx, "grpc request completed", attrs...)
	case codes.Internal, codes.Unknown, codes.DataLoss, codes.Unavailable:
		grpcAccessLog.ErrorContext(ctx, "grpc request failed", attrs...)
	default:
		grpcAccessLog.WarnContext(ctx, "grpc request rejected", attrs...)
	}
}

func grpcWithUnaryLogging(
	ctx context.Context,
	req any,
	info *grpc.UnaryServerInfo,
	handler grpc.UnaryHandler,
) (any, error) {
	start := time.Now()
	ctx, traceID, reqID := enrichGRPCContext(ctx)
	_ = grpc.SetHeader(ctx, metadata.Pairs(
		logging.MetadataTraceIDKey, traceID,
		logging.MetadataReqIDKey, reqID,
	))

	resp, err := handler(ctx, req)
	logCompletedGRPC(ctx, info.FullMethod, time.Since(start), err)
	return resp, err
}

func grpcWithStreamLogging(
	srv any,
	ss grpc.ServerStream,
	info *grpc.StreamServerInfo,
	handler grpc.StreamHandler,
) error {
	start := time.Now()
	ctx, traceID, reqID := enrichGRPCContext(ss.Context())
	_ = ss.SetHeader(metadata.Pairs(
		logging.MetadataTraceIDKey, traceID,
		logging.MetadataReqIDKey, reqID,
	))

	err := handler(srv, &wrappedServerStream{
		ServerStream: ss,
		ctx:          ctx,
	})
	logCompletedGRPC(ctx, info.FullMethod, time.Since(start), err)
	return err
}
