//go:build ignore

package main

import (
	"log"

	"entgo.io/contrib/entgql"
	"entgo.io/ent/entc"
	"entgo.io/ent/entc/gen"
)

func main() {
	log.Println("generating entgo")
	extensions, err := entgql.NewExtension(
		entgql.WithSchemaGenerator(),
		entgql.WithWhereInputs(true),
		entgql.WithSchemaPath("../graphql/schema/ent.graphql"),
		entgql.WithConfigPath("../graphql/gqlgen.yml"),
	)
	if err != nil {
		log.Fatalf("creating entgql extension: %v", err)
	}
	opts := []entc.Option{
		entc.Extensions(extensions),
		entc.FeatureNames("privacy", "sql/upsert"),
	}

	if err := entc.Generate(
		"./schema",
		&gen.Config{},
		opts...,
	); err != nil {
		log.Fatalf("running ent codegen: %v", err)
	}
}
