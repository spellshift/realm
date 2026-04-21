//go:build !linux

package sshecho

import "fmt"

// pamAuthenticate is not supported on non-Linux platforms.
func pamAuthenticate(username, password string) error {
	return fmt.Errorf("system authentication (PAM) is not supported on this platform")
}
