package shell

import "fmt"

// ErrInvalidShell occurs when the request did not provide a valid shell id.
// ErrShellNotFound occurs when the provided shell id did not correspond to a valid shell.
// ErrShellLookupFailed occurs when we fail to query for the shell.
// ErrFailedToQueryPortals occurs when we fail to determine if there are any open portals for the shell.
// ErrChannelClosed occurs when one of the synchronization channels for the shell closes.
var (
	ErrShellIDInvalid       = fmt.Errorf("must provide integer value for 'shell_id'")
	ErrShellNotFound        = fmt.Errorf("no shell found for provided 'shell_id'")
	ErrShellLookupFailed    = fmt.Errorf("failed to query shell information")
	ErrFailedToQueryPortals = fmt.Errorf("failed to query open portals for the shell")
	ErrChannelClosed        = fmt.Errorf("channel closed")
)
