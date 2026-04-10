#!/bin/bash
go run ./bin/e2e/sshecho &
ECHO_PID=$!
sleep 2

python3 -c "import socket, time; s = socket.socket(socket.AF_INET, socket.SOCK_STREAM); s.connect(('127.0.0.1', 2222)); s.send(b'SSH-2.0-Go\r\n'); time.sleep(1); s.send(b'\x00\x00\x00\x00\x00\x00'); time.sleep(1);" &
sleep 3

kill $ECHO_PID
