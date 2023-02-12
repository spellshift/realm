package main

import "net/http"

// OKStatusText is the body returned by the status handler when everything is running as expected.
const OKStatusText = "RUNNING"

func newStatusHandler() http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, err := w.Write([]byte(OKStatusText))
		if err != nil {
			panic(err)
		}
	})
}
