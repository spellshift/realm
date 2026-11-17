package logging

import (
	"bytes"
	"fmt"
	"io"
	"log/slog"
	"strconv"
	"strings"
	"sync"
	"time"
	"unicode/utf8"
)

const (
	ansiReset   = "\x1b[0m"
	ansiDim     = "\x1b[2m"
	ansiCyan    = "\x1b[36m"
	ansiGreen   = "\x1b[32m"
	ansiMagenta = "\x1b[35m"
	ansiYellow  = "\x1b[33m"
	ansiRed     = "\x1b[31m"

	textSubsystemColWidth = 16
	textShortTraceLen     = 8
)

var textBufPool = sync.Pool{
	New: func() any {
		return bytes.NewBuffer(make([]byte, 0, 256))
	},
}

type textSink struct {
	mu    sync.Mutex
	out   io.Writer
	color bool
}

func newTextSink(out io.Writer, color bool) *textSink {
	return &textSink{out: out, color: color}
}

func writeSpaces(buf *bytes.Buffer, n int) {
	for i := 0; i < n; i++ {
		buf.WriteByte(' ')
	}
}

func (s *textSink) WriteRecord(env recordEnvelope) error {
	buf := textBufPool.Get().(*bytes.Buffer)
	buf.Reset()
	defer textBufPool.Put(buf)

	attrBuf := textBufPool.Get().(*bytes.Buffer)
	attrBuf.Reset()
	defer textBufPool.Put(attrBuf)

	seen := make(map[string]struct{}, 16)
	writeKV := func(k string, v slog.Value) {
		if k == "" {
			return
		}
		seen[k] = struct{}{}
		v = v.Resolve()
		if IsSensitiveKey(k) {
			v = slog.StringValue(RedactedPlaceholder)
		}
		attrBuf.WriteByte(' ')
		if s.color {
			attrBuf.WriteString(ansiDim)
		}
		attrBuf.WriteString(k)
		attrBuf.WriteByte('=')
		if s.color {
			attrBuf.WriteString(ansiReset)
		}
		appendTextValue(attrBuf, v)
	}

	prefix := strings.Join(env.Groups, ".")
	qualify := func(k string) string {
		if prefix == "" {
			return k
		}
		return prefix + "." + k
	}

	for _, a := range env.BoundAttrs {
		flattenTextAttr(a, prefix, writeKV)
	}
	env.Record.Attrs(func(a slog.Attr) bool {
		flattenTextAttr(a, prefix, writeKV)
		return true
	})

	// Append context attribution fields not already explicitly logged on the record.
	if env.Meta.TraceID != "" {
		_, hasTrace := seen["trace"]
		_, hasTraceID := seen["trace_id"]
		if !hasTrace && !hasTraceID {
			shortTrace := env.Meta.TraceID
			if len(shortTrace) > textShortTraceLen {
				shortTrace = shortTrace[:textShortTraceLen]
			}
			writeKV("trace", slog.StringValue(shortTrace))
		}
	}
	if env.InstanceID != "" {
		if _, ok := seen["tavern_id"]; !ok {
			writeKV("tavern_id", slog.StringValue(env.InstanceID))
		}
	}
	if env.Meta.Actor.Name != "" {
		if _, ok := seen["actor_name"]; !ok {
			writeKV("actor_name", slog.StringValue(env.Meta.Actor.Name))
		}
	} else if env.Meta.Actor.ID != "" {
		if _, ok := seen["actor_id"]; !ok {
			writeKV("actor_id", slog.StringValue(env.Meta.Actor.ID))
		}
	}
	for _, sa := range env.Meta.StreamAttrs {
		qk := qualify(sa.Key)
		if _, ok := seen[qk]; !ok {
			writeKV(qk, sa.Value)
		}
	}

	ts := env.Record.Time
	if ts.IsZero() {
		ts = time.Now()
	}
	if s.color {
		buf.WriteString(ansiDim)
	}
	buf.WriteString(ts.Format("15:04:05.000"))
	if s.color {
		buf.WriteString(ansiReset)
	}
	buf.WriteByte(' ')

	lbl := LevelShortLabel(env.Record.Level, env.IsAudit)
	if s.color {
		buf.WriteString(levelColor(env.Record.Level, env.IsAudit))
	}
	buf.WriteString(lbl)
	if s.color {
		buf.WriteString(ansiReset)
	}
	buf.WriteByte(' ')

	sub := env.Subsystem
	if sub == "" {
		sub = "app"
	}
	if s.color {
		buf.WriteString(ansiCyan)
	}
	buf.WriteByte('[')
	buf.WriteString(sub)
	buf.WriteByte(']')
	if s.color {
		buf.WriteString(ansiReset)
	}

	hasBody := env.Record.Message != "" || attrBuf.Len() > 0
	if hasBody {
		subWidth := utf8.RuneCountInString(sub) + 2
		if pad := textSubsystemColWidth - subWidth; pad > 0 {
			writeSpaces(buf, pad)
		}
	}

	if env.Record.Message != "" {
		buf.WriteByte(' ')
		buf.WriteString(env.Record.Message)
	}

	if attrBuf.Len() > 0 {
		buf.Write(attrBuf.Bytes())
	}

	buf.WriteByte('\n')

	s.mu.Lock()
	_, err := s.out.Write(buf.Bytes())
	s.mu.Unlock()
	return err
}

func flattenTextAttr(a slog.Attr, prefix string, emit func(string, slog.Value)) {
	a.Value = a.Value.Resolve()
	if a.Equal(slog.Attr{}) {
		return
	}
	key := a.Key
	if prefix != "" && key != "" {
		key = prefix + "." + key
	} else if prefix != "" {
		key = prefix
	}
	if a.Value.Kind() == slog.KindGroup {
		for _, ga := range a.Value.Group() {
			flattenTextAttr(ga, key, emit)
		}
		return
	}
	emit(key, a.Value)
}

func appendTextValue(buf *bytes.Buffer, v slog.Value) {
	switch v.Kind() {
	case slog.KindString:
		s := v.String()
		if needsQuoting(s) {
			buf.WriteString(strconv.Quote(s))
			return
		}
		buf.WriteString(s)
	case slog.KindInt64:
		buf.WriteString(strconv.FormatInt(v.Int64(), 10))
	case slog.KindUint64:
		buf.WriteString(strconv.FormatUint(v.Uint64(), 10))
	case slog.KindFloat64:
		buf.WriteString(strconv.FormatFloat(v.Float64(), 'g', -1, 64))
	case slog.KindBool:
		buf.WriteString(strconv.FormatBool(v.Bool()))
	case slog.KindDuration:
		buf.WriteString(v.Duration().String())
	case slog.KindTime:
		buf.WriteString(v.Time().Format(time.RFC3339))
	default:
		s := fmt.Sprint(ScrubValue(v.Any()))
		if needsQuoting(s) {
			buf.WriteString(strconv.Quote(s))
			return
		}
		buf.WriteString(s)
	}
}

func needsQuoting(s string) bool {
	if s == "" {
		return true
	}
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c <= ' ' || c == '"' || c == '=' || c >= 0x7f {
			return true
		}
	}
	return false
}

func levelColor(l slog.Level, isAudit bool) string {
	if isAudit && l == slog.LevelInfo {
		return ansiMagenta
	}
	switch {
	case l <= LevelTrace:
		return ansiDim
	case l < slog.LevelInfo:
		return ansiCyan
	case l < slog.LevelWarn:
		return ansiGreen
	case l < slog.LevelError:
		return ansiYellow
	default:
		return ansiRed
	}
}
