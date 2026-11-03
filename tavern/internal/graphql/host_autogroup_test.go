package graphql

import (
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tag"
)

// seedProcessList creates a host, beacon and "Process list" task reporting
// the given executable paths.
func seedProcessList(t *testing.T, ctx context.Context, client *ent.Client, q *ent.Quest, name, ip string, platform c2pb.Host_Platform, paths []string) *ent.Host {
	t.Helper()
	h := client.Host.Create().
		SetIdentifier("id-" + name).
		SetName(name).
		SetPrimaryIP(ip).
		SetPlatform(platform).
		SaveX(ctx)
	b := client.Beacon.Create().SetHost(h).SetPrincipal("root").SetTransport(c2pb.Transport_TRANSPORT_GRPC).SaveX(ctx)
	task := client.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)
	for i, p := range paths {
		client.HostProcess.Create().
			SetHost(h).
			SetTask(task).
			SetPid(uint64(i + 1)).
			SetPpid(0).
			SetName(p).
			SetPath(p).
			SetStatus(epb.Process_STATUS_RUN).
			SaveX(ctx)
	}
	return h
}

func processListQuest(ctx context.Context, client *ent.Client, tomeName string) *ent.Quest {
	tm := client.Tome.Create().
		SetName(tomeName).
		SetDescription("d").
		SetAuthor("a").
		SetEldritch("e").
		SaveX(ctx)
	return client.Quest.Create().SetName("q").SetTome(tm).SaveX(ctx)
}

func TestRunHostAutogroup(t *testing.T) {
	ctx := context.Background()

	t.Run("groups hosts by role into randomly named service tags", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		q := processListQuest(ctx, client, "Process list")

		web := []string{"/usr/sbin/nginx", "/usr/sbin/sshd", "/usr/bin/php-fpm"}
		dc := []string{`C:\Windows\System32\lsass.exe`, `C:\Windows\System32\dns.exe`, `C:\Windows\System32\ntds.exe`}
		webHosts := map[int]bool{}
		dcHosts := map[int]bool{}
		for i := 0; i < 5; i++ {
			h := seedProcessList(t, ctx, client, q, fmt.Sprintf("web%d", i), fmt.Sprintf("10.0.1.%d", 10+i), c2pb.Host_PLATFORM_LINUX, web)
			webHosts[h.ID] = true
			h = seedProcessList(t, ctx, client, q, fmt.Sprintf("dc%d", i), fmt.Sprintf("10.0.2.%d", 10+i), c2pb.Host_PLATFORM_WINDOWS, dc)
			dcHosts[h.ID] = true
		}

		tags, err := runHostAutogroup(ctx, client, nil)
		require.NoError(t, err)
		require.GreaterOrEqual(t, len(tags), 2)

		seen := map[int]bool{}
		for _, tg := range tags {
			assert.Equal(t, tag.KindService, tg.Kind)
			assert.NotRegexp(t, `^group-\d+$`, tg.Name)
			assert.Contains(t, tg.Name, "-")

			hosts := tg.QueryHosts().AllX(ctx)
			require.NotEmpty(t, hosts)
			// Every cluster must be role-pure: all web or all dc.
			isWeb := webHosts[hosts[0].ID]
			for _, h := range hosts {
				assert.Equal(t, isWeb, webHosts[h.ID], "tag %q mixes web and dc hosts", tg.Name)
				assert.False(t, seen[h.ID], "host in more than one tag")
				seen[h.ID] = true
			}
		}
		assert.Len(t, seen, 10, "every host should be grouped")
	})

	t.Run("re-running leaves earlier tags in place", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		q := processListQuest(ctx, client, "Process list")
		for i := 0; i < 6; i++ {
			seedProcessList(t, ctx, client, q, fmt.Sprintf("h%d", i), fmt.Sprintf("10.0.0.%d", i), c2pb.Host_PLATFORM_LINUX,
				[]string{"/usr/sbin/sshd", fmt.Sprintf("/opt/app%d/bin", i%2)})
		}
		first, err := runHostAutogroup(ctx, client, nil)
		require.NoError(t, err)
		second, err := runHostAutogroup(ctx, client, nil)
		require.NoError(t, err)
		assert.Equal(t, len(first)+len(second), client.Tag.Query().CountX(ctx))
	})

	t.Run("k pins the exact number of groups", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		q := processListQuest(ctx, client, "Process list")
		profiles := [][]string{
			{"/usr/sbin/nginx", "/usr/sbin/sshd", "/usr/bin/php-fpm"},
			{`C:\Windows\System32\lsass.exe`, `C:\Windows\System32\dns.exe`},
			{"/usr/sbin/mysqld", "/usr/bin/mysqladmin"},
		}
		for g, procs := range profiles {
			for i := 0; i < 3; i++ {
				seedProcessList(t, ctx, client, q, fmt.Sprintf("h%d-%d", g, i), fmt.Sprintf("10.0.%d.%d", g, 10+i), c2pb.Host_PLATFORM_LINUX, procs)
			}
		}
		k := 3
		tags, err := runHostAutogroup(ctx, client, &k)
		require.NoError(t, err)
		assert.Len(t, tags, 3)

		total := 0
		for _, tg := range tags {
			total += tg.QueryHosts().CountX(ctx)
		}
		assert.Equal(t, 9, total)
	})

	t.Run("k must be positive", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		k := 0
		_, err := runHostAutogroup(ctx, client, &k)
		require.ErrorContains(t, err, "positive")
		assert.Zero(t, client.Tag.Query().CountX(ctx))

		k = -1
		_, err = runHostAutogroup(ctx, client, &k)
		require.ErrorContains(t, err, "positive")
	})

	t.Run("k must be less than the number of clusterable hosts", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		q := processListQuest(ctx, client, "Process list")
		for i := 0; i < 5; i++ {
			seedProcessList(t, ctx, client, q, fmt.Sprintf("h%d", i), fmt.Sprintf("10.0.0.%d", i), c2pb.Host_PLATFORM_LINUX, []string{"/bin/sh"})
		}
		k := 5
		_, err := runHostAutogroup(ctx, client, &k)
		require.ErrorContains(t, err, "too few")
		assert.Zero(t, client.Tag.Query().CountX(ctx))
	})

	t.Run("errors when no process list tasks exist", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		_, err := runHostAutogroup(ctx, client, nil)
		require.Error(t, err)
		assert.Zero(t, client.Tag.Query().CountX(ctx))
	})

	t.Run("errors when too few hosts to cluster", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		q := processListQuest(ctx, client, "Process list")
		for i := 0; i < 2; i++ {
			seedProcessList(t, ctx, client, q, fmt.Sprintf("h%d", i), fmt.Sprintf("10.0.0.%d", i), c2pb.Host_PLATFORM_LINUX, []string{"/bin/sh"})
		}
		_, err := runHostAutogroup(ctx, client, nil)
		require.ErrorContains(t, err, "too few")
		assert.Zero(t, client.Tag.Query().CountX(ctx))
	})

	t.Run("ignores tasks from other tomes", func(t *testing.T) {
		client := enttest.OpenTempDB(t)
		defer client.Close()
		q := processListQuest(ctx, client, "Something else")
		for i := 0; i < 6; i++ {
			seedProcessList(t, ctx, client, q, fmt.Sprintf("h%d", i), fmt.Sprintf("10.0.0.%d", i), c2pb.Host_PLATFORM_LINUX, []string{"/bin/sh"})
		}
		_, err := runHostAutogroup(ctx, client, nil)
		require.Error(t, err)
	})
}
