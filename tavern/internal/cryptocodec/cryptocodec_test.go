package cryptocodec

import (
	"runtime/debug"
	"testing"

	"github.com/stretchr/testify/assert"
)


func stacktrace() string {
	buf := debug.Stack()
	return string(buf)
}

func TestLRUCache(t *testing.T) {
	var session_pub_keys = NewSyncMap()
	session_pub_keys.Store(1, []byte{0x01, 0x02, 0x03})
	res, ok := session_pub_keys.Load(1)
	assert.True(t, ok)
	assert.Equal(t, []byte{0x01, 0x02, 0x03}, res)
	_, ok = session_pub_keys.Load(2)
	assert.False(t, ok)
}

func TestGoIdTrace(t *testing.T) {
	messages := make(chan string)

	go func() {
		messages <- stacktrace()
	}()

	res := <-messages

	t.Logf("res: %s", res)
}
