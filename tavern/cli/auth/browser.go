package auth

import (
	"fmt"
	"os"
	"os/exec"
	"runtime"
)

// DefaultBrowser provides a cross-platform way to open a browser.
// It implements the Browser interface.
type DefaultBrowser struct{}

// OpenURL opens the provided URL in the system's default browser.
func (b DefaultBrowser) OpenURL(url string) error {
	var err error
	switch runtime.GOOS {
	case "linux":
		// Check if we have a display available. If not, don't even try to open a browser.
		if os.Getenv("DISPLAY") == "" && os.Getenv("WAYLAND_DISPLAY") == "" {
			return fmt.Errorf("no display environment detected (DISPLAY or WAYLAND_DISPLAY)")
		}
		err = exec.Command("xdg-open", url).Start()
	case "windows":
		err = exec.Command("rundll32", "url.dll,FileProtocolHandler", url).Start()
	case "darwin":
		err = exec.Command("open", url).Start()
	default:
		err = fmt.Errorf("unsupported platform")
	}

	if err != nil {
		return fmt.Errorf("failed to open browser: %w", err)
	}
	return nil
}
