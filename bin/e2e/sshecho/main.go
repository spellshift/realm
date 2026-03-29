package main

import (
	"crypto/rand"
	"crypto/rsa"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/urfave/cli"
	"golang.org/x/crypto/ssh"
)

func newApp() *cli.App {
	app := cli.NewApp()
	app.Name = "sshecho"
	app.Usage = "A simple SSH server that echoes input back to the client"
	app.Flags = []cli.Flag{
		cli.IntFlag{
			Name:  "port, p",
			Value: 2222,
			Usage: "Port to listen on",
		},
		cli.StringFlag{
			Name:  "user, u",
			Usage: "Username for password authentication",
		},
		cli.StringFlag{
			Name:  "password, pass",
			Usage: "Password for password authentication",
		},
		cli.StringFlag{
			Name:  "pubkey, k",
			Usage: "Path to a public key file for public key authentication",
		},
	}

	app.Action = func(c *cli.Context) error {
		port := c.Int("port")
		user := c.String("user")
		password := c.String("password")
		pubkeyFile := c.String("pubkey")

		config := &ssh.ServerConfig{
			NoClientAuth: true,
		}

		if user != "" && password != "" {
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
				return fmt.Errorf("failed to read public key: %v", err)
			}
			allowedKey, _, _, _, err := ssh.ParseAuthorizedKey(pubkeyBytes)
			if err != nil {
				return fmt.Errorf("failed to parse public key: %v", err)
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
			return fmt.Errorf("failed to generate private key: %v", err)
		}

		signer, err := ssh.NewSignerFromKey(privateKey)
		if err != nil {
			return fmt.Errorf("failed to create signer: %v", err)
		}

		config.AddHostKey(signer)

		listener, err := net.Listen("tcp", fmt.Sprintf("0.0.0.0:%d", port))
		if err != nil {
			return fmt.Errorf("failed to listen on port %d: %v", port, err)
		}
		defer listener.Close()

		log.Printf("Listening on port %d...\n", port)

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

		sigCh := make(chan os.Signal, 1)
		signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

		if doneCh, ok := c.App.Metadata["done"].(chan struct{}); ok {
			select {
			case <-sigCh:
				log.Println("Shutting down...")
			case <-doneCh:
				log.Println("Context done, shutting down...")
			}
		} else {
			<-sigCh
			log.Println("Shutting down...")
		}

		return nil
	}

	return app
}

func main() {
	app := newApp()
	err := app.Run(os.Args)
	if err != nil {
		log.Fatal(err)
	}
}

func handleConnection(conn net.Conn, config *ssh.ServerConfig) {
	defer conn.Close()

	sshConn, chans, reqs, err := ssh.NewServerConn(conn, config)
	if err != nil {
		log.Printf("failed to handshake: %v\n", err)
		return
	}
	defer sshConn.Close()

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
			for req := range requests {
				switch req.Type {
				case "shell", "exec", "pty-req":
					req.Reply(true, nil)
				default:
					req.Reply(false, nil)
				}
			}
		}()

		// Echo loop
		go func() {
			defer channel.Close()
			_, err := io.Copy(channel, channel)
			if err != nil {
				log.Printf("io.Copy error: %v\n", err)
			}
		}()
	}
}
