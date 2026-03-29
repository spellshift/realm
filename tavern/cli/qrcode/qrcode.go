package qrcode

import (
	"fmt"

	qrcode "github.com/skip2/go-qrcode"
)

// PrintQRCode prints a URL as a scannable ASCII art QR code to standard output
func PrintQRCode(url string) error {
	qr, err := qrcode.New(url, qrcode.Medium)
	if err != nil {
		return err
	}
	fmt.Println(qr.ToSmallString(false))
	return nil
}
