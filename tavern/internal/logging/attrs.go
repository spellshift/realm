package logging

import (
	"log/slog"
	"strconv"
)

const (
	KeyTargetType = "target_type"
	KeyTargetID   = "target_id"
	KeyTargetName = "target_name"
)

// TargetAttrs returns standardized primary target attributes for an audit record.
func TargetAttrs(targetType, targetID, targetName string) []any {
	out := []any{
		slog.String(KeyTargetType, targetType),
		slog.String(KeyTargetID, targetID),
	}
	if targetName != "" {
		out = append(out, slog.String(KeyTargetName, targetName))
	}
	return out
}

// TargetIntAttrs is a convenience wrapper around TargetAttrs for integer Ent IDs.
func TargetIntAttrs(targetType string, targetID int, targetName string) []any {
	return TargetAttrs(targetType, strconv.Itoa(targetID), targetName)
}

// HostAttrs builds standardized host attributes.
func HostAttrs(id int, name, identifier string) []any {
	attrs := []any{slog.Int("host_id", id)}
	if name != "" {
		attrs = append(attrs, slog.String("host_name", name))
	}
	if identifier != "" {
		attrs = append(attrs, slog.String("host_identifier", identifier))
	}
	return attrs
}

// BeaconAttrs builds standardized beacon attributes.
func BeaconAttrs(id int, name, identifier string) []any {
	attrs := []any{slog.Int("beacon_id", id)}
	if name != "" {
		attrs = append(attrs, slog.String("beacon_name", name))
	}
	if identifier != "" {
		attrs = append(attrs, slog.String("beacon_identifier", identifier))
	}
	return attrs
}

// QuestAttrs builds standardized quest attributes, automatically scrubbing any secret keys in parameters JSON.
func QuestAttrs(id int, name, parametersJSON string) []any {
	attrs := []any{slog.Int("quest_id", id)}
	if name != "" {
		attrs = append(attrs, slog.String("quest_name", name))
	}
	if parametersJSON != "" {
		attrs = append(attrs, slog.Any("parameters", ScrubJSONValue(parametersJSON)))
	}
	return attrs
}

// TomeAttrs builds standardized tome attributes.
func TomeAttrs(id int, name, tactic string) []any {
	attrs := []any{slog.Int("tome_id", id)}
	if name != "" {
		attrs = append(attrs, slog.String("tome_name", name))
	}
	if tactic != "" {
		attrs = append(attrs, slog.String("tome_tactic", tactic))
	}
	return attrs
}

// TaskAttrs builds standardized task attributes.
func TaskAttrs(id int, questID, beaconID int) []any {
	attrs := []any{slog.Int("task_id", id)}
	if questID > 0 {
		attrs = append(attrs, slog.Int("quest_id", questID))
	}
	if beaconID > 0 {
		attrs = append(attrs, slog.Int("beacon_id", beaconID))
	}
	return attrs
}

// PortalAttrs builds standardized portal attributes.
func PortalAttrs(portalID, beaconID, hostID int) []any {
	attrs := []any{slog.Int("portal_id", portalID)}
	if beaconID > 0 {
		attrs = append(attrs, slog.Int("beacon_id", beaconID))
	}
	if hostID > 0 {
		attrs = append(attrs, slog.Int("host_id", hostID))
	}
	return attrs
}

// ShellAttrs builds standardized shell attributes.
func ShellAttrs(shellID, beaconID, hostID int) []any {
	attrs := []any{slog.Int("shell_id", shellID)}
	if beaconID > 0 {
		attrs = append(attrs, slog.Int("beacon_id", beaconID))
	}
	if hostID > 0 {
		attrs = append(attrs, slog.Int("host_id", hostID))
	}
	return attrs
}

// AssetAttrs builds standardized asset attributes.
func AssetAttrs(id int, name, hash string, size int) []any {
	attrs := []any{slog.Int("asset_id", id)}
	if name != "" {
		attrs = append(attrs, slog.String("asset_name", name))
	}
	if hash != "" {
		attrs = append(attrs, slog.String("asset_hash", hash))
	}
	if size >= 0 {
		attrs = append(attrs, slog.Int("asset_size", size))
	}
	return attrs
}
