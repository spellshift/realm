package user

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestUser(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var reqBody struct {
			Query string `json:"query"`
		}
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		var respData interface{}
		switch {
		case reqBody.Query == `query { hosts { edges { node { id beacons { id } } } } }`:
			respData = map[string]interface{}{
				"data": map[string]interface{}{
					"hosts": map[string]interface{}{
						"edges": []map[string]interface{}{
							{
								"node": map[string]interface{}{
									"id": "host-1",
									"beacons": []map[string]interface{}{
										{"id": "beacon-1"},
									},
								},
							},
						},
					},
				},
			}
		case reqBody.Query == `query { tomes { edges { node { id } } } }`:
			respData = map[string]interface{}{
				"data": map[string]interface{}{
					"tomes": map[string]interface{}{
						"edges": []map[string]interface{}{
							{
								"node": map[string]interface{}{
									"id": "tome-1",
								},
							},
						},
					},
				},
			}
		case reqBody.Query == `mutation { createQuest(input: { tomeID: "tome-1", beaconIDs: ["beacon-1"], parameters: "{}" }) { id } }`:
			respData = map[string]interface{}{
				"data": map[string]interface{}{
					"createQuest": map[string]interface{}{
						"id": "quest-1",
					},
				},
			}
		}
		json.NewEncoder(w).Encode(respData)
	}))
	defer server.Close()

	user, err := New(server.URL, 10*time.Millisecond, map[string]int{"createQuest": 1})
	require.NoError(t, err)

	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	err = user.Run(ctx)
	require.NoError(t, err)
}
