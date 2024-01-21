package c2

import (
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/host"
)

// convertHostPlatform converts a c2pb.Host_Platform to a host.Platform.
func convertHostPlatform(platform c2pb.Host_Platform) host.Platform {
	switch platform {
	case c2pb.Host_PLATFORM_WINDOWS:
		return host.PlatformWindows
	case c2pb.Host_PLATFORM_LINUX:
		return host.PlatformLinux
	case c2pb.Host_PLATFORM_MACOS:
		return host.PlatformMacOS
	case c2pb.Host_PLATFORM_BSD:
		return host.PlatformBSD
	}

	return host.PlatformUnknown
}
