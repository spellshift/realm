package cryptocodec

import (
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"log"

	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
)

func init() {
	log.Println("Loading xchacha20-poly1305")
	// Probably want to use the `ForceServerCodec` option instead.
	encoding.RegisterCodec(StreamDecryptCodec{})
}

// TODO decyrpt and encrypt in place

type StreamDecryptCodec struct {
	Csvc CryptoSvc
}

func NewStreamDecryptCodec() StreamDecryptCodec {
	return StreamDecryptCodec{}
}

func (s StreamDecryptCodec) Marshal(v any) ([]byte, error) {
	log.Println("Marshal")
	proto := encoding.GetCodec("proto")
	res, err := proto.Marshal(v)
	enc_res := s.Csvc.Encrypt(res)
	return enc_res, err
}

func (s StreamDecryptCodec) Unmarshal(buf []byte, v any) error {
	log.Println("Decrypt:", buf)
	dec_buf := s.Csvc.Decrypt(buf)
	log.Println("Unmarshal:", dec_buf)
	proto := encoding.GetCodec("proto")
	return proto.Unmarshal(buf, v)
}

func (s StreamDecryptCodec) Name() string {
	return "xchacha20-poly1305"
}

type CryptoSvc struct {
	Aead cipher.AEAD
}

func NewCryptoSvc(key []byte) CryptoSvc {
	hasher := sha256.New()
	hasher.Write(key)
	key = hasher.Sum(nil)
	aead, err := chacha20poly1305.NewX(key)

	if err != nil {
		panic(err)
	}

	nonce := make([]byte, aead.NonceSize())
	if _, err := rand.Read(nonce); err != nil {
		panic(err)
	}

	res := CryptoSvc{
		Aead: aead,
	}

	return res

}

func (csvc *CryptoSvc) Decrypt(in_arr []byte) []byte {
	if len(in_arr) < csvc.Aead.NonceSize() {
		fmt.Printf("Input bytes to short %d expected %d\n", len(in_arr), csvc.Aead.NonceSize())
		return []byte{}
	}

	nonce, ciphertext := in_arr[:csvc.Aead.NonceSize()], in_arr[csvc.Aead.NonceSize():]
	plaintext, err := csvc.Aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		fmt.Printf("Failed to decrypt %v\n", err)
		return []byte{}
	}

	return plaintext
}

func (csvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
	nonce := make([]byte, csvc.Aead.NonceSize(), csvc.Aead.NonceSize()+len(in_arr)+csvc.Aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		panic(err)
	}
	encryptedMsg := csvc.Aead.Seal(nonce, nonce, in_arr, nil)
	return encryptedMsg
}
