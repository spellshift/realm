package e2e_test

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"cloud.google.com/go/pubsub/v2/pstest"
	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
	"realm.pub/tests/e2e/utils/sshecho/sshecho"
)

type GraphQLResponse struct {
	Data   json.RawMessage `json:"data"`
	Errors []struct {
		Message string `json:"message"`
	} `json:"errors"`
}

func queryGraphQL(t *testing.T, tavernURL string, query string, variables map[string]interface{}) json.RawMessage {
	t.Helper()
	bodyObj := map[string]interface{}{
		"query": query,
	}
	if variables != nil {
		bodyObj["variables"] = variables
	}
	bodyBytes, err := json.Marshal(bodyObj)
	require.NoError(t, err)

	resp, err := http.Post(tavernURL+"/graphql", "application/json", bytes.NewReader(bodyBytes))
	require.NoError(t, err)
	defer resp.Body.Close()

	respBytes, err := io.ReadAll(resp.Body)
	require.NoError(t, err)

	var gqlResp GraphQLResponse
	require.NoError(t, json.Unmarshal(respBytes, &gqlResp), "failed to unmarshal graphql response: %s", string(respBytes))
	if len(gqlResp.Errors) > 0 {
		t.Fatalf("graphql returned errors: %v (response: %s)", gqlResp.Errors, string(respBytes))
	}
	return gqlResp.Data
}

func getFreePort(t *testing.T) int {
	t.Helper()
	l, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	port := l.Addr().(*net.TCPAddr).Port
	_ = l.Close()
	return port
}

func TestSSHPivotE2E(t *testing.T) {
	repoRoot, err := filepath.Abs("../..")
	require.NoError(t, err)

	// 1. Start mock SSH server (sshecho)
	sshPort := getFreePort(t)
	sshAddr := fmt.Sprintf("127.0.0.1:%d", sshPort)
	sshListener, err := sshecho.Run(sshAddr, "testuser", "testpass", "", false)
	require.NoError(t, err)
	defer sshListener.Close()
	t.Logf("sshecho mock SSH server listening on %s", sshAddr)

	// 2. Start GCP PubSub test emulator (pstest)
	psServer := pstest.NewServer()
	defer psServer.Close()
	t.Logf("pstest PubSub emulator listening on %s", psServer.Addr)

	// 3. Build Tavern binary if not present
	tavernBin := filepath.Join(repoRoot, "build", "tavern_e2e_bin")
	if _, err := os.Stat(tavernBin); os.IsNotExist(err) {
		t.Log("Building Tavern binary...")
		require.NoError(t, os.MkdirAll(filepath.Dir(tavernBin), 0755))
		buildCmd := exec.Command("go", "build", "-o", tavernBin, "./tavern")
		buildCmd.Dir = repoRoot
		out, err := buildCmd.CombinedOutput()
		require.NoError(t, err, "failed to build tavern: %s", string(out))
	}

	// 4. Start Tavern
	tavernPort := getFreePort(t)
	tavernAddr := fmt.Sprintf(":%d", tavernPort)
	tavernURL := fmt.Sprintf("http://127.0.0.1:%d", tavernPort)

	tavernCmd := exec.Command(tavernBin)
	tavernCmd.Dir = repoRoot
	tempSecretsFile := filepath.Join(t.TempDir(), "secrets.yaml")
	tavernCmd.Env = append(os.Environ(),
		fmt.Sprintf("HTTP_LISTEN_ADDR=%s", tavernAddr),
		"GCP_PROJECT_ID=test-project",
		fmt.Sprintf("PUBSUB_EMULATOR_HOST=%s", psServer.Addr),
		fmt.Sprintf("SECRETS_FILE_PATH=%s", tempSecretsFile),
		"ENABLE_DEBUG_LOGGING=true",
	)

	var tavernLog bytes.Buffer
	tavernCmd.Stdout = &tavernLog
	tavernCmd.Stderr = &tavernLog

	require.NoError(t, tavernCmd.Start())
	defer func() {
		if tavernCmd.Process != nil {
			_ = tavernCmd.Process.Kill()
		}
	}()

	// Wait for Tavern to be ready
	var pubkey string
	deadline := time.Now().Add(15 * time.Second)
	for time.Now().Before(deadline) {
		resp, err := http.Get(tavernURL + "/status")
		if err == nil && resp.StatusCode == http.StatusOK {
			var statusResp struct {
				Pubkey string `json:"Pubkey"`
			}
			if err := json.NewDecoder(resp.Body).Decode(&statusResp); err == nil && statusResp.Pubkey != "" {
				pubkey = statusResp.Pubkey
				resp.Body.Close()
				break
			}
			resp.Body.Close()
		}
		time.Sleep(200 * time.Millisecond)
	}
	require.NotEmpty(t, pubkey, "Tavern failed to start or return pubkey. Logs:\n%s", tavernLog.String())
	t.Logf("Tavern started on %s with Pubkey: %s", tavernURL, pubkey)

	// 5. Run imix agent
	imixBin := filepath.Join(repoRoot, "implants", "target", "debug", "imix")
	if _, err := os.Stat(imixBin); os.IsNotExist(err) {
		t.Log("Building imix binary...")
		buildCmd := exec.Command("cargo", "build", "--bin", "imix", "--manifest-path", filepath.Join(repoRoot, "implants", "imix", "Cargo.toml"))
		buildCmd.Dir = repoRoot
		out, err := buildCmd.CombinedOutput()
		require.NoError(t, err, "failed to build imix: %s", string(out))
	}

	imixCmd := exec.Command(imixBin)
	imixCmd.Dir = filepath.Join(repoRoot, "implants", "imix")
	imixCmd.Env = append(os.Environ(),
		fmt.Sprintf("IMIX_CALLBACK_URI=%s", tavernURL),
		fmt.Sprintf("IMIX_SERVER_PUBKEY=%s", pubkey),
		"IMIX_CALLBACK_INTERVAL=1",
		"RUST_LOG=debug",
	)

	var imixLog bytes.Buffer
	imixCmd.Stdout = &imixLog
	imixCmd.Stderr = &imixLog

	require.NoError(t, imixCmd.Start())
	defer func() {
		if imixCmd.Process != nil {
			_ = imixCmd.Process.Kill()
		}
	}()

	// 6. Wait for beacon to register in Tavern
	var beaconID string
	t.Log("Waiting for beacon callback...")
	beaconDeadline := time.Now().Add(20 * time.Second)
	for time.Now().Before(beaconDeadline) {
		data := queryGraphQL(t, tavernURL, `query { beacons { edges { node { id } } } }`, nil)
		var res struct {
			Beacons struct {
				Edges []struct {
					Node struct {
						ID string `json:"id"`
					} `json:"node"`
				} `json:"edges"`
			} `json:"beacons"`
		}
		if err := json.Unmarshal(data, &res); err == nil && len(res.Beacons.Edges) > 0 {
			beaconID = res.Beacons.Edges[0].Node.ID
			break
		}
		time.Sleep(500 * time.Millisecond)
	}
	require.NotEmpty(t, beaconID, "Agent failed to register beacon. Imix log:\n%s\nTavern log:\n%s", imixLog.String(), tavernLog.String())
	t.Logf("Beacon registered with ID: %s", beaconID)

	// 7. Find Tome for SOCKS5 Relay / Portal Create
	tomeData := queryGraphQL(t, tavernURL, `query { tomes { edges { node { id name } } } }`, nil)
	var tomesRes struct {
		Tomes struct {
			Edges []struct {
				Node struct {
					ID   string `json:"id"`
					Name string `json:"name"`
				} `json:"node"`
			} `json:"edges"`
		} `json:"tomes"`
	}
	require.NoError(t, json.Unmarshal(tomeData, &tomesRes))
	var portalTomeID string
	for _, edge := range tomesRes.Tomes.Edges {
		if strings.Contains(strings.ToLower(edge.Node.Name), "socks5") || strings.Contains(strings.ToLower(edge.Node.Name), "portal") {
			portalTomeID = edge.Node.ID
			break
		}
	}
	require.NotEmpty(t, portalTomeID, "could not find portal creation tome in Tavern")
	t.Logf("Found portal tome: ID %s", portalTomeID)

	// 8. Create Quest to launch the portal
	t.Log("Submitting quest to create portal...")
	questData := queryGraphQL(t, tavernURL, `
		mutation CreatePortalQuest($beaconIDs: [ID!]!, $input: CreateQuestInput!) {
			createQuest(beaconIDs: $beaconIDs, input: $input) {
				id
			}
		}
	`, map[string]interface{}{
		"beaconIDs": []string{beaconID},
		"input": map[string]interface{}{
			"name":   "e2e-ssh-portal-quest",
			"tomeID": portalTomeID,
		},
	})
	var questRes struct {
		CreateQuest struct {
			ID string `json:"id"`
		} `json:"createQuest"`
	}
	require.NoError(t, json.Unmarshal(questData, &questRes))
	t.Logf("Created quest with ID: %s", questRes.CreateQuest.ID)

	// 9. Wait for Portal to be created and active
	var portalID string
	t.Log("Waiting for portal to be created by agent...")
	portalDeadline := time.Now().Add(25 * time.Second)
	for time.Now().Before(portalDeadline) {
		pData := queryGraphQL(t, tavernURL, `query { portals { edges { node { id closedAt } } } }`, nil)
		var pRes struct {
			Portals struct {
				Edges []struct {
					Node struct {
						ID       string  `json:"id"`
						ClosedAt *string `json:"closedAt"`
					} `json:"node"`
				} `json:"edges"`
			} `json:"portals"`
		}
		if err := json.Unmarshal(pData, &pRes); err == nil && len(pRes.Portals.Edges) > 0 {
			for _, edge := range pRes.Portals.Edges {
				if edge.Node.ClosedAt == nil {
					portalID = edge.Node.ID
					break
				}
			}
			if portalID != "" {
				break
			}
		}
		time.Sleep(500 * time.Millisecond)
	}
	require.NotEmpty(t, portalID, "Portal was not created in time. Tavern log:\n%s\nImix log:\n%s", tavernLog.String(), imixLog.String())
	t.Logf("Portal is active with ID: %s", portalID)

	// 10. Connect via WebSocket to SSH Pivot
	wsURL := fmt.Sprintf("ws://127.0.0.1:%d/portals/ssh/ws?portal_id=%s&target=testuser:testpass@%s",
		tavernPort, portalID, sshAddr)
	t.Logf("Dialing SSH pivot websocket: %s", wsURL)

	ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer ws.Close()

	msgChan := make(chan map[string]interface{}, 100)
	errChan := make(chan error, 1)
	go func() {
		for {
			_, msg, err := ws.ReadMessage()
			if err != nil {
				errChan <- err
				return
			}
			var parsed map[string]interface{}
			if err := json.Unmarshal(msg, &parsed); err == nil {
				msgChan <- parsed
			}
		}
	}()

	waitForOutput := func(expected string, timeout time.Duration) string {
		var accumulated strings.Builder
		timer := time.NewTimer(timeout)
		defer timer.Stop()
		for {
			select {
			case m := <-msgChan:
				t.Logf("WS message received: %+v", m)
				if kind, ok := m["kind"].(string); ok {
					if kind == "OUTPUT" {
						if out, ok := m["output"].(string); ok {
							accumulated.WriteString(out)
							if strings.Contains(accumulated.String(), expected) {
								return accumulated.String()
							}
						}
					} else if kind == "ERROR" {
						t.Fatalf("Received ERROR from websocket: %v. Tavern log:\n%s\nImix log:\n%s", m["error"], tavernLog.String(), imixLog.String())
					}
				}
			case err := <-errChan:
				t.Fatalf("WS closed or failed: %v. Accumulated: %q. Tavern log:\n%s\nImix log:\n%s",
					err, accumulated.String(), tavernLog.String(), imixLog.String())
			case <-timer.C:
				t.Fatalf("Timed out waiting for %q, accumulated: %q. Tavern log:\n%s\nImix log:\n%s",
					expected, accumulated.String(), tavernLog.String(), imixLog.String())
			}
		}
	}

	// Wait for sshecho shell prompt
	waitForOutput("sshecho> ", 15*time.Second)
	t.Log("Received initial SSH prompt from mock SSH server!")

	// Send 'echo e2e-pivot-works' command
	sendInput := func(input string) {
		require.NoError(t, ws.WriteJSON(map[string]interface{}{
			"kind":  "INPUT",
			"input": input,
		}))
	}

	sendInput("echo e2e-pivot-works\n")
	waitForOutput("e2e-pivot-works", 5*time.Second)
	t.Log("Received command echo response through SSH pivot tunnel!")

	// Send 'whoami' command
	sendInput("whoami\n")
	waitForOutput("root", 5*time.Second)
	t.Log("Received whoami response through SSH pivot tunnel!")

	// Send 'exit' command
	sendInput("exit\n")
	t.Log("Sent exit command to SSH pivot session")

	// 11. Also test connecting via the pssh CLI tool
	t.Run("PSSH_CLI", func(t *testing.T) {
		psshBin := filepath.Join(repoRoot, "build", "pssh")
		if _, err := os.Stat(psshBin); os.IsNotExist(err) {
			t.Log("Building pssh binary...")
			bCmd := exec.Command("go", "build", "-o", psshBin, "./tests/e2e/utils/pssh")
			bCmd.Dir = repoRoot
			out, err := bCmd.CombinedOutput()
			require.NoError(t, err, "failed to build pssh: %s", string(out))
		}

		psshCmd := exec.Command(psshBin,
			"-portal", portalID,
			"-target", fmt.Sprintf("testuser:testpass@%s", sshAddr),
			"-url", fmt.Sprintf("ws://127.0.0.1:%d/portals/ssh/ws", tavernPort),
		)
		stdin, err := psshCmd.StdinPipe()
		require.NoError(t, err)
		stdout, err := psshCmd.StdoutPipe()
		require.NoError(t, err)

		require.NoError(t, psshCmd.Start())
		defer func() {
			_ = stdin.Close()
			if psshCmd.Process != nil {
				_ = psshCmd.Process.Kill()
			}
		}()

		reader := bufio.NewReader(stdout)
		readChan := make(chan string, 100)
		go func() {
			buf := make([]byte, 1024)
			for {
				n, err := reader.Read(buf)
				if n > 0 {
					readChan <- string(buf[:n])
				}
				if err != nil {
					return
				}
			}
		}()

		waitCliOutput := func(expected string, timeout time.Duration) {
			var accum strings.Builder
			timer := time.NewTimer(timeout)
			defer timer.Stop()
			for {
				select {
				case chunk := <-readChan:
					accum.WriteString(chunk)
					if strings.Contains(accum.String(), expected) {
						return
					}
				case <-timer.C:
					t.Fatalf("pssh CLI timed out waiting for %q. Accumulated:\n%s", expected, accum.String())
				}
			}
		}

		waitCliOutput("sshecho> ", 10*time.Second)
		_, _ = stdin.Write([]byte("echo pssh-cli-works\n"))
		waitCliOutput("pssh-cli-works", 5*time.Second)
		_, _ = stdin.Write([]byte("exit\n"))
		t.Log("pssh CLI successfully executed commands through SSH pivot tunnel!")
	})
}
