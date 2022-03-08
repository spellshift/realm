package main

import (
	"context"
	"fmt"
	"log"
	"net/http"

	"entgo.io/contrib/entgql"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/debug"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/kcarretto/realm/tavern/ent/migrate"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/urfave/cli"
)

// Version of Tavern being run
const Version = "v0.0.1"

func newApp(ctx context.Context, options ...func(*Config)) (app *cli.App) {
	app = cli.NewApp()
	app.Name = "tavern"
	app.Description = "Teamserver implementation for realm, see https://docs.realm.pub for more details"
	app.Version = Version
	app.Action = cli.ActionFunc(func(*cli.Context) error {
		return run(ctx, options...)
	})
	return
}

func run(ctx context.Context, options ...func(*Config)) error {
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

	// Listen & Serve HTTP Traffic
	addr := "0.0.0.0:80"
	log.Printf("listening on %s", addr)
	if err := http.ListenAndServe(addr, router); err != nil {
		return fmt.Errorf("stopped http server: %w", err)
	}

	return nil
}
