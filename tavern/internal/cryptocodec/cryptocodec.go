package cryptocodec

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"errors"
	"fmt"
	"log/slog"
	"runtime/debug"
	"strconv"

	"github.com/cloudflare/circl/dh/x25519"
	lru "github.com/hashicorp/golang-lru/v2"
	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/mem"
)

var session_pub_keys = NewSyncMap()

// This size limits the number of concurrent connections each server can handle.
// I can't imagine a single server handling more than 10k connections at once but just in case.
const LRUCACHE_SIZE = 10480

type SyncMap struct {
	Map *lru.Cache[int, []byte] // Example data map
}

func NewSyncMap() *SyncMap {
	l, err := lru.New[int, []byte](LRUCACHE_SIZE)
	if err != nil {
		slog.Error("Failed to create LRU cache")
	}
	return &SyncMap{Map: l}
}

func (s *SyncMap) String() string {
	var res string
	allkeys := s.Map.Keys()
	for _, k := range allkeys {
		v, _ := s.Map.Peek(k)
		res = fmt.Sprintf("%sid: %d pubkey: %x\n", res, k, v)
	}
	return res
}

func (s *SyncMap) Load(key int) ([]byte, bool) {
	return s.Map.Get(key)
}

func (s *SyncMap) Store(key int, value []byte) {
	s.Map.Add(key, value)
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
	encoding.RegisterCodecV2(StreamDecryptCodec{})
	slog.Debug("[cryptocodec] application-layer cryptography registered xchacha20-poly1305 gRPC codec")
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
	if len(in_arr) < x25519.Size {
		slog.Error(fmt.Sprintf("input bytes too short %d expected at least %d", len(in_arr), x25519.Size))
		return FAILURE_BYTES, FAILURE_BYTES
	}

	// CRITICAL FIX: Make a distinct copy of the public key.
	// Previously, 'in_arr[:32]' retained the capacity of the entire 'in_arr'.
	// This caused the subsequent 'Encrypt' to append into shared memory (Data Race).
	client_pub_key_bytes := make([]byte, x25519.Size)
	copy(client_pub_key_bytes, in_arr[:x25519.Size])


	ids, err := goAllIds()
	if err != nil {
		slog.Error("failed to get goid")
		return FAILURE_BYTES, FAILURE_BYTES
	}
	session_pub_keys.Store(ids.Id, client_pub_key_bytes)

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
	ids, err := goAllIds()
	if err != nil {
		slog.Error(fmt.Sprintf("unable to find GOID %s", err))
		return FAILURE_BYTES
	}

	var id int
	var client_pub_key_bytes []byte
	ok := false
	for _, id := range []int{ids.Id, ids.ParentId} {
		client_pub_key_bytes, ok = session_pub_keys.Load(id)
		if ok {
			break
		}
	}

	if !ok {
		slog.Error(fmt.Sprintf("public key not found for id: %d", id))
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
	return append(client_pub_key_bytes, encryptedMsg...)
}

type GoidTrace struct {
	Id       int
	ParentId int
	Others   []int
}

func goAllIds() (GoidTrace, error) {
	buf := debug.Stack()
	// slog.Info(fmt.Sprintf("debug stack: %s", buf))
	var ids []int
	elems := bytes.Fields(buf)
	for i, elem := range elems {
		if bytes.Equal(elem, []byte("goroutine")) && i+1 < len(elems) {
			id, err := strconv.Atoi(string(elems[i+1]))
			if err != nil {
				return GoidTrace{}, err
			}
			ids = append(ids, id)
		}
	}
	res := GoidTrace{
		Id:       ids[0],
		ParentId: ids[1],
		Others:   ids[2:],
	}
	return res, nil
}
