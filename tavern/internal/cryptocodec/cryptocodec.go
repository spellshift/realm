package cryptocodec

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"errors"
	"fmt"
	"log"
	"log/slog"
	"runtime"
	"runtime/debug"
	"strconv"
	"sync"

	"github.com/cloudflare/circl/dh/x25519"
	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/mem"
)

// TODO: Switch to a gomap and mutex.
var session_pub_keys = NewSyncMap()

type SyncMap struct {
	Mutex sync.RWMutex   // Read Write Mutex to allow for multiple readers
	Map   map[int][]byte // Example data map
}

func (s *SyncMap) String() string {
	res := ""
	defer s.Mutex.Unlock()
	s.Mutex.Lock()
	for k, v := range s.Map {
		res = fmt.Sprintf("%skey: %d val: %x\n", res, k, v)
	}
	return res
}


func NewSyncMap() *SyncMap {
	return &SyncMap{Mutex: sync.RWMutex{}, Map: make(map[int][]byte)}
}

func (s *SyncMap) Load(key int) ([]byte, bool) {
	defer s.Mutex.Unlock()
	s.Mutex.Lock()
	res, ok := s.Map[key]
	return res, ok
}

func (s *SyncMap) Store(key int, value []byte) {
	defer s.Mutex.Unlock()
	s.Mutex.Lock()
	s.Map[key] = value
}

func (s *SyncMap) Delete(key int) {
	defer s.Mutex.Unlock()
	s.Mutex.Lock()
	delete(s.Map, key)
}

// TODO: Should we make this a random long byte array in case it gets used anywhere to avoid encrypting data with a weak key? - Sliver handles errors in this way.
var FAILURE_BYTES = []byte{}

func castBytesToBufSlice(buf []byte) (mem.BufferSlice, error) {
	r := bytes.NewBuffer(buf)
	res, e := mem.ReadAll(r, mem.DefaultBufferPool())
	if e != nil {
		slog.Error(fmt.Sprintf("failed to read failure_bytes %s", e))
		return res, e
	}
	return res, nil
}

func init() {
	log.Println("[INFO] Loading xchacha20-poly1305")
	encoding.RegisterCodecV2(StreamDecryptCodec{})
}

type StreamDecryptCodec struct {
	Csvc CryptoSvc
}

func NewStreamDecryptCodec() StreamDecryptCodec {
	return StreamDecryptCodec{}
}

func (s StreamDecryptCodec) Marshal(v any) (mem.BufferSlice, error) {
	proto := encoding.GetCodecV2("proto")
	res, err := proto.Marshal(v)
	if err != nil {
		slog.Error("Unable to marshall data")
		return res, err
	}
	enc_res := s.Csvc.Encrypt(res.Materialize())
	byte_enc_res, err := castBytesToBufSlice(enc_res)

	return byte_enc_res, err
}

func (s StreamDecryptCodec) Unmarshal(buf mem.BufferSlice, v any) error {
	dec_buf, _ := s.Csvc.Decrypt(buf.Materialize())

	proto := encoding.GetCodecV2("proto")
	if proto == nil {
		slog.Error("'proto' codec is not registered")
		return errors.New("'proto' codec is not registered")
	}
	dec_mem_slice, err := castBytesToBufSlice(dec_buf)
	if err != nil {
		slog.Error("Unable to cast decrypted bytes to mem.BufferSlice")
		return err
	}
	return proto.Unmarshal(dec_mem_slice, v)
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

func (csvc *CryptoSvc) generate_shared_key(client_pub_key_bytes []byte) []byte {
	x22519_curve := ecdh.X25519()
	client_pub_key, err := x22519_curve.NewPublicKey(client_pub_key_bytes)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to create public key %v", err))
		return FAILURE_BYTES
	}

	shared_key, err := csvc.priv_key.ECDH(client_pub_key)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to get shared secret %v", err))
		return FAILURE_BYTES
	}

	return shared_key
}

func (csvc *CryptoSvc) Decrypt(in_arr []byte) ([]byte, []byte) {
	// Read in pub key
	if len(in_arr) < x25519.Size {
		slog.Error(fmt.Sprintf("input bytes to short %d expected at least %d", len(in_arr), x25519.Size))
		return FAILURE_BYTES, FAILURE_BYTES
	}

	client_pub_key_bytes := in_arr[:x25519.Size]

	id, err := goid()
	if err != nil {
		slog.Error("failed to get goid")
		return FAILURE_BYTES, FAILURE_BYTES
	}
	session_pub_keys.Store(id, client_pub_key_bytes)

	// Generate shared secret
	derived_key := csvc.generate_shared_key(client_pub_key_bytes)

	aead, err := chacha20poly1305.NewX(derived_key)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to create xchacha key %v", err))
		return FAILURE_BYTES, FAILURE_BYTES
	}

	// Progress in_arr buf
	in_arr = in_arr[x25519.Size:]

	// Read nonce & cipher text
	if len(in_arr) < aead.NonceSize() {
		slog.Error(fmt.Sprintf("input bytes to short %d expected at least %d", len(in_arr), aead.NonceSize()))
		return FAILURE_BYTES, FAILURE_BYTES
	}
	nonce, ciphertext := in_arr[:aead.NonceSize()], in_arr[aead.NonceSize():]

	// Decrypt
	plaintext, err := aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to decrypt %v", err))
		return FAILURE_BYTES, FAILURE_BYTES
	}

	return plaintext, client_pub_key_bytes
}

// TODO: Don't use [] ref.
func (csvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
	// Get the client pub key?
	ids, err := goAllIds()
	if err != nil {
		slog.Error(fmt.Sprintf("unable to find GOID: ", err))
		return FAILURE_BYTES
	}
	slog.Info(fmt.Sprintf("all ids: %v", ids))
	slog.Info(fmt.Sprintf("all keys: %v", session_pub_keys.String()))

	var id int
	var client_pub_key_bytes []byte
	ok := false
	for _, id = range ids {
		client_pub_key_bytes, ok = session_pub_keys.Load(id)
		if ok {
			break
		}
	}

	if !ok {
		slog.Error("Public key not found for any ID") // Assume this is a new connection.
		return FAILURE_BYTES
	}

	// Generate shared secret
	shared_key := csvc.generate_shared_key(client_pub_key_bytes)
	aead, err := chacha20poly1305.NewX(shared_key)
	if err != nil {
		slog.Error(fmt.Sprintf("failed to create xchacha key %v", err))
		return FAILURE_BYTES
	}

	nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(in_arr)+aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		slog.Error(fmt.Sprintf("Failed to encrypt %v", err))
		return FAILURE_BYTES
	}
	encryptedMsg := aead.Seal(nonce, nonce, in_arr, nil)

	// We should only need to use these once so delete it after use
	session_pub_keys.Delete(id)

	return append(client_pub_key_bytes, encryptedMsg...)
}


func goAllIds() ([]int, error) {
	buf := debug.Stack()
	var ids []int
	elems := bytes.Fields(buf)
	for i, elem := range elems {
		if bytes.Equal(elem, []byte("goroutine")) && i+1 < len(elems) {
			id, err := strconv.Atoi(string(elems[i+1]))
			if err != nil {
				return nil, err
			}
			ids = append(ids, id)
		}
	}
	return ids, nil
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
