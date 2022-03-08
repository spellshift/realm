package main

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConfigureMySQLFromEnv(t *testing.T) {
	// Always cleanup env
	defer func() {
		require.NoError(t, os.Unsetenv("MYSQL_ADDR"))
		require.NoError(t, os.Unsetenv("MYSQL_NET"))
		require.NoError(t, os.Unsetenv("MYSQL_USER"))
		require.NoError(t, os.Unsetenv("MYSQL_PASSWD"))
		require.NoError(t, os.Unsetenv("MYSQL_DB"))
	}()

	// No env vars set
	cfg := &Config{}
	ConfigureMySQLFromEnv()(cfg)
	assert.Empty(t, cfg.mysql, "Set MySQL DSN without any env config")

	// Missing Addr
	require.NoError(t, os.Setenv("MYSQL_NET", "unix"))
	require.NoError(t, os.Setenv("MYSQL_USER", "admin"))
	require.NoError(t, os.Setenv("MYSQL_PASSWD", "changeme"))
	require.NoError(t, os.Setenv("MYSQL_DB", "testdb"))
	ConfigureMySQLFromEnv()(cfg)
	assert.Empty(t, cfg.mysql, "Set MySQL DSN without MYSQL_ADDR in env")

	// w/ Addr
	require.NoError(t, os.Setenv("MYSQL_ADDR", "127.0.0.1"))
	ConfigureMySQLFromEnv()(cfg)
	assert.Equal(t, "admin:changeme@unix(127.0.0.1)/testdb?allowNativePasswords=false&checkConnLiveness=false&parseTime=true&maxAllowedPacket=0", cfg.mysql)

	// Defaults w/ Addr
	require.NoError(t, os.Unsetenv("MYSQL_NET"))
	require.NoError(t, os.Unsetenv("MYSQL_USER"))
	require.NoError(t, os.Unsetenv("MYSQL_PASSWD"))
	require.NoError(t, os.Unsetenv("MYSQL_DB"))
	ConfigureMySQLFromEnv()(cfg)
	assert.Equal(t, "root@tcp(127.0.0.1)/tavern?allowNativePasswords=false&checkConnLiveness=false&parseTime=true&maxAllowedPacket=0", cfg.mysql)
}
