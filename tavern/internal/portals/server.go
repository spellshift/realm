package portals

import (
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

type Server struct {
	graph    *ent.Client
	mux      *mux.Mux
	serverID string
	portalpb.UnimplementedPortalServer
}

func New(graph *ent.Client, mux *mux.Mux, serverID string) *Server {
	return &Server{
		graph:    graph,
		mux:      mux,
		serverID: serverID,
	}
}
