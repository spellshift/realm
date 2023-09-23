package build

import "embed"

// This file is a template that is copied into the npm build dir by the npm build script after a successful build.
// The template exists at: `/realm/tavern/internal/www/embed-template.go`
// The template destination is: `/realm/tavern/internal/www/build/embed.go`

// Content embedded from the application's build directory, includes the latest build of the UI.
//
//go:embed *.png *.html *.json *.txt *.ico
//go:embed static/*
var Content embed.FS
