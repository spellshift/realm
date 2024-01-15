package namegen_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/namegen"
)

func TestGetRandomName(t *testing.T) {
	t.Run("BasicName", func(t *testing.T) {
		name := namegen.GetRandomName()
		assert.NotEmpty(t, name)
	})

	t.Run("NoDuplicates", func(t *testing.T) {
		// Ensure we don't duplicate names over the course of many trials
		names := make(map[string]bool, 1000000)
		count := 0
		for i := 0; i < 1000000; i++ {
			name := namegen.GetRandomName()
			exists, ok := names[name]
			require.False(t, ok, "Name %s already exists - after %d attempts", name, count)
			assert.False(t, exists)
			names[name] = true
			count++
		}
	})

}
