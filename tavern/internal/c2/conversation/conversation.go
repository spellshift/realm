package conversation

import "sync"

// Conversation tracks state for a request-response exchange
type Conversation struct {
	mu               sync.Mutex
	ID               string
	MethodPath       string
	TotalChunks      uint32
	ExpectedCRC      uint32
	ExpectedDataSize uint32
	Chunks           map[uint32][]byte
	ResponseData     []byte
	ResponseChunks   [][]byte
	ResponseCRC      uint32
	Completed        bool
}
