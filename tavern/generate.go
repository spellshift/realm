package main

//go:generate go run -mod=mod ./ent/entc.go
//go:generate /bin/sh -c "cd ./graphql && go run -mod=mod github.com/99designs/gqlgen"

//go:generate /bin/sh -c "cat ./graphql/schema/* > ./graphql/schema.graphql"
//go:generate /bin/sh -c "cp ./graphql/schema.graphql ../implants/pkg/tavern/graphql/schema.graphql"
//go:generate /bin/sh -c "cd ../implants/pkg/tavern && ./codegen.sh"
