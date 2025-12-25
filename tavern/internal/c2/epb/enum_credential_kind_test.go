package epb_test

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/epb"
)

func TestCredentialKindValues(t *testing.T) {
	assert.NotEmpty(t, epb.Credential_Kind(0).Values())
}

func TestCredentialKindValue(t *testing.T) {
	val, err := epb.Credential_Kind(0).Value()
	require.NoError(t, err)
	require.NotNil(t, val)
}

func TestCredentialKindMarshalGraphQL(t *testing.T) {
	var buf bytes.Buffer
	epb.Credential_Kind(0).MarshalGQL(&buf)
	assert.Equal(t, `"KIND_UNSPECIFIED"`, buf.String())
}

func TestCredentialKindUnmarshalGraphQL(t *testing.T) {
	var kind epb.Credential_Kind
	assert.NoError(t, (*epb.Credential_Kind).UnmarshalGQL(&kind, `KIND_PASSWORD`))
	assert.Equal(t, epb.Credential_KIND_PASSWORD, kind)
}

func TestCredentialKindScan(t *testing.T) {
	tests := []struct {
		name     string
		scanVal  any
		wantKind epb.Credential_Kind
	}{
		{
			name:     "PASSWORD_String",
			scanVal:  "KIND_PASSWORD",
			wantKind: epb.Credential_KIND_PASSWORD,
		},
		{
			name:     "SSH_KEY_[]uint8",
			scanVal:  []uint8("KIND_SSH_KEY"),
			wantKind: epb.Credential_KIND_SSH_KEY,
		},
		{
			name:     "Invalid",
			scanVal:  "NOT_A_KIND",
			wantKind: epb.Credential_KIND_UNSPECIFIED,
		},
		{
			name:     "Empty",
			scanVal:  "",
			wantKind: epb.Credential_KIND_UNSPECIFIED,
		},
		{
			name:     "Nil",
			scanVal:  nil,
			wantKind: epb.Credential_KIND_UNSPECIFIED,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			kind := epb.Credential_Kind(0)
			err := (*epb.Credential_Kind).Scan(&kind, tc.scanVal)
			assert.NoError(t, err)
			assert.Equal(t, tc.wantKind, kind)
		})
	}
}
