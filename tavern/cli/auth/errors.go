package auth

import "fmt"

// ErrInvalidURL occurs if an invalid Tavern url is provided.
var (
	ErrInvalidURL = fmt.Errorf("must provide valid tavern url")
)
