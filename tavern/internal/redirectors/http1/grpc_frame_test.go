package http1

import (
	"testing"
)

func TestFrameHeaderNew(t *testing.T) {
	header := newFrameHeader(1234)
	if header.CompressionFlag != 0x00 {
		t.Errorf("Expected compression flag 0x00, got 0x%02x", header.CompressionFlag)
	}
	if header.MessageLength != 1234 {
		t.Errorf("Expected message length 1234, got %d", header.MessageLength)
	}
}

func TestFrameHeaderEncode(t *testing.T) {
	header := newFrameHeader(0x12345678)
	encoded := header.Encode()

	if len(encoded) != 5 {
		t.Errorf("Expected encoded length 5, got %d", len(encoded))
	}
	if encoded[0] != 0x00 {
		t.Errorf("Expected compression flag 0x00, got 0x%02x", encoded[0])
	}
	if encoded[1] != 0x12 || encoded[2] != 0x34 || encoded[3] != 0x56 || encoded[4] != 0x78 {
		t.Errorf("Expected big-endian length [0x12, 0x34, 0x56, 0x78], got [0x%02x, 0x%02x, 0x%02x, 0x%02x]",
			encoded[1], encoded[2], encoded[3], encoded[4])
	}
}

func TestTryDecodeSuccess(t *testing.T) {
	buffer := []byte{0x00, 0x00, 0x00, 0x01, 0x00}

	header, ok := tryDecode(buffer)
	if !ok {
		t.Fatal("Expected successful decode")
	}
	if header.CompressionFlag != 0x00 {
		t.Errorf("Expected compression flag 0x00, got 0x%02x", header.CompressionFlag)
	}
	if header.MessageLength != 256 {
		t.Errorf("Expected message length 256, got %d", header.MessageLength)
	}
}

func TestTryDecodeInsufficientData(t *testing.T) {
	buffer := []byte{0x00, 0x01, 0x02} // Only 3 bytes

	_, ok := tryDecode(buffer)
	if ok {
		t.Error("Expected decode to fail with insufficient data")
	}
}

func TestExtractFrameSuccess(t *testing.T) {
	// Header: no compression, 10 bytes
	// Message: 10 bytes of data
	buffer := []byte{0x00, 0x00, 0x00, 0x00, 0x0A}
	buffer = append(buffer, []byte("0123456789")...)

	header, message, remaining, ok := extractFrame(buffer)
	if !ok {
		t.Fatal("Expected successful frame extraction")
	}
	if header.MessageLength != 10 {
		t.Errorf("Expected message length 10, got %d", header.MessageLength)
	}
	if len(message) != 10 {
		t.Errorf("Expected message length 10, got %d", len(message))
	}
	if string(message) != "0123456789" {
		t.Errorf("Expected message '0123456789', got '%s'", string(message))
	}
	if len(remaining) != 0 {
		t.Errorf("Expected empty remaining buffer, got %d bytes", len(remaining))
	}
}

func TestExtractFrameIncomplete(t *testing.T) {
	// Header: no compression, 10 bytes
	// Message: only 5 bytes (incomplete)
	buffer := []byte{0x00, 0x00, 0x00, 0x00, 0x0A}
	buffer = append(buffer, []byte("01234")...)

	_, _, remaining, ok := extractFrame(buffer)
	if ok {
		t.Error("Expected frame extraction to fail with incomplete data")
	}
	if len(remaining) != 10 {
		t.Errorf("Expected buffer unchanged (10 bytes), got %d bytes", len(remaining))
	}
}

func TestExtractFrameMultiple(t *testing.T) {
	// First frame: 5 bytes
	buffer := []byte{0x00, 0x00, 0x00, 0x00, 0x05}
	buffer = append(buffer, []byte("AAAAA")...)

	// Second frame: 3 bytes
	buffer = append(buffer, []byte{0x00, 0x00, 0x00, 0x00, 0x03}...)
	buffer = append(buffer, []byte("BBB")...)

	// Extract first frame
	header1, msg1, remaining1, ok1 := extractFrame(buffer)
	if !ok1 {
		t.Fatal("Expected first frame extraction to succeed")
	}
	if header1.MessageLength != 5 {
		t.Errorf("Expected first message length 5, got %d", header1.MessageLength)
	}
	if string(msg1) != "AAAAA" {
		t.Errorf("Expected first message 'AAAAA', got '%s'", string(msg1))
	}

	// Extract second frame
	header2, msg2, remaining2, ok2 := extractFrame(remaining1)
	if !ok2 {
		t.Fatal("Expected second frame extraction to succeed")
	}
	if header2.MessageLength != 3 {
		t.Errorf("Expected second message length 3, got %d", header2.MessageLength)
	}
	if string(msg2) != "BBB" {
		t.Errorf("Expected second message 'BBB', got '%s'", string(msg2))
	}

	// No more frames
	_, _, _, ok3 := extractFrame(remaining2)
	if ok3 {
		t.Error("Expected no more frames to extract")
	}
	if len(remaining2) != 0 {
		t.Errorf("Expected empty remaining buffer, got %d bytes", len(remaining2))
	}
}

func TestExtractFrameZeroLength(t *testing.T) {
	buffer := []byte{0x00, 0x00, 0x00, 0x00, 0x00}

	header, message, _, ok := extractFrame(buffer)
	if !ok {
		t.Fatal("Expected successful extraction of zero-length frame")
	}
	if header.MessageLength != 0 {
		t.Errorf("Expected message length 0, got %d", header.MessageLength)
	}
	if len(message) != 0 {
		t.Errorf("Expected empty message, got %d bytes", len(message))
	}
}

func TestFrameHeaderMaxLength(t *testing.T) {
	header := newFrameHeader(0xFFFFFFFF) // uint32 max
	encoded := header.Encode()

	decoded, ok := tryDecode(encoded[:])
	if !ok {
		t.Fatal("Expected successful decode of max length header")
	}
	if decoded.MessageLength != 0xFFFFFFFF {
		t.Errorf("Expected message length 0xFFFFFFFF, got 0x%08x", decoded.MessageLength)
	}
}

func TestFrameHeaderCompressionFlag(t *testing.T) {
	buffer := []byte{0x01, 0x00, 0x00, 0x00, 0x00} // compression flag = 1

	header, ok := tryDecode(buffer)
	if !ok {
		t.Fatal("Expected successful decode")
	}
	if header.CompressionFlag != 0x01 {
		t.Errorf("Expected compression flag 0x01, got 0x%02x", header.CompressionFlag)
	}
}

func TestFrameHeaderPartialFrameAcrossReads(t *testing.T) {
	// Simulate first chunk: partial header
	buffer := []byte{0x00, 0x00}
	_, _, _, ok1 := extractFrame(buffer)
	if ok1 {
		t.Error("Expected extraction to fail with partial header")
	}

	// Simulate second chunk: rest of header + partial data
	buffer = append(buffer, []byte{0x00, 0x00, 0x05}...) // Complete header now
	buffer = append(buffer, []byte("AB")...)             // Partial data
	_, _, _, ok2 := extractFrame(buffer)
	if ok2 {
		t.Error("Expected extraction to fail with partial data")
	}

	// Simulate third chunk: rest of data
	buffer = append(buffer, []byte("CDE")...)
	header, message, _, ok3 := extractFrame(buffer)
	if !ok3 {
		t.Fatal("Expected successful extraction after receiving complete data")
	}
	if header.MessageLength != 5 {
		t.Errorf("Expected message length 5, got %d", header.MessageLength)
	}
	if string(message) != "ABCDE" {
		t.Errorf("Expected message 'ABCDE', got '%s'", string(message))
	}
}

func TestFrameHeaderRoundtrip(t *testing.T) {
	original := newFrameHeader(42)
	encoded := original.Encode()
	decoded, ok := tryDecode(encoded[:])

	if !ok {
		t.Fatal("Expected successful decode")
	}
	if original.CompressionFlag != decoded.CompressionFlag {
		t.Errorf("Compression flag mismatch: original 0x%02x, decoded 0x%02x",
			original.CompressionFlag, decoded.CompressionFlag)
	}
	if original.MessageLength != decoded.MessageLength {
		t.Errorf("Message length mismatch: original %d, decoded %d",
			original.MessageLength, decoded.MessageLength)
	}
}

func TestExtractFrameWithTrailingData(t *testing.T) {
	// Frame: 5 bytes + trailing data
	buffer := []byte{0x00, 0x00, 0x00, 0x00, 0x05}
	buffer = append(buffer, []byte("AAAAA")...)
	buffer = append(buffer, []byte("EXTRA")...) // Trailing data

	header, message, remaining, ok := extractFrame(buffer)
	if !ok {
		t.Fatal("Expected successful frame extraction")
	}
	if header.MessageLength != 5 {
		t.Errorf("Expected message length 5, got %d", header.MessageLength)
	}
	if string(message) != "AAAAA" {
		t.Errorf("Expected message 'AAAAA', got '%s'", string(message))
	}
	if string(remaining) != "EXTRA" {
		t.Errorf("Expected remaining 'EXTRA', got '%s'", string(remaining))
	}
}
