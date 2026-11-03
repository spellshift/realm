package sqldump

import (
	"os"
	"path/filepath"
	"testing"
)

const sampleDump = "CREATE DATABASE /*!32312 IF NOT EXISTS*/ `tavern`;\n" +
	"USE `tavern`;\n" +
	"DROP TABLE IF EXISTS `hosts`;\n" +
	"CREATE TABLE `hosts` (\n" +
	"  `id` bigint NOT NULL AUTO_INCREMENT,\n" +
	"  `identifier` varchar(255) NOT NULL,\n" +
	"  `name` varchar(255) DEFAULT NULL,\n" +
	"  `primary_ip` varchar(255) DEFAULT NULL,\n" +
	"  PRIMARY KEY (`id`),\n" +
	"  UNIQUE KEY `identifier` (`identifier`)\n" +
	") ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\n" +
	"INSERT INTO `hosts` VALUES (1,'abc-123','mail-server-01','10.0.0.5'),(2,'def-456','web\\'s box',NULL);\n" +
	"CREATE TABLE `tags` (\n" +
	"  `id` bigint NOT NULL AUTO_INCREMENT,\n" +
	"  `name` varchar(255) NOT NULL,\n" +
	"  `kind` enum('group','service') NOT NULL,\n" +
	"  PRIMARY KEY (`id`)\n" +
	") ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\n" +
	"INSERT INTO `tags` VALUES (1,'mail','service'),(2,'Team-01','group');\n"

func writeSample(t *testing.T) string {
	dir := t.TempDir()
	path := filepath.Join(dir, "sample.sql")
	if err := os.WriteFile(path, []byte(sampleDump), 0o644); err != nil {
		t.Fatal(err)
	}
	return path
}

func TestReadSchema(t *testing.T) {
	path := writeSample(t)
	schema, err := ReadSchema(path)
	if err != nil {
		t.Fatal(err)
	}
	wantHosts := []string{"id", "identifier", "name", "primary_ip"}
	if got := schema["hosts"]; !equalStrings(got, wantHosts) {
		t.Errorf("hosts columns: got %v, want %v", got, wantHosts)
	}
	wantTags := []string{"id", "name", "kind"}
	if got := schema["tags"]; !equalStrings(got, wantTags) {
		t.Errorf("tags columns: got %v, want %v", got, wantTags)
	}
}

func TestForEachRowParsesValuesAndSkipsUnwantedTables(t *testing.T) {
	path := writeSample(t)
	schema, err := ReadSchema(path)
	if err != nil {
		t.Fatal(err)
	}

	var hostRows []Row
	err = ForEachRow(path, map[string]bool{"hosts": true}, schema, func(table string, row Row) error {
		if table != "hosts" {
			t.Errorf("expected only 'hosts' rows (tags table should be skipped), got table=%q", table)
		}
		hostRows = append(hostRows, row)
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	if len(hostRows) != 2 {
		t.Fatalf("expected 2 host rows, got %d: %v", len(hostRows), hostRows)
	}

	// row 1: (1,'abc-123','mail-server-01','10.0.0.5')
	if got := hostRows[0][0]; got != int64(1) {
		t.Errorf("row0 id: got %v (%T), want int64(1)", got, got)
	}
	if got := hostRows[0][1]; got != "abc-123" {
		t.Errorf("row0 identifier: got %v, want abc-123", got)
	}

	// row 2: (2,'def-456','web\'s box',NULL) -- tests backslash-escaped
	// quote and NULL handling
	if got := hostRows[1][2]; got != "web's box" {
		t.Errorf("row1 name: got %q, want %q (escaped quote)", got, "web's box")
	}
	if got := hostRows[1][3]; got != nil {
		t.Errorf("row1 primary_ip: got %v, want nil (NULL)", got)
	}
}

func TestForEachRowMultipleTables(t *testing.T) {
	path := writeSample(t)
	schema, err := ReadSchema(path)
	if err != nil {
		t.Fatal(err)
	}

	counts := map[string]int{}
	err = ForEachRow(path, map[string]bool{"hosts": true, "tags": true}, schema, func(table string, row Row) error {
		counts[table]++
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	if counts["hosts"] != 2 || counts["tags"] != 2 {
		t.Errorf("expected 2 hosts + 2 tags, got %v", counts)
	}
}

func TestSchemaMismatchIsAnError(t *testing.T) {
	path := writeSample(t)
	// deliberately wrong schema (missing a column) to confirm a mismatch
	// is surfaced loudly rather than silently misaligning values
	badSchema := Schema{"hosts": {"id", "identifier", "name"}} // missing primary_ip
	err := ForEachRow(path, map[string]bool{"hosts": true}, badSchema, func(table string, row Row) error {
		return nil
	})
	if err == nil {
		t.Error("expected an error when row width doesn't match schema width, got nil")
	}
}

func equalStrings(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
