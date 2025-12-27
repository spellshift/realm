package portal

import (
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/portal/portalpb"
)

type Server struct {
	graph           *ent.Client
	portalClientMux *stream.Mux
	portalpb.UnimplementedPortalServer
}

func New(graph *ent.Client, portalClientMux *stream.Mux) *Server {
	return &Server{
		graph:           graph,
		portalClientMux: portalClientMux,
	}
}
