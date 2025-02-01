package main

import (
	"log/slog"
	"os"
)

func configureLogging() {
	var (
		logger         *slog.Logger
		logHandler     slog.Handler
		handlerOptions slog.HandlerOptions
	)

	// Configure Log Handler
	if EnvDebugLogging.IsUnset() {
		// Production
		handlerOptions = slog.HandlerOptions{Level: slog.LevelInfo}
	} else {
		// Debug
		handlerOptions = slog.HandlerOptions{Level: slog.LevelDebug, AddSource: true}
	}

	// Setup Log Format
	if EnvJSONLogging.IsSet() {
		logHandler = slog.NewJSONHandler(os.Stderr, &handlerOptions)
	} else {
		logHandler = slog.NewTextHandler(os.Stderr, &handlerOptions)
	}

	// Initialize Logger
	logger = slog.New(logHandler)

	// Configure logger to include Tavern Instance ID
	if EnvLogInstanceID.IsSet() {
		logger = logger.With("tavern_id", GlobalInstanceID)
	}

	slog.SetDefault(logger)
	slog.Debug("Debug logging enabled 🕵️ ")
}
