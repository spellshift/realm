package sshecho

import (
	"crypto/rand"
	"crypto/rsa"
	"fmt"
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
	app := NewApp()

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

func testInteractiveShell(t *testing.T, port int, clientConfig *ssh.ClientConfig) {
	conn, err := ssh.Dial("tcp", fmt.Sprintf("127.0.0.1:%d", port), clientConfig)
	require.NoError(t, err)
	defer conn.Close()

	session, err := conn.NewSession()
	require.NoError(t, err)
	defer session.Close()

	// Request pseudo terminal
	err = session.RequestPty("xterm", 80, 40, ssh.TerminalModes{
		ssh.ECHO:          0,     // disable echoing
		ssh.TTY_OP_ISPEED: 14400, // input speed = 14.4kbaud
		ssh.TTY_OP_OSPEED: 14400, // output speed = 14.4kbaud
	})
	require.NoError(t, err)

	stdin, err := session.StdinPipe()
	require.NoError(t, err)

	stdout, err := session.StdoutPipe()
	require.NoError(t, err)

	err = session.Shell()
	require.NoError(t, err)

	// Consume initial prompt
	buf := make([]byte, 256)
	n, err := stdout.Read(buf)
	require.NoError(t, err)
	require.Contains(t, string(buf[:n]), "sshecho> ")

	readUntilPrompt := func() string {
		out := make([]byte, 0, 1024)
		for {
			buf := make([]byte, 128)
			n, err := stdout.Read(buf)
			if err != nil {
				break
			}
			out = append(out, buf[:n]...)
			if len(out) >= 9 && string(out[len(out)-9:]) == "sshecho> " {
				break
			}
		}
		return string(out)
	}

	// Test "whoami" command
	_, err = stdin.Write([]byte("whoami\r"))
	require.NoError(t, err)

	outStr := readUntilPrompt()
	require.Contains(t, outStr, "root\r")

	// Test "ls" command
	_, err = stdin.Write([]byte("ls\r"))
	require.NoError(t, err)

	outStr = readUntilPrompt()
	require.Contains(t, outStr, "bin dev proc lib home etc\r")

	// Test "echo" command
	_, err = stdin.Write([]byte("echo hello world\r"))
	require.NoError(t, err)

	outStr = readUntilPrompt()
	require.Contains(t, outStr, "hello world\r")

	// Test "exit"
	_, err = stdin.Write([]byte("exit\r"))
	require.NoError(t, err)

	// Shell should exit cleanly
	err = session.Wait()
	require.NoError(t, err)

	// Test exec subsystem
	session2, err := conn.NewSession()
	require.NoError(t, err)
	defer session2.Close()

	out, err := session2.Output("whoami")
	require.NoError(t, err)
	require.Equal(t, string(out), "root\n")

	session3, err := conn.NewSession()
	require.NoError(t, err)
	defer session3.Close()

	out, err = session3.Output("echo hello world")
	require.NoError(t, err)
	require.Equal(t, string(out), "hello world\n")
}

func TestSSHEcho_NoAuth(t *testing.T) {
	port, cancel := startServer(t, []string{})
	defer cancel()

	clientConfig := &ssh.ClientConfig{
		User:            "test",
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}

	testInteractiveShell(t, port, clientConfig)
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

	testInteractiveShell(t, port, clientConfig)

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

	testInteractiveShell(t, port, clientConfig)

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
