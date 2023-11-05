package http

import "github.com/kcarretto/realm/tavern/internal/ent"

// An Option to configure an HTTP Endpoint.
type Option func(*Endpoint)

// WithAuthenticationCookie enables cookie authentication for the endpoint.
func WithAuthenticationCookie(graph *ent.Client) Option {
	return Option(func(endpoint *Endpoint) {
		endpoint.authenticator = &cookieAuthenticator{graph}
	})
}

// WithAuthenticationBypass enables requests to bypass authentication for the endpoint.
func WithAuthenticationBypass(graph *ent.Client) Option {
	return Option(func(endpoint *Endpoint) {
		endpoint.authenticator = &bypassAuthenticator{graph}
	})
}

/*

endpoints := tavernhttp.RouteMap{
	"/": tavernhttp.NewEndpoint(
			NewUIHandler(),
			tavernhttp.WithLogging("[HTTP][UI]"),
			tavernhttp.WithAuthenticationCookie(graph),
			tavernhttp.WithLoginRedirect(),
		),
	"/graphql": tavernhttp.NewEndpoint(
			NewGraphQLHandler(),
			tavernhttp.WithLogging("[HTTP][GraphQL]"),
			tavernhttp.WithAuthenticationCookie(graph),
		),
	"/grpc": tavernhttp.NewEndpoint(
			NewGRPCHandler(),
			tavernhttp.WithLogging("[HTTP][gRPC]"),
		),
	"/oauth/login": tavernhttp.NewEndpoint(
			auth.NewOAuthLoginHandler(cfg.oauth, privKey),
		),
	"/oauth/authorize": tavernhttp.NewEndpoint(
			auth.NewOAuthAuthorizationHandler(
				cfg.oauth,
				pubKey,
				client,
				"https://www.googleapis.com/oauth2/v3/userinfo",
			),
		),
	),
}
endpoints.Register(router)

*/
