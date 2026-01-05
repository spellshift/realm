package graphql

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"encoding/hex"
	"fmt"
	"strings"

	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
)

func createBundle(ctx context.Context, client *ent.Client, bundleAssets []*ent.Asset) (*ent.Asset, error) {
	// Calculate Bundle Hash
	bundleHash := newBundleHashDigest(bundleAssets...)
	bundleName := fmt.Sprintf("Bundle-%s", bundleHash)

	// Check if bundle exists
	bundle, err := client.Asset.Query().
		Where(asset.Name(bundleName)).
		First(ctx)
	if err != nil && !ent.IsNotFound(err) {
		return nil, fmt.Errorf("failed to query tome bundle: %w", err)
	}

	// Create a new bundle if it doesn't yet exist
	if bundle == nil || ent.IsNotFound(err) {
		bundleContent, err := encodeBundle(bundleAssets...)
		if err != nil {
			return nil, fmt.Errorf("failed to encode tome bundle: %w", err)
		}

		bundle, err = client.Asset.Create().
			SetName(bundleName).
			SetContent(bundleContent).
			Save(ctx)
		if err != nil || bundle == nil {
			return nil, fmt.Errorf("failed to create tome bundle: %w", err)
		}
	}

	return bundle, nil
}

func encodeBundle(assets ...*ent.Asset) ([]byte, error) {
	buf := &bytes.Buffer{}
	gw := gzip.NewWriter(buf)
	tw := tar.NewWriter(gw)
	for _, a := range assets {
		hdr := &tar.Header{
			Name: a.Name,
			Mode: 0644,
			Size: int64(len(a.Content)),
		}
		if err := tw.WriteHeader(hdr); err != nil {
			return nil, fmt.Errorf("failed to write header (%q;%q): %w", a.Name, a.Hash, err)
		}
		if _, err := tw.Write(a.Content); err != nil {
			return nil, fmt.Errorf("failed to write asset (%q;%q): %w", a.Name, a.Hash, err)
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

func newBundleHashDigest(assets ...*ent.Asset) string {
	hashes := make([]string, 0, len(assets))
	for _, a := range assets {
		hashes = append(hashes, a.Hash)
	}

	data := []byte(strings.Join(hashes, ""))
	hash := sha3.Sum256(data)
	return hex.EncodeToString(hash[:])
}
