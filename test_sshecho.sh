#!/bin/bash
go run ./bin/e2e/sshecho &
ECHO_PID=$!
sleep 2

kill $ECHO_PID
