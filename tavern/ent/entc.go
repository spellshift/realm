//go:build ignore

package main

import (
	"log"

	"entgo.io/contrib/entgql"
	"entgo.io/ent/entc"
	"entgo.io/ent/entc/gen"
)

func main() {
	extensions, err := entgql.NewExtension(
		entgql.WithSchemaGenerator(),
		entgql.WithWhereFilters(true),
		entgql.WithSchemaPath("./graphql/schema/schema.graphql"),
		entgql.WithConfigPath("./graphql/gqlgen.yml"),
	)
	if err != nil {
		log.Fatalf("creating entgql extension: %v", err)
	}
	opts := []entc.Option{
		entc.Extensions(extensions),
		entc.FeatureNames("privacy"),
	}

	if err := entc.Generate(
		"./ent/schema",
		&gen.Config{
			Templates: entgql.AllTemplates,
		},
		opts...,
	); err != nil {
		log.Fatalf("running ent codegen: %v", err)
	}
}
