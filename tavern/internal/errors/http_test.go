package errors_test

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/errors"
)

// TestWrapHandler asserts that wrapped handlers exhibit expected behavior.
func TestWrapHandler(t *testing.T) {
	t.Run("BasicError", newWrapHandlerTest(
		errors.NewHTTP("some error", 101),
		101,
	))
	t.Run("NoError", newWrapHandlerTest(
		nil,
		200,
	))
	t.Run("UnhandledError", newWrapHandlerTest(
		fmt.Errorf("unhandled oops"),
		http.StatusInternalServerError,
	))
}

func newWrapHandlerTest(err error, expectedStatusCode int) func(*testing.T) {
	return func(t *testing.T) {
		// Create an HTTP handler that just returns the expected error
		handler := errors.WrapHandler(func(http.ResponseWriter, *http.Request) error {
			return err
		})

		// Prepare a new ResponseWriter and Request
		w := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodPost, "/error/test", nil)

		// Invoke the handler
		handler.ServeHTTP(w, req)

		// Assert the result behaves as expected
		result := w.Result()
		assert.Equal(t, expectedStatusCode, result.StatusCode)
	}
}
