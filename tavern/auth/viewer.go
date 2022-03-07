package auth

import (
	"context"
	"fmt"
)

// ErrInvalidViewer occurs when an invalid type of viewer is retrieved from the context.
// ErrNoViewer occurs when no viewer can be retrieved from the context.
var (
	ErrInvalidViewer = fmt.Errorf("invalid viewer kind in context")
	ErrNoViewer      = fmt.Errorf("no authenticated viewer context")
)

// Viewer describes the query/mutation viewer-context.
type Viewer interface{}

// Implant viewers of the GraphAPI
type Implant struct {
	AuthToken string
}

// User viewers of the GraphAPI
type User struct{}

type ctxKey struct{}

// FromContext returns the Viewer stored in a context.
func FromContext(ctx context.Context) Viewer {
	v, _ := ctx.Value(ctxKey{}).(Viewer)
	return v
}

// ImplantFromContext returns an Implant viewer from a context.
// If the viewer was not an implant, returns an error.
func ImplantFromContext(ctx context.Context) (Implant, error) {
	v := FromContext(ctx)
	if v == nil {
		return Implant{}, ErrNoViewer
	}

	implant, ok := v.(Implant)
	if !ok {
		return Implant{}, ErrInvalidViewer
	}
	return implant, nil
}

// UserFromContext returns an User viewer from a context.
// If the viewer was not a user, returns an error.
func UserFromContext(ctx context.Context) (User, error) {
	v := FromContext(ctx)
	if v == nil {
		return User{}, ErrNoViewer
	}

	user, ok := v.(User)
	if !ok {
		return User{}, ErrInvalidViewer
	}
	return user, nil
}

// NewContext returns a copy of parent context with the given Viewer attached with it.
func NewContext(parent context.Context, v Viewer) context.Context {
	return context.WithValue(parent, ctxKey{}, v)
}
