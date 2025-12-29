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
	// Register metric if not already registered (in case this package is imported multiple times or tests)
	// In a real scenario, metrics definition should probably be centralized, but here we keep it with the service.
	// Since init() runs once per package load, this is safe.
	// However, if the main c2 package also registers it, we might have a conflict.
	// We will rely on the main c2 package handling registration or moving it here.
	// *Correction*: The user asked to decouple. If I move logic here, I should move metrics too.
	// But `prometheus.MustRegister` panics on duplicate registration.
	// For now, I will assume I can register it here. If `c2` package has it, I'll need to remove it from there.
	// Let's actually define the metric here and expose it or just use it.
}

// Service handles beacon and host registration.
type Service struct {
	client *ent.Client
}

// New creates a new Registration Service.
func New(client *ent.Client) *Service {
	// Check if we can/should register the metric here.
	// Since we are refactoring, we'll assume the metric moves here.
	// To avoid panic if already registered by the c2 package during the transition,
	// we can try to register and ignore error, or just assume we will delete it from c2.
	// I will attempt to register it in `init` but since I cannot easily coordinate with `c2` package's `init`,
	// I will just use the variable. The `init` block above will be empty or handle registration.
	// Let's actually define the metric here and expose it or just use it.
	return &Service{client: client}
}

// UpsertBeacon performs the logic to upsert the host and beacon, and record metrics.
func (s *Service) UpsertBeacon(ctx context.Context, req *c2pb.ClaimTasksRequest, clientIP string) (int, int, error) {
	now := time.Now()

	// Upsert the host
	hostID, err := s.client.Host.Create().
		SetIdentifier(req.Beacon.Host.Identifier).
		SetName(req.Beacon.Host.Name).
		SetPlatform(req.Beacon.Host.Platform).
		SetPrimaryIP(req.Beacon.Host.PrimaryIp).
		SetExternalIP(clientIP).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(time.Duration(req.Beacon.Interval) * time.Second)).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return 0, 0, fmt.Errorf("failed to upsert host entity: %w", err)
	}

	// Metrics
	// We run this asynchronously or defer it? The original code deferred it.
	// We can't defer in the caller easily if we want to encapsulate the logic.
	// We'll run it immediately here, but in a separate goroutine if we want to mimic "defer" behavior that doesn't block return?
	// Actually, the original defer ran *after* the function body but before return to caller? No, defer runs when function exits.
	// So it blocks the response. We can just run it.
	s.recordMetrics(ctx, hostID, req.Beacon.Host.Identifier)

	// Generate name for new beacons
	beaconExists, err := s.client.Beacon.Query().
		Where(beacon.IdentifierEQ(req.Beacon.Identifier)).
		Exist(ctx)
	if err != nil {
		return 0, 0, fmt.Errorf("failed to query beacon entity: %w", err)
	}

	var beaconNameAddr *string = nil
	if !beaconExists {
		beaconNameAddr, err = s.generateBeaconName(ctx)
		if err != nil {
			return 0, 0, err
		}
	}

	// Upsert the beacon
	beaconID, err := s.client.Beacon.Create().
		SetPrincipal(req.Beacon.Principal).
		SetIdentifier(req.Beacon.Identifier).
		SetAgentIdentifier(req.Beacon.Agent.Identifier).
		SetNillableName(beaconNameAddr).
		SetHostID(hostID).
		SetLastSeenAt(now).
		SetNextSeenAt(now.Add(time.Duration(req.Beacon.Interval) * time.Second)).
		SetInterval(req.Beacon.Interval).
		SetTransport(req.Beacon.Transport).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	if err != nil {
		return 0, 0, fmt.Errorf("failed to upsert beacon entity: %w", err)
	}

	return beaconID, hostID, nil
}

func (s *Service) generateBeaconName(ctx context.Context) (*string, error) {
	candidateNames := []string{
		namegen.NewSimple(),
		namegen.New(),
		namegen.NewComplex(),
	}

	collisions, err := s.client.Beacon.Query().
		Where(beacon.NameIn(candidateNames...)).
		All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query beacon entity: %w", err)
	}

	// Retry logic if all 3 collide
	if len(collisions) == 3 {
		candidateNames = []string{
			namegen.NewSimple(),
			namegen.New(),
			namegen.NewComplex(),
		}
		collisions, err = s.client.Beacon.Query().
			Where(beacon.NameIn(candidateNames...)).
			All(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to query beacon entity: %w", err)
		}
	}

	for _, candidate := range candidateNames {
		if !namegen.IsCollision(collisions, candidate) {
			return &candidate, nil
		}
	}
	// If all fail, return nil (which means unnamed/null in DB? Or should we error? Original code seemingly left it nil if loop finished)
	// Original code: var beaconNameAddr *string = nil ... if !beaconExists { ... loop ... beaconNameAddr = &canidate ... }
	// If loop finishes without match, it stays nil.
	return nil, nil
}

func (s *Service) recordMetrics(ctx context.Context, hostID int, hostIdentifier string) {
	var hostGroupTags []string
	var hostServiceTags []string
	var tagNames []struct {
		Name string
		Kind string
	}
	err := s.client.Host.Query().
		Where(host.ID(hostID)).
		QueryTags().
		Order(tag.ByKind()).
		Select(tag.FieldName, tag.FieldKind).
		Scan(ctx, &tagNames)
	if err != nil {
		slog.ErrorContext(ctx, "metrics failed to query host tags", "err", err, "host_id", hostID)
		// We don't fail the request for metrics
		return
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
			hostIdentifier,
			strings.Join(hostGroupTags, ","),
			strings.Join(hostServiceTags, ","),
		).
		Inc()
}

// RegisterMetrics allows the caller to register the metrics, or we can export the metric collector.
func RegisterMetrics(r prometheus.Registerer) {
	r.Register(metricHostCallbacksTotal)
}
