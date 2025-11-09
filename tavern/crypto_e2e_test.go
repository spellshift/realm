package main

import (
	"context"
	"crypto/ecdh"
	"crypto/rand"
	"log/slog"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	_ "gocloud.dev/pubsub/mempubsub"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cryptocodec"
)

type TestCryptoE2E struct {
	wg         sync.WaitGroup
	httpServer *Server
	httpClient *http.Client
	grpcClient c2pb.C2Client
}

func (s *TestCryptoE2E) Close() {
	s.httpServer.Close()
	s.wg.Wait()
}

func setupCryptoE2E(t *testing.T) *TestCryptoE2E {
	t.Helper()
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)

	var s TestCryptoE2E

	dir := t.TempDir()
	secretsDir := filepath.Join(dir, "secrets")
	require.NoError(t, os.MkdirAll(secretsDir, 0755))
	secretsFile := filepath.Join(secretsDir, "secrets.yaml")
	os.Setenv("SECRETS_FILE_PATH", secretsFile)
	os.Setenv("ENABLE_TEST_RUN_AND_EXIT", "true")
	os.Setenv("DISABLE_DEFAULT_TOMES", "true")
	t.Cleanup(func() {
		os.Unsetenv("ENABLE_TEST_RUN_AND_EXIT")
		os.Unsetenv("SECRETS_FILE_PATH")
		os.Unsetenv("DISABLE_DEFAULT_TOMES")
	})

	// Create new server
	srv, err := NewServer(ctx,
		ConfigureHTTPServerFromEnv(func(srv *http.Server) {
			srv.Addr = "127.0.0.1:0"
		}),
	)
	require.NoError(t, err)
	s.httpServer = srv
	t.Cleanup(func() {
		s.httpServer.Close()
	})

	// Listen on assigned port
	lis, err := net.Listen("tcp", s.httpServer.HTTP.Addr)
	require.NoError(t, err)
	s.httpServer.HTTP.Addr = lis.Addr().String()
	t.Logf("server listening on %s", s.httpServer.HTTP.Addr)

	// Start serving HTTP
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		if err := s.httpServer.HTTP.Serve(lis); err != nil {
			slog.Debug("test http server exited", "err", err)
		}
	}()

	// Wait for server to be ready
	time.Sleep(1 * time.Second)
	require.Eventually(t, func() bool {
		slog.Debug("checking for test http server readiness")
		res, err := http.Get("http://" + s.httpServer.HTTP.Addr + "/status")
		if err != nil {
			slog.Debug("test http server is not ready yet", "err", err)
			t.Logf("failed to get status: %v", err)
			return false
		}
		defer res.Body.Close()
		slog.Debug("test http server is ready", "status_code", res.StatusCode)
		return res.StatusCode == http.StatusOK
	}, 30*time.Second, 100*time.Millisecond)

	return &s
}

func TestCryptoE2E_ClaimTasks(t *testing.T) {
	s := setupCryptoE2E(t)
	defer s.Close()
	ctx := context.Background()

	// Create test entities
	user, err := s.httpServer.client.User.Create().SetName("test-user").SetOauthID("test-oauth-id").SetPhotoURL("http://example.com/photo.jpg").Save(ctx)
	require.NoError(t, err)
	host, err := s.httpServer.client.Host.Create().SetIdentifier("test-host").SetPlatform(c2pb.Host_PLATFORM_LINUX).Save(ctx)
	require.NoError(t, err)
	beacon, err := s.httpServer.client.Beacon.Create().SetHost(host).Save(ctx)
	require.NoError(t, err)
	tome, err := s.httpServer.client.Tome.Create().SetName("test-tome").SetDescription("test-desc").SetAuthor("test-author").SetEldritch("test-eldritch").SetUploader(user).Save(ctx)
	require.NoError(t, err)
	quest, err := s.httpServer.client.Quest.Create().SetName("test-quest").SetTome(tome).SetCreator(user).Save(ctx)
	require.NoError(t, err)
	task, err := s.httpServer.client.Task.Create().SetQuest(quest).SetBeacon(beacon).Save(ctx)
	require.NoError(t, err)

	curve := ecdh.X25519()
	priv, err := curve.GenerateKey(rand.Reader)
	require.NoError(t, err)

	// gRPC Client Setup
	codec := cryptocodec.NewStreamDecryptCodec()
	grpcConn, err := grpc.Dial(s.httpServer.HTTP.Addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(grpc.ForceCodec(codec)),
	)
	require.NoError(t, err)
	s.grpcClient = c2pb.NewC2Client(grpcConn)

	// Test ClaimTasks with cryptography enabled
	req := &c2pb.ClaimTasksRequest{
		Beacon: &c2pb.Beacon{
			Host: &c2pb.Host{
				Identifier: host.Identifier,
			},
		},
	}
	resp, err := s.grpcClient.ClaimTasks(ctx, req)
	require.NoError(t, err)
	assert.Len(t, resp.Tasks, 1)
	assert.Equal(t, int64(task.ID), resp.Tasks[0].Id)
}
