package automation

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/robfig/cron/v3"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tome"
)

var (
	MetricErrors = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "tavern_tome_automation_errors_total",
			Help: "The total number of errors encountered during tome automation",
		},
	)
)

func init() {
	prometheus.MustRegister(MetricErrors)
}

type Service struct {
	graph *ent.Client
}

func NewService(graph *ent.Client) *Service {
	return &Service{graph: graph}
}

func (s *Service) RunTomeAutomation(ctx context.Context, beaconID int, hostID int, isNewBeacon bool, isNewHost bool, now time.Time, interval time.Duration) {
	// Tome Automation Logic
	candidateTomes, err := s.graph.Tome.Query().
		Where(tome.Or(
			tome.RunOnNewBeaconCallback(true),
			tome.RunOnFirstHostCallback(true),
			tome.RunOnScheduleNEQ(""),
		)).
		All(ctx)

	if err != nil {
		slog.ErrorContext(ctx, "failed to query candidate tomes for automation", "err", err)
		MetricErrors.Inc()
		return
	}

	selectedTomes := make(map[int]*ent.Tome)
	parser := cron.NewParser(cron.Minute | cron.Hour | cron.Dom | cron.Month | cron.Dow)
	cutoff := now.Add(interval)

	for _, t := range candidateTomes {
		shouldRun := false

		// Check RunOnNewBeaconCallback
		if isNewBeacon && t.RunOnNewBeaconCallback {
			shouldRun = true
		}

		// Check RunOnFirstHostCallback
		if !shouldRun && isNewHost && t.RunOnFirstHostCallback {
			shouldRun = true
		}

		// Check RunOnSchedule
		if !shouldRun && t.RunOnSchedule != "" {
			sched, err := parser.Parse(t.RunOnSchedule)
			if err == nil {
				isMatch := false
				// If schedule contains a range (hyphen), checking for strict current match
				// without factoring in callback interval.
				if strings.Contains(t.RunOnSchedule, "-") {
					currentMinute := now.Truncate(time.Minute)
					// Verify if 'currentMinute' is a valid trigger time.
					// We check if the next trigger after (currentMinute - 1s) is exactly currentMinute.
					next := sched.Next(currentMinute.Add(-1 * time.Second))
					if next.Equal(currentMinute) {
						isMatch = true
					}
				} else {
					// Check if any point between now and the next expected check-in matches the run schedule
					// Next(now-1sec) <= now + interval?
					next := sched.Next(now.Add(-1 * time.Second))
					if !next.After(cutoff) {
						isMatch = true
					}
				}

				if isMatch {
					// Check scheduled_hosts constraint
					hostCount, err := t.QueryScheduledHosts().Count(ctx)
					if err != nil {
						slog.ErrorContext(ctx, "failed to count scheduled hosts for automation", "err", err, "tome_id", t.ID)
						MetricErrors.Inc()
						continue
					}
					if hostCount == 0 {
						shouldRun = true
					} else {
						hostExists, err := t.QueryScheduledHosts().
							Where(host.ID(hostID)).
							Exist(ctx)
						if err != nil {
							slog.ErrorContext(ctx, "failed to check host existence for automation", "err", err, "tome_id", t.ID)
							MetricErrors.Inc()
							continue
						}
						if hostExists {
							shouldRun = true
						}
					}
				}
			} else {
				// Don't log cron parse errors for now, as it might be spammy if stored in DB
				// MetricErrors.Inc()
			}
		}

		if shouldRun {
			selectedTomes[t.ID] = t
		}
	}

	// Create Quest and Task for each selected Tome
	for _, t := range selectedTomes {
		q, err := s.graph.Quest.Create().
			SetName(fmt.Sprintf("Automated: %s", t.Name)).
			SetTome(t).
			SetParamDefsAtCreation(t.ParamDefs).
			SetEldritchAtCreation(t.Eldritch).
			Save(ctx)
		if err != nil {
			slog.ErrorContext(ctx, "failed to create automated quest", "err", err, "tome_id", t.ID)
			MetricErrors.Inc()
			continue
		}

		_, err = s.graph.Task.Create().
			SetQuest(q).
			SetBeaconID(beaconID).
			Save(ctx)
		if err != nil {
			slog.ErrorContext(ctx, "failed to create automated task", "err", err, "quest_id", q.ID)
			MetricErrors.Inc()
		}
	}
}
