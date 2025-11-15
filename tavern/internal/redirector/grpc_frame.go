package redirector

import "encoding/binary"

// FrameHeaderSize is the size of gRPC frame header: [compression_flag(1)][length(4)]
const FrameHeaderSize = 5

// FrameHeader represents a gRPC wire protocol frame header
type FrameHeader struct {
	CompressionFlag uint8
	MessageLength   uint32
}

// NewFrameHeader creates a new frame header with no compression
func NewFrameHeader(messageLength uint32) FrameHeader {
	return FrameHeader{
		CompressionFlag: 0x00,
		MessageLength:   messageLength,
	}
}

// Encode encodes the frame header to a 5-byte array
func (h FrameHeader) Encode() [5]byte {
	var header [5]byte
	header[0] = h.CompressionFlag
	binary.BigEndian.PutUint32(header[1:], h.MessageLength)
	return header
}

// TryDecode attempts to decode a frame header from the buffer
// Returns (header, ok) where ok indicates if enough data was available
func TryDecode(buffer []byte) (FrameHeader, bool) {
	if len(buffer) < FrameHeaderSize {
		return FrameHeader{}, false
	}

	return FrameHeader{
		CompressionFlag: buffer[0],
		MessageLength:   binary.BigEndian.Uint32(buffer[1:5]),
	}, true
}

// ExtractFrame extracts a complete gRPC frame from the buffer
// Returns (header, message, remainingBuffer, ok)
func ExtractFrame(buffer []byte) (FrameHeader, []byte, []byte, bool) {
	header, ok := TryDecode(buffer)
	if !ok {
		return FrameHeader{}, nil, buffer, false
	}

	totalSize := FrameHeaderSize + int(header.MessageLength)
	if len(buffer) < totalSize {
		return FrameHeader{}, nil, buffer, false
	}

	message := buffer[FrameHeaderSize:totalSize]
	remaining := buffer[totalSize:]

	return header, message, remaining, true
}
