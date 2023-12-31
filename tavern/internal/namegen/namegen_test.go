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
		name1 := namegen.GetRandomName(namegen.Simple)
		assert.NotEmpty(t, name1)
		name2 := namegen.GetRandomName(namegen.Moderate)
		assert.NotEmpty(t, name2)
		name3 := namegen.GetComplexRandomName()
		assert.NotEmpty(t, name3)
	})

	t.Run("NoDuplicates", func(t *testing.T) {
		// Ensure we don't duplicate names over the course of many trials
		names := make(map[string]bool, 1000000)
		count := 0
		for i := 0; i < 1000000; i++ {
			name := namegen.GetRandomName(namegen.Moderate)
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
