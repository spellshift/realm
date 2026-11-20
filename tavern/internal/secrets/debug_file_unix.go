//go:build unix

package secrets

import (
	"fmt"
	"os"
	"syscall"
)

func openSecretsFileReadOnly(path string) (*os.File, error) {
	return os.OpenFile(path, os.O_RDONLY|syscall.O_NOFOLLOW, 0)
}

func validateSecretsFileOwner(path string, info os.FileInfo) error {
	stat, ok := info.Sys().(*syscall.Stat_t)
	if !ok {
		return fmt.Errorf("failed to inspect owner of secrets file %q", path)
	}
	euid := uint32(os.Geteuid())
	if stat.Uid != euid {
		return fmt.Errorf("secrets file %q is owned by uid %d, expected %d", path, stat.Uid, euid)
	}
	return nil
}
