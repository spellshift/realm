package c2

//go:generate protoc -I=./proto --go_out=./epb --go_opt=paths=source_relative --go-grpc_out=./epb --go-grpc_opt=paths=source_relative eldritch.proto
//go:generate protoc -I=./proto --go_out=./c2pb --go_opt=paths=source_relative --go-grpc_out=./c2pb --go-grpc_opt=paths=source_relative c2.proto
