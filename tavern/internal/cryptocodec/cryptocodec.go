package cryptocodec

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"errors"
	"fmt"
	"log/slog"
	"runtime"
	"strconv"
	"strings"

	"github.com/cloudflare/circl/dh/x25519"
	lru "github.com/hashicorp/golang-lru/v2"
	"golang.org/x/crypto/chacha20poly1305"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/mem"
)

type SessionKey struct {
	ClientPubKey []byte
	SharedKey    []byte
}

var session_pub_keys = NewSyncMap()

// This size limits the number of concurrent connections each server can handle.
// I can't imagine a single server handling more than 10k connections at once but just in case.
const LRUCACHE_SIZE = 10480

type SyncMap struct {
	Map *lru.Cache[int, SessionKey]
}

func NewSyncMap() *SyncMap {
	l, err := lru.New[int, SessionKey](LRUCACHE_SIZE)
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
		res = fmt.Sprintf("%sid: %d pubkey: %x\n", res, k, v.ClientPubKey)
	}
	return res
}

func (s *SyncMap) Load(key int) (SessionKey, bool) {
	return s.Map.Get(key)
}

func (s *SyncMap) Store(key int, value SessionKey) {
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
	// Read in pub key
	if len(in_arr) < x25519.Size {
		slog.Error(fmt.Sprintf("input bytes to short %d expected at least %d", len(in_arr), x25519.Size))
		return FAILURE_BYTES, FAILURE_BYTES
	}

	client_pub_key_bytes := in_arr[:x25519.Size]

	ids, err := goAllIds()
	if err != nil {
		slog.Error("failed to get goid")
		return FAILURE_BYTES, FAILURE_BYTES
	}

	// Optimization: Check if we already have the derived shared key for this session
	// This saves us from doing ECDH on every packet if the client pub key is stable.
	var derived_key []byte
	cached, found := session_pub_keys.Load(ids.Id)

	if found && bytes.Equal(cached.ClientPubKey, client_pub_key_bytes) {
		derived_key = cached.SharedKey
	} else {
		// Generate shared secret
		derived_key = csvc.generate_shared_key(client_pub_key_bytes)
		session_pub_keys.Store(ids.Id, SessionKey{
			ClientPubKey: client_pub_key_bytes,
			SharedKey:    derived_key,
		})
	}

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

	var sessionKey SessionKey
	ok := false
	for _, id := range []int{ids.Id, ids.ParentId} {
		sessionKey, ok = session_pub_keys.Load(id)
		if ok {
			break
		}
	}

	if !ok {
		slog.Error(fmt.Sprintf("public key not found for id: %d", ids.Id))
		return FAILURE_BYTES
	}

	// Use cached shared key directly
	shared_key := sessionKey.SharedKey
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
	return append(sessionKey.ClientPubKey, encryptedMsg...)
}

type GoidTrace struct {
	Id       int
	ParentId int
	Others   []int
}

// goAllIds returns the current goroutine ID and its creator's ID.
// Optimized to avoid debug.Stack() which is very expensive.
func goAllIds() (GoidTrace, error) {
	// 4096 bytes is enough for "goroutine 1 [running]:\n" plus a reasonable stack trace
	// to ensure the "created by ... in goroutine X" footer is captured.
	// A small stack trace is usually ~200-300 bytes, but deep stacks can be larger.
	var buf [4096]byte
	n := runtime.Stack(buf[:], false)
	s := string(buf[:n])

	var res GoidTrace

	// Parse current ID: "goroutine <id> [running]:"
	if !strings.HasPrefix(s, "goroutine ") {
		return res, errors.New("invalid stack trace format")
	}

	s = s[10:] // Skip "goroutine "
	i := strings.IndexByte(s, ' ')
	if i == -1 {
		return res, errors.New("invalid stack trace format")
	}
	idStr := s[:i]
	id, err := strconv.Atoi(idStr)
	if err != nil {
		return res, err
	}
	res.Id = id

	// Parse creator ID: "created by ... in goroutine <id>"
	// It is usually the last line.
	// Search from the end for "in goroutine "
	lastIdx := strings.LastIndex(s, "in goroutine ")
	if lastIdx != -1 {
		parentStr := s[lastIdx+13:]
		// parentStr might have newline at the end
		parentStr = strings.TrimSpace(parentStr)
		parentId, err := strconv.Atoi(parentStr)
		if err == nil {
			res.ParentId = parentId
			// We can populate others if needed, but the original logic only really used parentId correctly (ids[1])
			// because ids[2:] would likely be empty or garbage.
			res.Others = []int{}
		}
	}

	return res, nil
}
