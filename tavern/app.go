package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/kcarretto/realm/tavern/tomes"

	"entgo.io/contrib/entgql"
	gqlgraphql "github.com/99designs/gqlgen/graphql"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/kcarretto/realm/tavern/auth"
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/migrate"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/kcarretto/realm/tavern/internal/cdn"
	"github.com/kcarretto/realm/tavern/internal/www"
	"github.com/urfave/cli"
)

// Version of Tavern being run
const Version = "v0.0.1"

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
	defer srv.client.Close()

	// Listen & Serve HTTP Traffic
	log.Printf("Starting HTTP server on %s", srv.HTTP.Addr)
	if err := srv.HTTP.ListenAndServe(); err != nil {
		return fmt.Errorf("stopped http server: %w", err)
	}

	return nil
}

// Server responsible for handling Tavern requests.
type Server struct {
	HTTP   *http.Server
	client *ent.Client
}

// Close should always be called to clean up a Tavern server.
func (srv *Server) Close() error {
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
	if err := tomes.UploadTomes(ctx, client, tomes.FileSystem); err != nil {
		client.Close()
		return nil, fmt.Errorf("failed to upload default tomes: %w", err)
	}

	// Initialize Test Data
	if cfg.IsTestDataEnabled() {
		createTestData(ctx, client)
	}

	// Create GraphQL Handler
	srv := handler.NewDefaultServer(graphql.NewSchema(client))
	srv.Use(entgql.Transactioner{TxOpener: client})

	// GraphQL Logging
	gqlLogger := log.New(os.Stderr, "[GraphQL] ", log.Flags())
	srv.AroundOperations(func(ctx context.Context, next gqlgraphql.OperationHandler) gqlgraphql.ResponseHandler {
		oc := gqlgraphql.GetOperationContext(ctx)
		reqVars, err := json.Marshal(oc.Variables)
		if err != nil {
			gqlLogger.Printf("[ERROR] failed to marshal variables to JSON: %v", err)
			return next(ctx)
		}

		authName := "unknown"
		id := auth.IdentityFromContext(ctx)
		if id != nil {
			authName = id.String()
		}

		gqlLogger.Printf("%s (%s): %s", oc.OperationName, authName, string(reqVars))
		return next(ctx)
	})

	// Setup HTTP Handlers
	router := http.NewServeMux()
	router.Handle("/status", newStatusHandler())
	router.Handle("/playground",
		playground.Handler("Tavern", "/graphql"),
	)
	router.Handle("/", www.NewAppHandler(""))
	router.Handle("/graphql", http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Headers", "*")
		srv.ServeHTTP(w, req)
	}))
	router.Handle("/oauth/login", auth.NewOAuthLoginHandler(cfg.oauth, privKey))
	router.Handle("/oauth/authorize", auth.NewOAuthAuthorizationHandler(cfg.oauth, pubKey, client, "https://www.googleapis.com/oauth2/v3/userinfo"))
	router.Handle("/cdn/", cdn.NewDownloadHandler(client))
	router.Handle("/cdn/upload", cdn.NewUploadHandler(client))

	// Log Middleware
	httpLogger := log.New(os.Stderr, "[HTTP] ", log.Flags())
	handlerWithLogging := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authName := "unknown"
		id := auth.IdentityFromContext(r.Context())
		if id != nil {
			authName = id.String()
		}

		httpLogger.Printf("%s (%s) %s %s\n", r.RemoteAddr, authName, r.Method, r.URL)
		router.ServeHTTP(w, r)
	})

	// Auth Middleware
	var endpoint http.Handler
	if cfg.oauth.ClientID != "" {
		endpoint = auth.Middleware(handlerWithLogging, client)
	} else {
		endpoint = auth.AuthDisabledMiddleware(handlerWithLogging, client)
	}

	// Initialize HTTP Server
	if cfg.srv == nil {
		cfg.srv = &http.Server{
			Addr:    "0.0.0.0:80",
			Handler: endpoint,
		}
	} else {
		cfg.srv.Handler = endpoint
	}

	return &Server{
		HTTP:   cfg.srv,
		client: client,
	}, nil
}
