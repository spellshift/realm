package namegen_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/namegen"
)

func TestGetRandomName(t *testing.T) {
	t.Run("BasicName", func(t *testing.T) {
		name1 := namegen.NewSimple()
		assert.NotEmpty(t, name1)
		name2 := namegen.New()
		assert.NotEmpty(t, name2)
		name3 := namegen.NewComplex()
		assert.NotEmpty(t, name3)
	})

	t.Run("NoDuplicates", func(t *testing.T) {
		maxIterations := 100000

		// Ensure we don't duplicate names over the course of many trials
		names := make(map[string]bool, maxIterations)
		count := 0
		for i := 0; i < maxIterations; i++ {
			name := namegen.NewComplex()
			exists, ok := names[name]
			require.False(t, ok, "Name %s already exists - after %d attempts", name, count)
			assert.False(t, exists)
			names[name] = true
			count++
		}
	})

}

// TestBeaconnameinstring tests the Beaconnameinstring function
func TestBeaconnameinstring(t *testing.T) {
	testCases := []struct {
		name     string
		beacons  []*ent.Beacon
		str      string
		expected bool
	}{
		{
			name: "String matches a beacon name",
			beacons: []*ent.Beacon{
				{Name: "Alpha"},
				{Name: "Beta"},
			},
			str:      "Beta",
			expected: true,
		},
		{
			name: "String does not match any beacon name",
			beacons: []*ent.Beacon{
				{Name: "Alpha"},
				{Name: "Beta"},
			},
			str:      "Gamma",
			expected: false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			result := namegen.IsCollision(tc.beacons, tc.str)
			if result != tc.expected {
				t.Errorf("Beaconnameinstring(%v, %s) = %v; expected %v", tc.beacons, tc.str, result, tc.expected)
			}
		})
	}
}
