package http1

import (
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// TestStreamConfigFetchAsset tests the FetchAsset stream configuration
func TestStreamConfigFetchAsset(t *testing.T) {
	if fetchAssetStream.MethodPath != "/c2.C2/FetchAsset" {
		t.Errorf("Expected method path '/c2.C2/FetchAsset', got '%s'", fetchAssetStream.MethodPath)
	}
	if !fetchAssetStream.Desc.ServerStreams {
		t.Error("FetchAsset should have server streaming enabled")
	}
	if fetchAssetStream.Desc.ClientStreams {
		t.Error("FetchAsset should not have client streaming enabled")
	}
}

// TestStreamConfigReportFile tests the ReportFile stream configuration
func TestStreamConfigReportFile(t *testing.T) {
	if reportFileStream.MethodPath != "/c2.C2/ReportFile" {
		t.Errorf("Expected method path '/c2.C2/ReportFile', got '%s'", reportFileStream.MethodPath)
	}
	if reportFileStream.Desc.ServerStreams {
		t.Error("ReportFile should not have server streaming enabled")
	}
	if !reportFileStream.Desc.ClientStreams {
		t.Error("ReportFile should have client streaming enabled")
	}
}

// TestCreateStreamWithContext tests stream creation with context timeout
func TestCreateStreamWithContext(t *testing.T) {
	conn, err := grpcTestConnection()
	if err != nil {
		t.Fatalf("Failed to create test connection: %v", err)
	}
	defer conn.Close()

	ctx, cancel := createRequestContext(5 * time.Second)
	defer cancel()

	// This will fail because there's no real server, but we're testing the context creation
	_, err = createStream(ctx, conn, fetchAssetStream)
	if err == nil {
		t.Fatalf("Expected error from unavailable server")
	}

	// Verify context can be used for timeout checks
	select {
	case <-ctx.Done():
		t.Error("Context should not be canceled yet")
	default:
		// Expected
	}
}

// TestStreamConfigProperties tests properties of stream configurations
func TestStreamConfigProperties(t *testing.T) {
	configs := []struct {
		name         string
		cfg          streamConfig
		expectServer bool
		expectClient bool
	}{
		{"FetchAsset", fetchAssetStream, true, false},
		{"ReportFile", reportFileStream, false, true},
	}

	for _, tc := range configs {
		t.Run(tc.name, func(t *testing.T) {
			if tc.cfg.Desc.ServerStreams != tc.expectServer {
				t.Errorf("%s: ServerStreams mismatch", tc.name)
			}
			if tc.cfg.Desc.ClientStreams != tc.expectClient {
				t.Errorf("%s: ClientStreams mismatch", tc.name)
			}
			if tc.cfg.Desc.StreamName == "" {
				t.Errorf("%s: StreamName should not be empty", tc.name)
			}
			if tc.cfg.MethodPath == "" {
				t.Errorf("%s: MethodPath should not be empty", tc.name)
			}
		})
	}
}

// grpcTestConnection creates a test gRPC connection to localhost
func grpcTestConnection() (*grpc.ClientConn, error) {
	conn, err := grpc.NewClient(
		"localhost:0",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	return conn, err
}
