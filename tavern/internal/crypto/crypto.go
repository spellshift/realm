package crypto

import (
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"fmt"

	"golang.org/x/crypto/chacha20poly1305"
)

type CryptoSvc struct {
	Aead cipher.AEAD
}

func NewCryptoSvc(key []byte) CryptoSvc {
	hasher := sha256.New()
	hasher.Write(key)
	aead, err := chacha20poly1305.NewX(hasher.Sum(nil))

	if err != nil {
		panic(err)
	}

	res := CryptoSvc{
		Aead: aead,
	}

	return res
}

func (cryptosvc *CryptoSvc) Decrypt(in_arr []byte) []byte {
	if len(in_arr) < cryptosvc.Aead.NonceSize() {
		fmt.Printf("Input bytes to short %d expected %d\n", len(in_arr), cryptosvc.Aead.NonceSize())
		return []byte{}
	}

	nonce, ciphertext := in_arr[:cryptosvc.Aead.NonceSize()], in_arr[cryptosvc.Aead.NonceSize():]
	plaintext, err := cryptosvc.Aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		fmt.Printf("Failed to decrypt %v\n", err)
		return []byte{}
	}

	return plaintext
}

func (cryptosvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
	if len(in_arr) == 0 {
		fmt.Println("Encrypt in bytes is 0 bytes long")
		return []byte{}
	}

	nonce := make([]byte, cryptosvc.Aead.NonceSize(), cryptosvc.Aead.NonceSize()+len(in_arr)+cryptosvc.Aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		fmt.Println("Encrypt failed ", err)
		return []byte{}
	}

	// Encrypt the message and append the ciphertext to the nonce.
	encryptedMsg := cryptosvc.Aead.Seal(nonce, nonce, in_arr, nil)
	return encryptedMsg
}
