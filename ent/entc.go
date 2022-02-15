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
		entgql.WithWhereFilters(true),
		entgql.WithSchemaPath("../graphql/schema/ent.graphql"),
		entgql.WithConfigPath("../graphql/gqlgen.yml"),
	)
	if err != nil {
		log.Fatalf("creating entgql extension: %v", err)
	}
	if err := entc.Generate(
		"./schema",
		&gen.Config{},
		entc.Extensions(extensions),
	); err != nil {
		log.Fatalf("running ent codegen: %v", err)
	}
}
