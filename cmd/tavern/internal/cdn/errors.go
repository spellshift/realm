package cdn

import (
	"net/http"

	"github.com/kcarretto/realm/cmd/tavern/internal/errors"
)

// ErrInvalidFileName occurs when an invalid file name is provided to the API.
// ErrInvalidFileContent occurs when an upload is attempted with invalid file content.
// ErrFileNotModified occurs when a download is attempted using the If-None-Match header and the file has not been modified.
// ErrFileNotFound occurs when a download is attempted for a file that does not exist.
var (
	ErrInvalidFileName    = errors.NewHTTP("invalid file name provided", http.StatusBadRequest)
	ErrInvalidFileContent = errors.NewHTTP("invalid file content provided", http.StatusBadRequest)
	ErrFileNotModified    = errors.NewHTTP("file has not been modified", http.StatusNotModified)
	ErrFileNotFound       = errors.NewHTTP("file not found", http.StatusNotFound)
)
