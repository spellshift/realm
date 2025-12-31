package stream

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCircularBuffer(t *testing.T) {
	cb := NewCircularBuffer(10)

	// Test basic write
	cb.Write([]byte("hello"))
	assert.Equal(t, []byte("hello"), cb.Bytes())

	// Test append within size
	cb2 := NewCircularBuffer(10)
	cb2.Write([]byte("hello "))
	cb2.Write([]byte("world"))
	assert.Equal(t, []byte("ello world"), cb2.Bytes())

	// Test write larger than size
	cb3 := NewCircularBuffer(5)
	cb3.Write([]byte("1234567890"))
	assert.Equal(t, []byte("67890"), cb3.Bytes())

	// Test write exact size
	cb4 := NewCircularBuffer(5)
	cb4.Write([]byte("12345"))
	assert.Equal(t, []byte("12345"), cb4.Bytes())
	cb4.Write([]byte("6"))
	assert.Equal(t, []byte("23456"), cb4.Bytes())
}
