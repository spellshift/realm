package main

import (
	"context"
	"fmt"
	"testing"
	"time"

	"entgo.io/ent/dialect/sql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/migrate"
)

func openTestSQLiteUTCClient(t *testing.T, dbName string) *ent.Client {
	t.Helper()

	dsn := fmt.Sprintf("file:%s_%d?mode=memory&cache=shared&_fk=1&_loc=UTC", dbName, time.Now().UnixNano())
	drv, err := sql.Open("sqlite3", dsn)
	require.NoError(t, err)

	client := ent.NewClient(ent.Driver(&utcDriver{Driver: drv}))
	t.Cleanup(func() {
		_ = client.Close()
	})

	require.NoError(t, client.Schema.Create(
		context.Background(),
		migrate.WithGlobalUniqueID(true),
	))
	return client
}

func TestSQLiteUTCDriver_NextSeenAtFiltering(t *testing.T) {
	newTimezoneTest := func(dbName string, loc *time.Location) func(*testing.T) {
		return func(t *testing.T) {
			ctx := context.Background()
			client := openTestSQLiteUTCClient(t, dbName)

			nowLocal := time.Date(2026, 11, 19, 13, 30, 0, 0, time.UTC).In(loc)
			activeNextSeen := nowLocal.Add(60 * time.Second)
			offlineNextSeen := nowLocal.Add(-5 * time.Minute)

			h, err := client.Host.Create().
				SetIdentifier("host-" + dbName).
				SetName("test-host").
				SetPlatform(c2pb.Host_PLATFORM_LINUX).
				SetLastSeenAt(nowLocal).
				SetNextSeenAt(activeNextSeen).
				Save(ctx)
			require.NoError(t, err)

			activeBeacon, err := client.Beacon.Create().
				SetIdentifier("active-" + dbName).
				SetPrincipal("root").
				SetAgentIdentifier("imix").
				SetHostID(h.ID).
				SetTransport(c2pb.Transport_TRANSPORT_GRPC).
				SetInterval(60).
				SetLastSeenAt(nowLocal).
				SetNextSeenAt(activeNextSeen).
				Save(ctx)
			require.NoError(t, err)

			_, err = client.Beacon.Create().
				SetIdentifier("offline-" + dbName).
				SetPrincipal("root").
				SetAgentIdentifier("imix").
				SetHostID(h.ID).
				SetTransport(c2pb.Transport_TRANSPORT_GRPC).
				SetInterval(60).
				SetLastSeenAt(offlineNextSeen.Add(-60 * time.Second)).
				SetNextSeenAt(offlineNextSeen).
				Save(ctx)
			require.NoError(t, err)

			// Matches Create a Quest's useOnlineBeaconIds filter (UTC RFC3339).
			cutoffUTC := nowLocal.Add(-30 * time.Second).UTC()
			online, err := client.Beacon.Query().
				Where(beacon.NextSeenAtGTE(cutoffUTC)).
				All(ctx)
			require.NoError(t, err)
			require.Len(t, online, 1)
			assert.Equal(t, activeBeacon.ID, online[0].ID)
			assert.Equal(t, time.UTC, online[0].NextSeenAt.Location())
			assert.True(t, online[0].NextSeenAt.Equal(activeNextSeen))
		}
	}

	t.Run("NegativeOffset_EST", newTimezoneTest("utc_est", time.FixedZone("EST", -5*3600)))
	t.Run("NegativeOffset_PST", newTimezoneTest("utc_pst", time.FixedZone("PST", -8*3600)))
	t.Run("PositiveOffset_CET", newTimezoneTest("utc_cet", time.FixedZone("CET", 1*3600)))
	t.Run("PositiveOffset_JST", newTimezoneTest("utc_jst", time.FixedZone("JST", 9*3600)))
	t.Run("UTC", newTimezoneTest("utc_utc", time.UTC))
}

func TestSQLiteUTCDriver_Transactions(t *testing.T) {
	ctx := context.Background()
	est := time.FixedZone("EST", -5*3600)
	nowEST := time.Date(2026, 11, 19, 13, 30, 0, 0, time.UTC).In(est)
	nextSeenEST := nowEST.Add(60 * time.Second)
	cutoffUTC := nowEST.Add(-30 * time.Second).UTC()

	runTxTest := func(dbName string, begin func(*ent.Client) (*ent.Tx, error)) func(*testing.T) {
		return func(t *testing.T) {
			client := openTestSQLiteUTCClient(t, dbName)

			tx, err := begin(client)
			require.NoError(t, err)

			h, err := tx.Host.Create().
				SetIdentifier("tx-host-" + dbName).
				SetName("tx-host").
				SetPlatform(c2pb.Host_PLATFORM_LINUX).
				SetLastSeenAt(nowEST).
				SetNextSeenAt(nextSeenEST).
				Save(ctx)
			require.NoError(t, err)

			created, err := tx.Beacon.Create().
				SetIdentifier("tx-beacon-" + dbName).
				SetPrincipal("root").
				SetAgentIdentifier("imix").
				SetHostID(h.ID).
				SetTransport(c2pb.Transport_TRANSPORT_GRPC).
				SetInterval(60).
				SetLastSeenAt(nowEST).
				SetNextSeenAt(nextSeenEST).
				Save(ctx)
			require.NoError(t, err)

			inTx, err := tx.Beacon.Query().
				Where(beacon.NextSeenAtGTE(cutoffUTC)).
				All(ctx)
			require.NoError(t, err)
			require.Len(t, inTx, 1)
			assert.Equal(t, created.ID, inTx[0].ID)

			require.NoError(t, tx.Commit())

			afterCommit, err := client.Beacon.Query().
				Where(beacon.NextSeenAtGTE(cutoffUTC)).
				All(ctx)
			require.NoError(t, err)
			require.Len(t, afterCommit, 1)
			assert.Equal(t, created.ID, afterCommit[0].ID)
		}
	}

	t.Run("Tx", runTxTest("utc_tx", func(c *ent.Client) (*ent.Tx, error) {
		return c.Tx(ctx)
	}))
	t.Run("BeginTx", runTxTest("utc_begintx", func(c *ent.Client) (*ent.Tx, error) {
		return c.BeginTx(ctx, nil)
	}))
}

func TestConfigConnectSQLiteNormalizesUTC(t *testing.T) {
	ctx := context.Background()
	cfg := &Config{}
	client, err := cfg.Connect()
	require.NoError(t, err)
	t.Cleanup(func() {
		_ = client.Close()
	})

	require.NoError(t, client.Schema.Create(
		ctx,
		migrate.WithGlobalUniqueID(true),
	))

	est := time.FixedZone("EST", -5*3600)
	nowEST := time.Date(2026, 11, 19, 13, 30, 0, 0, time.UTC).In(est)
	nextSeenEST := nowEST.Add(60 * time.Second)
	cutoffUTC := nowEST.Add(-30 * time.Second).UTC()

	suffix := fmt.Sprintf("%d", time.Now().UnixNano())

	h, err := client.Host.Create().
		SetIdentifier("connect-utc-host-" + suffix).
		SetName("connect-utc-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetLastSeenAt(nowEST).
		SetNextSeenAt(nextSeenEST).
		Save(ctx)
	require.NoError(t, err)
	t.Cleanup(func() {
		_ = client.Host.DeleteOneID(h.ID).Exec(ctx)
	})

	b, err := client.Beacon.Create().
		SetIdentifier("connect-utc-beacon-" + suffix).
		SetPrincipal("root").
		SetAgentIdentifier("imix").
		SetHostID(h.ID).
		SetTransport(c2pb.Transport_TRANSPORT_GRPC).
		SetInterval(60).
		SetLastSeenAt(nowEST).
		SetNextSeenAt(nextSeenEST).
		Save(ctx)
	require.NoError(t, err)
	t.Cleanup(func() {
		_ = client.Beacon.DeleteOneID(b.ID).Exec(ctx)
	})

	found, err := client.Beacon.Query().
		Where(
			beacon.ID(b.ID),
			beacon.NextSeenAtGTE(cutoffUTC),
		).
		All(ctx)
	require.NoError(t, err)
	require.Len(t, found, 1)
	assert.Equal(t, b.ID, found[0].ID)
}

func TestNormalizeTimeArgsUTC(t *testing.T) {
	est := time.FixedZone("EST", -5*3600)
	ts := time.Date(2026, 11, 19, 8, 30, 0, 0, est)
	tsPtr := ts
	nullValid := sql.NullTime{Time: ts, Valid: true}
	nullValidPtr := &sql.NullTime{Time: ts, Valid: true}
	nullInvalid := sql.NullTime{Time: ts, Valid: false}
	var nilTimePtr *time.Time
	var nilNullTimePtr *sql.NullTime

	orig := []any{
		"plain",
		ts,
		&tsPtr,
		nullValid,
		nullValidPtr,
		nullInvalid,
		nilTimePtr,
		nilNullTimePtr,
		ts.UTC(),
	}

	got, ok := normalizeTimeArgsUTC(orig).([]any)
	require.True(t, ok)

	assert.Equal(t, "plain", got[0])
	assert.Equal(t, time.UTC, got[1].(time.Time).Location())
	assert.True(t, got[1].(time.Time).Equal(ts))
	assert.Equal(t, time.UTC, got[2].(*time.Time).Location())
	assert.True(t, got[2].(*time.Time).Equal(ts))
	assert.Equal(t, time.UTC, got[3].(sql.NullTime).Time.Location())
	assert.Equal(t, time.UTC, got[4].(*sql.NullTime).Time.Location())
	assert.Equal(t, est, got[5].(sql.NullTime).Time.Location())
	assert.Nil(t, got[6])
	assert.Nil(t, got[7])
	assert.Equal(t, time.UTC, got[8].(time.Time).Location())

	// Caller's slice and pointed-to values must not be mutated.
	assert.Equal(t, est, orig[1].(time.Time).Location())
	assert.Equal(t, est, tsPtr.Location())
	assert.Equal(t, est, nullValidPtr.Time.Location())
}
