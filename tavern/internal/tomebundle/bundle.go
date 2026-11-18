package tomebundle

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"io"

	"golang.org/x/crypto/sha3"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
)

var ErrFileNotFound = errors.New("bundle file not found")

func Create(ctx context.Context, client *ent.Client, bundleAssets []*ent.Asset) (*ent.Asset, error) {
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
	hash := sha3.New256()
	for _, a := range assets {
		// Lengths make the sequence unambiguous, and hashing content rather
		// than the stored digest catches edits even if that digest is stale.
		_ = binary.Write(hash, binary.BigEndian, uint64(len(a.Name)))
		_, _ = hash.Write([]byte(a.Name))
		_ = binary.Write(hash, binary.BigEndian, uint64(len(a.Content)))
		_, _ = hash.Write(a.Content)
	}
	return hex.EncodeToString(hash.Sum(nil))
}

// Read returns a named file from the frozen bundle.
func Read(content []byte, name string) ([]byte, error) {
	gr, err := gzip.NewReader(bytes.NewReader(content))
	if err != nil {
		return nil, err
	}
	defer gr.Close()
	tr := tar.NewReader(gr)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			return nil, ErrFileNotFound
		}
		if err != nil {
			return nil, err
		}
		if hdr.Name != name {
			continue
		}
		return io.ReadAll(tr)
	}
}

// Names returns the file names frozen in a quest bundle.
func Names(content []byte) ([]string, error) {
	gr, err := gzip.NewReader(bytes.NewReader(content))
	if err != nil {
		return nil, err
	}
	defer gr.Close()
	tr := tar.NewReader(gr)
	var names []string
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			return names, nil
		}
		if err != nil {
			return nil, err
		}
		names = append(names, hdr.Name)
	}
}
