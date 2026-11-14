package c2

import (
	"bytes"
	"encoding/binary"
	"hash/crc32"
	"image"
	"image/color"
	"image/draw"
	"image/png"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// newTestPNG builds a valid, solid-color PNG of the given dimensions.
func newTestPNG(t *testing.T, width, height int) []byte {
	t.Helper()
	img := image.NewRGBA(image.Rect(0, 0, width, height))
	draw.Draw(img, img.Bounds(), &image.Uniform{C: color.RGBA{100, 150, 200, 255}}, image.Point{}, draw.Src)

	var buf bytes.Buffer
	require.NoError(t, png.Encode(&buf, img))
	return buf.Bytes()
}

// newTestPNGHeader builds a minimal, structurally valid PNG signature + IHDR chunk declaring
// the given (attacker-controlled) dimensions, without any pixel data. This is enough for
// image.DecodeConfig to succeed (it only needs the IHDR chunk), while never actually holding
// width*height pixels in memory - exactly what a crafted "decompression bomb" style PNG would
// look like on the wire.
func newTestPNGHeader(t *testing.T, width, height uint32) []byte {
	t.Helper()

	ihdrData := make([]byte, 13)
	binary.BigEndian.PutUint32(ihdrData[0:4], width)
	binary.BigEndian.PutUint32(ihdrData[4:8], height)
	ihdrData[8] = 8  // bit depth
	ihdrData[9] = 2  // color type: truecolor
	ihdrData[10] = 0 // compression method
	ihdrData[11] = 0 // filter method
	ihdrData[12] = 0 // interlace method

	var buf bytes.Buffer
	buf.Write([]byte("\x89PNG\r\n\x1a\n")) // PNG signature

	var lenBuf [4]byte
	binary.BigEndian.PutUint32(lenBuf[:], uint32(len(ihdrData)))
	buf.Write(lenBuf[:])

	chunkTypeAndData := append([]byte("IHDR"), ihdrData...)
	buf.Write(chunkTypeAndData)

	var crcBuf [4]byte
	binary.BigEndian.PutUint32(crcBuf[:], crc32.ChecksumIEEE(chunkTypeAndData))
	buf.Write(crcBuf[:])

	return buf.Bytes()
}

func TestGenerateScreenshotThumbnail(t *testing.T) {
	t.Run("DownscalesLargeImage", func(t *testing.T) {
		content := newTestPNG(t, 1920, 1080)

		thumbBytes, err := generateScreenshotThumbnail(content)
		require.NoError(t, err)

		thumb, _, err := image.Decode(bytes.NewReader(thumbBytes))
		require.NoError(t, err)

		bounds := thumb.Bounds()
		assert.LessOrEqual(t, bounds.Dx(), thumbnailMaxWidth)
		assert.LessOrEqual(t, bounds.Dy(), thumbnailMaxHeight)

		// Aspect ratio should be preserved (16:9).
		assert.Equal(t, thumbnailMaxWidth, bounds.Dx())
		assert.InDelta(t, float64(thumbnailMaxWidth)*9/16, float64(bounds.Dy()), 1)
	})

	t.Run("DoesNotUpscaleSmallImage", func(t *testing.T) {
		content := newTestPNG(t, 50, 30)

		thumbBytes, err := generateScreenshotThumbnail(content)
		require.NoError(t, err)

		thumb, _, err := image.Decode(bytes.NewReader(thumbBytes))
		require.NoError(t, err)

		bounds := thumb.Bounds()
		assert.Equal(t, 50, bounds.Dx())
		assert.Equal(t, 30, bounds.Dy())
	})

	t.Run("InvalidContent", func(t *testing.T) {
		_, err := generateScreenshotThumbnail([]byte("not an image"))
		assert.Error(t, err)
	})

	t.Run("RejectsOversizedDimension", func(t *testing.T) {
		// A single huge dimension (width here) with the other kept small still exceeds
		// maxDecodeDimension and must be rejected without attempting a full decode.
		header := newTestPNGHeader(t, maxDecodeDimension+1, 10)

		_, err := generateScreenshotThumbnail(header)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "exceed limit")
	})

	t.Run("RejectsOversizedPixelCount", func(t *testing.T) {
		// Neither dimension alone exceeds maxDecodeDimension, but the product does, e.g. a
		// maliciously crafted "decompression bomb" style image header.
		const dim = maxDecodeDimension - 1 // stays under the per-dimension cap
		header := newTestPNGHeader(t, dim, dim)
		require.Greater(t, int64(dim)*int64(dim), int64(maxDecodePixels))

		_, err := generateScreenshotThumbnail(header)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "too many pixels")
	})

	t.Run("AcceptsHeaderWithinLimits", func(t *testing.T) {
		// Sanity check that newTestPNGHeader itself produces a header image.DecodeConfig can
		// parse; a within-limits header should fail later (no pixel data), not at the dimension
		// check, proving the cap isn't rejecting valid images.
		header := newTestPNGHeader(t, 100, 100)

		_, err := generateScreenshotThumbnail(header)
		require.Error(t, err)
		assert.NotContains(t, err.Error(), "exceed limit")
		assert.NotContains(t, err.Error(), "too many pixels")
	})
}
