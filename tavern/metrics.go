package main

import (
	"context"
	"log"
	"time"

	gcppubsub "cloud.google.com/go/pubsub/v2"
	ocprometheus "contrib.go.opencensus.io/exporter/prometheus"
	"github.com/prometheus/client_golang/prometheus"
	"go.opencensus.io/stats/view"
	"google.golang.org/grpc"
)

var (
	metricGRPCRequests = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "tavern_grpc_requests_total",
			Help: "Total number of requests received.",
		},
		[]string{"method"},
	)

	metricGRPCLatency = prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "tavern_grpc_request_duration_seconds",
			Help:    "Latency of requests.",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"method"},
	)

	metricGRPCErrors = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "tavern_grpc_request_errors",
			Help: "Total number of errors.",
		},
		[]string{"method"},
	)
)

func init() {
	// Register metrics with Prometheus
	prometheus.MustRegister(metricGRPCRequests, metricGRPCLatency, metricGRPCErrors)

	// Register OpenCensus views for PubSub metrics
	if err := view.Register(
		&view.View{
			Name:        "pubsub.googleapis.com/stream/open_count",
			Description: "Count of stream opens",
			Measure:     gcppubsub.StreamOpenCount,
			Aggregation: view.Sum(),
		},
		&view.View{
			Name:        "pubsub.googleapis.com/publish/latency",
			Description: "Latency of publish operations",
			Measure:     gcppubsub.PublishLatency,
			// Latency buckets in milliseconds, from 0ms to 10s
			Aggregation: view.Distribution(0, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000),
		},
	); err != nil {
		log.Fatalf("Failed to register OpenCensus views: %v", err)
	}

	// Create and register OpenCensus Prometheus exporter
	pe, err := ocprometheus.NewExporter(ocprometheus.Options{
		Registry: prometheus.DefaultRegisterer.(*prometheus.Registry),
	})
	if err != nil {
		log.Fatalf("Failed to create OpenCensus Prometheus exporter: %v", err)
	}
	view.RegisterExporter(pe)
}

func grpcWithUnaryMetrics(
	ctx context.Context,
	req interface{},
	info *grpc.UnaryServerInfo,
	handler grpc.UnaryHandler,
) (interface{}, error) {
	var (
		start = time.Now()
		h     any
		err   error
	)

	defer func() {
		// Increment total requests counter
		metricGRPCRequests.WithLabelValues(info.FullMethod).Inc()

		// Record the latency
		metricGRPCLatency.WithLabelValues(info.FullMethod).Observe(time.Since(start).Seconds())

		// Record if there was an error
		if err != nil {
			metricGRPCErrors.WithLabelValues(info.FullMethod).Inc()
		}
	}()

	// Call the handler
	h, err = handler(ctx, req)

	return h, err
}

func grpcWithStreamMetrics(
	srv interface{},
	ss grpc.ServerStream,
	info *grpc.StreamServerInfo,
	handler grpc.StreamHandler,
) error {
	var (
		start = time.Now()
		err   error
	)

	defer func() {
		// Increment total requests counter
		metricGRPCRequests.WithLabelValues(info.FullMethod).Inc()

		// Record the latency
		metricGRPCLatency.WithLabelValues(info.FullMethod).Observe(time.Since(start).Seconds())

		// Record if there was an error
		if err != nil {
			metricGRPCErrors.WithLabelValues(info.FullMethod).Inc()
		}
	}()

	// Call the handler
	err = handler(srv, ss)

	return err
}
