package graphql

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/file"
	"golang.org/x/crypto/sha3"
)

func createBundle(ctx context.Context, client *ent.Client, bundleFiles []*ent.File) (*ent.File, error) {
	// Calculate Bundle Hash
	bundleHash := newBundleHashDigest(bundleFiles...)
	bundleName := fmt.Sprintf("Bundle-%s", bundleHash)

	// Check if bundle exists
	bundle, err := client.File.Query().
		Where(file.Name(bundleName)).
		First(ctx)
	if err != nil && !ent.IsNotFound(err) {
		return nil, fmt.Errorf("failed to query tome bundle: %w", err)
	}

	// Create a new bundle if it doesn't yet exist
	if bundle == nil || ent.IsNotFound(err) {
		bundleContent, err := encodeBundle(bundleFiles...)
		if err != nil {
			return nil, fmt.Errorf("failed to encode tome bundle: %w", err)
		}

		bundle, err = client.File.Create().
			SetName(bundleName).
			SetContent(bundleContent).
			Save(ctx)
		if err != nil || bundle == nil {
			return nil, fmt.Errorf("failed to create tome bundle: %w", err)
		}
	}

	return bundle, nil
}

func encodeBundle(files ...*ent.File) ([]byte, error) {
	buf := &bytes.Buffer{}
	gw := gzip.NewWriter(buf)
	tw := tar.NewWriter(gw)
	for _, f := range files {
		hdr := &tar.Header{
			Name: f.Name,
			Mode: 0644,
			Size: int64(len(f.Content)),
		}
		if err := tw.WriteHeader(hdr); err != nil {
			return nil, fmt.Errorf("failed to write header (%q;%q): %w", f.Name, f.Hash, err)
		}
		if _, err := tw.Write(f.Content); err != nil {
			return nil, fmt.Errorf("failed to write file (%q;%q): %w", f.Name, f.Hash, err)
		}
	}
	if err := tw.Close(); err != nil {
		return nil, fmt.Errorf("failed to close tar writer: %w", err)
	}
	if err := gw.Close(); err != nil {
		return nil, fmt.Errorf("failed to close gzip writer: %w", err)
	}

	return buf.Bytes(), nil
}

func newBundleHashDigest(files ...*ent.File) string {
	hashes := make([]string, 0, len(files))
	for _, f := range files {
		hashes = append(hashes, f.Hash)
	}

	return hex.EncodeToString(
		sha3.New256().Sum([]byte(strings.Join(hashes, ""))),
	)
}
