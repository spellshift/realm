package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/urfave/cli"
	"realm.pub/tests/e2e/utils/sshecho/sshecho"
)

func NewApp() *cli.App {
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

		listener, err := sshecho.Run(fmt.Sprintf("0.0.0.0:%d", port), user, password, pubkeyFile)
		if err != nil {
			return err
		}
		defer listener.Close()

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
	app := NewApp()
	err := app.Run(os.Args)
	if err != nil {
		log.Fatal(err)
	}
}
