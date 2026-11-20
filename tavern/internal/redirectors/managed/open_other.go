//go:build !unix

package managed

import "os"

func openIdentityFile(path string) (*os.File, error) {
	return os.Open(path)
}

func verifyFileOwnership(os.FileInfo) error {
	return nil
}
