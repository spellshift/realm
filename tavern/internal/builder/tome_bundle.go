package builder

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"fmt"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
)

// PackageTomeEntity packages an already loaded tome's eldritch script and assets
// into a tar.gz archive. Asset names are preserved as-is (including directory
// structure like "example/linux/test-file"). The eldritch script is stored as
// "main.eldritch" in the archive root.
func PackageTomeEntity(ctx context.Context, t *ent.Tome) ([]byte, error) {
	assets, err := t.QueryAssets().Order(ent.Asc(asset.FieldID)).All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query assets for tome %d: %w", t.ID, err)
	}

	buf := &bytes.Buffer{}
	gw := gzip.NewWriter(buf)
	tw := tar.NewWriter(gw)

	// Add the eldritch script as main.eldritch.
	if t.Eldritch != "" {
		hdr := &tar.Header{
			Name: "main.eldritch",
			Mode: 0644,
			Size: int64(len(t.Eldritch)),
		}
		if err := tw.WriteHeader(hdr); err != nil {
			return nil, fmt.Errorf("failed to write eldritch header for tome %d: %w", t.ID, err)
		}
		if _, err := tw.Write([]byte(t.Eldritch)); err != nil {
			return nil, fmt.Errorf("failed to write eldritch content for tome %d: %w", t.ID, err)
		}
	}

	// Add each asset, preserving its server-side name.
	for _, a := range assets {
		hdr := &tar.Header{
			Name: a.Name,
			Mode: 0644,
			Size: int64(len(a.Content)),
		}
		if err := tw.WriteHeader(hdr); err != nil {
			return nil, fmt.Errorf("failed to write asset header %q for tome %d: %w", a.Name, t.ID, err)
		}
		if _, err := tw.Write(a.Content); err != nil {
			return nil, fmt.Errorf("failed to write asset content %q for tome %d: %w", a.Name, t.ID, err)
		}
	}

	if err := tw.Close(); err != nil {
		return nil, fmt.Errorf("failed to close tar writer for tome %d: %w", t.ID, err)
	}
	if err := gw.Close(); err != nil {
		return nil, fmt.Errorf("failed to close gzip writer for tome %d: %w", t.ID, err)
	}

	return buf.Bytes(), nil
}
