package http

import "fmt"

// ErrReadingAuthCookie occurs when tavern is unable to read the auth cookie of the request.
// ErrInvalidAuthCookie occurs when the auth cookie provided by the request is invalid.
// ErrInvalidAccessToken occurs when the access token provided by the request is invalid.
var (
	ErrReadingAuthCookie  = fmt.Errorf("failed to read auth cookie")
	ErrInvalidAuthCookie  = fmt.Errorf("invalid auth cookie")
	ErrInvalidAccessToken = fmt.Errorf("invalid access token")
)
