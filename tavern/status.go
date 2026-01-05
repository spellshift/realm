package main

import (
	"encoding/base64"
	"encoding/json"
	"net/http"

	"realm.pub/tavern/internal/keyservice"
)

type Status struct {
	OKStatusText string
	Pubkey       string
}

// OKStatusText is the body returned by the status handler when everything is running as expected.
const OKStatusText = "RUNNING"

func newStatusHandler(keySvc *keyservice.KeyService) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		pub := keySvc.GetEd25519PublicKey()
		res, err := json.Marshal(Status{
			OKStatusText: OKStatusText,
			Pubkey:       base64.StdEncoding.EncodeToString(pub),
		})
		if err != nil {
			panic(err)
		}

		_, err = w.Write(res)
		if err != nil {
			panic(err)
		}
	})
}
