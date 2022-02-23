package main

import (
	"context"
	"log"
	"net/http"

	"entgo.io/contrib/entgql"
	"github.com/kcarretto/realm/ent"
	"github.com/kcarretto/realm/ent/credential"
	"github.com/kcarretto/realm/ent/migrate"
	_ "github.com/kcarretto/realm/ent/runtime"
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
	target := client.Target.Create().
		SetName("Test").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)
	client.Credential.Create().
		SetPrincipal("root").
		SetSecret("changeme").
		SetKind(credential.KindPassword).
		SetTarget(target).
		SaveX(ctx)
	client.Credential.Create().
		SetPrincipal("admin").
		SetSecret("password1!").
		SetKind(credential.KindPassword).
		SetTarget(target).
		SaveX(ctx)

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
		log.Printf("stopped http server: %v", err)
	}

}
