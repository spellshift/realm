package cryptocodec

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"errors"
	"log"
	"runtime"
	"strconv"
	"sync"

	"github.com/cloudflare/circl/dh/x25519"
	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
)

// TODO: Switch to a gomap and mutex.
var session_pub_keys = sync.Map{}

// TODO: Should we make this a random long byte array in case it gets used anywhere to avoid encrypting data with a week key? - Sliver handles errors in this way.
var FAILURE_BYTES = []byte{}

func init() {
	log.Println("[INFO] Loading xchacha20-poly1305")
	encoding.RegisterCodec(StreamDecryptCodec{})
}

type StreamDecryptCodec struct {
	Csvc CryptoSvc
}

func NewStreamDecryptCodec() StreamDecryptCodec {
	return StreamDecryptCodec{}
}

func (s StreamDecryptCodec) Marshal(v any) ([]byte, error) {
	id, err := goid()
	if err != nil {
		log.Println("[ERROR] Unable to find GOID ", id)
		return FAILURE_BYTES, err
	}
	proto := encoding.GetCodec("proto")
	res, err := proto.Marshal(v)
	enc_res := s.Csvc.Encrypt(res)
	return enc_res, err
}

func (s StreamDecryptCodec) Unmarshal(buf []byte, v any) error {
	id, err := goid()
	if err != nil {
		log.Println("[ERROR] Unable to find GOID ", id)
		return err
	}
	dec_buf, pub_key := s.Csvc.Decrypt(buf)
	s.Csvc.SetAgentPubkey(pub_key)
	proto := encoding.GetCodec("proto")
	return proto.Unmarshal(dec_buf, v)
}

func (s StreamDecryptCodec) Name() string {
	return "xchacha20-poly1305"
}

type CryptoSvc struct {
	// Aead cipher.AEAD
	priv_key *ecdh.PrivateKey
}

func NewCryptoSvc(priv_key *ecdh.PrivateKey) CryptoSvc {
	return CryptoSvc{
		priv_key: priv_key,
	}
}

func (csvc *CryptoSvc) GetAgentPubkey() []byte {
	id, err := goid()
	if err != nil {
		log.Println("[ERROR] Unable to find GOID ", id)
		return FAILURE_BYTES
	}
	res, ok := session_pub_keys.Load(id)
	if !ok {
		log.Println("[ERROR] Public key not found")
	}
	return res.([]byte)
}

func (csvc *CryptoSvc) SetAgentPubkey(client_pub_key []byte) {
	id, err := goid()
	if err != nil {
		log.Println("[ERROR] Failed to get goid")
	}
	session_pub_keys.Store(id, client_pub_key)
}

func (csvc *CryptoSvc) generate_shared_key(client_pub_key_bytes []byte) []byte {
	x22519_curve := ecdh.X25519()
	client_pub_key, err := x22519_curve.NewPublicKey(client_pub_key_bytes)
	if err != nil {
		log.Printf("[ERROR] Failed to create public key %v\n", err)
		return FAILURE_BYTES
	}

	shared_key, err := csvc.priv_key.ECDH(client_pub_key)
	if err != nil {
		log.Printf("[ERROR] Failed to get shared secret %v\n", err)
		return FAILURE_BYTES
	}

	return shared_key
}

func (csvc *CryptoSvc) Decrypt(in_arr []byte) ([]byte, []byte) {
	// Read in pub key
	if len(in_arr) < x25519.Size {
		log.Printf("[ERROR] Input bytes to short %d expected at least %d\n", len(in_arr), x25519.Size)
		return FAILURE_BYTES, FAILURE_BYTES
	}

	client_pub_key_bytes := in_arr[:x25519.Size]
	csvc.SetAgentPubkey(client_pub_key_bytes)

	// Generate shared secret
	derived_key := csvc.generate_shared_key(client_pub_key_bytes)

	aead, err := chacha20poly1305.NewX(derived_key)
	if err != nil {
		log.Printf("[ERROR] Failed to create xchacha key %v\n", err)
		return FAILURE_BYTES, FAILURE_BYTES
	}

	// Progress in_arr buf
	in_arr = in_arr[x25519.Size:]

	// Read nonce
	if len(in_arr) < aead.NonceSize() {
		log.Printf("[ERROR] Input bytes to short %d expected at least %d\n", len(in_arr), aead.NonceSize())
		return FAILURE_BYTES, FAILURE_BYTES
	}
	nonce, ciphertext := in_arr[:aead.NonceSize()], in_arr[aead.NonceSize():]

	// Decrypt
	plaintext, err := aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		log.Printf("[ERROR] Failed to decrypt %v\n", err)
		return FAILURE_BYTES, FAILURE_BYTES
	}

	return plaintext, client_pub_key_bytes
}

// TODO: Don't use [] ref.
func (csvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
	// Get the client pub key?
	client_pub_key_bytes := csvc.GetAgentPubkey()

	// Generate shared secret
	shared_key := csvc.generate_shared_key(client_pub_key_bytes)
	aead, err := chacha20poly1305.NewX(shared_key)
	if err != nil {
		log.Printf("[ERROR] Failed to create xchacha key %v\n", err)
		return FAILURE_BYTES
	}

	nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(in_arr)+aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		log.Printf("[ERROR] Failed to encrypt %v\n", err)
		return FAILURE_BYTES
	}
	encryptedMsg := aead.Seal(nonce, nonce, in_arr, nil)
	return append(client_pub_key_bytes, encryptedMsg...)
}

// TODO: Find a better way
// This is terrible, slow, and should never be used.
func goid() (int, error) {
	buf := make([]byte, 32)
	n := runtime.Stack(buf, false)
	buf = buf[:n]
	// goroutine 1 [running]: ...
	var goroutinePrefix = []byte("goroutine ")
	var errBadStack = errors.New("invalid runtime.Stack output")
	buf, ok := bytes.CutPrefix(buf, goroutinePrefix)
	if !ok {
		return 0, errBadStack
	}

	i := bytes.IndexByte(buf, ' ')
	if i < 0 {
		return 0, errBadStack
	}

	return strconv.Atoi(string(buf[:i]))
}
