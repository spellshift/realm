//go:build linux && pam

// This file uses the cgo-based github.com/msteinert/pam/v2 package, which
// requires libpam0g-dev (security/pam_appl.h) to be installed. It is only
// compiled when the "pam" build tag is set (e.g. `go build -tags pam`), so
// that default builds and tests do not depend on PAM headers being present.

package sshecho

import (
	"fmt"

	"github.com/msteinert/pam/v2"
)

// pamAuthenticate authenticates a user with the given password using PAM.
func pamAuthenticate(username, password string) error {
	t, err := pam.StartFunc("sshd", username, func(s pam.Style, msg string) (string, error) {
		switch s {
		case pam.PromptEchoOff:
			return password, nil
		case pam.PromptEchoOn:
			return "", nil
		case pam.ErrorMsg, pam.TextInfo:
			return "", nil
		default:
			return "", fmt.Errorf("unsupported PAM style: %v", s)
		}
	})
	if err != nil {
		return fmt.Errorf("PAM start failed: %w", err)
	}
	defer t.End()

	if err := t.Authenticate(0); err != nil {
		return fmt.Errorf("PAM authentication failed: %w", err)
	}

	return nil
}
