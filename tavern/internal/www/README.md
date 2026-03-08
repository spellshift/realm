# Tavern UI

This package contains the relevant code for the Tavern UI. It primarily uses [React](https://reactjs.org/docs/getting-started.html) + [Apollo](https://www.apollographql.com/docs/react) in combination with [TypeScript](https://www.typescriptlang.org/), but depends on code generation to function properly. Read on for more development information.

## Code Generation

Any relevant code generation can be executed by running `go generate ./cmd/tavern/internal/www/generate.go` or will automatically be executed when running `go generate ./...`. Code generation is responsible for:

* Constructing the GraphQL schema by concatenating all schema files under the [/graphql/schema directory](https://github.com/spellshift/realm/tree/main/graphql/schema).

## Testing Changes

_If this is your first time contributing WWW changes in this dev environment, remember to run `npm install` to ensure you have all required dependencies_

1. Run `go generate ./...` to ensure all code generation is up to date
2. Run `go run ./tavern` to start the teamserver (for the GraphQL API) run in the project root
    * Note: to run the teamserver with test data (useful for UI development), run `ENABLE_TEST_DATA=1 go run ./tavern` instead
3. In a separate terminal, navigate to the [UI Root /cmd/tavern/internal/www](https://github.com/spellshift/realm/tree/main/cmd/tavern/internal/www) and run `npm start`

## Building the Application

When you're ready to include changes to the UI in the tavern binary, you'll need to run `npm run build` to update files in the `build/` directory. These files will automatically be bundled into new compilations of the tavern binary.
