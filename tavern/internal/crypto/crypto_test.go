package crypto

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGrpcEncryptDecrypt(t *testing.T) {
	test_str := []byte("My SuperSecret Message!")
	cryptosvc_test := NewCryptoSvc([]byte("My Secret key"))
	enc := cryptosvc_test.Encrypt(test_str)
	dec := cryptosvc_test.Decrypt(enc)
	assert.Equal(t, test_str, dec)
}
