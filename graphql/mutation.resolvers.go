package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"

	"github.com/kcarretto/realm/ent"
)

func (r *mutationResolver) CreateTarget(ctx context.Context, target CreateTargetInput) (*ent.Target, error) {
	return r.client.Target.Create().
		SetName(target.Name).
		SetForwardConnectIP(target.ForwardConnectIP).
		Save(ctx)
}

func (r *mutationResolver) CreateCredential(ctx context.Context, credential CreateCredentialInput) (*ent.Credential, error) {
	return r.client.Credential.Create().
		SetTargetID(credential.TargetID).
		SetPrincipal(credential.Principal).
		SetSecret(credential.Secret).
		SetKind(credential.Kind).
		Save(ctx)
}

// Mutation returns MutationResolver implementation.
func (r *Resolver) Mutation() MutationResolver { return &mutationResolver{r} }

type mutationResolver struct{ *Resolver }
