package tools

import (
	"context"
	"fmt"
	"io/fs"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
)

// UploadTools traverses the provided filesystem and creates assets using the provided graph.
// It skips directories and imports files as assets to be downloaded via the CDN.
func UploadTools(ctx context.Context, graph *ent.Client, fileSystem fs.ReadDirFS) error {
	tx, err := graph.Tx(ctx)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	if err := fs.WalkDir(fileSystem, ".", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to traverse file system: %w", err))
		}

		// Skip directories
		if d.IsDir() {
			return nil
		}

		// Skip embed.go and parse.go if they are in the directory
		if path == "embed.go" || path == "parse.go" {
			return nil
		}

		content, err := fs.ReadFile(fileSystem, path)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to read tool file %q: %w", path, err))
		}

		// Check if asset already exists
		exists, err := tx.Asset.Query().
			Where(asset.NameEQ(path)).
			Exist(ctx)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to check if asset already exists: %w", err))
		}

		if exists {
			// Update existing
			_, err = tx.Asset.Update().
				Where(asset.NameEQ(path)).
				SetContent(content).
				Save(ctx)
			if err != nil {
				return rollback(tx, fmt.Errorf("failed to update tool asset %q: %w", path, err))
			}
		} else {
			// Create new
			_, err = tx.Asset.Create().
				SetName(path).
				SetContent(content).
				Save(ctx)
			if err != nil {
				return rollback(tx, fmt.Errorf("failed to upload tool asset %q: %w", path, err))
			}
		}

		return nil
	}); err != nil {
		return rollback(tx, fmt.Errorf("failed to upload tools: %w", err))
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("failed to commit transaction: %w", err)
	}

	return nil
}

func rollback(tx *ent.Tx, err error) error {
	if rerr := tx.Rollback(); rerr != nil {
		err = fmt.Errorf("%w: %v", err, rerr)
	}
	return err
}
