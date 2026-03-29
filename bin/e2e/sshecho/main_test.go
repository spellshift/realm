package main

import (
	"bytes"
	"crypto/rand"
	"crypto/rsa"
	"fmt"
	"io"
	"net"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/ssh"
)

func getFreePort() (int, error) {
	addr, err := net.ResolveTCPAddr("tcp", "localhost:0")
	if err != nil {
		return 0, err
	}

	l, err := net.ListenTCP("tcp", addr)
	if err != nil {
		return 0, err
	}
	defer l.Close()
	return l.Addr().(*net.TCPAddr).Port, nil
}

func startServer(t *testing.T, args []string) (int, func()) {
	port, err := getFreePort()
	require.NoError(t, err)

	appArgs := append([]string{"sshecho", "--port", fmt.Sprintf("%d", port)}, args...)

	done := make(chan struct{})
	app := newApp()

	if app.Metadata == nil {
		app.Metadata = make(map[string]interface{})
	}
	app.Metadata["done"] = done

	go func() {
		app.Run(appArgs)
	}()

	// Wait a moment for the server goroutine to start listening
	time.Sleep(500 * time.Millisecond)

	return port, func() {
		close(done)
	}
}

func testEcho(t *testing.T, port int, clientConfig *ssh.ClientConfig) {
	conn, err := ssh.Dial("tcp", fmt.Sprintf("127.0.0.1:%d", port), clientConfig)
	require.NoError(t, err)
	defer conn.Close()

	session, err := conn.NewSession()
	require.NoError(t, err)
	defer session.Close()

	stdin, err := session.StdinPipe()
	require.NoError(t, err)

	stdout, err := session.StdoutPipe()
	require.NoError(t, err)

	err = session.Shell()
	require.NoError(t, err)

	msg := []byte("hello world!\n")
	_, err = stdin.Write(msg)
	require.NoError(t, err)

	// Close stdin to signal end of input
	err = stdin.Close()
	require.NoError(t, err)

	var output bytes.Buffer
	_, err = io.Copy(&output, stdout)
	require.NoError(t, err)

	require.Equal(t, msg, output.Bytes())
}

func TestSSHEcho_NoAuth(t *testing.T) {
	port, cancel := startServer(t, []string{})
	defer cancel()

	clientConfig := &ssh.ClientConfig{
		User:            "test",
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}

	testEcho(t, port, clientConfig)
}

func TestSSHEcho_PasswordAuth(t *testing.T) {
	port, cancel := startServer(t, []string{"--user", "user1", "--password", "pass1"})
	defer cancel()

	clientConfig := &ssh.ClientConfig{
		User: "user1",
		Auth: []ssh.AuthMethod{
			ssh.Password("pass1"),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}

	testEcho(t, port, clientConfig)

	// Test invalid password
	invalidConfig := &ssh.ClientConfig{
		User: "user1",
		Auth: []ssh.AuthMethod{
			ssh.Password("wrong"),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}
	_, err := ssh.Dial("tcp", fmt.Sprintf("127.0.0.1:%d", port), invalidConfig)
	require.Error(t, err)
}

func TestSSHEcho_PublicKeyAuth(t *testing.T) {
	// Generate a key pair for the client
	privateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)

	pubKey, err := ssh.NewPublicKey(&privateKey.PublicKey)
	require.NoError(t, err)
	pubKeyBytes := ssh.MarshalAuthorizedKey(pubKey)

	// Save public key to a temp file
	tmpFile, err := os.CreateTemp("", "pubkey-*.pub")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	_, err = tmpFile.Write(pubKeyBytes)
	require.NoError(t, err)
	err = tmpFile.Close()
	require.NoError(t, err)

	port, cancel := startServer(t, []string{"--pubkey", tmpFile.Name()})
	defer cancel()

	signer, err := ssh.NewSignerFromKey(privateKey)
	require.NoError(t, err)

	clientConfig := &ssh.ClientConfig{
		User: "user1",
		Auth: []ssh.AuthMethod{
			ssh.PublicKeys(signer),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}

	testEcho(t, port, clientConfig)

	// Test invalid key
	wrongPrivateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)
	wrongSigner, err := ssh.NewSignerFromKey(wrongPrivateKey)
	require.NoError(t, err)

	invalidConfig := &ssh.ClientConfig{
		User: "user1",
		Auth: []ssh.AuthMethod{
			ssh.PublicKeys(wrongSigner),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}
	_, err = ssh.Dial("tcp", fmt.Sprintf("127.0.0.1:%d", port), invalidConfig)
	require.Error(t, err)
}
