package c2

import (
	"bytes"
	"fmt"
	"image"
	"image/png"
)

const (
	// thumbnailMaxWidth is the maximum width (in pixels) of a generated screenshot thumbnail.
	thumbnailMaxWidth = 320
	// thumbnailMaxHeight is the maximum height (in pixels) of a generated screenshot thumbnail.
	thumbnailMaxHeight = 240
	// maxDecodeDimension is the maximum width or height (in pixels) of a source image we are
	// willing to decode. Image decoders allocate a pixel buffer sized directly from the
	// width/height declared in the file header, so this bounds that allocation against a
	// maliciously crafted (or simply huge) image before any real decoding work happens.
	maxDecodeDimension = 16384
	// maxDecodePixels further bounds the total pixel count (width * height) of a source image
	// we are willing to decode, independent of maxDecodeDimension, so e.g. a 16383x16383 image
	// (~1GB as RGBA) is still rejected even though neither dimension alone exceeds the cap.
	maxDecodePixels = 40_000_000 // ~40 megapixels (~160MB as decoded RGBA)
)

// generateScreenshotThumbnail decodes the provided screenshot image content and returns a
// downscaled, PNG-encoded copy suitable for use as a thumbnail preview. The aspect ratio of
// the original image is preserved, and the image is never upscaled.
//
// The content is untrusted (reported by a beacon), so the image header is checked and bounded
// before the full image is decoded, to avoid an attacker-controlled allocation of arbitrary size.
func generateScreenshotThumbnail(content []byte) ([]byte, error) {
	cfg, _, err := image.DecodeConfig(bytes.NewReader(content))
	if err != nil {
		return nil, fmt.Errorf("failed to decode screenshot image header: %w", err)
	}
	if cfg.Width <= 0 || cfg.Height <= 0 {
		return nil, fmt.Errorf("invalid screenshot image dimensions: %dx%d", cfg.Width, cfg.Height)
	}
	if cfg.Width > maxDecodeDimension || cfg.Height > maxDecodeDimension {
		return nil, fmt.Errorf("screenshot image dimensions exceed limit: %dx%d", cfg.Width, cfg.Height)
	}
	if pixels := int64(cfg.Width) * int64(cfg.Height); pixels > maxDecodePixels {
		return nil, fmt.Errorf("screenshot image has too many pixels (%d > %d)", pixels, maxDecodePixels)
	}

	src, _, err := image.Decode(bytes.NewReader(content))
	if err != nil {
		return nil, fmt.Errorf("failed to decode screenshot image: %w", err)
	}

	bounds := src.Bounds()
	srcWidth, srcHeight := bounds.Dx(), bounds.Dy()
	if srcWidth <= 0 || srcHeight <= 0 {
		return nil, fmt.Errorf("invalid screenshot image dimensions: %dx%d", srcWidth, srcHeight)
	}

	// Determine the scale factor required to fit within the thumbnail bounds, never upscaling.
	scale := 1.0
	if widthScale := float64(thumbnailMaxWidth) / float64(srcWidth); widthScale < scale {
		scale = widthScale
	}
	if heightScale := float64(thumbnailMaxHeight) / float64(srcHeight); heightScale < scale {
		scale = heightScale
	}

	dstWidth, dstHeight := srcWidth, srcHeight
	if scale < 1.0 {
		dstWidth = max(1, int(float64(srcWidth)*scale))
		dstHeight = max(1, int(float64(srcHeight)*scale))
	}

	dst := image.NewRGBA(image.Rect(0, 0, dstWidth, dstHeight))
	for y := 0; y < dstHeight; y++ {
		srcY := bounds.Min.Y + (y*srcHeight)/dstHeight
		for x := 0; x < dstWidth; x++ {
			srcX := bounds.Min.X + (x*srcWidth)/dstWidth
			dst.Set(x, y, src.At(srcX, srcY))
		}
	}

	var buf bytes.Buffer
	if err := png.Encode(&buf, dst); err != nil {
		return nil, fmt.Errorf("failed to encode screenshot thumbnail: %w", err)
	}
	return buf.Bytes(), nil
}
