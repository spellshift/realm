package cdn_test

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strconv"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/cdn"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/logging"
)

type cdnAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *cdnAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *cdnAuditRecorder) byAction(action string) []logging.AuditEntry {
	r.mu.Lock()
	defer r.mu.Unlock()
	var matched []logging.AuditEntry
	for _, entry := range r.entries {
		if entry.Action == action {
			matched = append(matched, entry)
		}
	}
	return matched
}

func detailsMap(t *testing.T, raw map[string]any) map[string]any {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return out
}

func TestCDNAuditLogging(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	rec := &cdnAuditRecorder{}
	logging.SetAuditPersister(rec)
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
	})

	t.Run("AssetUploadNewAndOverwritten", func(t *testing.T) {
		uploadHandler := cdn.NewUploadHandler(graph)

		// 1. Upload brand new asset
		req1 := newUploadRequest(t, "StagePayload.bin", []byte("initial_payload_bytes"))
		req1 = req1.WithContext(logging.WithActor(req1.Context(), logging.Actor{
			Kind: logging.ActorUser,
			ID:   "42",
			Name: "redteam-op",
		}))
		w1 := httptest.NewRecorder()
		uploadHandler.ServeHTTP(w1, req1)
		require.Equal(t, http.StatusOK, w1.Result().StatusCode)

		// 2. Re-upload same asset name (overwrite)
		req2 := newUploadRequest(t, "StagePayload.bin", []byte("updated_payload_bytes_v2"))
		w2 := httptest.NewRecorder()
		uploadHandler.ServeHTTP(w2, req2)
		require.Equal(t, http.StatusOK, w2.Result().StatusCode)

		entries := rec.byAction("cdn.asset_uploaded")
		require.Len(t, entries, 2)

		assert.Equal(t, "audit.cdn", entries[0].Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, entries[0].Outcome)
		assert.Equal(t, logging.ActorUser, entries[0].Actor.Kind)
		assert.Equal(t, "42", entries[0].Actor.ID)
		assert.Equal(t, "asset", entries[0].TargetType)
		assert.Equal(t, "StagePayload.bin", entries[0].TargetName)

		details1 := detailsMap(t, entries[0].Details)
		details2 := detailsMap(t, entries[1].Details)
		assert.Equal(t, false, details1["overwritten"])
		assert.Equal(t, float64(len("initial_payload_bytes")), details1["asset_size"])
		assert.Equal(t, true, details2["overwritten"])
		assert.Equal(t, float64(len("updated_payload_bytes_v2")), details2["asset_size"])
	})

	t.Run("LinkDownloadedAndDenied", func(t *testing.T) {
		payloadAsset := newAsset(graph, "ImixBeacon.exe", []byte("MZ_PAYLOAD"))
		limit := 1
		oneShotLink := graph.Link.Create().
			SetAsset(payloadAsset).
			SetExpiresAt(time.Now().Add(1 * time.Hour)).
			SetDownloadLimit(limit).
			SaveX(ctx)

		expiredLink := graph.Link.Create().
			SetAsset(payloadAsset).
			SetExpiresAt(time.Now().Add(-5 * time.Minute)).
			SaveX(ctx)

		linkHandler := cdn.NewLinkDownloadHandler(graph, "/cdn/")

		// 1. Valid one-shot link download -> cdn.link_downloaded (downloads_remaining == 0)
		validReq := httptest.NewRequest(http.MethodGet, "/cdn/"+oneShotLink.Path, nil)
		validReq.RemoteAddr = "203.0.113.55:49152"
		validRec := httptest.NewRecorder()
		linkHandler.ServeHTTP(validRec, validReq)
		require.Equal(t, http.StatusOK, validRec.Result().StatusCode)

		// 2. Second attempt on exhausted link -> cdn.link_denied (limit_reached)
		exhaustedRec := httptest.NewRecorder()
		linkHandler.ServeHTTP(exhaustedRec, httptest.NewRequest(http.MethodGet, "/cdn/"+oneShotLink.Path, nil))
		require.Equal(t, http.StatusNotFound, exhaustedRec.Result().StatusCode)

		// 3. Expired link -> cdn.link_denied (expired)
		expiredRec := httptest.NewRecorder()
		linkHandler.ServeHTTP(expiredRec, httptest.NewRequest(http.MethodGet, "/cdn/"+expiredLink.Path, nil))
		require.Equal(t, http.StatusNotFound, expiredRec.Result().StatusCode)

		// 4. Nonexistent path -> cdn.link_denied (not_found)
		missingRec := httptest.NewRecorder()
		linkHandler.ServeHTTP(missingRec, httptest.NewRequest(http.MethodGet, "/cdn/unknown-stage-path", nil))
		require.Equal(t, http.StatusNotFound, missingRec.Result().StatusCode)

		// 5. Empty path -> cdn.link_denied (invalid_path)
		emptyRec := httptest.NewRecorder()
		linkHandler.ServeHTTP(emptyRec, httptest.NewRequest(http.MethodGet, "/cdn/", nil))
		require.Equal(t, http.StatusNotFound, emptyRec.Result().StatusCode)

		dlEntries := rec.byAction("cdn.link_downloaded")
		require.Len(t, dlEntries, 1)
		dlEntry := dlEntries[0]
		assert.Equal(t, "audit.cdn", dlEntry.Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, dlEntry.Outcome)
		assert.Equal(t, "203.0.113.55", dlEntry.ClientIP)
		assert.Equal(t, "link", dlEntry.TargetType)
		assert.Equal(t, strconv.Itoa(oneShotLink.ID), dlEntry.TargetID)
		assert.Equal(t, oneShotLink.Path, dlEntry.TargetName)

		dlDetails := detailsMap(t, dlEntry.Details)
		assert.Equal(t, float64(oneShotLink.ID), dlDetails["link_id"])
		assert.Equal(t, oneShotLink.Path, dlDetails["link_path"])
		assert.Equal(t, float64(payloadAsset.ID), dlDetails["asset_id"])
		assert.Equal(t, payloadAsset.Name, dlDetails["asset_name"])
		assert.Equal(t, float64(0), dlDetails["downloads_remaining"])

		deniedEntries := rec.byAction("cdn.link_denied")
		require.Len(t, deniedEntries, 4)

		expectedReasons := []string{"limit_reached", "expired", "not_found", "invalid_path"}
		for i, entry := range deniedEntries {
			assert.Equal(t, "audit.cdn", entry.Subsystem)
			assert.Equal(t, logging.OutcomeDenied, entry.Outcome)
			d := detailsMap(t, entry.Details)
			assert.Equal(t, expectedReasons[i], d["reason"])
		}
	})

	t.Run("AssetHostFileAndScreenshotDownloads", func(t *testing.T) {
		assetObj := newAsset(graph, "DirectDownloadAsset.bin", []byte("direct_bytes"))
		dlHandler := cdn.NewDownloadHandler(graph, "/cdn/assets/")
		dlRec := httptest.NewRecorder()
		dlHandler.ServeHTTP(dlRec, httptest.NewRequest(http.MethodGet, "/cdn/assets/"+assetObj.Name, nil))
		require.Equal(t, http.StatusOK, dlRec.Result().StatusCode)

		// Host + Task + HostFile + Screenshot
		h := graph.Host.Create().
			SetIdentifier("loot-host-1").
			SetPlatform(c2pb.Host_PLATFORM_LINUX).
			SaveX(ctx)
		b := graph.Beacon.Create().
			SetHost(h).
			SetIdentifier("loot-beacon-1").
			SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
			SaveX(ctx)
		tm := graph.Tome.Create().
			SetName("LootTome").
			SetDescription("desc").
			SetAuthor("op").
			SetEldritch("print(1)").
			SaveX(ctx)
		q := graph.Quest.Create().SetName("LootQuest").SetTome(tm).SaveX(ctx)
		tsk := graph.Task.Create().SetQuest(q).SetBeacon(b).SaveX(ctx)
		hf := graph.HostFile.Create().
			SetHost(h).
			SetTask(tsk).
			SetPath("/etc/shadow").
			SetContent([]byte("root:*:19000:0:99999:7:::")).
			SaveX(ctx)
		sc := graph.Screenshot.Create().
			SetHost(h).
			SetName("desktop.png").
			SetContent([]byte("png_bytes")).
			SaveX(ctx)

		hfHandler := cdn.NewHostFileDownloadHandler(graph, "/cdn/hostfiles/")
		hfRec := httptest.NewRecorder()
		hfHandler.ServeHTTP(hfRec, httptest.NewRequest(http.MethodGet, fmt.Sprintf("/cdn/hostfiles/%d", hf.ID), nil))
		require.Equal(t, http.StatusOK, hfRec.Result().StatusCode)

		scSingle := cdn.NewScreenshotDownloadHandler(graph, "/cdn/screenshots/")
		scSingleRec := httptest.NewRecorder()
		scSingle.ServeHTTP(scSingleRec, httptest.NewRequest(http.MethodGet, fmt.Sprintf("/cdn/screenshots/%d", sc.ID), nil))
		require.Equal(t, http.StatusOK, scSingleRec.Result().StatusCode)

		scThumb := cdn.NewScreenshotThumbnailDownloadHandler(graph, "/cdn/thumbnails/")
		scThumbRec := httptest.NewRecorder()
		scThumb.ServeHTTP(scThumbRec, httptest.NewRequest(http.MethodGet, fmt.Sprintf("/cdn/thumbnails/%d", sc.ID), nil))
		require.Equal(t, http.StatusOK, scThumbRec.Result().StatusCode)

		scAll := cdn.NewHostScreenshotsDownloadHandler(graph, "/cdn/host-screenshots/")
		scAllRec := httptest.NewRecorder()
		scAll.ServeHTTP(scAllRec, httptest.NewRequest(http.MethodGet, fmt.Sprintf("/cdn/host-screenshots/%d", h.ID), nil))
		require.Equal(t, http.StatusOK, scAllRec.Result().StatusCode)

		assetDlEntries := rec.byAction("cdn.asset_downloaded")
		require.Len(t, assetDlEntries, 1)
		assert.Equal(t, strconv.Itoa(assetObj.ID), assetDlEntries[0].TargetID)

		hfEntries := rec.byAction("cdn.hostfile_downloaded")
		require.Len(t, hfEntries, 1)
		hfDetails := detailsMap(t, hfEntries[0].Details)
		assert.Equal(t, float64(hf.ID), hfDetails["host_file_id"])
		assert.Equal(t, float64(h.ID), hfDetails["host_id"])
		assert.Equal(t, "/etc/shadow", hfDetails["path"])

		scEntries := rec.byAction("cdn.screenshot_downloaded")
		require.Len(t, scEntries, 3)

		wantModes := []string{"single", "thumbnail", "all"}
		for i, entry := range scEntries {
			d := detailsMap(t, entry.Details)
			assert.Equal(t, wantModes[i], d["mode"])
		}
	})
}
