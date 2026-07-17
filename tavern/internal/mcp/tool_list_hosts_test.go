package mcp_test

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/tag"
	tavernmcp "realm.pub/tavern/internal/mcp"
)

func TestHandleListHosts(t *testing.T) {
	client := setupTestDB(t)
	defer client.Close()

	ctx := context.WithValue(context.Background(), tavernmcp.ContextKey{}, client)

	// Create tags
	tag1 := client.Tag.Create().SetName("web").SetKind(tag.KindService).SaveX(ctx)
	tag2 := client.Tag.Create().SetName("db").SetKind(tag.KindService).SaveX(ctx)

	// Create mock hosts
	host1 := client.Host.Create().
		SetIdentifier("host-1").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SetName("web-server").
		SetPrimaryIP("192.168.1.10").
		SetExternalIP("203.0.113.10").
		SetLastSeenAt(time.Now()).
		AddTags(tag1).
		SaveX(ctx)

	// Create mock beacons and attach them to host1
	client.Beacon.Create().
		SetHost(host1).
		SetName("beacon-1").
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SetPrincipal("admin").
		SaveX(ctx)

	host2 := client.Host.Create().
		SetIdentifier("host-2").
		SetPlatform(c2pb.Host_PLATFORM_WINDOWS).
		SetName("db-server").
		SetPrimaryIP("192.168.1.20").
		SetExternalIP("203.0.113.20").
		SetLastSeenAt(time.Now()).
		AddTags(tag2).
		SaveX(ctx)
	_ = host2

	req := mcp.CallToolRequest{}
	res, err := tavernmcp.HandleListHosts(ctx, req)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.False(t, res.IsError)

	var results []map[string]interface{}
	textContent := res.Content[0].(mcp.TextContent).Text
	err = json.Unmarshal([]byte(textContent), &results)
	require.NoError(t, err)

	assert.Len(t, results, 2)

	// Verify host 1
	var h1Res map[string]interface{}
	var h2Res map[string]interface{}
	if results[0]["id"].(float64) == float64(host1.ID) {
		h1Res = results[0]
		h2Res = results[1]
	} else {
		h1Res = results[1]
		h2Res = results[0]
	}

	assert.Equal(t, "host-1", h1Res["identifier"])
	assert.Equal(t, "web-server", h1Res["name"])
	assert.Equal(t, "PLATFORM_LINUX", h1Res["platform"])
	assert.Equal(t, "192.168.1.10", h1Res["primaryIP"])
	assert.Equal(t, "203.0.113.10", h1Res["externalIP"])

	tags1 := h1Res["tags"].([]interface{})
	require.Len(t, tags1, 1)
	tagRes := tags1[0].(map[string]interface{})
	assert.Equal(t, "web", tagRes["name"])
	assert.Equal(t, "service", tagRes["kind"])

	beacons1 := h1Res["beacons"].([]interface{})
	require.Len(t, beacons1, 1)
	beaconRes := beacons1[0].(map[string]interface{})
	assert.Equal(t, "beacon-1", beaconRes["name"])
	assert.Equal(t, "admin", beaconRes["principal"])

	// Verify host 2
	assert.Equal(t, "host-2", h2Res["identifier"])
	assert.Equal(t, "db-server", h2Res["name"])
	assert.Equal(t, "PLATFORM_WINDOWS", h2Res["platform"])
	assert.Equal(t, "192.168.1.20", h2Res["primaryIP"])
	assert.Equal(t, "203.0.113.20", h2Res["externalIP"])
	tags2 := h2Res["tags"].([]interface{})
	require.Len(t, tags2, 1)
	tagRes2 := tags2[0].(map[string]interface{})
	assert.Equal(t, "db", tagRes2["name"])
	assert.Equal(t, "service", tagRes2["kind"])

	// test no client error
	res, err = tavernmcp.HandleListHosts(context.Background(), req)
	require.NoError(t, err)
	require.True(t, res.IsError)
	assert.Contains(t, res.Content[0].(mcp.TextContent).Text, "internal error: no database client")
}
