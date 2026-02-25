package schema

import (
	"context"
	"encoding/base64"
	"fmt"

	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/ent/hook"
	"realm.pub/tavern/internal/ent/schema/hostfilepreviewtype"

	"entgo.io/contrib/entgql"
	"entgo.io/ent"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// HostFile holds the schema definition for the HostFile entity.
type HostFile struct {
	ent.Schema
}

// Fields of the HostFile.
func (HostFile) Fields() []ent.Field {
	return []ent.Field{
		field.String("path").
			NotEmpty().
			Annotations(
				entgql.OrderField("NAME"),
			).
			Comment("Path to the file on the host system."),
		field.String("owner").
			Optional().
			Comment("User who owns the file on the host system."),
		field.String("group").
			Optional().
			Comment("Group who owns the file on the host system."),
		field.String("permissions").
			Optional().
			Comment("Permissions for the file on the host system."),
		field.Uint64("size").
			Default(0).
			Min(0).
			Annotations(
				entgql.Type("Uint64"),
				entgql.OrderField("SIZE"),
			).
			Comment("The size of the file in bytes"),
		field.String("hash").
			Optional().
			MaxLen(100).
			Comment("A SHA3-256 digest of the content field"),
		field.Bytes("content").
			Optional().
			Annotations(
				entgql.Skip(), // Don't return file content in GraphQL queries
			).
			Comment("The content of the file"),
		field.String("preview").
			Optional().
			Comment("Preview of the file content (text or base64-encoded image), max 512KB."),
		field.Enum("preview_type").
			GoType(hostfilepreviewtype.HostFilePreviewType("")).
			Default(string(hostfilepreviewtype.None)).
			Comment("The type of preview available for this file."),
	}
}

// Edges of the ent.
func (HostFile) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("host", Host.Type).
			Required().
			Unique().
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Host the file was reported on."),
		edge.From("task", Task.Type).
			Required().
			Unique().
			Ref("reported_files").
			Annotations(
				entsql.OnDelete(entsql.Cascade),
			).
			Comment("Task that reported this file."),
	}
}

// Annotations describes additional information for the ent.
func (HostFile) Annotations() []schema.Annotation {
	return []schema.Annotation{}
}

// Mixin defines common shared properties for the ent.
func (HostFile) Mixin() []ent.Mixin {
	return []ent.Mixin{
		MixinHistory{}, // created_at, last_modified_at
	}
}

// Hooks defines middleware for mutations for the ent.
func (HostFile) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(HookDeriveHostFileInfo(), ent.OpCreate|ent.OpUpdate|ent.OpUpdateOne),
	}
}

// HookDeriveHostFileInfo will update file info (e.g. size, hash, preview) whenever it is mutated.
func HookDeriveHostFileInfo() ent.Hook {
	// Get the relevant methods from the HostFile Mutation
	// See this example: https://github.com/ent/ent/blob/master/entc/integration/hooks/ent/schema/user.go#L98
	type fMutation interface {
		Content() ([]byte, bool)
		SetSize(i uint64)
		SetHash(s string)
		SetPreview(s string)
		ClearPreview()
		SetPreviewType(hostfilepreviewtype.HostFilePreviewType)
	}

	return func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			// Get the file mutation
			f, ok := m.(fMutation)
			if !ok {
				return nil, fmt.Errorf("expected hostfile mutation in schema hook, got: %+v", m)
			}

			// Set the new size
			content, _ := f.Content()
			f.SetSize(uint64(len(content)))

			// Set the new hash (if content exists)
			if len(content) > 0 {
				f.SetHash(fmt.Sprintf("%x", sha3.Sum256(content)))
			}

			// Derive preview type and content
			previewType, preview := derivePreview(content)
			f.SetPreviewType(previewType)
			if preview != "" {
				f.SetPreview(preview)
			} else {
				f.ClearPreview()
			}

			return next.Mutate(ctx, m)
		})
	}
}

const maxPreviewSize = 524288 // 512KB

// derivePreview examines file content and returns the preview type and preview string.
// Text previews are stored as-is. Image previews are base64-encoded.
func derivePreview(content []byte) (hostfilepreviewtype.HostFilePreviewType, string) {
	if len(content) == 0 {
		return hostfilepreviewtype.None, ""
	}

	if isImage(content) {
		if len(content) <= maxPreviewSize {
			return hostfilepreviewtype.Image, base64.StdEncoding.EncodeToString(content)
		}
		return hostfilepreviewtype.None, ""
	}

	if isHumanReadableText(content) {
		if len(content) > maxPreviewSize {
			return hostfilepreviewtype.Text, string(content[:maxPreviewSize])
		}
		return hostfilepreviewtype.Text, string(content)
	}

	return hostfilepreviewtype.None, ""
}

// isImage checks if the content starts with known image format magic bytes.
func isImage(content []byte) bool {
	if len(content) < 3 {
		return false
	}

	// JPEG: FF D8 FF
	if content[0] == 0xFF && content[1] == 0xD8 && content[2] == 0xFF {
		return true
	}

	if len(content) < 4 {
		return false
	}

	// PNG: 89 50 4E 47
	if content[0] == 0x89 && content[1] == 0x50 && content[2] == 0x4E && content[3] == 0x47 {
		return true
	}

	// BMP: 42 4D
	if content[0] == 0x42 && content[1] == 0x4D {
		return true
	}

	// GIF: "GIF87a" or "GIF89a"
	if len(content) >= 6 &&
		content[0] == 'G' && content[1] == 'I' && content[2] == 'F' &&
		content[3] == '8' && (content[4] == '7' || content[4] == '9') && content[5] == 'a' {
		return true
	}

	// WEBP: "RIFF" + 4 bytes + "WEBP"
	if len(content) >= 12 &&
		content[0] == 'R' && content[1] == 'I' && content[2] == 'F' && content[3] == 'F' &&
		content[8] == 'W' && content[9] == 'E' && content[10] == 'B' && content[11] == 'P' {
		return true
	}

	return false
}

// isHumanReadableText checks if the first 20 bytes of content appear to be human-readable text.
func isHumanReadableText(content []byte) bool {
	if len(content) == 0 {
		return false
	}

	sample := content
	if len(sample) > 20 {
		sample = sample[:20]
	}

	for _, b := range sample {
		if b >= 0x20 && b <= 0x7E { // printable ASCII
			continue
		}
		if b == 0x09 || b == 0x0A || b == 0x0D { // tab, LF, CR
			continue
		}
		if b >= 0x80 { // UTF-8 multibyte sequences
			continue
		}
		return false
	}
	return true
}
