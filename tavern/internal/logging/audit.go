package logging

import (
	"context"
	"fmt"
	"log/slog"
	"runtime"
	"strings"
	"time"
)

// AuditOutcome describes the result of an audited action.
type AuditOutcome string

const (
	OutcomeSuccess AuditOutcome = "success"
	OutcomeFailure AuditOutcome = "failure"
	OutcomeDenied  AuditOutcome = "denied"
)

// AuditEntry is the canonical semantic audit record emitted to structured logs
// (and any registered AuditPersister hook).
type AuditEntry struct {
	Timestamp  time.Time
	Subsystem  string
	Action     string
	Outcome    AuditOutcome
	Summary    string
	Actor      Actor
	Entrypoint string
	ClientIP   string
	TraceID    string
	RequestID  string
	TargetType string
	TargetID   string
	TargetName string
	Details    map[string]any
}

// AuditPersister receives semantic audit entries asynchronously or synchronously
// alongside the structured log stream (primarily used by tests and external sinks).
type AuditPersister interface {
	Enqueue(entry AuditEntry)
}

// Audit emits a successful semantic audit event under subsystem "audit.<domain>".
func Audit(ctx context.Context, domain, action, summary string, args ...any) {
	emitAudit(ctx, 2, domain, action, OutcomeSuccess, summary, args...)
}

// AuditSuccess emits a successful semantic audit event under subsystem "audit.<domain>".
func AuditSuccess(ctx context.Context, domain, action, summary string, args ...any) {
	emitAudit(ctx, 2, domain, action, OutcomeSuccess, summary, args...)
}

// AuditFailure emits a failed semantic audit event under subsystem "audit.<domain>".
func AuditFailure(ctx context.Context, domain, action, summary string, args ...any) {
	emitAudit(ctx, 2, domain, action, OutcomeFailure, summary, args...)
}

// AuditDenied emits an authorization/authentication-denied audit event under subsystem "audit.<domain>".
func AuditDenied(ctx context.Context, domain, action, summary string, args ...any) {
	emitAudit(ctx, 2, domain, action, OutcomeDenied, summary, args...)
}

// AuditRecord emits an audit event with an explicit AuditOutcome.
func AuditRecord(ctx context.Context, domain, action string, outcome AuditOutcome, summary string, args ...any) {
	emitAudit(ctx, 2, domain, action, outcome, summary, args...)
}

func emitAudit(ctx context.Context, skip int, domain, action string, outcome AuditOutcome, summary string, args ...any) {
	if ctx == nil {
		ctx = context.Background()
	}

	domain = strings.TrimPrefix(strings.ToLower(strings.TrimSpace(domain)), "audit.")
	subsystem := "audit"
	if domain != "" {
		subsystem = "audit." + domain
	}
	if outcome == "" {
		outcome = OutcomeSuccess
	}
	if summary == "" {
		summary = action
	}

	now := time.Now()
	meta := MetaFromContext(ctx)

	var pcs [1]uintptr
	runtime.Callers(skip+1, pcs[:])

	level := slog.LevelInfo
	if outcome == OutcomeDenied || outcome == OutcomeFailure {
		level = slog.LevelWarn
	}

	rec := slog.NewRecord(now, level, summary, pcs[0])
	rec.AddAttrs(
		slog.String("log_kind", "audit"),
		slog.String("audit_action", action),
		slog.String("audit_outcome", string(outcome)),
	)
	rec.Add(args...)

	details := make(map[string]any)
	var targetType, targetID, targetName string

	extractFromAttr := func(a slog.Attr) {
		a.Value = a.Value.Resolve()
		if a.Equal(slog.Attr{}) || a.Key == "" {
			return
		}
		switch a.Key {
		case "log_kind", "audit_action", "audit_outcome":
			return
		case KeyTargetType:
			targetType = a.Value.String()
			return
		case KeyTargetID:
			targetID = fmt.Sprint(slogValueToJSON(a.Value))
			return
		case KeyTargetName:
			targetName = a.Value.String()
			return
		}
		insertJSONAttr(details, nil, a)
	}

	for _, sa := range meta.StreamAttrs {
		extractFromAttr(sa)
	}
	rec.Attrs(func(a slog.Attr) bool {
		extractFromAttr(a)
		return true
	})

	if targetType == "" {
		targetType, targetID, targetName = inferPrimaryTarget(details)
	}

	st := currentState()
	env := recordEnvelope{
		Record:     rec,
		Subsystem:  subsystem,
		IsAudit:    true,
		InstanceID: st.instanceIDForRecord(),
		GCPProject: st.gcpProjectID,
		Meta:       meta,
	}

	// 1. Write to primary log sink if enabled by current --log filter.
	if rec.Level >= st.resolveLevel(subsystem) && st.primarySink != nil {
		_ = st.primarySink.WriteRecord(env)
	}

	// 2. Always tee to TAVERN_AUDIT_FILE when configured, regardless of console filter.
	if st.auditFileSink != nil {
		_ = st.auditFileSink.WriteRecord(env)
	}

	// 3. Notify registered AuditPersister hook when present.
	if p := currentPersister(); p != nil {
		actor := meta.Actor
		if actor.Kind == "" {
			actor.Kind = ActorAnonymous
		}
		p.Enqueue(AuditEntry{
			Timestamp:  now,
			Subsystem:  subsystem,
			Action:     action,
			Outcome:    outcome,
			Summary:    summary,
			Actor:      actor,
			Entrypoint: meta.Entrypoint,
			ClientIP:   meta.ClientIP,
			TraceID:    meta.TraceID,
			RequestID:  meta.RequestID,
			TargetType: targetType,
			TargetID:   targetID,
			TargetName: targetName,
			Details:    details,
		})
	}
}

type inferredTargetRule struct {
	tType   string
	idKey   string
	nameKey string
}

var targetInferenceOrder = []inferredTargetRule{
	{tType: "quest", idKey: "quest_id", nameKey: "quest_name"},
	{tType: "shell_task", idKey: "shell_task_id", nameKey: ""},
	{tType: "shell", idKey: "shell_id", nameKey: ""},
	{tType: "pivot", idKey: "pivot_id", nameKey: "target"},
	{tType: "portal", idKey: "portal_id", nameKey: ""},
	{tType: "task", idKey: "task_id", nameKey: ""},
	{tType: "scheduled_task", idKey: "scheduled_task_id", nameKey: "scheduled_task_name"},
	{tType: "build_task", idKey: "build_task_id", nameKey: "profile_name"},
	{tType: "build_profile", idKey: "build_profile_id", nameKey: "profile_name"},
	{tType: "builder", idKey: "builder_id", nameKey: "builder_identifier"},
	{tType: "redirector", idKey: "redirector_id", nameKey: "redirector_identifier"},
	{tType: "link", idKey: "link_id", nameKey: "link_path"},
	{tType: "asset", idKey: "asset_id", nameKey: "asset_name"},
	{tType: "tome", idKey: "tome_id", nameKey: "tome_name"},
	{tType: "repository", idKey: "repository_id", nameKey: "repository_url"},
	{tType: "user", idKey: "target_user_id", nameKey: "target_user_name"},
	{tType: "beacon", idKey: "beacon_id", nameKey: "beacon_name"},
	{tType: "host", idKey: "host_id", nameKey: "host_name"},
}

func inferPrimaryTarget(details map[string]any) (string, string, string) {
	for _, rule := range targetInferenceOrder {
		val, ok := details[rule.idKey]
		if !ok || val == nil {
			continue
		}
		idStr := fmt.Sprint(val)
		if idStr == "" || idStr == "0" {
			continue
		}
		var nameStr string
		if rule.nameKey != "" {
			if nv, ok := details[rule.nameKey]; ok && nv != nil {
				nameStr = fmt.Sprint(nv)
			}
		}
		return rule.tType, idStr, nameStr
	}
	return "", "", ""
}
