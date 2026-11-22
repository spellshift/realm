package qrcode

import (
	"fmt"
	"io"
	"os"

	qrcode "github.com/skip2/go-qrcode"
)

// FprintQRCode writes a URL as a scannable ASCII art QR code to w.
func FprintQRCode(w io.Writer, url string) error {
	qr, err := qrcode.New(url, qrcode.Medium)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(w, qr.ToSmallString(false))
	return err
}

// PrintQRCode prints a URL as a scannable ASCII art QR code to standard error.
func PrintQRCode(url string) error {
	return FprintQRCode(os.Stderr, url)
}
