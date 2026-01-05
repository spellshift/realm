package tomes

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/url"
	"path/filepath"
	"strings"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/filemode"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/go-git/go-git/v5/plumbing/transport"
	gitssh "github.com/go-git/go-git/v5/plumbing/transport/ssh"
	"github.com/go-git/go-git/v5/storage/memory"
	"golang.org/x/crypto/ssh"
	"gopkg.in/yaml.v3"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/tome"
)

// GitImportOption provides configuration for creating a new GitImporter.
type GitImportOption func(*GitImporter)

// NewGitImporter initializes and returns a new GitImporter.
// If no SSH Private Key is provided, a new ECDSA P256 private key is generated.
// This panics if a new SSH Private Key cannot be generated.
func NewGitImporter(graph *ent.Client, options ...GitImportOption) *GitImporter {
	importer := &GitImporter{
		graph: graph,
	}
	for _, opt := range options {
		opt(importer)
	}

	return importer
}

// A GitImporter imports tomes from a provided Git URL.
type GitImporter struct {
	graph *ent.Client
}

// Import clones a git repository from the provided URL in memory.
// It walks the directory structure, looking for 'main.eldritch' files.
// For each 'main.eldritch' file found, it's parent directory is treated as the tome's root.
// All files in that directory and it's subdirectories (recursively) aside from the reserved
// metadata.yml file are uploaded as the tome's assets.
//
// Provided filters on tome paths may be used to limit included directories by returning true if the
// result should be included.
func (importer *GitImporter) Import(ctx context.Context, entRepo *ent.Repository, filters ...func(path string) bool) error {
	// Parse URL
	repoURL, err := url.Parse(entRepo.URL)
	if err != nil {
		return fmt.Errorf("failed to parse repo url %q: %w", entRepo.URL, err)
	}

	// Use Private Key Auth for SSH
	var authMethod transport.AuthMethod
	if repoURL.Scheme == "ssh" {
		privKey, err := ssh.ParsePrivateKey([]byte(entRepo.PrivateKey))
		if err != nil {
			return fmt.Errorf("failed to parse private key for repository: %w", err)
		}
		user := "git"
		if repoURL.User != nil && repoURL.User.Username() != "" {
			user = repoURL.User.Username()
		}
		authMethod = transport.AuthMethod(&gitssh.PublicKeys{
			User:   user,
			Signer: privKey,
			HostKeyCallbackHelper: gitssh.HostKeyCallbackHelper{
				// Ignore Host Keys
				HostKeyCallback: ssh.InsecureIgnoreHostKey(),
			},
		})
	}

	// Clone Repository (In-Memory)
	storage := memory.NewStorage()
	repo, err := git.CloneContext(ctx, storage, nil, &git.CloneOptions{
		URL:          entRepo.URL,
		SingleBranch: true,
		Depth:        1,
		Auth:         authMethod,
		Tags:         git.NoTags,
	})
	if err != nil {
		return fmt.Errorf("failed to clone: %w", err)
	}

	// Get HEAD
	head, err := repo.Head()
	if err != nil {
		return fmt.Errorf("failed to get repository HEAD: %w", err)
	}

	// Get Commit
	commit, err := repo.CommitObject(head.Hash())
	if err != nil {
		return fmt.Errorf("failed to get commit object (HEAD): %w", err)
	}

	// Get Root File Tree
	tree, err := repo.TreeObject(commit.TreeHash)
	if err != nil {
		return fmt.Errorf("failed to get tree (%q): %w", commit.Hash, err)
	}

	// Get Tome Paths
	tomePaths, err := findTomePaths(tree)
	if err != nil {
		return err
	}

	// Import Tomes
	namespace := parseNamespaceFromGit(entRepo.URL)
	for _, path := range tomePaths {
		// Apply Filters
		include := true
		for _, filter := range filters {
			if !filter(path) {
				include = false
				break
			}
		}
		if !include {
			continue
		}

		// Import Tome
		if err := importer.importFromGitTree(ctx, repo, entRepo, namespace, tree, path); err != nil {
			return fmt.Errorf("failed to import tome (%q): %w", path, err)
		}
	}

	return nil
}

// ImportFromGitTree imports a tome based on the provided path
func (importer *GitImporter) importFromGitTree(ctx context.Context, repo *git.Repository, entRepo *ent.Repository, namespace string, root *object.Tree, path string) error {
	tree, err := root.Tree(path)
	if err != nil {
		return fmt.Errorf("failed to get tome tree (%q): %w", path, err)
	}

	walker := object.NewTreeWalker(tree, true, make(map[plumbing.Hash]bool))
	defer walker.Close()

	var metadata MetadataDefinition
	var eldritch string
	var tomeFileIDs []int
	// Iterate Tome Files
	for {
		name, entry, err := walker.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("failed to walk tome tree (%q): %w", name, err)
		}

		// Skip Directory Files
		if entry.Mode == filemode.Dir {
			continue
		}

		// Read File Data
		blob, err := repo.BlobObject(entry.Hash)
		if err != nil {
			return fmt.Errorf("failed to get tome blob (%q): %w", name, err)
		}
		reader, err := blob.Reader()
		if err != nil {
			return fmt.Errorf("failed to get tome blob reader (%q): %w", name, err)
		}
		defer reader.Close()
		data, err := io.ReadAll(reader)
		if err != nil {
			return fmt.Errorf("failed to read tome file (%q): %w", name, err)
		}

		// Parse metadata.yml
		if filepath.Base(name) == "metadata.yml" {
			if err := yaml.Unmarshal(data, &metadata); err != nil {
				return fmt.Errorf("failed to parse tome metadata %q: %w", name, err)
			}
			if err := metadata.Validate(); err != nil {
				return fmt.Errorf("invalid tome metadata %q: %w", name, err)
			}

			continue
		}

		// Parse main.eldritch
		if filepath.Base(name) == "main.eldritch" {
			eldritch = string(data)
			continue
		}

		// Upload other assets
		// TODO: Namespace tomes to prevent multi-repo conflicts
		assetID, err := importer.graph.Asset.Create().
			SetName(fmt.Sprintf("%s/%s", filepath.Base(path), name)).
			SetContent(data).
			OnConflict().
			UpdateNewValues().
			ID(ctx)
		if err != nil {
			return fmt.Errorf("failed to upload tome asset %q: %w", name, err)
		}
		tomeFileIDs = append(tomeFileIDs, assetID)
	}

	// Ensure Metadata was found
	if metadata.Name == "" {
		return fmt.Errorf("tome must include 'metadata.yml' file (%q)", path)
	}

	// Ensure Eldritch not empty
	if eldritch == "" {
		return fmt.Errorf("tome must include non-empty 'eldritch.main' file (%q)", path)
	}

	// Marshal Params
	paramdefs, err := json.Marshal(metadata.ParamDefs)
	if err != nil {
		return fmt.Errorf("failed to marshal param defs for %q: %w", path, err)
	}

	// Create the tome
	_, err = importer.graph.Tome.Create().
		SetName(fmt.Sprintf("%s::%s", namespace, metadata.Name)).
		SetDescription(metadata.Description).
		SetAuthor(metadata.Author).
		SetParamDefs(string(paramdefs)).
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetTactic(tome.Tactic(metadata.Tactic)).
		SetEldritch(eldritch).
		SetRepository(entRepo).
		AddAssetIDs(tomeFileIDs...).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return fmt.Errorf("failed to create tome %q: %w", metadata.Name, err)
	}
	return nil
}

// findTomePaths returns a list of valid paths to the root directory of a Tome.
// This is based on all of the 'main.eldritch' files found in the repository.
func findTomePaths(tree *object.Tree) ([]string, error) {
	var tomePaths []string
	walker := object.NewTreeWalker(tree, true, make(map[plumbing.Hash]bool))
	defer walker.Close()
	for {
		// Fetch next entry
		name, _, err := walker.Next()
		if err == io.EOF {
			// No more entries
			break
		}
		if err != nil {
			return nil, fmt.Errorf("failed to walk repo tree: %w", err)
		}

		// If 'main.eldritch' is present, the parent directory is the tome root
		base := filepath.Base(name)
		if base == "main.eldritch" {
			// Cannot use filepath.Dir on Windows, git does not use \ separators
			tomePaths = append(
				tomePaths,
				strings.TrimSuffix(
					strings.TrimSuffix(name, base), // Get the directory name
					"/",                            // Remove the trailing /
				),
			)
		}
	}

	return tomePaths, nil
}

// parseNamespaceFromGit attempts to return a shortend namespace for the tome based on the git URL.
// If it cannot or something goes wrong, this will return the provided git URL as the namespace.
func parseNamespaceFromGit(gitURLStr string) string {
	gitURL, err := url.Parse(gitURLStr)
	if err != nil {
		return gitURLStr
	}

	// Support more pleasant names for github
	if gitURL.Host == "github.com" {
		return strings.Trim(gitURL.Path, "/")
	}

	return gitURLStr
}
