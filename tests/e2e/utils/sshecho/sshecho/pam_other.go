//go:build !linux || !pam

package sshecho

import "fmt"

// pamAuthenticate is only available when built for Linux with the "pam" build
// tag (which requires libpam0g-dev to be installed). Without it, system
// authentication always fails so that a misconfigured server never silently
// accepts credentials.
func pamAuthenticate(username, password string) error {
	return fmt.Errorf("system authentication (PAM) is not compiled in on this platform (rebuild with -tags pam)")
}
