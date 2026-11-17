package logging

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"runtime"
	"sync"
	"time"
)

var jsonBufPool = sync.Pool{
	New: func() any {
		return bytes.NewBuffer(make([]byte, 0, 512))
	},
}

type gcpSourceLocation struct {
	File     string `json:"file,omitempty"`
	Line     int    `json:"line,omitempty"`
	Function string `json:"function,omitempty"`
}

type jsonSink struct {
	mu     sync.Mutex
	out    io.Writer
	closer io.Closer
}

func newJSONSink(out io.Writer, closer io.Closer) *jsonSink {
	return &jsonSink{out: out, closer: closer}
}

func (s *jsonSink) Close() error {
	if s == nil || s.closer == nil {
		return nil
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.closer.Close()
}

func (s *jsonSink) WriteRecord(env recordEnvelope) error {
	payload := make(map[string]any, 20)

	ts := env.Record.Time
	if ts.IsZero() {
		ts = time.Now()
	}
	payload["time"] = ts.UTC().Format(time.RFC3339Nano)
	payload["severity"] = GCPSeverity(env.Record.Level, env.IsAudit)
	payload["message"] = env.Record.Message

	sub := env.Subsystem
	if sub == "" {
		sub = "app"
	}
	payload["subsystem"] = sub
	if env.IsAudit {
		payload["log_kind"] = "audit"
	}
	if env.InstanceID != "" {
		payload["tavern_id"] = env.InstanceID
	}

	// Context correlation & attribution defaults (explicit record attrs can refine if needed).
	if env.Meta.TraceID != "" {
		payload["trace_id"] = env.Meta.TraceID
		if env.GCPProject != "" {
			payload["logging.googleapis.com/trace"] = fmt.Sprintf("projects/%s/traces/%s", env.GCPProject, env.Meta.TraceID)
		} else {
			payload["logging.googleapis.com/trace"] = env.Meta.TraceID
		}
	}
	if env.Meta.RequestID != "" {
		payload["request_id"] = env.Meta.RequestID
		payload["logging.googleapis.com/spanId"] = env.Meta.RequestID
	}
	if env.Meta.Entrypoint != "" {
		payload["entrypoint"] = env.Meta.Entrypoint
	}
	if env.Meta.ClientIP != "" {
		payload["client_ip"] = env.Meta.ClientIP
	}
	if env.Meta.Actor.Kind != "" {
		payload["actor_kind"] = string(env.Meta.Actor.Kind)
	}
	if env.Meta.Actor.ID != "" {
		payload["actor_id"] = env.Meta.Actor.ID
	}
	if env.Meta.Actor.Name != "" {
		payload["actor_name"] = env.Meta.Actor.Name
	}
	if env.Meta.Actor.Kind == ActorUser {
		payload["actor_is_admin"] = env.Meta.Actor.IsAdmin
	}
	if env.Meta.Actor.AuthMethod != "" {
		payload["actor_auth_method"] = env.Meta.Actor.AuthMethod
	}
	for _, sa := range env.Meta.StreamAttrs {
		insertJSONAttr(payload, nil, sa)
	}

	// Source location on warnings/errors and debug/trace diagnostics.
	if env.Record.PC != 0 && (env.Record.Level >= slog.LevelWarn || env.Record.Level <= slog.LevelDebug) {
		frames := runtime.CallersFrames([]uintptr{env.Record.PC})
		f, _ := frames.Next()
		if f.File != "" {
			payload["logging.googleapis.com/sourceLocation"] = gcpSourceLocation{
				File:     f.File,
				Line:     f.Line,
				Function: f.Function,
			}
		}
	}

	for _, a := range env.BoundAttrs {
		insertJSONAttr(payload, env.Groups, a)
	}
	env.Record.Attrs(func(a slog.Attr) bool {
		insertJSONAttr(payload, env.Groups, a)
		return true
	})

	buf := jsonBufPool.Get().(*bytes.Buffer)
	buf.Reset()
	defer jsonBufPool.Put(buf)

	enc := json.NewEncoder(buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(payload); err != nil {
		return err
	}

	s.mu.Lock()
	_, err := s.out.Write(buf.Bytes())
	s.mu.Unlock()
	return err
}

func insertJSONAttr(dst map[string]any, groups []string, a slog.Attr) {
	a.Value = a.Value.Resolve()
	if a.Equal(slog.Attr{}) {
		return
	}

	target := dst
	for _, g := range groups {
		if g == "" {
			continue
		}
		sub, ok := target[g].(map[string]any)
		if !ok {
			sub = make(map[string]any)
			target[g] = sub
		}
		target = sub
	}

	if a.Value.Kind() == slog.KindGroup {
		groupAttrs := a.Value.Group()
		if len(groupAttrs) == 0 {
			return
		}
		subTarget := target
		if a.Key != "" {
			existing, ok := target[a.Key].(map[string]any)
			if !ok {
				existing = make(map[string]any, len(groupAttrs))
				target[a.Key] = existing
			}
			subTarget = existing
		}
		for _, ga := range groupAttrs {
			insertJSONAttr(subTarget, nil, ga)
		}
		return
	}

	if a.Key == "" {
		return
	}
	if IsSensitiveKey(a.Key) {
		target[a.Key] = RedactedPlaceholder
		return
	}
	target[a.Key] = slogValueToJSON(a.Value)
}

func slogValueToJSON(v slog.Value) any {
	switch v.Kind() {
	case slog.KindString:
		return v.String()
	case slog.KindInt64:
		return v.Int64()
	case slog.KindUint64:
		return v.Uint64()
	case slog.KindFloat64:
		return v.Float64()
	case slog.KindBool:
		return v.Bool()
	case slog.KindDuration:
		return v.Duration().String()
	case slog.KindTime:
		return v.Time().UTC().Format(time.RFC3339Nano)
	default:
		raw := v.Any()
		if err, ok := raw.(error); ok {
			return err.Error()
		}
		return ScrubValue(raw)
	}
}
