package logging

import (
	"context"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

type recordSink interface {
	WriteRecord(env recordEnvelope) error
}

type recordEnvelope struct {
	Record     slog.Record
	Subsystem  string
	IsAudit    bool
	InstanceID string
	GCPProject string
	Meta       ContextMeta
	BoundAttrs []slog.Attr
	Groups     []string
}

type routerState struct {
	spec               Spec
	format             Format
	primarySink        recordSink
	auditFileSink      *jsonSink
	instanceID         string
	includeInstanceID  bool
	logGraphQLRawQuery bool
	gcpProjectID       string
	levelCache         sync.Map // map[string]slog.Level
}

func (s *routerState) resolveLevel(subsystem string) slog.Level {
	if cached, ok := s.levelCache.Load(subsystem); ok {
		return cached.(slog.Level)
	}
	lvl := s.spec.ResolveLevel(subsystem)
	s.levelCache.Store(subsystem, lvl)
	return lvl
}

func (s *routerState) instanceIDForRecord() string {
	if !s.includeInstanceID {
		return ""
	}
	return s.instanceID
}

var (
	activeState     atomic.Pointer[routerState]
	activePersister atomic.Pointer[persisterHolder]
)

type persisterHolder struct {
	p AuditPersister
}

func init() {
	defaultSpec, _ := ParseSpec(string(PresetDev))
	st := &routerState{
		spec:        defaultSpec,
		format:      FormatText,
		primarySink: newTextSink(os.Stderr, detectTTYColor(os.Stderr)),
	}
	activeState.Store(st)
	slog.SetDefault(slog.New(&subsystemHandler{subsystem: "app"}))
}

func currentState() *routerState {
	return activeState.Load()
}

func currentPersister() AuditPersister {
	h := activePersister.Load()
	if h == nil {
		return nil
	}
	return h.p
}

// SetAuditPersister registers (or clears when nil) the async AuditPersister used by Audit(...).
func SetAuditPersister(p AuditPersister) {
	if p == nil {
		activePersister.Store(nil)
		return
	}
	activePersister.Store(&persisterHolder{p: p})
}

// LogGraphQLRawQueryEnabled reports whether GraphQL operation logs should include raw_query.
func LogGraphQLRawQueryEnabled() bool {
	return currentState().logGraphQLRawQuery
}

// ActiveSpec returns a copy-safe view of the currently configured filter Spec.
func ActiveSpec() Spec {
	return currentState().spec
}

// Options configures Tavern's global subsystem router and output sinks.
type Options struct {
	// Spec is the `--log` / `TAVERN_LOG` preset & subsystem filter string (e.g. "prod", "dev", "portals=debug").
	Spec string

	// Format overrides the preset's default output format ("text" or "json") when non-empty.
	Format Format

	// Output overrides the destination writer (defaults to os.Stderr).
	Output io.Writer

	// AuditFilePath tees all semantic `audit.*` records as JSONL to the given file path when non-empty.
	AuditFilePath string

	// InstanceID is the unique Tavern server instance identifier (`tavern_id`).
	InstanceID string

	// IncludeInstanceID explicitly overrides whether `tavern_id` is emitted on records.
	IncludeInstanceID *bool

	// LogGraphQLRawQuery explicitly overrides whether raw GraphQL query strings are logged.
	LogGraphQLRawQuery *bool

	// GCPProjectID formats `logging.googleapis.com/trace` as `projects/<id>/traces/<trace_id>` when set.
	GCPProjectID string

	// Color explicitly enables or disables ANSI colors in Text mode (auto-detected on TTY when nil).
	Color *bool

	// Persister optionally registers a background AuditPersister at configuration time.
	Persister AuditPersister
}

// Configure parses opts, atomically updates all `logging.For(...)` loggers and `slog.Default()`,
// and closes any previous dedicated audit file sink.
func Configure(opts Options) error {
	spec, err := ParseSpec(opts.Spec)
	if err != nil {
		return err
	}

	format := opts.Format
	if format == FormatAuto {
		format = spec.DefaultFormat
	}
	if format == FormatAuto {
		format = FormatText
	}

	out := opts.Output
	if out == nil {
		out = os.Stderr
	}

	includeInstanceID := spec.IncludeInstanceID
	if opts.IncludeInstanceID != nil {
		includeInstanceID = *opts.IncludeInstanceID
	}

	logRawQuery := spec.LogGraphQLRawQuery
	if opts.LogGraphQLRawQuery != nil {
		logRawQuery = *opts.LogGraphQLRawQuery
	}

	var primary recordSink
	if format == FormatJSON {
		primary = newJSONSink(out, nil)
	} else {
		useColor := detectTTYColor(out)
		if opts.Color != nil {
			useColor = *opts.Color
		}
		primary = newTextSink(out, useColor)
	}

	var auditSink *jsonSink
	if auditPath := strings.TrimSpace(opts.AuditFilePath); auditPath != "" {
		if dir := filepath.Dir(auditPath); dir != "" && dir != "." {
			if err := os.MkdirAll(dir, 0o750); err != nil {
				return err
			}
		}
		f, err := os.OpenFile(auditPath, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0o600)
		if err != nil {
			return err
		}
		auditSink = newJSONSink(f, f)
	}

	next := &routerState{
		spec:               spec,
		format:             format,
		primarySink:        primary,
		auditFileSink:      auditSink,
		instanceID:         opts.InstanceID,
		includeInstanceID:  includeInstanceID,
		logGraphQLRawQuery: logRawQuery,
		gcpProjectID:       strings.TrimSpace(opts.GCPProjectID),
	}

	prev := activeState.Swap(next)
	if prev != nil && prev.auditFileSink != nil {
		_ = prev.auditFileSink.Close()
	}

	if opts.Persister != nil {
		SetAuditPersister(opts.Persister)
	}

	slog.SetDefault(slog.New(&subsystemHandler{subsystem: "app"}))
	return nil
}

func detectTTYColor(w io.Writer) bool {
	if os.Getenv("NO_COLOR") != "" {
		return false
	}
	f, ok := w.(*os.File)
	if !ok {
		return false
	}
	info, err := f.Stat()
	if err != nil {
		return false
	}
	return (info.Mode() & os.ModeCharDevice) != 0
}

// Logger is a subsystem-scoped structured logger compatible with *slog.Logger,
// adding ergonomic Trace / TraceContext helpers for wire-level diagnostics.
type Logger struct {
	*slog.Logger
	subsystem string
}

// For returns a dynamically filtered, O(1)-checked structured logger bound to a dotted subsystem name
// (e.g. `logging.For("portals.ssh")`). Safe to declare as a package-level variable (`var log = logging.For(...)`).
func For(subsystem string) *Logger {
	sub := strings.ToLower(strings.TrimSpace(subsystem))
	if sub == "" {
		sub = "app"
	}
	registerSubsystemRoot(sub)
	h := &subsystemHandler{
		subsystem: sub,
		isAudit:   sub == "audit" || strings.HasPrefix(sub, "audit."),
	}
	return &Logger{
		Logger:    slog.New(h),
		subsystem: sub,
	}
}

// Subsystem returns the dotted subsystem name bound to this Logger.
func (l *Logger) Subsystem() string {
	return l.subsystem
}

// Slog returns the underlying *slog.Logger.
func (l *Logger) Slog() *slog.Logger {
	return l.Logger
}

// With returns a child Logger with additional bound attributes while preserving Subsystem and Trace methods.
func (l *Logger) With(args ...any) *Logger {
	if len(args) == 0 {
		return l
	}
	return &Logger{
		Logger:    l.Logger.With(args...),
		subsystem: l.subsystem,
	}
}

// Trace emits a LevelTrace record without an explicit context.
func (l *Logger) Trace(msg string, args ...any) {
	l.emitAtLevel(context.Background(), LevelTrace, msg, args...)
}

// TraceContext emits a LevelTrace record carrying ctx correlation metadata.
func (l *Logger) TraceContext(ctx context.Context, msg string, args ...any) {
	l.emitAtLevel(ctx, LevelTrace, msg, args...)
}

func (l *Logger) emitAtLevel(ctx context.Context, level slog.Level, msg string, args ...any) {
	if ctx == nil {
		ctx = context.Background()
	}
	if !l.Logger.Enabled(ctx, level) {
		return
	}
	var pcs [1]uintptr
	runtime.Callers(3, pcs[:])
	rec := slog.NewRecord(time.Now(), level, msg, pcs[0])
	rec.Add(args...)
	_ = l.Logger.Handler().Handle(ctx, rec)
}

type subsystemHandler struct {
	subsystem string
	isAudit   bool
	attrs     []slog.Attr
	groups    []string
}

func (h *subsystemHandler) Enabled(_ context.Context, level slog.Level) bool {
	st := currentState()
	if h.isAudit && st.auditFileSink != nil {
		return true
	}
	return level >= st.resolveLevel(h.subsystem)
}

func (h *subsystemHandler) Handle(ctx context.Context, r slog.Record) error {
	st := currentState()
	meta := MetaFromContext(ctx)

	isAudit := h.isAudit
	if !isAudit {
		r.Attrs(func(a slog.Attr) bool {
			if a.Key == "log_kind" && a.Value.String() == "audit" {
				isAudit = true
				return false
			}
			return true
		})
	}

	env := recordEnvelope{
		Record:     r,
		Subsystem:  h.subsystem,
		IsAudit:    isAudit,
		InstanceID: st.instanceIDForRecord(),
		GCPProject: st.gcpProjectID,
		Meta:       meta,
		BoundAttrs: h.attrs,
		Groups:     h.groups,
	}

	var firstErr error
	if r.Level >= st.resolveLevel(h.subsystem) && st.primarySink != nil {
		if err := st.primarySink.WriteRecord(env); err != nil {
			firstErr = err
		}
	}
	if isAudit && st.auditFileSink != nil {
		if err := st.auditFileSink.WriteRecord(env); err != nil && firstErr == nil {
			firstErr = err
		}
	}
	return firstErr
}

func (h *subsystemHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	if len(attrs) == 0 {
		return h
	}
	combined := make([]slog.Attr, 0, len(h.attrs)+len(attrs))
	combined = append(combined, h.attrs...)
	combined = append(combined, attrs...)
	return &subsystemHandler{
		subsystem: h.subsystem,
		isAudit:   h.isAudit,
		attrs:     combined,
		groups:    h.groups,
	}
}

func (h *subsystemHandler) WithGroup(name string) slog.Handler {
	if name == "" {
		return h
	}
	combined := make([]string, 0, len(h.groups)+1)
	combined = append(combined, h.groups...)
	combined = append(combined, name)
	return &subsystemHandler{
		subsystem: h.subsystem,
		isAudit:   h.isAudit,
		attrs:     h.attrs,
		groups:    combined,
	}
}
