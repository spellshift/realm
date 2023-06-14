#!/bin/sh
if [ ! -x "$(which graphql-client)" ] ; then
    echo "Installing codegen utility 'graphql_client_cli'";
    cargo install graphql_client_cli;
fi
echo "[Tavern][Rust] Generating GraphQL Code...";
graphql-client generate --output-directory ./src --schema-path ./graphql/schema.graphql --custom-scalars-module='crate::scalars' --response-derives='Serialize,Clone' ./graphql/mutations.graphql
echo "[Tavern][Rust] Code Generation Complete";
