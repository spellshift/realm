package tagrules

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func testHost() HostSnapshot {
	return HostSnapshot{
		Platform:   "PLATFORM_LINUX",
		PrimaryIP:  "10.0.0.5",
		ExternalIP: "203.0.113.7",
		Name:       "db1-prod",
	}
}

func TestMatchRowExact(t *testing.T) {
	h := testHost()
	assert.True(t, MatchRow(h, Condition{Field: "platform", Predicate: "is", Values: []string{"PLATFORM_LINUX"}}))
	assert.False(t, MatchRow(h, Condition{Field: "platform", Predicate: "is", Values: []string{"PLATFORM_WINDOWS"}}))
	assert.True(t, MatchRow(h, Condition{Field: "platform", Predicate: "is-not", Values: []string{"PLATFORM_WINDOWS"}}))
	assert.True(t, MatchRow(h, Condition{Field: "platform", Predicate: "is-in", Values: []string{"PLATFORM_WINDOWS", "PLATFORM_LINUX"}}))
	assert.False(t, MatchRow(h, Condition{Field: "platform", Predicate: "is-not-in", Values: []string{"PLATFORM_WINDOWS", "PLATFORM_LINUX"}}))
	assert.True(t, MatchRow(h, Condition{Field: "platform", Predicate: "is-not-in", Values: []string{"PLATFORM_WINDOWS"}}))
}

func TestMatchRowFreeText(t *testing.T) {
	h := testHost()
	assert.True(t, MatchRow(h, Condition{Field: "host", Predicate: "is", Values: []string{"db1-prod"}}))
	assert.True(t, MatchRow(h, Condition{Field: "host", Predicate: "contains", Values: []string{"db1"}}))
	assert.False(t, MatchRow(h, Condition{Field: "host", Predicate: "contains", Values: []string{"web"}}))
	assert.True(t, MatchRow(h, Condition{Field: "host", Predicate: "does-not-contain", Values: []string{"web"}}))
	assert.False(t, MatchRow(h, Condition{Field: "host", Predicate: "does-not-contain", Values: []string{"db1"}}))
	assert.True(t, MatchRow(h, Condition{Field: "primaryIP", Predicate: "contains", Values: []string{"10.0.0"}}))
	assert.True(t, MatchRow(h, Condition{Field: "externalIP", Predicate: "is-in", Values: []string{"198.51.100.1", "203.0.113.7"}}))
}

func TestMatchesAND(t *testing.T) {
	h := testHost()
	conds := []Condition{
		{Field: "platform", Predicate: "is", Values: []string{"PLATFORM_LINUX"}},
		{Field: "host", Predicate: "contains", Values: []string{"db1"}},
	}
	assert.True(t, Matches(h, conds))
	conds[1].Values = []string{"web"}
	assert.False(t, Matches(h, conds))
}

func TestMatchesJSONRoundTrip(t *testing.T) {
	conds := []Condition{
		{Field: "platform", Predicate: "is-in", Values: []string{"PLATFORM_LINUX", "PLATFORM_WINDOWS"}},
		{Field: "primaryIP", Predicate: "contains", Values: []string{"10.0.0"}},
	}
	raw, err := MarshalConditions(conds)
	require.NoError(t, err)
	parsed, err := ParseConditions(raw)
	require.NoError(t, err)
	assert.Equal(t, conds, parsed)
	assert.True(t, Matches(testHost(), parsed))
}

func TestValidateConditionRejects(t *testing.T) {
	assert.Error(t, ValidateCondition(Condition{Field: "status", Predicate: "is", Values: []string{"x"}}))
	assert.Error(t, ValidateCondition(Condition{Field: "platform", Predicate: "contains", Values: []string{"LINUX"}}))
	assert.Error(t, ValidateCondition(Condition{Field: "host", Predicate: "is", Values: []string{}}))
	assert.Error(t, ValidateCondition(Condition{Field: "host", Predicate: "is", Values: []string{"a", "b"}}))
	assert.Error(t, ValidateCondition(Condition{Field: "transport", Predicate: "is", Values: []string{"x"}}))
	assert.Error(t, ValidateCondition(Condition{Field: "service", Predicate: "is", Values: []string{"x"}}))
}

func TestParseSweepMode(t *testing.T) {
	m, err := ParseSweepMode("fill-empty")
	require.NoError(t, err)
	assert.Equal(t, SweepFillEmpty, m)
	m, err = ParseSweepMode("OVERWRITE")
	require.NoError(t, err)
	assert.Equal(t, SweepOverwrite, m)
	_, err = ParseSweepMode("bogus")
	assert.Error(t, err)
}
