package ent

//go:generate go run -mod=mod generate_code.go
//go:generate /bin/sh -c "cd ../graphql && go run -mod=mod github.com/99designs/gqlgen"
//go:generate /bin/sh -c "cat ../graphql/schema/* > ../graphql/schema.graphql"
//go:generate /bin/sh -c "cp ../graphql/schema.graphql ../www/schema.graphql"
//go:generate /bin/sh -c "cp ../graphql/schema.graphql ../../../implants/lib/tavern/graphql/schema.graphql"
//go:generate /bin/sh -c "cd ../../../implants/lib/tavern && ./codegen.sh"
