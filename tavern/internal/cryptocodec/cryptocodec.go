package cryptocodec

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"crypto/sha256"
	"errors"
	"fmt"
	"log"
	"runtime"
	"strconv"
	"sync"

	"github.com/cloudflare/circl/dh/x25519"
	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
)

// var last_seen_client_pub_key []byte
var session_pub_keys = sync.Map{}

func init() {
	log.Println("Loading xchacha20-poly1305")
	// Probably want to use the `ForceServerCodec` option instead.
	encoding.RegisterCodec(StreamDecryptCodec{})
}

type StreamDecryptCodec struct {
	Csvc CryptoSvc
}

func NewStreamDecryptCodec() StreamDecryptCodec {
	return StreamDecryptCodec{}
}

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

func (s StreamDecryptCodec) Marshal(v any) ([]byte, error) {
	id, _ := goid()
	fmt.Printf("GO_ID: %d Marshal\n", id)
	proto := encoding.GetCodec("proto")
	res, err := proto.Marshal(v)
	enc_res := s.Csvc.Encrypt(res)
	return enc_res, err
}

func (s StreamDecryptCodec) Unmarshal(buf []byte, v any) error {
	id, _ := goid()
	fmt.Printf("GO_ID: %d Unmarshal\n", id)
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

func hashitup(in_key []byte) []byte {
	hasher := sha256.New()
	hasher.Write(in_key)
	key := hasher.Sum(nil)
	// aead, err := chacha20poly1305.NewX(key)

	// if err != nil {
	// 	panic(err)
	// }

	// nonce := make([]byte, aead.NonceSize())
	// if _, err := rand.Read(nonce); err != nil {
	// 	panic(err)
	// }
	return key

}

func NewCryptoSvc(priv_key *ecdh.PrivateKey) CryptoSvc {

	res := CryptoSvc{
		// Aead: aead,
		priv_key: priv_key,
	}

	return res

}

func (csvc *CryptoSvc) GetAgentPubkey() []byte {
	id, _ := goid()
	res, _ := session_pub_keys.Load(id)
	return res.([]byte)
}

func (csvc *CryptoSvc) SetAgentPubkey(client_pub_key []byte) {
	// fmt.Printf("Old last_seen_client_pub_key: %v\n", last_seen_client_pub_key)
	// last_seen_client_pub_key = client_pub_key
	// fmt.Printf("New last_seen_client_pub_key: %v\n", last_seen_client_pub_key)
	id, _ := goid()
	session_pub_keys.Store(id, client_pub_key)
}

func (csvc *CryptoSvc) generate_shared_key(client_pub_key_bytes []byte) []byte {
	fmt.Println("[DEBUG] client_pub_key_bytes: ", client_pub_key_bytes)
	x22519_curve := ecdh.X25519()
	client_pub_key, err := x22519_curve.NewPublicKey(client_pub_key_bytes)
	if err != nil {
		log.Printf("[ERROR] Failed to create public key %v", err)
		return []byte{}
	}

	shared_key, err := csvc.priv_key.ECDH(client_pub_key)
	if err != nil {
		log.Printf("[ERROR] Failed to get shared secret %v", err)
		return []byte{}
	}

	return shared_key
}

func (csvc *CryptoSvc) Decrypt(in_arr []byte) ([]byte, []byte) {
	// Read in pub key
	if len(in_arr) < x25519.Size {
		fmt.Printf("Input bytes to short %d expected at least %d\n", len(in_arr), x25519.Size)
		return []byte{}, []byte{}
	}

	client_pub_key_bytes := in_arr[:x25519.Size]
	csvc.SetAgentPubkey(client_pub_key_bytes)

	// // Generate shared secret
	// log.Printf("[DEBUG] shared_key: %v\n", shared_key)
	// aead, err := chacha20poly1305.NewX(shared_key)
	// derived_key := hashitup([]byte("I Don't care how small the room is I cast fireball"))

	derived_key := csvc.generate_shared_key(client_pub_key_bytes)

	log.Println("[DEBUG] derived_key: ", derived_key)
	aead, err := chacha20poly1305.NewX(derived_key)
	if err != nil {
		log.Printf("[ERROR] Failed to create xchacha key %v", err)
		return []byte{}, []byte{}
	}

	// // Progress in_arr buf
	// fmt.Println("[DEBUG] in_arr 1:", in_arr)
	in_arr = in_arr[x25519.Size:]
	// fmt.Println("[DEBUG] in_arr 2:", in_arr)

	// Read nonce
	if len(in_arr) < aead.NonceSize() {
		fmt.Printf("Input bytes to short %d expected at least %d\n", len(in_arr), aead.NonceSize())
		return []byte{}, []byte{}
	}
	nonce, ciphertext := in_arr[:aead.NonceSize()], in_arr[aead.NonceSize():]

	// Decrypt
	plaintext, err := aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		fmt.Printf("Failed to decrypt %v\n", err)
		return []byte{}, []byte{}
	}

	return plaintext, client_pub_key_bytes
}

// TODO: Don't use [] ref.
func (csvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
	// Get the client pub key?
	client_pub_key_bytes := csvc.GetAgentPubkey()
	fmt.Println("[DEBUG] Got pub key: ", client_pub_key_bytes)

	// Generate shared secret
	shared_key := csvc.generate_shared_key(client_pub_key_bytes)
	aead, err := chacha20poly1305.NewX(shared_key)
	if err != nil {
		log.Printf("[ERROR] Failed to create xchacha key %v", err)
		return []byte{}
	}

	nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(in_arr)+aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		fmt.Printf("Failed to encrypt %v\n", err)
		return []byte{}
	}
	encryptedMsg := aead.Seal(nonce, nonce, in_arr, nil)
	return append(client_pub_key_bytes, encryptedMsg...)
}
