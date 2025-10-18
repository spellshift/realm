package main

import (
	"context"
	"crypto/ecdh"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"encoding/base64"
	"fmt"
	"log"
	"log/slog"
	"net/http"
	"net/http/pprof"
	"os"
	"strings"

	"entgo.io/contrib/entgql"
	gqlgraphql "github.com/99designs/gqlgen/graphql"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/urfave/cli"
	"golang.org/x/net/http2"
	"golang.org/x/net/http2/h2c"
	"google.golang.org/grpc"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/cryptocodec"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/migrate"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/secrets"
	"realm.pub/tavern/internal/www"
	"realm.pub/tavern/tomes"
)

func init() {
	configureLogging()
}

func newApp(ctx context.Context, options ...func(*Config)) (app *cli.App) {
	app = cli.NewApp()
	app.Name = "tavern"
	app.Description = "Teamserver implementation for Realm, see https://docs.realm.pub for more details"
	app.Usage = "Time for an Adventure!"
	app.Version = Version
	app.Action = cli.ActionFunc(func(*cli.Context) error {
		return run(ctx, options...)
	})
	return
}

func run(ctx context.Context, options ...func(*Config)) error {
	srv, err := NewServer(ctx, options...)
	if err != nil {
		return err
	}
	defer srv.Close()

	// Start Metrics Server (if configured)
	if srv.MetricsHTTP != nil {
		if srv.HTTP.Addr == srv.MetricsHTTP.Addr {
			return fmt.Errorf(
				"tavern and metrics http must have different listen configurations (tavern=%q, metrics=%q)",
				srv.HTTP.Addr,
				srv.MetricsHTTP.Addr,
			)
		}
		go func(ctx context.Context) {
			slog.InfoContext(ctx, "metrics http server started", "metrics_addr", srv.MetricsHTTP.Addr)
			if err := srv.MetricsHTTP.ListenAndServe(); err != nil {
				slog.WarnContext(ctx, "metrics http server stopped", "err", err)
			}
		}(ctx)
	}

	// Listen & Serve HTTP Traffic
	slog.InfoContext(ctx, "http server started", "http_addr", srv.HTTP.Addr)
	if err := srv.HTTP.ListenAndServe(); err != nil {
		slog.ErrorContext(ctx, "http server stopped", "err", err)
		return fmt.Errorf("http server stopped: %w", err)
	}

	return nil
}

// Server responsible for handling Tavern requests.
type Server struct {
	HTTP        *http.Server
	MetricsHTTP *http.Server
	client      *ent.Client
}

// Close should always be called to clean up a Tavern server.
func (srv *Server) Close() error {
	srv.HTTP.Shutdown(context.Background())
	if srv.MetricsHTTP == nil {
		return srv.client.Close()
	}
	srv.MetricsHTTP.Shutdown(context.Background())
	return srv.client.Close()
}

// NewServer initializes a Tavern HTTP server with the provided configuration.
func NewServer(ctx context.Context, options ...func(*Config)) (*Server, error) {
	// Generate server key pair
	pubKey, privKey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		log.Fatalf("[FATAL] failed to generate ed25519 keypair: %v", err)
	}
	// Initialize Config
	cfg := &Config{}
	for _, opt := range options {
		opt(cfg)
	}

	// Create Ent Client
	client, err := cfg.Connect()
	if err != nil {
		return nil, fmt.Errorf("failed to open graph: %w", err)
	}

	// Initialize Graph Schema
	if err := client.Schema.Create(
		ctx,
		migrate.WithGlobalUniqueID(true),
	); err != nil {
		client.Close()
		return nil, fmt.Errorf("failed to initialize graph schema: %w", err)
	}

	// Load Default Tomes
	if cfg.IsDefaultTomeImportEnabled() {
		if err := tomes.UploadTomes(ctx, client, tomes.FileSystem); err != nil {
			slog.ErrorContext(ctx, "failed to upload default tomes", "err", err)
		}
	}

	// Initialize Git Tome Importer
	git := cfg.NewGitImporter(client)

	// Initialize Test Data
	if cfg.IsTestDataEnabled() {
		createTestData(ctx, client)
	}

	// Configure Authentication
	var withAuthentication tavernhttp.Option
	if cfg.oauth.ClientID != "" {
		withAuthentication = tavernhttp.WithAuthentication(client)
	} else {
		withAuthentication = tavernhttp.WithAuthenticationBypass(client)
	}

	// Configure Request Logging
	httpLogger := log.New(os.Stderr, "[HTTP] ", log.Flags())

	// Configure Shell Muxes
	wsShellMux, grpcShellMux := cfg.NewShellMuxes(ctx)
	go func() {
		if err := wsShellMux.Start(ctx); err != nil {
			slog.ErrorContext(ctx, "websocket shell mux stopped", "err", err)
		}
	}()
	go func() {
		if err := grpcShellMux.Start(ctx); err != nil {
			slog.ErrorContext(ctx, "grpc shell mux stopped", "err", err)
		}
	}()

	// Route Map
	routes := tavernhttp.RouteMap{
		"/status": tavernhttp.Endpoint{
			Handler:              newStatusHandler(),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/access_token/redirect": tavernhttp.Endpoint{
			Handler:          auth.NewTokenRedirectHandler(),
			LoginRedirectURI: "/oauth/login",
		},
		"/oauth/login": tavernhttp.Endpoint{
			Handler:              auth.NewOAuthLoginHandler(cfg.oauth, privKey),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/oauth/authorize": tavernhttp.Endpoint{
			Handler: auth.NewOAuthAuthorizationHandler(
				cfg.oauth,
				pubKey,
				client,
				cfg.userProfiles,
			),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/graphql": tavernhttp.Endpoint{
			Handler:          newGraphQLHandler(client, git),
			AllowUnactivated: true,
		},
		"/c2.C2/": tavernhttp.Endpoint{
			Handler:              newGRPCHandler(client, grpcShellMux),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/cdn/": tavernhttp.Endpoint{
			Handler:              cdn.NewDownloadHandler(client, "/cdn/"),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/cdn/hostfiles/": tavernhttp.Endpoint{
			Handler: cdn.NewHostFileDownloadHandler(client, "/cdn/hostfiles/"),
		},
		"/cdn/upload": tavernhttp.Endpoint{
			Handler: cdn.NewUploadHandler(client),
		},
		"/shell/ws": tavernhttp.Endpoint{
			Handler: stream.NewShellHandler(client, wsShellMux),
		},
		"/": tavernhttp.Endpoint{
			Handler:          www.NewHandler(httpLogger),
			LoginRedirectURI: "/oauth/login",
			AllowUnactivated: true,
		},
		"/playground": tavernhttp.Endpoint{
			Handler:          playground.Handler("Realm - Red Team Engagement Platform", "/graphql"),
			LoginRedirectURI: "/oauth/login",
		},
	}

	// Setup Profiling
	if cfg.IsPProfEnabled() {
		slog.WarnContext(ctx, "performance profiling is enabled, do not use in production as this may leak sensitive information")
		registerProfiler(routes)
	}

	// Create Tavern HTTP Server
	srv := tavernhttp.NewServer(
		routes,
		withAuthentication,
	)

	// Configure HTTP/2 (support for without TLS)
	handler := h2c.NewHandler(srv, &http2.Server{})

	// Initialize HTTP Server
	if cfg.srv == nil {
		cfg.srv = &http.Server{
			Addr:    "0.0.0.0:80",
			Handler: handler,
		}
	} else {
		cfg.srv.Handler = handler
	}

	// Enable HTTP/2
	if err := http2.ConfigureServer(cfg.srv, &http2.Server{}); err != nil {
		return nil, fmt.Errorf("failed to configure http/2: %w", err)
	}

	// Initialize Server
	tSrv := &Server{
		HTTP:   cfg.srv,
		client: client,
	}

	// Setup Metrics
	if cfg.IsMetricsEnabled() {
		slog.WarnContext(ctx, "metrics reporting is enabled, unauthenticated /metrics endpoint will be available", "metrics_addr", EnvHTTPMetricsListenAddr.String())
		tSrv.MetricsHTTP = newMetricsServer()
	}

	// Shutdown for Test Run & Exit
	if cfg.IsTestRunAndExitEnabled() {
		go func() {
			tSrv.Close()
		}()
	}

	return tSrv, nil
}

func newGraphQLHandler(client *ent.Client, repoImporter graphql.RepoImporter) http.Handler {
	srv := handler.NewDefaultServer(graphql.NewSchema(client, repoImporter))
	srv.Use(entgql.Transactioner{TxOpener: client})

	// Configure Raw Query Logging
	logRawQuery := EnvLogGraphQLRawQuery.IsSet()

	// GraphQL Logging
	srv.AroundOperations(func(ctx context.Context, next gqlgraphql.OperationHandler) gqlgraphql.ResponseHandler {
		// Authentication Information
		var (
			authID          = "unknown"
			isActivated     = false
			isAdmin         = false
			isAuthenticated = false
			authUserID      = 0
			authUserName    = ""
		)
		id := auth.IdentityFromContext(ctx)
		if id != nil {
			authID = id.String()
			isActivated = id.IsActivated()
			isAdmin = id.IsAdmin()
			isAuthenticated = id.IsAuthenticated()
		}
		if authUser := auth.UserFromContext(ctx); authUser != nil {
			authUserID = authUser.ID
			authUserName = authUser.Name
		}

		// Operation Context
		oc := gqlgraphql.GetOperationContext(ctx)

		// Determine if Raw Query should be logged
		args := []any{
			"auth_generic_id", authID,
			"auth_user_name", authUserName,
			"auth_user_id", authUserID,
			"is_admin", isAdmin,
			"is_activated", isActivated,
			"is_authenticated", isAuthenticated,
			"operation", oc.OperationName,
			"variables", oc.Variables,
		}
		if logRawQuery {
			args = append(args, "raw_query", oc.RawQuery)
		}

		// Log Request
		slog.InfoContext(ctx, "tavern graphql request", args...)
		return next(ctx)
	})

	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Headers", "*")
		srv.ServeHTTP(w, req)
	})
}

func generateKeyPair() (*ecdh.PublicKey, *ecdh.PrivateKey, error) {
	curve := ecdh.X25519()
	privateKey, err := curve.GenerateKey(rand.Reader)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to generate private key: %v\n", err))
		return nil, nil, err
	}
	publicKey, err := curve.NewPublicKey(privateKey.PublicKey().Bytes())
	if err != nil {
		slog.Error(fmt.Sprintf("failed to generate public key: %v\n", err))
		return nil, nil, err
	}

	return publicKey, privateKey, nil
}

func GetPubKey() (*ecdh.PublicKey, error) {
	pub, _, err := getKeyPair()
	if err != nil {
		return nil, err
	}
	return pub, nil
}

func getKeyPair() (*ecdh.PublicKey, *ecdh.PrivateKey, error) {
	curve := ecdh.X25519()

	var secretsManager secrets.SecretsManager
	var err error

	if EnvSecretsManagerPath.String() == "" {
		secretsManager, err = secrets.NewGcp("")
	} else {
		secretsManager, err = secrets.NewDebugFileSecrets(EnvSecretsManagerPath.String())
	}
	if err != nil {
		slog.Error("unable to setup secrets manager")
		slog.Error("if you're running locally try setting `export SECRETS_FILE_PATH='/tmp/secrets'` \n")
		return nil, nil, fmt.Errorf("unable to connect to secrets manager: %s", err.Error())
	}

	// Check if we already have a key
	privateKeyString, err := secretsManager.GetValue("tavern_encryption_private_key")
	if err != nil {
		// Generate a new one if it doesn't exist
		pubKey, privateKey, err := generateKeyPair()
		if err != nil {
			return nil, nil, fmt.Errorf("key generation failed: %v", err)
		}

		privateKeyBytes, err := x509.MarshalPKCS8PrivateKey(privateKey)
		if err != nil {
			return nil, nil, fmt.Errorf("unable to marshal private key: %v", err)
		}
		_, err = secretsManager.SetValue("tavern_encryption_private_key", privateKeyBytes)
		if err != nil {
			return nil, nil, fmt.Errorf("unable to set 'tavern_encryption_private_key' using secrets manager: %v", err)
		}
		return pubKey, privateKey, nil
	}

	// Parse private key bytes
	tmp, err := x509.ParsePKCS8PrivateKey(privateKeyString)
	if err != nil {
		return nil, nil, fmt.Errorf("unable to parse private key: %v", err)
	}
	privateKey := tmp.(*ecdh.PrivateKey)

	publicKey, err := curve.NewPublicKey(privateKey.PublicKey().Bytes())
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate public key: %v", err)
	}

	return publicKey, privateKey, nil
}

func newGRPCHandler(client *ent.Client, grpcShellMux *stream.Mux) http.Handler {
	pub, priv, err := getKeyPair()
	if err != nil {
		panic(err)
	}
	slog.Info(fmt.Sprintf("public key: %s", base64.StdEncoding.EncodeToString(pub.Bytes())))

	c2srv := c2.New(client, grpcShellMux)
	xchacha := cryptocodec.StreamDecryptCodec{
		Csvc: cryptocodec.NewCryptoSvc(priv),
	}
	grpcSrv := grpc.NewServer(
		grpc.ForceServerCodecV2(xchacha),
		grpc.UnaryInterceptor(grpcWithUnaryMetrics),
		grpc.StreamInterceptor(grpcWithStreamMetrics),
	)
	c2pb.RegisterC2Server(grpcSrv, c2srv)
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.ProtoMajor != 2 {
			http.Error(w, "grpc requires HTTP/2", http.StatusBadRequest)
			return
		}

		if contentType := r.Header.Get("Content-Type"); !strings.HasPrefix(contentType, "application/grpc") {
			http.Error(w, "must specify Content-Type application/grpc", http.StatusBadRequest)
			return
		}

		grpcSrv.ServeHTTP(w, r)
	})
}

func newMetricsServer() *http.Server {
	router := http.NewServeMux()
	router.Handle("/metrics", promhttp.Handler())
	return &http.Server{
		Addr:    EnvHTTPMetricsListenAddr.String(),
		Handler: router,
	}
}

func registerProfiler(router tavernhttp.RouteMap) {
	router.HandleFunc("/debug/pprof/", pprof.Index)
	router.HandleFunc("/debug/pprof/cmdline", pprof.Cmdline)
	router.HandleFunc("/debug/pprof/profile", pprof.Profile)
	router.HandleFunc("/debug/pprof/symbol", pprof.Symbol)

	// Manually add support for paths linked to by index page at /debug/pprof/
	router.Handle("/debug/pprof/goroutine", pprof.Handler("goroutine"))
	router.Handle("/debug/pprof/heap", pprof.Handler("heap"))
	router.Handle("/debug/pprof/threadcreate", pprof.Handler("threadcreate"))
	router.Handle("/debug/pprof/block", pprof.Handler("block"))
}
