package cryptoold

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/aead/chacha20poly1305"
	"github.com/stretchr/testify/assert"
)

func TestGrpcEncryptDecrypt(t *testing.T) {
	mem := bytes.NewBuffer(nil)
	test_str := []byte("My SuperSecret Message!")
	cryptosvc_test := NewCryptoSvc([]byte("My Secret key"))

	encryptedWriter, err := chacha20poly1305.EncryptWriter(mem, cryptosvc_test.key, cryptosvc_test.encrypt_nonce)
	if err != nil {
		fmt.Println("Failed to create encrypt writer", err)
	}
	n, err := encryptedWriter.Write(test_str)
	assert.Nil(t, err)
	assert.Greater(t, n, 1)
	assert.NotEqual(t, test_str, mem)
	dec := cryptosvc_test.Decrypt(mem.Bytes())
	assert.Equal(t, test_str, dec)
}
