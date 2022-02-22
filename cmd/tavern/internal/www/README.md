# Tavern UI
This package contains the relevant code for the Tavern UI. It primarily uses [React](https://reactjs.org/docs/getting-started.html) + [Relay](https://relay.dev/docs/guided-tour/) in combination with [TypeScript](https://www.typescriptlang.org/), but depends on code generation to function properly. Read on for more development information.

## Code Generation
Any relevant code generation can be executed by running `go generate ./cmd/tavern/internal/www/generate.go` or will automatically be executed when running `go generate ./...`. Code generation is responsible for:

* Constructing the GraphQL schema by concatenating all schema files under the [/graphql/schema directory](https://github.com/KCarretto/realm/tree/main/graphql/schema).

## Testing Changes

1. Run `go generate ./...` to ensure all code generation is up to date
2. Run `go run ./cmd/tavern` to start the teamserver (for the GraphQL API)
3. In a separate terminal, navigate to the [UI Root /cmd/tavern/internal/www](https://github.com/KCarretto/realm/tree/main/cmd/tavern/internal/www) and run `npm start` (Note this will also run the [Relay compiler](https://relay.dev/docs/guides/compiler/)) 
