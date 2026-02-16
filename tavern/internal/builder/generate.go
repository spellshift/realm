package builder

//go:generate protoc -I=./proto --go_out=./builderpb --go_opt=paths=source_relative --go-grpc_out=./builderpb --go-grpc_opt=paths=source_relative builder.proto
