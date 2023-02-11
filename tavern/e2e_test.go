package main_test

import (
	"context"
	"fmt"
	"io"
	"net/http/httptest"
	"testing"

	tavern "github.com/kcarretto/realm/tavern"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEndToEnd(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())

	srv, err := tavern.NewServer(ctx)
	require.NoError(t, err)

	ts := httptest.NewUnstartedServer(srv.Handler)
	ts.Config = srv
	ts.Start()
	defer ts.Close()
	// var wg sync.WaitGroup
	// wg.Add(1)
	// go func() {
	// 	defer wg.Done()
	// 	defer ts.Close()
	// 	ts.Start()
	// }()

	t.Run("StatusHandler", func(t *testing.T) {
		resp, err := ts.Client().Get(fmt.Sprintf("%s/status", ts.URL))
		require.NoError(t, err)
		body, err := io.ReadAll(resp.Body)
		require.NoError(t, err)
		assert.Equal(t, tavern.OKStatusText, string(body))
	})

	cancel()
	// ts.Close()
	// wg.Wait()
}
