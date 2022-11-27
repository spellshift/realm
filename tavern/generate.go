package main

//go:generate go run -mod=mod ./ent/entc.go
//go:generate /bin/sh -c "cd ./graphql && go run -mod=mod github.com/99designs/gqlgen"
