#!/bin/bash

set +e

keep_trying() {
    DOMAIN="$1"
    for (( i=1 ; i<=180; i++)); do
        RES=$(curl -q --fail $DOMAIN/status 2>/dev/null)
        if [[ $? -eq 0 && -n $RES ]]; then
            echo $RES
            exit
        fi
        sleep 10
    done
}

keep_trying $1
