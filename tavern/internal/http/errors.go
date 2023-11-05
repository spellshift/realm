package http

// An Error for an HTTP request.
type Error struct {
	Message string
	Code    int
}

// Error message associated with the error.
func (err *Error) Error() string {
	return err.Message
}
