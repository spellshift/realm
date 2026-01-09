package registration

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/namegen"
)

var (
	metricHostCallbacksTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "tavern_host_callbacks_total",
			Help: "The total number of ClaimTasks gRPC calls, provided with host labeling",
		},
		[]string{"host_identifier", "host_groups", "host_services"},
	)
)

func init() {
	prometheus.MustRegister(metricHostCallbacksTotal)
}

type Service struct {
	graph *ent.Client
}

func NewService(graph *ent.Client) *Service {
	return &Service{graph: graph}
}

// RegisterHost ensures the host exists or creates it, updates its LastSeen/NextSeen,
// and returns the Host ID and whether it was newly created.
func (s *Service) RegisterHost(ctx context.Context, req *c2pb.ClaimTasksRequest, clientIP string, now time.Time, interval time.Duration) (int, bool, error) {
	// Check if host exists
	hostExists, err := s.graph.Host.Query().
		Where(host.IdentifierEQ(req.Beacon.Host.Identifier)).
		Exist(ctx)
	if err != nil {
		return 0, false, fmt.Errorf("failed to query host existence: %w", err)
	}
	isNewHost := !hostExists

	// Upsert the host
	hostID, err := s.graph.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(req.Beacon.Host.Platform).
		SetPrimaryIP(req.Beacon.Host.PrimaryIp).
		SetExternalIP(clientIP).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(interval)).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return 0, false, fmt.Errorf("failed to upsert host entity: %w", err)
	}

	// Metrics
	go func() {
		var hostGroupTags []string
		var hostServiceTags []string
		var tagNames []struct {
			Name string
			Kind string
		}
		err := s.graph.Host.Query().
			Where(host.ID(hostID)).
			QueryTags().
			Order(tag.ByKind()).
			Select(tag.FieldName, tag.FieldKind).
			Scan(ctx, &tagNames)
		if err != nil {
			slog.ErrorContext(ctx, "metrics failed to query host tags", "err", err, "host_id", hostID)
		}
		for _, t := range tagNames {
			if t.Kind == string(tag.KindGroup) {
				hostGroupTags = append(hostGroupTags, t.Name)
			}
			if t.Kind == string(tag.KindService) {
				hostServiceTags = append(hostServiceTags, t.Name)
			}
		}
		metricHostCallbacksTotal.
			WithLabelValues(
				req.Beacon.Host.Identifier,
				strings.Join(hostGroupTags, ","),
				strings.Join(hostServiceTags, ","),
			).
			Inc()
	}()

	return hostID, isNewHost, nil
}

// RegisterBeacon ensures the beacon exists or creates it, handling name collision,
// updates its LastSeen/NextSeen, and returns the Beacon ID and whether it was newly created.
func (s *Service) RegisterBeacon(ctx context.Context, req *c2pb.ClaimTasksRequest, hostID int, now time.Time, interval time.Duration, transportType int32) (int, bool, error) {
	beaconExists, err := s.graph.Beacon.Query().
		Where(beacon.IdentifierEQ(req.Beacon.Identifier)).
		Exist(ctx)
	if err != nil {
		return 0, false, fmt.Errorf("failed to query beacon entity: %w", err)
	}
	isNewBeacon := !beaconExists

	var beaconNameAddr *string = nil
	if isNewBeacon {
		candidateNames := []string{
			namegen.NewSimple(),
			namegen.New(),
			namegen.NewComplex(),
		}

		collisions, err := s.graph.Beacon.Query().
			Where(beacon.NameIn(candidateNames...)).
			All(ctx)
		if err != nil {
			return 0, false, fmt.Errorf("failed to query beacon entity: %w", err)
		}
		if len(collisions) == 3 {
			candidateNames = []string{
				namegen.NewSimple(),
				namegen.New(),
				namegen.NewComplex(),
			}

			collisions, err = s.graph.Beacon.Query().
				Where(beacon.NameIn(candidateNames...)).
				All(ctx)
			if err != nil {
				return 0, false, fmt.Errorf("failed to query beacon entity: %w", err)
			}
		}
		for _, candidate := range candidateNames {
			if !namegen.IsCollision(collisions, candidate) {
				name := candidate // copy for address
				beaconNameAddr = &name
				break
			}
		}
	}

	// Upsert the beacon
	beaconID, err := s.graph.Beacon.Create().
		SetPrincipal(req.Beacon.Principal).
		SetIdentifier(req.Beacon.Identifier).
		SetAgentIdentifier(req.Beacon.Agent.Identifier).
		SetNillableName(beaconNameAddr).
		SetHostID(hostID).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(interval)).
		SetInterval(uint64(interval.Seconds())).
		SetTransport(c2pb.Transport_Type(transportType)).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return 0, false, fmt.Errorf("failed to upsert beacon entity: %w", err)
	}

	return beaconID, isNewBeacon, nil
}
