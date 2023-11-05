package beacon

//go:generate protoc --go_out=./beaconpb --go_opt=paths=source_relative --go-grpc_out=./beaconpb --go-grpc_opt=paths=source_relative beacon.proto
