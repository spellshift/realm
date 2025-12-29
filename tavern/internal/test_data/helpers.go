package test_data

import (
	"context"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/user"
)

func User(ctx context.Context, client *ent.Client) *ent.User {
	u, _ := client.User.Query().Where(user.Name("User")).First(ctx)
	if u == nil {
		CreateTestData(ctx, client)
		u, _ = client.User.Query().Where(user.Name("User")).First(ctx)
	}
	return u
}

func Beacon(ctx context.Context, client *ent.Client, u *ent.User) *ent.Beacon {
	b, _ := client.Beacon.Query().First(ctx)
	if b == nil {
		CreateTestData(ctx, client)
		b, _ = client.Beacon.Query().First(ctx)
	}
	return b
}

func Tome(ctx context.Context, client *ent.Client, u *ent.User) *ent.Tome {
	t, _ := client.Tome.Query().First(ctx)
	if t == nil {
		CreateTestData(ctx, client)
		t, _ = client.Tome.Query().First(ctx)
	}
	return t
}

func Quest(ctx context.Context, client *ent.Client, u *ent.User, t *ent.Tome) *ent.Quest {
	q, _ := client.Quest.Query().First(ctx)
	if q == nil {
		CreateTestData(ctx, client)
		q, _ = client.Quest.Query().First(ctx)
	}
	return q
}

func Task(ctx context.Context, client *ent.Client, q *ent.Quest, b *ent.Beacon) *ent.Task {
	// Create a fresh task for the benchmark to ensure clean state if needed
	// But test data already has tasks.
	// Let's create a new one to be safe and specific.
	t, err := client.Task.Create().
		SetBeacon(b).
		SetQuest(q).
		Save(ctx)
	if err != nil {
		// fallback to existing
		t, _ = client.Task.Query().First(ctx)
	}
	return t
}
