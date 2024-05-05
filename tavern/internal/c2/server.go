package c2

import (
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/http/stream"
)

type Server struct {
	MaxFileChunkSize uint64
	graph            *ent.Client
	mux              *stream.Mux
	c2pb.UnimplementedC2Server
}

func New(graph *ent.Client, mux *stream.Mux) *Server {
	return &Server{
		MaxFileChunkSize: 1024 * 1024, // 1 MB
		graph:            graph,
		mux:              mux,
	}
}
