package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"fmt"
	"log"
	"net/http"

	"entgo.io/contrib/entgql"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/debug"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/kcarretto/realm/tavern/auth"
	"github.com/kcarretto/realm/tavern/ent/migrate"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/kcarretto/realm/tavern/internal/cdn"
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
		return fmt.Errorf("failed to open graph: %w", err)
	}
	defer client.Close()

	// Initialize Graph Schema
	if err := client.Schema.Create(
		ctx,
		migrate.WithGlobalUniqueID(true),
	); err != nil {
		return fmt.Errorf("failed to initialize graph schema: %w", err)
	}

	// Initialize Test Data
	createTestData(ctx, client)

	// Create GraphQL Handler
	srv := handler.NewDefaultServer(graphql.NewSchema(client))
	srv.Use(entgql.Transactioner{TxOpener: client})
	srv.Use(&debug.Tracer{})

	// Setup HTTP Handler
	router := http.NewServeMux()
	router.Handle("/",
		playground.Handler("Tavern", "/graphql"),
	)
	router.Handle("/graphql", http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Headers", "*")
		srv.ServeHTTP(w, req)
	}))
	router.Handle("/oauth/login", auth.NewOAuthLoginHandler(cfg.oauth, privKey))
	router.Handle("/oauth/authorize", auth.NewOAuthAuthorizationHandler(cfg.oauth, pubKey, client, "https://www.googleapis.com/oauth2/v3/userinfo"))
	router.Handle("/cdn/", cdn.NewDownloadHandler(client))
	router.Handle("/cdn/upload", cdn.NewUploadHandler(client))

	// Auth Middleware
	var endpoint http.Handler
	if cfg.oauth.ClientID != "" {
		endpoint = auth.Middleware(router, cfg.client)
	} else {
		endpoint = auth.AuthDisabledMiddleware(router)
	}

	// Listen & Serve HTTP Traffic
	addr := "0.0.0.0:80"
	log.Printf("listening on %s", addr)
	if err := http.ListenAndServe(addr, endpoint); err != nil {
		return fmt.Errorf("stopped http server: %w", err)
	}

	return nil
}
