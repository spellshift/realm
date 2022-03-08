package main

import (
	"context"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/credential"
)

func createTestData(ctx context.Context, client *ent.Client) {
	target := client.Target.Create().
		SetName("Test").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)
	client.Credential.Create().
		SetPrincipal("root").
		SetSecret("changeme").
		SetKind(credential.KindPassword).
		SetTarget(target).
		SaveX(ctx)
	client.Credential.Create().
		SetPrincipal("admin").
		SetSecret("password1!").
		SetKind(credential.KindPassword).
		SetTarget(target).
		SaveX(ctx)
}
