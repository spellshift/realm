package main

import (
	"context"
	"crypto/ecdh"
	"crypto/ed25519"
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
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/crypto"
	"realm.pub/tavern/internal/cryptocodec"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/migrate"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/redirectors"
	"realm.pub/tavern/internal/secrets"
	"realm.pub/tavern/internal/www"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/tomes"

	_ "realm.pub/tavern/internal/redirectors/dns"
	_ "realm.pub/tavern/internal/redirectors/grpc"
	_ "realm.pub/tavern/internal/redirectors/http1"
)

func init() {
	configureLogging()
}

func newApp(ctx context.Context) (app *cli.App) {
	app = cli.NewApp()
	app.Name = "tavern"
	app.Description = "Teamserver implementation for Realm, see https://docs.realm.pub for more details"
	app.Usage = "Time for an Adventure!"
	app.Version = Version
	app.Action = func(c *cli.Context) error {
		return runTavern(
			ctx,
			ConfigureHTTPServerFromEnv(),
			ConfigureMySQLFromEnv(),
			ConfigureOAuthFromEnv("/oauth/authorize"),
		)
	}
	app.Commands = []cli.Command{
		{
			Name:      "redirector",
			Usage:     "Run a redirector connecting agents using a specific transport to the server",
			ArgsUsage: "[upstream_address]",
			Flags: []cli.Flag{
				cli.StringFlag{
					Name:  "listen",
					Usage: "Address to listen on for incoming redirector traffic (default: :8080)",
					Value: ":8080",
				},
				cli.StringFlag{
					Name:  "transport",
					Usage: "Transport protocol to use for redirector",
					Value: "grpc",
				},
			},
			Action: func(c *cli.Context) error {
				var (
					upstream  = c.Args().First()
					listenOn  = c.String("listen")
					transport = c.String("transport")
				)
				if upstream == "" {
					return fmt.Errorf("gRPC upstream address is required (first argument)")
				}
				if listenOn == "" {
					listenOn = ":8080"
				}
				if transport == "" {
					transport = "grpc"
				}
				slog.InfoContext(ctx, "starting redirector", "upstream", upstream, "transport", transport, "listen_on", listenOn)
				return redirectors.Run(ctx, transport, listenOn, upstream)
			},
			Subcommands: cli.Commands{
				cli.Command{
					Name:  "list",
					Usage: "List available redirectors",
					Action: func(c *cli.Context) error {
						redirectorNames := redirectors.List()
						if len(redirectorNames) == 0 {
							fmt.Println("No redirectors registered")
							return nil
						}
						fmt.Println("Available redirectors:")
						for _, name := range redirectorNames {
							fmt.Printf("- %s\n", name)
						}
						return nil
					},
				},
			},
		},
		{
			Name:  "builder",
			Usage: "Run a builder that compiles agents for target platforms",
			Flags: []cli.Flag{
				cli.StringFlag{
					Name:  "config",
					Usage: "Path to the builder YAML configuration file",
				},
			},
			Action: func(c *cli.Context) error {
				configPath := c.String("config")
				if configPath == "" {
					return fmt.Errorf("--config flag is required")
				}

				cfg, err := builder.ParseConfig(configPath)
				if err != nil {
					return fmt.Errorf("failed to parse builder config: %w", err)
				}

				slog.InfoContext(ctx, "starting builder",
					"config", configPath,
					"supported_targets", cfg.SupportedTargets,
				)

				return builder.Run(ctx, cfg)
			},
		},
	}
	return
}

func runTavern(ctx context.Context, options ...func(*Config)) error {
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
	// Get server key pair from secrets manager (for OAuth)
	pubKey, privKey, err := getKeyPairEd25519()
	if err != nil {
		log.Fatalf("[FATAL] failed to get ed25519 key pair: %v", err)
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

	// Initialize Builder CA
	builderCACert, builderCAKey, err := getBuilderCA()
	if err != nil {
		client.Close()
		return nil, fmt.Errorf("failed to initialize builder CA: %w", err)
	}

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

	// Configure Portal Mux
	portalMux := cfg.NewPortalMux(ctx)

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
			Handler:          newGraphQLHandler(client, git, builderCACert, builderCAKey),
			AllowUnactivated: true,
		},
		"/c2.C2/": tavernhttp.Endpoint{
			Handler:              newGRPCHandler(client, grpcShellMux, portalMux),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/portal.Portal/": tavernhttp.Endpoint{
			Handler: newPortalGRPCHandler(client, portalMux),
		},
		"/builder.Builder/": tavernhttp.Endpoint{
			Handler:              newBuilderGRPCHandler(client, builderCACert),
			AllowUnauthenticated: true,
			AllowUnactivated:     true,
		},
		"/cdn/": tavernhttp.Endpoint{
			Handler:              cdn.NewLinkDownloadHandler(client, "/cdn/"),
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
		"/shell/ping": tavernhttp.Endpoint{
			Handler: stream.NewPingHandler(client, wsShellMux),
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

func newGraphQLHandler(client *ent.Client, repoImporter graphql.RepoImporter, builderCACert *x509.Certificate, builderCAKey ed25519.PrivateKey) http.Handler {
	srv := handler.NewDefaultServer(graphql.NewSchema(client, repoImporter, builderCACert, builderCAKey))
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

func newSecretsManager() (secrets.SecretsManager, error) {
	if EnvGCPProjectID.String() == "" && EnvSecretsManagerPath.String() == "" {
		slog.Error("No configuration provided for secret manager path, using a potentially insecure default.")
		return secrets.NewDebugFileSecrets("/tmp/tavern-secrets")
	}
	if EnvSecretsManagerPath.String() == "" {
		return secrets.NewGcp(EnvGCPProjectID.String())
	}

	return secrets.NewDebugFileSecrets(EnvSecretsManagerPath.String())
}

func GetPubKey() (*ecdh.PublicKey, error) {
	pub, _, err := getKeyPairX25519()
	return pub, err
}

// getKeyPairX25519 returns the server's X25519 key pair (derived from ED25519)
func getKeyPairX25519() (pubKey *ecdh.PublicKey, privKey *ecdh.PrivateKey, err error) {
	secretsManager, err := newSecretsManager()
	if err != nil {
		return nil, nil, err
	}

	pubKey, err = crypto.GetPubKeyX25519(secretsManager)
	if err != nil {
		return nil, nil, err
	}

	privKey, err = crypto.GetPrivKeyX25519(secretsManager)
	if err != nil {
		return nil, nil, err
	}

	return pubKey, privKey, nil
}

// getKeyPairEd25519 returns the server's ED25519 key pair
func getKeyPairEd25519() (pubKey []byte, privKey []byte, err error) {
	secretsManager, err := newSecretsManager()
	if err != nil {
		return nil, nil, err
	}

	pubKey, err = crypto.GetPubKeyED25519(secretsManager)
	if err != nil {
		return nil, nil, err
	}

	privKey, err = crypto.GetPrivKeyED25519(secretsManager)
	if err != nil {
		return nil, nil, err
	}

	return pubKey, privKey, nil
}

// getBuilderCA returns the Builder CA certificate and private key for signing builder certificates.
// It uses the existing ED25519 key from the secrets manager.
func getBuilderCA() (caCert *x509.Certificate, caKey ed25519.PrivateKey, err error) {
	secretsManager, err := newSecretsManager()
	if err != nil {
		return nil, nil, err
	}

	caKey, err = crypto.GetPrivKeyED25519(secretsManager)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to get ED25519 private key: %w", err)
	}

	caCert, err = builder.CreateCA(caKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create builder CA: %w", err)
	}

	return caCert, caKey, nil
}

func newPortalGRPCHandler(graph *ent.Client, portalMux *mux.Mux) http.Handler {
	portalSrv := portals.New(graph, portalMux)
	grpcSrv := grpc.NewServer(
		grpc.UnaryInterceptor(grpcWithUnaryMetrics),
		grpc.StreamInterceptor(grpcWithStreamMetrics),
	)
	portalpb.RegisterPortalServer(grpcSrv, portalSrv)
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

func newBuilderGRPCHandler(client *ent.Client, caCert *x509.Certificate) http.Handler {
	builderSrv := builder.New(client)
	grpcSrv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			builder.NewMTLSAuthInterceptor(caCert, client),
			grpcWithUnaryMetrics,
		),
		grpc.StreamInterceptor(grpcWithStreamMetrics),
	)
	builderpb.RegisterBuilderServer(grpcSrv, builderSrv)
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

func newGRPCHandler(client *ent.Client, grpcShellMux *stream.Mux, portalMux *mux.Mux) http.Handler {
	pub, priv, err := getKeyPairX25519()
	if err != nil {
		panic(err)
	}
	slog.Info(fmt.Sprintf("public key: %s", base64.StdEncoding.EncodeToString(pub.Bytes())))

	// Get ED25519 private key for JWT signing
	ed25519PubKey, ed25519PrivKey, err := getKeyPairEd25519()
	if err != nil {
		panic(err)
	}

	c2srv := c2.New(client, grpcShellMux, portalMux, ed25519PubKey, ed25519PrivKey)
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
