package main

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
)

type Status struct {
	OKStatusText string `json:"OKStatusText"`
	Pubkey       string `json:"Pubkey"`
	Version      string `json:"version"`
}

// OKStatusText is the body returned by the status handler when everything is running as expected.
const OKStatusText = "RUNNING"
const NOPUBKEY = "UNAVAILABLE"

func newStatusHandler() http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		pubStr := NOPUBKEY
		pub, err := GetPubKey()
		if err == nil {
			pubStr = base64.StdEncoding.EncodeToString(pub.Bytes())
		}

		res, err := json.Marshal(Status{
			OKStatusText: OKStatusText,
			Pubkey:       pubStr,
			Version:      Version,
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
