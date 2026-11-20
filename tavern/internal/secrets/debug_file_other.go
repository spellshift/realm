//go:build !unix

package secrets

import "os"

func openSecretsFileReadOnly(path string) (*os.File, error) {
	return os.Open(path)
}

func validateSecretsFileOwner(_ string, _ os.FileInfo) error {
	return nil
}
