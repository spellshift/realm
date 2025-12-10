package http1

import "encoding/binary"

// frameHeaderSize is the size of gRPC frame header: [compression_flag(1)][length(4)]
const frameHeaderSize = 5

// frameHeader represents a gRPC wire protocol frame header
type frameHeader struct {
	CompressionFlag uint8
	MessageLength   uint32
}

// NewFrameHeader creates a new frame header with no compression
func newFrameHeader(messageLength uint32) frameHeader {
	return frameHeader{
		CompressionFlag: 0x00,
		MessageLength:   messageLength,
	}
}

// Encode encodes the frame header to a 5-byte array
func (h frameHeader) Encode() [5]byte {
	var header [5]byte
	header[0] = h.CompressionFlag
	binary.BigEndian.PutUint32(header[1:], h.MessageLength)
	return header
}

// tryDecode attempts to decode a frame header from the buffer
// Returns (header, ok) where ok indicates if enough data was available
func tryDecode(buffer []byte) (frameHeader, bool) {
	if len(buffer) < frameHeaderSize {
		return frameHeader{}, false
	}

	return frameHeader{
		CompressionFlag: buffer[0],
		MessageLength:   binary.BigEndian.Uint32(buffer[1:5]),
	}, true
}

// extractFrame extracts a complete gRPC frame from the buffer
// Returns (header, message, remainingBuffer, ok)
func extractFrame(buffer []byte) (frameHeader, []byte, []byte, bool) {
	header, ok := tryDecode(buffer)
	if !ok {
		return frameHeader{}, nil, buffer, false
	}

	totalSize := frameHeaderSize + int(header.MessageLength)
	if len(buffer) < totalSize {
		return frameHeader{}, nil, buffer, false
	}

	message := buffer[frameHeaderSize:totalSize]
	remaining := buffer[totalSize:]

	return header, message, remaining, true
}
