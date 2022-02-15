package main

import (
	"context"
	"log"
	"net/http"

	"entgo.io/contrib/entgql"
	"github.com/kcarretto/realm/ent"
	"github.com/kcarretto/realm/ent/migrate"
	"github.com/kcarretto/realm/graphql"

	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/debug"
	"github.com/99designs/gqlgen/graphql/playground"
	_ "github.com/mattn/go-sqlite3"
)

func main() {
	// Server Context
	ctx := context.Background()

	// Create Ent Client
	client, err := ent.Open(
		"sqlite3",
		"file:ent?mode=memory&cache=shared&_fk=1",
	)
	if err != nil {
		log.Fatalf("failed to open graph: %v", err)
	}
	defer client.Close()

	// Initialize Graph Schema
	if err := client.Schema.Create(
		ctx,
		migrate.WithGlobalUniqueID(true),
	); err != nil {
		log.Fatalf("failed to initialize graph schema: %v", err)
	}

	// Initialize Test Data
	client.Target.Create().
		SetName("Test").
		SetForwardConnectIP("10.0.0.1").
		ExecX(ctx)

	// Create GraphQL Handler
	srv := handler.NewDefaultServer(graphql.NewSchema(client))
	srv.Use(entgql.Transactioner{TxOpener: client})
	srv.Use(&debug.Tracer{})

	// Setup HTTP Handler
	http.Handle("/",
		playground.Handler("Tavern", "/query"),
	)
	http.Handle("/query", srv)

	// Listen & Serve HTTP Traffic
	addr := "0.0.0.0:80"
	log.Printf("listening on %s", addr)
	if err := http.ListenAndServe(addr, nil); err != nil {
		log.Printf("stopped http server: %v", err)
	}

}
