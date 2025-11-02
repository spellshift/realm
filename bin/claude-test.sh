#!/bin/bash


export SECRETS_FILE_PATH='/tmp/secrets'

go run ./tavern/... > /tmp/tavern-out.log 2>&1 &

sleep 5

export IMIX_SERVER_PUBKEY="$(curl -q http://localhost/status | jq -r '.Pubkey')"
cd implants && cargo run --bin imix > /tmp/imix-out.log  2>&1 & cd -

sleep 60
kill %2
kill %1
