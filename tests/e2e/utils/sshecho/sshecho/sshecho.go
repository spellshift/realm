package sshecho

import (
	"crypto/rand"
	"crypto/rsa"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"strings"

	"golang.org/x/crypto/ssh"
	"golang.org/x/term"
)

// Run starts the SSH echo server on the given address, with optional user, password, and public key file for auth.
// If systemAuth is true, password authentication is performed against the system (e.g. PAM) instead of
// using the hardcoded user/password. In this mode, the user and password flags are ignored for password auth.
// It returns a net.Listener that can be closed to stop the server.
func Run(addr string, user string, password string, pubkeyFile string, systemAuth bool) (net.Listener, error) {
	config := &ssh.ServerConfig{
		NoClientAuth: true,
	}

	if systemAuth {
		config.NoClientAuth = false
		config.PasswordCallback = func(c ssh.ConnMetadata, pass []byte) (*ssh.Permissions, error) {
			if err := pamAuthenticate(c.User(), string(pass)); err != nil {
				return nil, fmt.Errorf("system auth rejected for %q: %w", c.User(), err)
			}
			return nil, nil
		}
		log.Println("System authentication (PAM) enabled")
	} else if user != "" && password != "" {
		config.NoClientAuth = false
		config.PasswordCallback = func(c ssh.ConnMetadata, pass []byte) (*ssh.Permissions, error) {
			if c.User() == user && string(pass) == password {
				return nil, nil
			}
			return nil, fmt.Errorf("password rejected for %q", c.User())
		}
	}

	if pubkeyFile != "" {
		config.NoClientAuth = false
		pubkeyBytes, err := os.ReadFile(pubkeyFile)
		if err != nil {
			return nil, fmt.Errorf("failed to read public key: %v", err)
		}
		allowedKey, _, _, _, err := ssh.ParseAuthorizedKey(pubkeyBytes)
		if err != nil {
			return nil, fmt.Errorf("failed to parse public key: %v", err)
		}

		config.PublicKeyCallback = func(c ssh.ConnMetadata, pubKey ssh.PublicKey) (*ssh.Permissions, error) {
			if string(allowedKey.Marshal()) == string(pubKey.Marshal()) {
				return nil, nil
			}
			return nil, fmt.Errorf("public key rejected for %q", c.User())
		}
	}

	privateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, fmt.Errorf("failed to generate private key: %v", err)
	}

	signer, err := ssh.NewSignerFromKey(privateKey)
	if err != nil {
		return nil, fmt.Errorf("failed to create signer: %v", err)
	}

	config.AddHostKey(signer)

	listener, err := net.Listen("tcp", addr)
	if err != nil {
		return nil, fmt.Errorf("failed to listen on addr %s: %v", addr, err)
	}

	log.Printf("Listening on %s...\n", addr)

	go func() {
		for {
			conn, err := listener.Accept()
			if err != nil {
				// We just return if the listener is closed
				return
			}

			go handleConnection(conn, config)
		}
	}()

	return listener, nil
}

func handleConnection(conn net.Conn, config *ssh.ServerConfig) {
	defer conn.Close()

	sshConn, chans, reqs, err := ssh.NewServerConn(conn, config)
	if err != nil {
		log.Printf("failed to handshake: %v\n", err)
		return
	}
	defer sshConn.Close()

	log.Printf("Client connected: %s\n", sshConn.RemoteAddr())
	defer log.Printf("Client disconnected: %s\n", sshConn.RemoteAddr())

	go ssh.DiscardRequests(reqs)

	for newChannel := range chans {
		if newChannel.ChannelType() != "session" {
			newChannel.Reject(ssh.UnknownChannelType, "unknown channel type")
			continue
		}

		channel, requests, err := newChannel.Accept()
		if err != nil {
			log.Printf("could not accept channel: %v\n", err)
			continue
		}

		go func() {
			defer channel.Close()

			for req := range requests {
				switch req.Type {
				case "pty-req":
					log.Printf("Accepted request type: %s\n", req.Type)
					req.Reply(true, nil)

				case "shell":
					log.Printf("Accepted request type: %s\n", req.Type)
					req.Reply(true, nil)

					// We can use golang.org/x/term to run a proper terminal emulator
					// which allows the built-in `ssh` client to connect and interact
					// with a prompt.
					terminal := term.NewTerminal(channel, "sshecho> ")

					go func() {
						for {
							line, err := terminal.ReadLine()
							if err != nil {
								if err != io.EOF {
									log.Printf("terminal read error: %v\n", err)
								}
								return
							}

							line = strings.TrimSpace(line)
							if line == "" {
								continue
							}

							log.Printf("Received command: %q\n", line)

							if line == "whoami" {
								terminal.Write([]byte("root\r\n"))
							} else if line == "ls" {
								terminal.Write([]byte("bin dev proc lib home etc\r\n"))
							} else if strings.HasPrefix(line, "echo ") {
								msg := strings.TrimPrefix(line, "echo ")
								terminal.Write([]byte(msg + "\r\n"))
							} else if line == "exit" || line == "quit" {
								channel.SendRequest("exit-status", false, ssh.Marshal(struct{ uint32 }{0}))
								channel.Close()
								return
							} else {
								terminal.Write([]byte(fmt.Sprintf("sshecho: %s: command not found\r\n", line)))
							}
						}
					}()

				case "exec":
					log.Printf("Accepted request type: %s\n", req.Type)
					req.Reply(true, nil)

					var execReq struct {
						Command string
					}
					if err := ssh.Unmarshal(req.Payload, &execReq); err != nil {
						log.Printf("Failed to unmarshal exec payload: %v", err)
						channel.Close()
						return
					}

					line := strings.TrimSpace(execReq.Command)
					log.Printf("Received exec command: %q\n", line)

					if line == "whoami" {
						channel.Write([]byte("root\n"))
					} else if line == "ls" {
						channel.Write([]byte("bin dev proc lib home etc\n"))
					} else if strings.HasPrefix(line, "echo ") {
						msg := strings.TrimPrefix(line, "echo ")
						channel.Write([]byte(msg + "\n"))
					} else {
						channel.Write([]byte(fmt.Sprintf("sshecho: %s: command not found\n", line)))
					}

					channel.SendRequest("exit-status", false, ssh.Marshal(struct{ uint32 }{0}))
					channel.Close()
					return

				default:
					log.Printf("Rejected request type: %s\n", req.Type)
					req.Reply(false, nil)
				}
			}
		}()
	}
}
