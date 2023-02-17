package auth

import (
	"context"
	"fmt"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/user"
)

var (
	// ErrPermissionDenied indicates the identity did not have sufficient permissions to perform an action.
	ErrPermissionDenied = fmt.Errorf("permission denied")
)

// ctxKey is used to manage values stored inside of context, it is unexported to limit modifications of these values to this package.
type ctxKey struct{}

// An Identity making a request.
type Identity interface {
	// String representation of the identity, used for logging
	String() string

	// IsAuthenticated should only return true if the identity has been authenticated.
	IsAuthenticated() bool

	// IsActivated should only return true if the identity is allowed to make sensitive API requests.
	IsActivated() bool

	// IsAdmin should only return true if the identity represents an administrator.
	IsAdmin() bool
}

// A userIdentity represents a user.
type userIdentity struct {
	Authenticated bool
	*ent.User
}

// String returns the underlying user identities username.
func (u *userIdentity) String() string {
	return u.Name
}

// IsAuthenticated returns true if the user has been authenticated.
func (u *userIdentity) IsAuthenticated() bool {
	if u == nil || u.User == nil {
		return false
	}
	return u.Authenticated
}

// IsActivated returns true if the user account has been activated.
func (u *userIdentity) IsActivated() bool {
	if u == nil || u.User == nil {
		return false
	}
	return u.User.IsActivated
}

// IsAdmin returns true if the user is an administrator.
func (u *userIdentity) IsAdmin() bool {
	if u == nil || u.User == nil {
		return false
	}
	return u.User.IsAdmin
}

// ContextFromIdentity returns a copy of parent context with the given Identity associated with it.
func ContextFromIdentity(ctx context.Context, id Identity) context.Context {
	return context.WithValue(ctx, ctxKey{}, id)
}

// ContextFromSessionToken returns a copy of parent context with a user Identity associated with it (if it exists).
func ContextFromSessionToken(ctx context.Context, graph *ent.Client, token string) (context.Context, error) {
	u, err := graph.User.Query().
		Where(user.SessionToken(token)).
		Only(ctx)
	if err != nil {
		return nil, err
	}

	return ContextFromIdentity(ctx, &userIdentity{true, u}), nil
}

// IdentityFromContext returns the identity associated with the provided context, or nil if no identity is associated.
func IdentityFromContext(ctx context.Context) Identity {
	val := ctx.Value(ctxKey{})
	id, ok := val.(Identity)
	if !ok {
		return nil
	}
	return id
}

// IsAuthenticatedContext returns true if the context is associated with an authenticated identity, false otherwise.
func IsAuthenticatedContext(ctx context.Context) bool {
	v, ok := ctx.Value(ctxKey{}).(Identity)
	if !ok || v == nil {
		return false
	}
	return v.IsAuthenticated()
}

// IsActivatedContext returns true if the context is associated with an activated identity, false otherwise.
func IsActivatedContext(ctx context.Context) bool {
	v, ok := ctx.Value(ctxKey{}).(Identity)
	if !ok || v == nil {
		return false
	}
	return v.IsActivated()
}

// IsAdminContext returns true if the context is associated with an admin identity, false otherwise.
func IsAdminContext(ctx context.Context) bool {
	v, ok := ctx.Value(ctxKey{}).(Identity)
	if !ok || v == nil {
		return false
	}
	return v.IsAdmin()
}
