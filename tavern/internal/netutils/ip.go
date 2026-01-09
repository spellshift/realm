package netutils

import (
	"net"
)

// ValidateIP checks if the provided string is a valid IP address or "unknown".
func ValidateIP(ipaddr string) bool {
	return net.ParseIP(ipaddr) != nil || ipaddr == "unknown"
}
