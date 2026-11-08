package ocr

import (
	"context"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"google.golang.org/api/idtoken"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/screenshot"
)

const Audience = "/internal/ocr-check"

func VerifyLoopback(r *http.Request) error {
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil || !net.ParseIP(host).IsLoopback() {
		return fmt.Errorf("OCR worker request is not from loopback")
	}
	return nil
}

func OIDCVerifier(serviceAccountEmail, audience string) func(*http.Request) error {
	return func(r *http.Request) error {
		auth := strings.Fields(r.Header.Get("Authorization"))
		if len(auth) != 2 || !strings.EqualFold(auth[0], "Bearer") {
			return fmt.Errorf("missing OIDC bearer token")
		}
		claims, err := idtoken.Validate(r.Context(), auth[1], audience)
		if err != nil {
			return fmt.Errorf("invalid OIDC token: %w", err)
		}
		if claims.Issuer != "https://accounts.google.com" && claims.Issuer != "accounts.google.com" {
			return fmt.Errorf("unexpected OIDC issuer")
		}
		if claims.Claims["email"] != serviceAccountEmail || claims.Claims["email_verified"] != true {
			return fmt.Errorf("unexpected OIDC identity")
		}
		return nil
	}
}

func NewHandler(graph *ent.Client, recognizer Recognizer, verify func(*http.Request) error) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		if verify(r) != nil {
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
		Limit(10).
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
			if _, saveErr := graph.Screenshot.Update().Where(screenshot.ID(image.ID), screenshot.OcrRequested(true), screenshot.OcrAttemptsEQ(image.OcrAttempts+1)).SetOcrRequested(false).ClearOcrLeaseUntil().Save(ctx); saveErr != nil {
				slog.ErrorContext(ctx, "failed to mark screenshot OCR as failed", "screenshot_id", image.ID, "err", saveErr)
			}
			return
		}
		if _, saveErr := graph.Screenshot.Update().Where(screenshot.ID(image.ID), screenshot.OcrRequested(true), screenshot.OcrAttemptsEQ(image.OcrAttempts+1)).SetOcrLeaseUntil(time.Now().Add(retryDelay(image.OcrAttempts + 1))).Save(ctx); saveErr != nil {
			slog.ErrorContext(ctx, "failed to schedule screenshot OCR retry", "screenshot_id", image.ID, "err", saveErr)
		}
		return
	}
	if _, err := graph.Screenshot.Update().Where(screenshot.ID(image.ID), screenshot.OcrRequested(true), screenshot.OcrAttemptsEQ(image.OcrAttempts+1)).
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
