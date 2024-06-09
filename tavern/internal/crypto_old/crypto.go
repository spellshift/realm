package cryptoold

import (
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"io"
	"net/http"

	"github.com/aead/chacha20poly1305"
	"realm.pub/tavern/internal/c2/c2pb"
)

type CryptoSvc struct {
	Aead          cipher.AEAD
	key           []byte
	encrypt_nonce []byte
}

func NewCryptoSvc(key []byte) CryptoSvc {
	hasher := sha256.New()
	hasher.Write(key)
	key = hasher.Sum(nil)
	aead, err := chacha20poly1305.NewXCipher(key)

	if err != nil {
		panic(err)
	}

	nonce := make([]byte, aead.NonceSize())
	if _, err := rand.Read(nonce); err != nil {
		panic(err)
	}

	res := CryptoSvc{
		Aead:          aead,
		key:           key,
		encrypt_nonce: nonce,
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

type RequestBodyWrapper struct {
	Csvc          CryptoSvc
	Body          io.ReadCloser
	stream_reader c2pb.C2_ReportFileServer
}

// Close implements io.ReadCloser.
func (r RequestBodyWrapper) Close() error {
	return r.Body.Close()
}

// Read implements io.ReadCloser.
func (r RequestBodyWrapper) Read(p []byte) (n int, err error) {
	// tmp := []byte{}
	bytes_read, byte_err := r.Body.Read(p)
	if byte_err != nil {
		fmt.Println("Failed to read body: ", byte_err)
		return bytes_read, byte_err
	}
	fmt.Println("Encrypted request: ", p[:bytes_read])
	res := r.Csvc.Decrypt(p[:bytes_read])
	fmt.Println("Decrypted request: ", res)
	copy(p, res)

	return bytes_read, byte_err
}

// func (cryptosvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
// 	if len(in_arr) == 0 {
// 		fmt.Println("Encrypt in bytes is 0 bytes long")
// 		return []byte{}
// 	}

// 	// Encrypt the message and append the ciphertext to the nonce.
// 	fmt.Printf("Encrypt Nonce:       %v\n", cryptosvc.encrypt_nonce)
// 	in_arr = cryptosvc.Aead.Seal([]byte{}, cryptosvc.encrypt_nonce, in_arr, nil)
// 	return in_arr
// }

// Custom writer to enable encryption
type ResponseWriterWrapper struct {
	csvc CryptoSvc
	hw   http.ResponseWriter
	ew   io.WriteCloser
}

// Header implements http.ResponseWriter.
func (r ResponseWriterWrapper) Header() http.Header {
	return r.hw.Header()
}

// Write implements http.ResponseWriter.
func (r ResponseWriterWrapper) Write(buf []byte) (int, error) {
	fmt.Println("Decrypted response: ", buf)
	return r.ew.Write(buf)
}

// WriteHeader implements http.ResponseWriter.
func (r ResponseWriterWrapper) WriteHeader(statusCode int) {
	r.hw.WriteHeader(statusCode)
}

// Header implements http.ResponseWriter.
func (i ResponseWriterWrapper) Flush() {
	// REVIEW: I am not confidenty calling Close is the right thing here
	// in some situations http may call flush mid message
	// but this seems to be working so far
	// i.ew.Close()
	if err := i.ew.Close(); err != nil {
		fmt.Printf("Failed to finish encryption: %v\n", err)
	}
	i.hw.(http.Flusher).Flush()
}

func NewResponseWriterWrapper(csvc CryptoSvc, w http.ResponseWriter) ResponseWriterWrapper {

	// We need to prepend the nonce to the body
	// Can't do this in Write or it may be written
	// multiple times to the same body
	_, err := w.Write(csvc.encrypt_nonce)
	if err != nil {
		fmt.Println("Failed to write nonce", err)
	}

	encryptedWriter, err := chacha20poly1305.EncryptWriter(w, csvc.key, csvc.encrypt_nonce)
	if err != nil {
		fmt.Println("Failed to create encrypt writer", err)
	}

	fmt.Println("Encrypt Nonce: ", csvc.encrypt_nonce)
	fmt.Println("Encrypt Key:   ", csvc.key)

	return ResponseWriterWrapper{
		csvc: csvc,
		hw:   w,
		ew:   encryptedWriter,
	}
}
