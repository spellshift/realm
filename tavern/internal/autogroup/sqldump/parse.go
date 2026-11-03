// Package sqldump is a minimal, dependency-free parser for mysqldump .sql
// files, narrow and built specifically for mysqldump's own output format
// (one `CREATE TABLE` block per table, one or more `INSERT INTO ... VALUES
// (...),(...),...;` statements per table, each potentially a single very
// long line). It is NOT a general SQL parser.
//
// This exists purely to support `host-autogroup benchmark`: scoring this
// tool's clustering against a past competition's own already-applied
// tags, read directly from a raw mysqldump backup instead of a live
// server. It is intentionally never used for anything that reads an
// un-tagged, "real" new-engagement dump -- see benchmark/benchmark.go.
package sqldump

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"regexp"
	"strconv"
	"strings"
)

// Row is one parsed INSERT tuple's values, in column order as declared by
// that table's own CREATE TABLE statement (NOT assumed to be the same
// across different dump files/versions -- see Schema).
type Row []any

// Schema maps table name -> ordered column names, read directly from the
// dump's own CREATE TABLE statements so column drift between dump
// versions (confirmed to happen in practice: extra FK columns appended,
// or columns missing entirely in an older schema) never silently
// misaligns values.
type Schema map[string][]string

var createTableRe = regexp.MustCompile("^CREATE TABLE `([a-zA-Z0-9_]+)` \\(")
var columnRe = regexp.MustCompile("^\\s*`([a-zA-Z0-9_]+)`\\s+\\S")

var skipPrefixes = []string{"PRIMARY KEY", "UNIQUE KEY", "KEY ", "CONSTRAINT", "FOREIGN KEY"}

func hasSkipPrefix(s string) bool {
	for _, p := range skipPrefixes {
		if strings.HasPrefix(s, p) {
			return true
		}
	}
	return false
}

// ReadSchema scans the whole file once and returns column order for every
// table it finds a CREATE TABLE for.
func ReadSchema(path string) (Schema, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	schema := Schema{}
	r := bufio.NewReaderSize(f, 1<<20)

	var curTable string
	var curCols []string
	inBlock := false

	for {
		line, err := readLongLine(r)
		if line == "" && err != nil {
			break
		}
		if !inBlock {
			if m := createTableRe.FindStringSubmatch(line); m != nil {
				curTable = m[1]
				curCols = nil
				inBlock = true
			}
			if err != nil {
				break
			}
			continue
		}
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, ")") {
			schema[curTable] = curCols
			inBlock = false
			if err != nil {
				break
			}
			continue
		}
		if !hasSkipPrefix(trimmed) {
			if m := columnRe.FindStringSubmatch(line); m != nil {
				curCols = append(curCols, m[1])
			}
		}
		if err != nil {
			break
		}
	}
	return schema, nil
}

// readLongLine reads one line of arbitrary length (mysqldump INSERT lines
// can be many megabytes -- bufio.Scanner's fixed max-token-size would
// error on those, so this uses bufio.Reader.ReadString instead, which has
// no such limit).
func readLongLine(r *bufio.Reader) (string, error) {
	return r.ReadString('\n')
}

// ForEachRow streams through the dump, calling fn for every row of any
// table named in wantTables. Rows for tables not in wantTables are
// skipped without being parsed (cheap prefix check only), so this stays
// fast even on multi-GB dumps dominated by tables we don't care about
// (e.g. `assets`, which can have single INSERT lines tens of MB long).
func ForEachRow(path string, wantTables map[string]bool, schema Schema, fn func(table string, row Row) error) error {
	f, err := os.Open(path)
	if err != nil {
		return err
	}
	defer f.Close()

	r := bufio.NewReaderSize(f, 1<<20)
	for {
		line, err := readLongLine(r)
		if line != "" && strings.HasPrefix(line, "INSERT INTO `") {
			table, rest := parseInsertHeader(line)
			if table != "" && wantTables[table] {
				cols := schema[table]
				rows, perr := parseValuesTuples(rest)
				if perr != nil {
					return fmt.Errorf("table %s: %w", table, perr)
				}
				for _, row := range rows {
					if len(cols) > 0 && len(row) != len(cols) {
						return fmt.Errorf("table %s: row has %d values but schema has %d columns (%v)",
							table, len(row), len(cols), cols)
					}
					if ferr := fn(table, row); ferr != nil {
						return ferr
					}
				}
			}
		}
		if err != nil {
			if err == io.EOF {
				return nil
			}
			return err
		}
	}
}

var insertHeaderRe = regexp.MustCompile("^INSERT INTO `([a-zA-Z0-9_]+)` VALUES ")

// parseInsertHeader splits "INSERT INTO `table` VALUES (...)...;\n" into
// the table name and the remaining tuple text.
func parseInsertHeader(line string) (table, rest string) {
	m := insertHeaderRe.FindStringSubmatchIndex(line)
	if m == nil {
		return "", ""
	}
	table = line[m[2]:m[3]]
	rest = line[m[1]:]
	return table, rest
}

// parseValuesTuples parses "(...),(...),...;\n" into a slice of Rows.
// Handles single-quoted strings with backslash escaping, NULL, integers/
// floats, and 0x... hex blob literals (skipped as raw bytes, not needed
// by anything host-autogroup reads) -- the same narrow grammar as the
// mysqldump_parse.py reference it mirrors.
func parseValuesTuples(s string) ([]Row, error) {
	var rows []Row
	i := 0
	n := len(s)
	for i < n {
		for i < n && (s[i] == ' ' || s[i] == '\n' || s[i] == '\r') {
			i++
		}
		if i >= n || s[i] != '(' {
			break
		}
		row, next, err := parseTuple(s, i)
		if err != nil {
			return nil, err
		}
		rows = append(rows, row)
		i = next
		for i < n && s[i] == ',' {
			i++
		}
	}
	return rows, nil
}

func parseTuple(s string, i int) (Row, int, error) {
	n := len(s)
	if s[i] != '(' {
		return nil, i, fmt.Errorf("expected '(' at %d", i)
	}
	i++
	var row Row
	for {
		for i < n && s[i] == ' ' {
			i++
		}
		val, next, err := parseValue(s, i)
		if err != nil {
			return nil, i, err
		}
		row = append(row, val)
		i = next
		for i < n && s[i] == ' ' {
			i++
		}
		if i >= n {
			return nil, i, fmt.Errorf("unexpected end of input inside tuple")
		}
		if s[i] == ',' {
			i++
			continue
		}
		if s[i] == ')' {
			i++
			break
		}
		return nil, i, fmt.Errorf("unexpected char %q at %d inside tuple", s[i], i)
	}
	return row, i, nil
}

func parseValue(s string, i int) (any, int, error) {
	n := len(s)
	if i >= n {
		return nil, i, fmt.Errorf("unexpected end of input")
	}
	switch {
	case s[i] == '\'':
		return parseString(s, i)
	case s[i] == 'N' && strings.HasPrefix(s[i:], "NULL"):
		return nil, i + 4, nil
	case s[i] == '0' && i+1 < n && s[i+1] == 'x':
		j := i + 2
		for j < n && isHexDigit(s[j]) {
			j++
		}
		return s[i:j], j, nil // hex blob kept as raw text, never dereferenced
	default:
		j := i
		if s[j] == '-' {
			j++
		}
		for j < n && (isDigit(s[j]) || s[j] == '.') {
			j++
		}
		if j == i {
			return nil, i, fmt.Errorf("unrecognized value starting %q at %d", s[i:min(i+20, n)], i)
		}
		tok := s[i:j]
		if strings.Contains(tok, ".") {
			f, err := strconv.ParseFloat(tok, 64)
			return f, j, err
		}
		v, err := strconv.ParseInt(tok, 10, 64)
		return v, j, err
	}
}

func parseString(s string, i int) (string, int, error) {
	n := len(s)
	i++ // skip opening quote
	var sb strings.Builder
	for i < n {
		c := s[i]
		if c == '\\' {
			if i+1 >= n {
				return "", i, fmt.Errorf("truncated escape at %d", i)
			}
			next := s[i+1]
			switch next {
			case 'n':
				sb.WriteByte('\n')
			case 't':
				sb.WriteByte('\t')
			case 'r':
				sb.WriteByte('\r')
			case '0':
				sb.WriteByte(0)
			case 'Z':
				sb.WriteByte(0x1a)
			case 'b':
				sb.WriteByte('\b')
			default:
				sb.WriteByte(next)
			}
			i += 2
			continue
		}
		if c == '\'' {
			if i+1 < n && s[i+1] == '\'' {
				sb.WriteByte('\'')
				i += 2
				continue
			}
			return sb.String(), i + 1, nil
		}
		sb.WriteByte(c)
		i++
	}
	return "", i, fmt.Errorf("unterminated string starting before %d", i)
}

func isDigit(c byte) bool    { return c >= '0' && c <= '9' }
func isHexDigit(c byte) bool { return isDigit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F') }
