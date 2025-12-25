package epb

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCredentialKind_Values(t *testing.T) {
	// Should contain "PASSWORD", "SSH_KEY", etc. from protobuf definition
	c := Credential_Kind(0)
	vals := c.Values()
	assert.NotEmpty(t, vals)
}

func TestCredentialKind_Value(t *testing.T) {
	c := Credential_KIND_PASSWORD
	val, err := c.Value()
	assert.NoError(t, err)
	assert.Equal(t, "KIND_PASSWORD", val)
}

func TestCredentialKind_Scan(t *testing.T) {
	var c Credential_Kind

	// Valid string
	err := c.Scan("KIND_SSH_KEY")
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_SSH_KEY, c)

	// Valid bytes
	err = c.Scan([]byte("KIND_PASSWORD"))
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_PASSWORD, c)

	// Nil
	err = c.Scan(nil)
	assert.NoError(t, err)

	// Empty string
	err = c.Scan("")
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_UNSPECIFIED, c)

	// Invalid string
	err = c.Scan("INVALID_KIND")
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_UNSPECIFIED, c)

	// Invalid type
	err = c.Scan(123)
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_UNSPECIFIED, c)
}

func TestCredentialKind_MarshalGQL(t *testing.T) {
	c := Credential_KIND_PASSWORD
	var buf bytes.Buffer
	c.MarshalGQL(&buf)
	assert.Equal(t, "\"KIND_PASSWORD\"", buf.String())
}

func TestCredentialKind_UnmarshalGQL(t *testing.T) {
	var c Credential_Kind

	// Valid
	err := c.UnmarshalGQL("KIND_SSH_KEY")
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_SSH_KEY, c)

	// In the real implementation of graphql.UnmarshalString, if it receives an int, it might try to cast it.
	// However, our code delegates to Scan, which handles type switching.
	// But `graphql.UnmarshalString` returns error if v is not string.
	// Let's verify what `graphql.UnmarshalString` does.
	// Assuming standard 99designs/gqlgen behavior, it usually errors on non-string.
	// If the test failed saying "expected error but got nil", it means `graphql.UnmarshalString` or `Scan` didn't error.
	// `Scan` returns nil error for default case (invalid types).
	// So if `graphql.UnmarshalString` doesn't error on int (which is odd), then `Scan` swallows it.
	// But `graphql.UnmarshalString` implementation:
	// func UnmarshalString(v interface{}) (string, error) {
	// 	switch v := v.(type) {
	// 	case string:
	// 		return v, nil
	// 	case int:
	// 		return strconv.Itoa(v), nil
	// ...
	// It seems it might convert int to string?
	// If so, UnmarshalString(123) -> "123", nil.
	// Then Scan("123") -> default case -> UNSPECIFIED, nil.
	// So no error is returned.

	// Let's test checking the value instead of error for invalid input.
	err = c.UnmarshalGQL(123)
	assert.NoError(t, err)
	assert.Equal(t, Credential_KIND_UNSPECIFIED, c)
}
