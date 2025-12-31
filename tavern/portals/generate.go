package portals

//go:generate protoc -I=./proto --go_out=./portalpb --go_opt=paths=source_relative --go-grpc_out=./portalpb --go-grpc_opt=paths=source_relative portal.proto
//go:generate protoc -I=./proto --go_out=./tracepb --go_opt=paths=source_relative --go-grpc_out=./tracepb --go-grpc_opt=paths=source_relative trace.proto
