package build

import "embed"

// Content embedded from the application's build directory, includes the latest build of the UI.
//
//go:embed *.png *.html *.json *.txt *.ico
//go:embed static/*
var Content embed.FS
