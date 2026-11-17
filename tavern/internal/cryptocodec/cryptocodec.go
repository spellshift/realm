package cryptocodec

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"errors"
	"fmt"
	"runtime/debug"
	"strconv"

	"github.com/cloudflare/circl/dh/x25519"
	lru "github.com/hashicorp/golang-lru/v2"
	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/mem"
	"realm.pub/tavern/internal/logging"
)

var (
	log              = logging.For("c2.crypto")
	session_pub_keys = NewSyncMap()
)

// This size limits the number of concurrent connections each server can handle.
// I can't imagine a single server handling more than 10k connections at once but just in case.
const LRUCACHE_SIZE = 10480

type SyncMap struct {
	Map *lru.Cache[int, []byte] // Example data map
}

func NewSyncMap() *SyncMap {
	l, err := lru.New[int, []byte](LRUCACHE_SIZE)
	if err != nil {
		log.Error("failed to create session key LRU cache", "error", err)
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
		log.Error("failed to read buffer slice", "error", e)
		return res, e
	}
	return res, nil
}

func init() {
	encoding.RegisterCodecV2(StreamDecryptCodec{})
	log.Debug("registered xchacha20-poly1305 gRPC codec")
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
		log.Error("failed to marshal protobuf message", "error", err)
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
		log.Error("proto codec is not registered")
		return errors.New("'proto' codec is not registered")
	}
	dec_mem_slice, err := castBytesToBufSlice(dec_buf)
	if err != nil {
		log.Error("failed to cast decrypted bytes to buffer slice", "error", err)
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
		log.Debug("failed to parse client x25519 public key", "key_len", len(client_pub_key_bytes), "error", err)
		return FAILURE_BYTES
	}

	shared_key, err := csvc.priv_key.ECDH(client_pub_key)
	if err != nil {
		log.Debug("failed to derive x25519 shared secret", "error", err)
		return FAILURE_BYTES
	}

	return shared_key
}

func (csvc *CryptoSvc) Decrypt(in_arr []byte) ([]byte, []byte) {
	if len(in_arr) == 0 {
		log.Trace("received empty encrypted c2 frame")
		return FAILURE_BYTES, FAILURE_BYTES
	}
	if len(in_arr) < x25519.Size {
		log.Debug("encrypted c2 frame shorter than x25519 public key", "input_len", len(in_arr), "min_len", x25519.Size)
		return FAILURE_BYTES, FAILURE_BYTES
	}

	// CRITICAL FIX: Make a distinct copy of the public key.
	// Previously, 'in_arr[:32]' retained the capacity of the entire 'in_arr'.
	// This caused the subsequent 'Encrypt' to append into shared memory (Data Race).
	client_pub_key_bytes := make([]byte, x25519.Size)
	copy(client_pub_key_bytes, in_arr[:x25519.Size])

	ids, err := goAllIds()
	if err != nil {
		log.Error("failed to inspect goroutine id", "error", err)
		return FAILURE_BYTES, FAILURE_BYTES
	}
	session_pub_keys.Store(ids.Id, client_pub_key_bytes)

	// Generate shared secret
	derived_key := csvc.generate_shared_key(client_pub_key_bytes)

	aead, err := chacha20poly1305.NewX(derived_key)
	if err != nil {
		log.Debug("failed to initialize xchacha20-poly1305 cipher", "error", err)
		return FAILURE_BYTES, FAILURE_BYTES
	}

	// Progress in_arr buf
	in_arr = in_arr[x25519.Size:]

	// Read nonce & cipher text
	if len(in_arr) < aead.NonceSize() {
		log.Debug("encrypted c2 payload shorter than nonce", "payload_len", len(in_arr), "min_len", aead.NonceSize())
		return FAILURE_BYTES, FAILURE_BYTES
	}
	nonce, ciphertext := in_arr[:aead.NonceSize()], in_arr[aead.NonceSize():]

	// Decrypt
	plaintext, err := aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		log.Warn("failed to decrypt c2 frame", "error", err)
		return FAILURE_BYTES, FAILURE_BYTES
	}

	return plaintext, client_pub_key_bytes
}

// RegisterCurrentGoroutine associates the current goroutine ID with client public key bytes.
func RegisterCurrentGoroutine(clientPubKey []byte) {
	if len(clientPubKey) == 0 {
		return
	}
	ids, err := goAllIds()
	if err != nil {
		return
	}
	session_pub_keys.Store(ids.Id, clientPubKey)
}

// GetSessionPubKey attempts to retrieve the client public key for the current goroutine or its ancestors.
func GetSessionPubKey() ([]byte, bool) {
	ids, err := goAllIds()
	if err != nil {
		return nil, false
	}
	candidateIDs := append([]int{ids.Id, ids.ParentId}, ids.Others...)
	for _, id := range candidateIDs {
		if id == 0 {
			continue
		}
		if pk, ok := session_pub_keys.Load(id); ok {
			return pk, true
		}
	}
	return nil, false
}

// TODO: Don't use [] ref.
func (csvc *CryptoSvc) Encrypt(in_arr []byte) []byte {
	ids, err := goAllIds()
	if err != nil {
		log.Error("failed to inspect goroutine id", "error", err)
		return FAILURE_BYTES
	}

	var client_pub_key_bytes []byte
	ok := false
	candidateIDs := append([]int{ids.Id, ids.ParentId}, ids.Others...)
	for _, candID := range candidateIDs {
		if candID == 0 {
			continue
		}
		client_pub_key_bytes, ok = session_pub_keys.Load(candID)
		if ok {
			break
		}
	}

	if !ok {
		log.Debug("session public key not found for goroutine", "goid", ids.Id, "parent_goid", ids.ParentId)
		return FAILURE_BYTES
	}

	// Generate shared secret
	shared_key := csvc.generate_shared_key(client_pub_key_bytes)
	aead, err := chacha20poly1305.NewX(shared_key)
	if err != nil {
		log.Error("failed to initialize xchacha20-poly1305 cipher", "error", err)
		return FAILURE_BYTES
	}

	nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(in_arr)+aead.Overhead())
	if _, err := rand.Read(nonce); err != nil {
		log.Error("failed to generate xchacha20-poly1305 nonce", "error", err)
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
