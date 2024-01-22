package validators_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/ent/schema/validators"
)

func TestNewJSONStringString(t *testing.T) {
	tests := []struct {
		name    string
		data    string
		wantErr error
	}{
		{
			name:    "Empty",
			data:    ``,
			wantErr: nil,
		},
		{
			name:    "Valid",
			data:    `{"data":"stuff"}`,
			wantErr: nil,
		},
		{
			name:    "Invalid",
			data:    `blah`,
			wantErr: &json.SyntaxError{},
		},
		{
			name:    "Partial",
			data:    `{"blah":"stuff"`,
			wantErr: &json.SyntaxError{},
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := validators.NewJSONStringString()(tc.data)
			if tc.wantErr == nil {
				assert.NoError(t, err)
				return
			}
			assert.ErrorAs(t, err, &tc.wantErr)
		})
	}
}

func TestNewTomeParameterDefinitions(t *testing.T) {
	tests := []struct {
		name    string
		data    string
		wantErr error
	}{
		{
			name:    "Empty",
			data:    ``,
			wantErr: nil,
		},
		{
			name:    "Int32",
			data:    `[{"name":"an-int","type": "int32"}]`,
			wantErr: nil,
		},
		{
			name:    "Multiple",
			data:    `[{"name":"an-int","type":"int32"},{"name":"a-str","type": "string"}]`,
			wantErr: nil,
		},
		{
			name:    "Valid",
			data:    `[{"name":"stuff","type":"string"}]`,
			wantErr: nil,
		},
		{
			name:    "Invalid",
			data:    `blah`,
			wantErr: &json.SyntaxError{},
		},
		{
			name:    "Partial",
			data:    `{"blah":"stuff"`,
			wantErr: &json.SyntaxError{},
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := validators.NewTomeParameterDefinitions()(tc.data)
			if tc.wantErr == nil {
				assert.NoError(t, err)
				return
			}
			assert.ErrorAs(t, err, &tc.wantErr)
		})
	}
}
