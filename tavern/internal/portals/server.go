package portals

import (
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

type Server struct {
	graph *ent.Client
	mux   *mux.Mux
	portalpb.UnimplementedPortalServer
}

func New(graph *ent.Client, mux *mux.Mux) *Server {
	return &Server{
		graph: graph,
		mux:   mux,
	}
}
