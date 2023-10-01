package tomes

import (
	"context"
	"fmt"
	"io/fs"
	"path/filepath"

	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/tome"
	"gopkg.in/yaml.v3"
)

type tomeMetadata struct {
	Name        string
	Description string
	ParamDefs   string
}

// UploadTomes traverses the provided filesystem and creates tomes using the provided graph.
// Each directory at the root of the filesystem is a tome, and must contain the required
// "metadata.yml" and "main.eldritch" files. You may use the tomes.FileSystem to include the
// default tomes specified here.
func UploadTomes(ctx context.Context, graph *ent.Client, fileSystem fs.ReadDirFS) error {
	entries, err := fileSystem.ReadDir(".")
	if err != nil {
		return fmt.Errorf("failed to read filesystem: %w", err)
	}

	tx, err := graph.Tx(ctx)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}

		exists, err := graph.Tome.Query().
			Where(tome.Name(entry.Name())).
			Exist(ctx)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to check if tome already exists: %w", err))
		}
		if exists {
			continue
		}

		var metadata tomeMetadata
		var eldritch string
		var tomeFiles []*ent.File
		if err := fs.WalkDir(fileSystem, entry.Name(), func(path string, d fs.DirEntry, err error) error {
			// Skip directories
			if d.IsDir() {
				return nil
			}

			// Parse File
			if err != nil {
				return rollback(tx, fmt.Errorf("failed to parse tome: %w", err))
			}
			content, err := fs.ReadFile(fileSystem, path)
			if err != nil {
				return rollback(tx, fmt.Errorf("failed to parse tome file %q: %w", path, err))
			}

			// Parse metadata.yml
			if filepath.Base(path) == "metadata.yml" {
				if err := yaml.Unmarshal(content, &metadata); err != nil {
					return rollback(tx, fmt.Errorf("failed to parse %q: %w", path, err))
				}
				return nil
			}

			// Parse main.eldritch
			if filepath.Base(path) == "main.eldritch" {
				eldritch = string(content)
				return nil
			}

			// Upload other files
			f, err := graph.File.Create().
				SetName(path).
				SetContent(content).
				Save(ctx)
			if err != nil {
				return rollback(tx, fmt.Errorf("failed to upload tome file %q: %w", path, err))
			}
			tomeFiles = append(tomeFiles, f)

			return nil
		}); err != nil {
			return rollback(tx, fmt.Errorf("failed to parse and upload tome %q: %w", entry.Name(), err))
		}

		// Create the tome
		if _, err := graph.Tome.Create().
			SetName(metadata.Name).
			SetDescription(metadata.Description).
			SetParamDefs(metadata.ParamDefs).
			SetEldritch(eldritch).
			AddFiles(tomeFiles...).
			Save(ctx); err != nil {
			return rollback(tx, fmt.Errorf("failed to create tome %q: %w", entry.Name(), err))
		}
	}
	return nil
}

func rollback(tx *ent.Tx, err error) error {
	if rerr := tx.Rollback(); rerr != nil {
		err = fmt.Errorf("%w: %v", err, rerr)
	}
	return err
}
