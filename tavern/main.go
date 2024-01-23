package main

import (
	"context"
	"errors"
	"log"
	"net/http"
	"os"

	_ "realm.pub/tavern/internal/ent/runtime"

	_ "github.com/mattn/go-sqlite3"
)

func main() {
	ctx := context.Background()
	app := newApp(ctx,
		ConfigureHTTPServer("0.0.0.0:80"),
		ConfigureMySQLFromEnv(),
		ConfigureOAuthFromEnv("/oauth/authorize"),
	)
	if err := app.Run(os.Args); err != nil && !errors.Is(err, http.ErrServerClosed) {
		log.Fatalf("fatal error: %v", err)
	}
}
