package http

import "github.com/prometheus/client_golang/prometheus"

var (
	metricHTTPRequests = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "tavern_http_requests_total",
			Help: "Total number of requests received.",
		},
		[]string{"request_uri", "method"},
	)

	metricHTTPLatency = prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "tavern_http_request_duration_seconds",
			Help:    "Latency of requests.",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"request_uri", "method"},
	)

	metricHTTPErrors = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "tavern_http_request_errors",
			Help: "Total number of errors.",
		},
		[]string{"request_uri", "method"},
	)
)

func init() {
	// Register metrics with Prometheus
	prometheus.MustRegister(metricHTTPRequests, metricHTTPLatency, metricHTTPErrors)
}
