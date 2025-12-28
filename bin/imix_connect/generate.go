package main

//go:generate protoc -I=../../tavern/internal/portal/proto --go_out=./portalpb --go_opt=paths=source_relative --go-grpc_out=./portalpb --go-grpc_opt=paths=source_relative ../../tavern/internal/portal/proto/portal.proto
