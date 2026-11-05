//go:build ignore

package main

import (
	"fmt"
	"os"
	"regexp"
	"sort"
	"strings"
)

// This program generates MatchRegex filter extensions for every searchable
// string field in the entgql-generated WhereInput structs. It runs
// automatically inside `go generate ./tavern/...` (see generate.go), right
// after ent codegen and before gqlgen. It reads gql_where_input.go in this
// directory and writes:
//
//	tavern/internal/graphql/schema/regex.graphql    - `extend input` blocks
//	tavern/internal/graphql/regex.resolvers.go      - resolver implementations
//	tavern/internal/graphql/regex_test.go           - table-driven coverage
//
// Usage (run from tavern/internal/ent):
//
//	go run -mod=mod generate_regex.go --check      # survey only, no writes
//	go run -mod=mod generate_regex.go --generate   # write the three files

type regexField struct {
	structField  string // e.g. PrimaryIP
	graphqlBase  string // e.g. primaryIP
	graphqlField string // e.g. primaryIPMatchRegex
	method       string // e.g. PrimaryIPMatchRegex
	columnConst  string // e.g. FieldPrimaryIP
}

var (
	typeRe   = regexp.MustCompile(`^type (\w+)WhereInput struct \{$`)
	memberRe = regexp.MustCompile("^\t([A-Za-z0-9_]+)\\s+(\\S+)\\s+`json:\"([^\",]+)(?:,omitempty)?\"`")
)

// initialisms mirrors the whole-word acronym handling in gqlgen's GoName
// (IP, ID, URL, ...). Applied on camel-case word boundaries so words like
// "Identifier" are untouched.
var initialisms = map[string]bool{
	"ip": true, "id": true, "url": true, "api": true,
	"http": true, "ui": true, "os": true,
}

func isUpper(r rune) bool { return r >= 'A' && r <= 'Z' }
func isLower(r rune) bool { return r >= 'a' && r <= 'z' }

func lowerFirst(s string) string {
	if s == "" {
		return s
	}
	return strings.ToLower(s[:1]) + s[1:]
}

func splitWords(s string) []string {
	rs := []rune(s)
	var words []string
	start := 0
	for i := 1; i < len(rs); i++ {
		cur, prev := rs[i], rs[i-1]
		boundary := false
		if isUpper(cur) && !isUpper(prev) {
			boundary = true
		} else if isUpper(cur) && isUpper(prev) && i+1 < len(rs) && isLower(rs[i+1]) {
			boundary = true
		}
		if boundary {
			words = append(words, string(rs[start:i]))
			start = i
		}
	}
	return append(words, string(rs[start:]))
}

func goMethodName(graphqlField string) string {
	var b strings.Builder
	for _, w := range splitWords(graphqlField) {
		lw := strings.ToLower(w)
		switch {
		case initialisms[lw]:
			b.WriteString(strings.ToUpper(w))
		case w == strings.ToUpper(w):
			b.WriteString(w)
		default:
			b.WriteString(strings.ToUpper(w[:1]) + strings.ToLower(w[1:]))
		}
	}
	return b.String()
}

func packageName(typename string) string {
	return strings.ToLower(typename)
}

// collectFields returns searchable string fields per WhereInput type, keyed
// by type name. A field qualifies when its WhereInput struct has a *string
// member ending in "Contains".
func collectFields(src string) (map[string][]regexField, error) {
	members := map[string]map[string]string{}
	tags := map[string]map[string]bool{}
	var current string
	for _, line := range strings.Split(src, "\n") {
		if m := typeRe.FindStringSubmatch(line); m != nil {
			current = m[1]
			members[current] = map[string]string{}
			tags[current] = map[string]bool{}
			continue
		}
		if current == "" {
			continue
		}
		if line == "}" {
			current = ""
			continue
		}
		if m := memberRe.FindStringSubmatch(line); m != nil {
			name, gotype, tag := m[1], m[2], m[3]
			if i := strings.Index(tag, ","); i >= 0 {
				tag = tag[:i]
			}
			members[current][name] = gotype
			tags[current][tag] = true
		}
	}

	fields := map[string][]regexField{}
	for _, typename := range sortedKeys(members) {
		var found []regexField
		for _, name := range sortedKeys(members[typename]) {
			gotype := members[typename][name]
			if gotype != "*string" || !strings.HasSuffix(name, "Contains") {
				continue
			}
			structField := strings.TrimSuffix(name, "Contains")
			if structField == "" {
				return nil, fmt.Errorf("empty base for %s.%s", typename, name)
			}
			// Recover the exact json tag for this member.
			tagRe := regexp.MustCompile(
				"^\t" + regexp.QuoteMeta(name) + `\s+\*string\s+` +
					"`json:\"([^\"]+)\"`$")
			tag := ""
			for _, line := range strings.Split(src, "\n") {
				if m := tagRe.FindStringSubmatch(line); m != nil {
					tag = m[1]
					break
				}
			}
			if i := strings.Index(tag, ","); i >= 0 {
				tag = tag[:i]
			}
			if !strings.HasSuffix(tag, "Contains") {
				return nil, fmt.Errorf("unexpected tag %s for %s.%s", tag, typename, name)
			}
			base := strings.TrimSuffix(tag, "Contains")
			if base == "" {
				return nil, fmt.Errorf("empty base for %s.%s", typename, name)
			}
			if tags[typename][base+"MatchRegex"] {
				return nil, fmt.Errorf("collision: %s already present on %sWhereInput",
					base+"MatchRegex", typename)
			}
			found = append(found, regexField{
				structField:  structField,
				graphqlBase:  base,
				graphqlField: base + "MatchRegex",
				method:       goMethodName(base + "MatchRegex"),
				columnConst:  "Field" + structField,
			})
		}
		if len(found) > 0 {
			sort.Slice(found, func(i, j int) bool {
				return found[i].graphqlField < found[j].graphqlField
			})
			fields[typename] = found
		}
	}
	return fields, nil
}

func sortedKeys[V any](m map[string]V) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// checkMappings verifies every derived column constant exists in the ent
// package on disk.
func checkMappings(fields map[string][]regexField) []string {
	var problems []string
	constRe := func(columnConst string) *regexp.Regexp {
		return regexp.MustCompile(`(?m)^\s*` + regexp.QuoteMeta(columnConst) + `(\s|=)`)
	}
	for _, typename := range sortedKeys(fields) {
		pkg := packageName(typename)
		baseFile := pkg + "/" + pkg + ".go"
		content, err := os.ReadFile(baseFile)
		if err != nil {
			problems = append(problems, fmt.Sprintf("%s: cannot read %s: %v", typename, baseFile, err))
			continue
		}
		for _, r := range fields[typename] {
			if !constRe(r.columnConst).Match(content) {
				problems = append(problems, fmt.Sprintf("%s: missing column const %s in %s",
					typename, r.columnConst, baseFile))
			}
		}
	}
	return problems
}

func sortedTypenames(fields map[string][]regexField) []string {
	return sortedKeys(fields)
}

func emitSchema(fields map[string][]regexField, b *strings.Builder) {
	b.WriteString("# Code generated by tavern/internal/ent/generate_regex.go. DO NOT EDIT.\n")
	b.WriteString("# Rebuilt automatically by `go generate ./tavern/...`.\n\n")
	for _, typename := range sortedTypenames(fields) {
		fmt.Fprintf(b, "extend input %sWhereInput {\n", typename)
		for _, r := range fields[typename] {
			fmt.Fprintf(b, "  %s: String\n", r.graphqlField)
		}
		b.WriteString("}\n")
	}
}

func emitImports(b *strings.Builder, extra []string) {
	imports := append([]string{"realm.pub/tavern/internal/ent"}, extra...)
	imports = append(imports, "realm.pub/tavern/internal/ent/predicate")
	sort.Strings(imports)
	for _, imp := range imports {
		fmt.Fprintf(b, "\t%q\n", imp)
	}
}

func emitResolvers(fields map[string][]regexField, b *strings.Builder) {
	b.WriteString("package graphql\n\n")
	b.WriteString("// Code generated by tavern/internal/ent/generate_regex.go. DO NOT EDIT.\n")
	b.WriteString("// Rebuilt automatically by `go generate ./tavern/...`. Resolver\n")
	b.WriteString("// implementations are preserved across runs so long as method names\n")
	b.WriteString("// match gqlgen's canonical GoName for each field.\n\n")
	b.WriteString("import (\n")
	b.WriteString("\t\"context\"\n")
	b.WriteString("\t\"fmt\"\n")
	b.WriteString("\t\"regexp\"\n\n")
	b.WriteString("\t\"entgo.io/ent/dialect/sql\"\n\n")
	var pkgs []string
	for _, typename := range sortedTypenames(fields) {
		pkgs = append(pkgs, "realm.pub/tavern/internal/ent/"+packageName(typename))
	}
	emitImports(b, pkgs)
	b.WriteString(")\n")
	for _, typename := range sortedTypenames(fields) {
		recv := lowerFirst(typename) + "WhereInputResolver"
		for _, r := range fields[typename] {
			b.WriteString("\n")
			fmt.Fprintf(b, "// %s is the resolver for the %s field.\n", r.method, r.graphqlField)
			fmt.Fprintf(b, "func (r *%s) %s(ctx context.Context, obj *ent.%sWhereInput, data *string) error {\n",
				recv, r.method, typename)
			b.WriteString("\tif data == nil {\n\t\treturn nil\n\t}\n")
			fmt.Fprintf(b, "\tif _, err := regexp.Compile(*data); err != nil {\n")
			fmt.Fprintf(b, "\t\treturn fmt.Errorf(\"invalid %s %%q: %%w\", *data, err)\n", r.graphqlField)
			b.WriteString("\t}\n")
			b.WriteString("\tpattern := *data\n")
			fmt.Fprintf(b, "\tobj.AddPredicates(predicate.%s(func(s *sql.Selector) {\n", typename)
			fmt.Fprintf(b, "\t\ts.Where(sql.ExprP(s.C(%s.%s)+\" REGEXP ?\", pattern))\n",
				packageName(typename), r.columnConst)
			b.WriteString("\t}))\n")
			b.WriteString("\treturn nil\n")
			b.WriteString("}\n")
		}
	}
}

func emitTest(fields map[string][]regexField, b *strings.Builder) {
	b.WriteString("package graphql\n\n")
	b.WriteString("// Code generated by tavern/internal/ent/generate_regex.go. DO NOT EDIT.\n")
	b.WriteString("// Rebuilt automatically by `go generate ./tavern/...`. Covers every\n")
	b.WriteString("// generated MatchRegex resolver: invalid patterns error out, valid\n")
	b.WriteString("// patterns attach a REGEXP predicate on the right column.\n\n")
	b.WriteString("import (\n")
	b.WriteString("\t\"context\"\n")
	b.WriteString("\t\"testing\"\n\n")
	b.WriteString("\t\"entgo.io/ent/dialect\"\n")
	b.WriteString("\t\"entgo.io/ent/dialect/sql\"\n\n")
	var pkgs []string
	for _, typename := range sortedTypenames(fields) {
		pkgs = append(pkgs, "realm.pub/tavern/internal/ent/"+packageName(typename))
	}
	emitImportsTest(b, pkgs)
	b.WriteString(")\n")
	b.WriteString("\nfunc TestMatchRegexResolversInvalidPattern(t *testing.T) {\n")
	b.WriteString("\tbad := \"([0-9+\"\n")
	b.WriteString("\tctx := context.Background()\n")
	for _, typename := range sortedTypenames(fields) {
		recv := lowerFirst(typename) + "WhereInputResolver"
		for _, r := range fields[typename] {
			b.WriteString("\t{\n")
			fmt.Fprintf(b, "\t\tr := &%s{&Resolver{}}\n", recv)
			fmt.Fprintf(b, "\t\twhere := &ent.%sWhereInput{}\n", typename)
			fmt.Fprintf(b, "\t\terr := r.%s(ctx, where, &bad)\n", r.method)
			b.WriteString("\t\trequire.Error(t, err)\n")
			b.WriteString("\t\tassert.Empty(t, where.Predicates)\n")
			b.WriteString("\t}\n")
		}
	}
	b.WriteString("}\n")
	b.WriteString("\nfunc TestMatchRegexResolversAttachRegexp(t *testing.T) {\n")
	b.WriteString("\tpattern := \"^10\\\\.0\\\\.\"\n")
	b.WriteString("\n\tctx := context.Background()\n")
	for _, typename := range sortedTypenames(fields) {
		recv := lowerFirst(typename) + "WhereInputResolver"
		for _, r := range fields[typename] {
			b.WriteString("\t{\n")
			fmt.Fprintf(b, "\t\tr := &%s{&Resolver{}}\n", recv)
			fmt.Fprintf(b, "\t\twhere := &ent.%sWhereInput{}\n", typename)
			fmt.Fprintf(b, "\t\trequire.NoError(t, r.%s(ctx, where, &pattern))\n", r.method)
			b.WriteString("\t\trequire.Len(t, where.Predicates, 1)\n")
			fmt.Fprintf(b, "\t\tsel := sql.Select(\"*\").From(sql.Table(%s.Table))\n", packageName(typename))
			b.WriteString("\t\tsel.SetDialect(dialect.MySQL)\n")
			b.WriteString("\t\tfor _, p := range where.Predicates {\n")
			b.WriteString("\t\t\tp(sel)\n")
			b.WriteString("\t\t}\n")
			b.WriteString("\t\tquery, args := sel.Query()\n")
			b.WriteString("\t\tassert.Contains(t, query, \"REGEXP\")\n")
			fmt.Fprintf(b, "\t\tassert.Contains(t, query, %s.%s)\n", packageName(typename), r.columnConst)
			b.WriteString("\t\tassert.Contains(t, args, pattern)\n")
			b.WriteString("\t}\n")
		}
	}
	b.WriteString("}\n")
}

func emitImportsTest(b *strings.Builder, pkgs []string) {
	imports := append([]string{"realm.pub/tavern/internal/ent"}, pkgs...)
	sort.Strings(imports)
	for _, imp := range imports {
		fmt.Fprintf(b, "\t%q\n", imp)
	}
	b.WriteString("\n\t\"github.com/stretchr/testify/assert\"\n")
	b.WriteString("\t\"github.com/stretchr/testify/require\"\n")
}

func writeFile(path, content string) error {
	return os.WriteFile(path, []byte(content), 0644)
}

func run(checkOnly bool) int {
	src, err := os.ReadFile("gql_where_input.go")
	if err != nil {
		fmt.Fprintln(os.Stderr, "read gql_where_input.go:", err)
		return 1
	}
	fields, err := collectFields(string(src))
	if err != nil {
		fmt.Fprintln(os.Stderr, "collect fields:", err)
		return 1
	}
	total := 0
	for _, typename := range sortedTypenames(fields) {
		var names []string
		for _, r := range fields[typename] {
			names = append(names, r.graphqlField)
		}
		fmt.Printf("  %s: %s\n", typename, strings.Join(names, ", "))
		total += len(names)
	}
	fmt.Printf("types with string searches: %d\n", len(fields))
	fmt.Printf("total MatchRegex fields: %d\n", total)
	if problems := checkMappings(fields); len(problems) > 0 {
		for _, p := range problems {
			fmt.Println("PROBLEM:", p)
		}
		return 1
	}
	fmt.Println("mappings OK")
	if checkOnly {
		return 0
	}
	var schema, resolvers, test strings.Builder
	emitSchema(fields, &schema)
	emitResolvers(fields, &resolvers)
	emitTest(fields, &test)
	for path, content := range map[string]string{
		"../graphql/schema/regex.graphql": schema.String(),
		"../graphql/regex.resolvers.go":   resolvers.String(),
		"../graphql/regex_test.go":        test.String(),
	} {
		if err := writeFile(path, content); err != nil {
			fmt.Fprintln(os.Stderr, "write", path+":", err)
			return 1
		}
		fmt.Println("wrote", path)
	}
	return 0
}

func main() {
	if len(os.Args) != 2 || (os.Args[1] != "--check" && os.Args[1] != "--generate") {
		fmt.Fprintln(os.Stderr, "usage: go run -mod=mod generate_regex.go --check|--generate")
		os.Exit(2)
	}
	os.Exit(run(os.Args[1] == "--check"))
}
