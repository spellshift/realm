# Agents

Welcome to our repository! Most commands need to be run from the root of the project directory (e.g. where this `AGENT.md` file is located)

## Project Structure

* `tavern/` includes our Golang server implementation, which hosts a GraphQL API, User-Interface (typescript), and a gRPC API used by agents.
  * Our user interface is located in `tavern/internal/www` and we managed dependencies within that directory using `npm`
* `implants/` contains Rust code that is deployed to target machines, such as our agent located in `implants/imix`.

## Golang Tests

To run all Golang tests in our repository, please run `go test ./...` from the project root.

## Code Generation

This project heavily relies on generated code. When making changes to ent schemas, GraphQL, or user interface / front-end changes in the `tavern/internal/www/` directory, you will need to run `go generate ./...` from the project root directory to re-generate some critical files.

## Additional Documentation

Our user-facing documentation for the project is located in `docs/_docs` and can be referenced for additional information.
