package ocr

import (
	"context"
	"crypto/ed25519"
	"fmt"
	"log/slog"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
)

const Audience = "/internal/ocr-check"

func NewToken(key ed25519.PrivateKey) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodEdDSA, jwt.RegisteredClaims{
		Audience: jwt.ClaimStrings{Audience},
	})
	return token.SignedString(key)
}

func verifyToken(raw string, key ed25519.PublicKey) error {
	token, err := jwt.ParseWithClaims(raw, &jwt.RegisteredClaims{}, func(token *jwt.Token) (any, error) {
		if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
			return nil, fmt.Errorf("unexpected signing method")
		}
		return key, nil
	}, jwt.WithAudience(Audience))
	if err != nil {
		return fmt.Errorf("invalid OCR token: %w", err)
	}
	if !token.Valid {
		return fmt.Errorf("invalid OCR token")
	}
	return nil
}

func NewHandler(graph *ent.Client, key ed25519.PublicKey, recognizer Recognizer) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		auth := strings.Fields(r.Header.Get("Authorization"))
		if len(auth) != 2 || !strings.EqualFold(auth[0], "Bearer") || verifyToken(auth[1], key) != nil {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		if err := processPending(r.Context(), graph, recognizer); err != nil {
			slog.ErrorContext(r.Context(), "OCR poll failed", "err", err)
			http.Error(w, "internal error", http.StatusInternalServerError)
			return
		}
		w.WriteHeader(http.StatusOK)
	})
}

func processPending(ctx context.Context, graph *ent.Client, recognizer Recognizer) error {
	now := time.Now()
	images, err := graph.Screenshot.Query().
		Where(
			screenshot.OcrRequested(true),
			screenshot.Or(screenshot.OcrLeaseUntilIsNil(), screenshot.OcrLeaseUntilLT(now)),
		).
		Select(screenshot.FieldID, screenshot.FieldOcrAttempts).
		Limit(25).
		All(ctx)
	if err != nil {
		return fmt.Errorf("query pending screenshots: %w", err)
	}
	var workers sync.WaitGroup
	capacity := make(chan struct{}, 5)
	for _, image := range images {
		capacity <- struct{}{}
		workers.Add(1)
		go func(image *ent.Screenshot) {
			defer workers.Done()
			defer func() { <-capacity }()
			processImage(ctx, graph, recognizer, image, now)
		}(image)
	}
	workers.Wait()
	return nil
}

func processImage(ctx context.Context, graph *ent.Client, recognizer Recognizer, image *ent.Screenshot, now time.Time) {
	claimed, err := graph.Screenshot.Update().
		Where(
			screenshot.ID(image.ID),
			screenshot.OcrRequested(true),
			screenshot.OcrAttemptsEQ(image.OcrAttempts),
			screenshot.Or(screenshot.OcrLeaseUntilIsNil(), screenshot.OcrLeaseUntilLT(now)),
		).
		AddOcrAttempts(1).
		SetOcrLeaseUntil(time.Now().Add(2 * time.Minute)).
		Save(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "failed to claim screenshot OCR", "screenshot_id", image.ID, "err", err)
		return
	}
	if claimed == 0 {
		return
	}
	stored, err := graph.Screenshot.Get(ctx, image.ID)
	if err != nil {
		slog.ErrorContext(ctx, "failed to load screenshot for OCR", "screenshot_id", image.ID, "err", err)
		return
	}
	callCtx, cancel := context.WithTimeout(ctx, 25*time.Second)
	text, err := recognizer.Recognize(callCtx, stored.Content)
	cancel()
	if err != nil {
		slog.ErrorContext(ctx, "screenshot OCR failed", "screenshot_id", image.ID, "attempt", image.OcrAttempts+1, "err", err)
		if isPermanent(err) {
			if _, saveErr := graph.Screenshot.UpdateOneID(image.ID).SetOcrRequested(false).ClearOcrLeaseUntil().Save(ctx); saveErr != nil {
				slog.ErrorContext(ctx, "failed to mark screenshot OCR as failed", "screenshot_id", image.ID, "err", saveErr)
			}
			return
		}
		if _, saveErr := graph.Screenshot.UpdateOneID(image.ID).SetOcrLeaseUntil(time.Now().Add(retryDelay(image.OcrAttempts + 1))).Save(ctx); saveErr != nil {
			slog.ErrorContext(ctx, "failed to schedule screenshot OCR retry", "screenshot_id", image.ID, "err", saveErr)
		}
		return
	}
	if _, err := graph.Screenshot.UpdateOneID(image.ID).
		SetOcrText(text).
		SetOcrRequested(false).
		ClearOcrLeaseUntil().
		Save(ctx); err != nil {
		slog.ErrorContext(ctx, "failed to save screenshot OCR text", "screenshot_id", image.ID, "err", err)
	}
}

func retryDelay(attempt int) time.Duration {
	delay := time.Minute
	for i := 1; i < attempt && delay < time.Hour; i++ {
		delay *= 2
	}
	if delay > time.Hour {
		return time.Hour
	}
	return delay
}
