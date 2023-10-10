package main

import (
	"context"
	"log"
	"os"

	_ "github.com/kcarretto/realm/tavern/internal/ent/runtime"

	_ "github.com/mattn/go-sqlite3"
)

func main() {
	ctx := context.Background()
	app := newApp(ctx,
		ConfigureHTTPServer(),
		ConfigureMySQLFromEnv(),
		ConfigureOAuthFromEnv("/oauth/authorize"),
	)
	if err := app.Run(os.Args); err != nil {
		log.Fatalf("fatal error: %v", err)
	}
}
