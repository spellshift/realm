package c2

import (
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

type Server struct {
	MaxFileChunkSize uint64
	graph            *ent.Client
	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client) *Server {
	return &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
	}
}
