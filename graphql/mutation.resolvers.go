package graphql

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.

import (
	"context"

	"github.com/kcarretto/realm/ent"
	"github.com/kcarretto/realm/ent/implant"
	"github.com/kcarretto/realm/ent/implantconfig"
	"github.com/kcarretto/realm/ent/viewer"
)

func (r *mutationResolver) Callback(ctx context.Context, info CallbackInput) (*CallbackResponse, error) {
	// Get the viewer
	vc, err := viewer.ImplantFromContext(ctx)
	if err != nil {
		return nil, err
	}

	// Get the implant config based on the viewer
	configID, err := r.client.ImplantConfig.Query().
		Where(
			implantconfig.AuthToken(vc.AuthToken),
		).
		OnlyID(ctx)
	if err != nil {
		return nil, err
	}

	// Load the target
	target, err := r.client.Target.Get(ctx, info.TargetID)
	if err != nil {
		return nil, err
	}

	// Upsert the implant
	impQuery := r.client.Implant.Query().
		Where(implant.SessionID(info.SessionID))

	var imp *ent.Implant
	if exists := impQuery.Clone().ExistX(ctx); exists {
		impID := impQuery.OnlyIDX(ctx)
		imp = r.client.Implant.UpdateOneID(impID).
			SetProcessName(info.ProcessName).
			SaveX(ctx)
	} else {
		imp = r.client.Implant.Create().
			SetSessionID(info.SessionID).
			SetProcessName(info.ProcessName).
			SetConfigID(configID).
			SetTarget(target).
			SaveX(ctx)
	}

	// Format the response
	resp := CallbackResponse{
		Implant: imp,
	}

	return &resp, nil
}

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
