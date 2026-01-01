package c2

import (
	"net"
)

func validateIP(ipaddr string) bool {
	return net.ParseIP(ipaddr) != nil || ipaddr == "unknown"
}
