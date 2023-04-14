#!/bin/sh
graphql-client generate --output-directory ./src --schema-path ./graphql/schema.graphql --custom-scalars-module='crate::scalars' --response-derives='Serialize' ./graphql/mutations.graphql