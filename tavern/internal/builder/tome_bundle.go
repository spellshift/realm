package builder

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"fmt"

	"realm.pub/tavern/internal/ent"
)

// PackageTome loads a tome by ID and packages its eldritch script and assets
// into a tar.gz archive. Asset names are preserved as-is (including directory
// structure like "example/linux/test-file"). The eldritch script is stored as
// "main.eldritch" in the archive root.
func PackageTome(ctx context.Context, graph *ent.Client, tomeID int) ([]byte, error) {
	t, err := graph.Tome.Get(ctx, tomeID)
	if err != nil {
		return nil, fmt.Errorf("failed to load tome %d: %w", tomeID, err)
	}

	assets, err := t.QueryAssets().All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query assets for tome %d: %w", tomeID, err)
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
			return nil, fmt.Errorf("failed to write eldritch header for tome %d: %w", tomeID, err)
		}
		if _, err := tw.Write([]byte(t.Eldritch)); err != nil {
			return nil, fmt.Errorf("failed to write eldritch content for tome %d: %w", tomeID, err)
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
			return nil, fmt.Errorf("failed to write asset header %q for tome %d: %w", a.Name, tomeID, err)
		}
		if _, err := tw.Write(a.Content); err != nil {
			return nil, fmt.Errorf("failed to write asset content %q for tome %d: %w", a.Name, tomeID, err)
		}
	}

	if err := tw.Close(); err != nil {
		return nil, fmt.Errorf("failed to close tar writer for tome %d: %w", tomeID, err)
	}
	if err := gw.Close(); err != nil {
		return nil, fmt.Errorf("failed to close gzip writer for tome %d: %w", tomeID, err)
	}

	return buf.Bytes(), nil
}
