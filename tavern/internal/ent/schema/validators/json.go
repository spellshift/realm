package validators

import (
	"encoding/json"

	"realm.pub/tavern/tomes"
)

// NewJSONStringString returns a validator that errors if the string field has a value that cannot be JSON unmarshalled to a map[string]string.
func NewJSONStringString() func(string) error {
	return func(data string) error {
		if data == "" {
			return nil
		}
		var dataMap map[string]string
		return json.Unmarshal([]byte(data), &dataMap)
	}
}

// NewTomeParameterDefinitions returns a validator that errors if the string field has a value that cannot be JSON unmarshalled to a []tomes.TomeParamDefinition.
func NewTomeParameterDefinitions() func(string) error {
	return func(data string) error {
		if data == "" {
			return nil
		}
		var paramDefs []tomes.ParamDefinition
		if err := json.Unmarshal([]byte(data), &paramDefs); err != nil {
			return err
		}

		// Validate parameters
		for _, paramDef := range paramDefs {
			if err := paramDef.Validate(); err != nil {
				return err
			}
		}

		return nil
	}
}
