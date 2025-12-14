package tomes

import (
	"context"
	"encoding/json"
	"fmt"
	"io/fs"
	"path/filepath"

	"gopkg.in/yaml.v3"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/tome"
)

// ErrParamNameInvalid occurs when a parameter definition specifies an invalid parameter name.
// ErrParamTypeUnsupported occurs when a parameter definition specifies an unsupported parameter type.
var (
	ErrParamNameInvalid     = fmt.Errorf("invalid name in parameter definition")
	ErrParamTypeUnsupported = fmt.Errorf("unsupported type in parameter definition")
)

// ParamDefinition provides structured information for a tome to define a parameter.
type ParamDefinition struct {
	Name        string `yaml:"name" json:"name"`
	Label       string `yaml:"label" json:"label"`
	Type        string `yaml:"type" json:"type"`
	Placeholder string `yaml:"placeholder" json:"placeholder"`
}

// Validate the parameter definition, returning an error if an invalid definition has been defined.
func (paramDef ParamDefinition) Validate() error {
	if paramDef.Name == "" {
		return fmt.Errorf("%w: %q", ErrParamNameInvalid, paramDef.Name)
	}
	// TODO: Support Types
	// if paramDef.Type != "string" {
	// 	return fmt.Errorf("%w: %v is of type %v", ErrParamTypeUnsupported, paramDef.Name, paramDef.Type)
	// }
	return nil
}

// MetadataDefinition defines the contents that should be present in all tome metadata.yml files
type MetadataDefinition struct {
	Name         string `yaml:"name"`
	Description  string `yaml:"description"`
	Author       string `yaml:"author"`
	SupportModel string `yaml:"support_model"`
	Tactic       string `yaml:"tactic"`
	ParamDefs    []ParamDefinition
}
// Validate ensures the Tome metadata has been properly configured.
func (meta MetadataDefinition) Validate() error {
	if meta.Name == "" {
		return fmt.Errorf("must set 'name'")
	}
	if meta.Author == "" {
		return fmt.Errorf("must set 'author'")
	}
	if meta.Description == "" {
		return fmt.Errorf("must set 'description'")
	}
	if meta.Tactic == "" {
		return fmt.Errorf("must set 'tactic'")
	}
	for _, paramDef := range meta.ParamDefs {
		if err := paramDef.Validate(); err != nil {
			return fmt.Errorf("invalid parameter definition (%q): %w", meta.Name, err)
		}
	}
	return nil
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

		var metadata MetadataDefinition
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
				// Validate Metadata
				if err := metadata.Validate(); err != nil {
					return rollback(tx, fmt.Errorf("invalid tome metadata %q: %w", entry.Name(), err))
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

		// Marshal Params
		paramdefs, err := json.Marshal(metadata.ParamDefs)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to parse param defs for %q: %w", metadata.Name, err))
		}

		// Create the tome
		if _, err := graph.Tome.Create().
			SetName(metadata.Name).
			SetDescription(metadata.Description).
			SetAuthor(metadata.Author).
			SetParamDefs(string(paramdefs)).
			SetSupportModel(tome.SupportModel(metadata.SupportModel)).
			SetTactic(tome.Tactic(metadata.Tactic)).
			SetEldritch(eldritch).
			AddFiles(tomeFiles...).
			Save(ctx); err != nil {
			return rollback(tx, fmt.Errorf("failed to create tome %q: %w", metadata.Name, err))
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
