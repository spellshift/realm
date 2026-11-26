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
	Map *lru.Cache[int, []byte]
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

func (s *SyncMap) Remove(key int) {
	s.Map.Remove(key)
}

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
	if proto == nil {
		log.Error("proto codec is not registered")
		return nil, errors.New("'proto' codec is not registered")
	}
	res, err := proto.Marshal(v)
	if err != nil {
		log.Error("failed to marshal protobuf message", "error", err)
		return nil, err
	}
	defer res.Free()

	enc_res, err := s.Csvc.Encrypt(res.Materialize())
	if err != nil {
		return nil, err
	}
	return castBytesToBufSlice(enc_res)
}

func (s StreamDecryptCodec) Unmarshal(buf mem.BufferSlice, v any) error {
	dec_buf, _, err := s.Csvc.Decrypt(buf.Materialize())
	if err != nil {
		return err
	}

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
	defer dec_mem_slice.Free()

	return proto.Unmarshal(dec_mem_slice, v)
}

func (s StreamDecryptCodec) Name() string {
	return "xchacha20-poly1305"
}

type CryptoSvc struct {
	priv_key *ecdh.PrivateKey
}

func NewCryptoSvc(priv_key *ecdh.PrivateKey) CryptoSvc {
	return CryptoSvc{
		priv_key: priv_key,
	}
}

func (csvc *CryptoSvc) generate_shared_key(client_pub_key_bytes []byte) ([]byte, error) {
	if csvc.priv_key == nil {
		return nil, errors.New("server private key is nil")
	}
	x25519Curve := ecdh.X25519()
	client_pub_key, err := x25519Curve.NewPublicKey(client_pub_key_bytes)
	if err != nil {
		log.Debug("failed to parse client x25519 public key", "key_len", len(client_pub_key_bytes), "error", err)
		return nil, fmt.Errorf("failed to parse client x25519 public key: %w", err)
	}

	shared_key, err := csvc.priv_key.ECDH(client_pub_key)
	if err != nil {
		log.Debug("failed to derive x25519 shared secret", "error", err)
		return nil, fmt.Errorf("failed to derive x25519 shared secret: %w", err)
	}

	return shared_key, nil
}

func (csvc *CryptoSvc) Decrypt(in_arr []byte) ([]byte, []byte, error) {
	if len(in_arr) == 0 {
		log.Trace("received empty encrypted c2 frame")
		return nil, nil, errors.New("empty encrypted c2 frame")
	}
	if len(in_arr) < x25519.Size {
		log.Debug("encrypted c2 frame shorter than x25519 public key", "input_len", len(in_arr), "min_len", x25519.Size)
		return nil, nil, fmt.Errorf("encrypted c2 frame shorter than x25519 public key: got %d, need at least %d", len(in_arr), x25519.Size)
	}

	// Make a distinct copy of the public key so it does not alias the input buffer.
	client_pub_key_bytes := make([]byte, x25519.Size)
	copy(client_pub_key_bytes, in_arr[:x25519.Size])

	// Generate shared secret
	derived_key, err := csvc.generate_shared_key(client_pub_key_bytes)
	if err != nil {
		return nil, nil, err
	}

	aead, err := chacha20poly1305.NewX(derived_key)
	if err != nil {
		log.Debug("failed to initialize xchacha20-poly1305 cipher", "error", err)
		return nil, nil, fmt.Errorf("failed to initialize xchacha20-poly1305 cipher: %w", err)
	}

	// Progress in_arr buf
	payload := in_arr[x25519.Size:]

	// Read nonce & cipher text
	minPayloadLen := aead.NonceSize() + aead.Overhead()
	if len(payload) < minPayloadLen {
		log.Debug("encrypted c2 payload shorter than nonce and auth tag", "payload_len", len(payload), "min_len", minPayloadLen)
		return nil, nil, fmt.Errorf("encrypted c2 payload shorter than nonce and auth tag: got %d, need at least %d", len(payload), minPayloadLen)
	}
	nonce, ciphertext := payload[:aead.NonceSize()], payload[aead.NonceSize():]

	// Decrypt and authenticate before mutating session state
	plaintext, err := aead.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		log.Warn("failed to decrypt c2 frame", "error", err)
		return nil, nil, fmt.Errorf("failed to decrypt c2 frame: %w", err)
	}

	ids, err := goAllIds()
	if err != nil {
		log.Error("failed to inspect goroutine id", "error", err)
		return nil, nil, fmt.Errorf("failed to inspect goroutine id: %w", err)
	}
	session_pub_keys.Store(ids.Id, client_pub_key_bytes)

	return plaintext, client_pub_key_bytes, nil
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
	keyCopy := make([]byte, len(clientPubKey))
	copy(keyCopy, clientPubKey)
	session_pub_keys.Store(ids.Id, keyCopy)
}

func lookupSessionPubKey(ids GoidTrace) ([]byte, bool) {
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

// GetSessionPubKey attempts to retrieve the client public key for the current goroutine or its ancestors.
func GetSessionPubKey() ([]byte, bool) {
	ids, err := goAllIds()
	if err != nil {
		return nil, false
	}
	return lookupSessionPubKey(ids)
}

func (csvc *CryptoSvc) Encrypt(in_arr []byte) ([]byte, error) {
	ids, err := goAllIds()
	if err != nil {
		log.Error("failed to inspect goroutine id", "error", err)
		return nil, fmt.Errorf("failed to inspect goroutine id: %w", err)
	}

	client_pub_key_bytes, ok := lookupSessionPubKey(ids)
	if !ok {
		log.Debug("session public key not found for goroutine", "goid", ids.Id, "parent_goid", ids.ParentId)
		return nil, errors.New("session public key not found for goroutine")
	}

	// Generate shared secret
	shared_key, err := csvc.generate_shared_key(client_pub_key_bytes)
	if err != nil {
		return nil, err
	}
	aead, err := chacha20poly1305.NewX(shared_key)
	if err != nil {
		log.Error("failed to initialize xchacha20-poly1305 cipher", "error", err)
		return nil, fmt.Errorf("failed to initialize xchacha20-poly1305 cipher: %w", err)
	}

	// Allocate a single output buffer: [client_pub_key || nonce || ciphertext+tag]
	// without mutating the cached client_pub_key_bytes slice.
	pubLen := len(client_pub_key_bytes)
	nonceLen := aead.NonceSize()
	out := make([]byte, pubLen+nonceLen, pubLen+nonceLen+len(in_arr)+aead.Overhead())
	copy(out, client_pub_key_bytes)
	nonce := out[pubLen : pubLen+nonceLen]
	if _, err := rand.Read(nonce); err != nil {
		log.Error("failed to generate xchacha20-poly1305 nonce", "error", err)
		return nil, fmt.Errorf("failed to generate xchacha20-poly1305 nonce: %w", err)
	}
	return aead.Seal(out, nonce, in_arr, nil), nil
}

type GoidTrace struct {
	Id       int
	ParentId int
	Others   []int
}

func goAllIds() (GoidTrace, error) {
	return parseGoroutineIDs(debug.Stack())
}

func parseGoroutineIDs(stack []byte) (GoidTrace, error) {
	var ids []int
	elems := bytes.Fields(stack)
	for i, elem := range elems {
		if bytes.Equal(elem, []byte("goroutine")) && i+1 < len(elems) {
			id, err := strconv.Atoi(string(elems[i+1]))
			if err != nil {
				continue
			}
			ids = append(ids, id)
		}
	}
	if len(ids) == 0 {
		return GoidTrace{}, errors.New("no goroutine id found in stack trace")
	}
	res := GoidTrace{
		Id: ids[0],
	}
	if len(ids) > 1 {
		res.ParentId = ids[1]
	}
	if len(ids) > 2 {
		res.Others = ids[2:]
	}
	return res, nil
}
