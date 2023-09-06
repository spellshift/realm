//go:build ignore

package main

import (
	"log"

	"entgo.io/contrib/entgql"
	"entgo.io/ent/entc"
	"entgo.io/ent/entc/gen"
)

//go:generate go run -mod=mod generate.go
//go:generate /bin/sh -c "cd ../graphql && go run -mod=mod github.com/99designs/gqlgen"
//go:generate /bin/sh -c "cat ../graphql/schema/* > ../graphql/schema.graphql"
//go:generate /bin/sh -c "cp ../graphql/schema.graphql ../www/schema.graphql"
//go:generate /bin/sh -c "cp ../graphql/schema.graphql ../../../implants/lib/tavern/graphql/schema.graphql"
//go:generate /bin/sh -c "cd ../../../implants/lib/tavern && ./codegen.sh"

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
		entc.FeatureNames("privacy"),
	}

	if err := entc.Generate(
		"./schema",
		&gen.Config{},
		opts...,
	); err != nil {
		log.Fatalf("running ent codegen: %v", err)
	}
}
