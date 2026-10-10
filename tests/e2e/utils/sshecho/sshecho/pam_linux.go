//go:build linux

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
