package namegen_test

import (
	"testing"

	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/namegen"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGetRandomName(t *testing.T) {
	t.Run("BasicName", func(t *testing.T) {
		name := namegen.GetRandomName(1)
		assert.NotEmpty(t, name)
	})

	t.Run("NoDuplicates", func(t *testing.T) {
		// Ensure we don't duplicate names over the course of many trials
		names := make(map[string]bool, 1000000)
		count := 0
		for i := 0; i < 1000000; i++ {
			name := namegen.GetRandomName(2)
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
			result := namegen.Beaconnameinstring(tc.beacons, tc.str)
			if result != tc.expected {
				t.Errorf("Beaconnameinstring(%v, %s) = %v; expected %v", tc.beacons, tc.str, result, tc.expected)
			}
		})
	}
}
