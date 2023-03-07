package main

import (
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
)

// TestConfigureMySQLFromEnv ensures environment variables set the proper config values.
func TestConfigureMySQLFromEnv(t *testing.T) {
	cleanup := func() {
		require.NoError(t, os.Unsetenv(EnvMySQLAddr.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLNet.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLUser.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLPasswd.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLDB.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLMaxIdleConns.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLMaxOpenConns.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLMaxConnLifetime.Key))
	}

	t.Run("NoEnvVarsSet", func(t *testing.T) {
		defer cleanup()

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)

		assert.Empty(t, cfg.mysqlDSN, "Set MySQL DSN without any env config")
	})

	t.Run("MissingAddr", func(t *testing.T) {
		defer cleanup()
		require.NoError(t, os.Setenv(EnvMySQLNet.Key, "unix"))
		require.NoError(t, os.Setenv(EnvMySQLUser.Key, "admin"))
		require.NoError(t, os.Setenv(EnvMySQLPasswd.Key, "changeme"))
		require.NoError(t, os.Setenv(EnvMySQLDB.Key, "testdb"))

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)
		assert.Empty(t, cfg.mysqlDSN, "Set MySQL DSN without MYSQL_ADDR in env")
	})

	t.Run("ValuesWithAddr", func(t *testing.T) {
		defer cleanup()
		require.NoError(t, os.Setenv(EnvMySQLNet.Key, "unix"))
		require.NoError(t, os.Setenv(EnvMySQLUser.Key, "admin"))
		require.NoError(t, os.Setenv(EnvMySQLPasswd.Key, "changeme"))
		require.NoError(t, os.Setenv(EnvMySQLDB.Key, "testdb"))
		require.NoError(t, os.Setenv(EnvMySQLAddr.Key, "127.0.0.1"))

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)

		assert.Equal(t, "admin:changeme@unix(127.0.0.1)/testdb?parseTime=true", cfg.mysqlDSN)
	})

	t.Run("DefaultsWithAddr", func(t *testing.T) {
		defer cleanup()
		require.NoError(t, os.Setenv(EnvMySQLAddr.Key, "127.0.0.1"))

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)

		assert.Equal(t, "root@tcp(127.0.0.1)/tavern?parseTime=true", cfg.mysqlDSN)
	})

	t.Run("DBConnLimits", func(t *testing.T) {
		defer cleanup()
		require.NoError(t, os.Setenv(EnvMySQLAddr.Key, "127.0.0.1"))
		require.NoError(t, os.Setenv(EnvMySQLMaxIdleConns.Key, "1337"))
		require.NoError(t, os.Setenv(EnvMySQLMaxOpenConns.Key, "420"))
		require.NoError(t, os.Setenv(EnvMySQLMaxConnLifetime.Key, "5"))

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)

		assert.Equal(t, 1337, cfg.mysqlMaxIdleConns)
		assert.Equal(t, 420, cfg.mysqlMaxOpenConns)
		assert.Equal(t, 5*time.Second, cfg.mysqlMaxConnLifetime)
	})
}

// TestConfigureOAuthFromEnv ensures environment variables set the proper config values.
func TestConfigureOAuthFromEnv(t *testing.T) {
	cleanup := func() {
		require.NoError(t, os.Unsetenv(EnvOAuthClientID.Key))
		require.NoError(t, os.Unsetenv(EnvOAuthClientSecret.Key))
		require.NoError(t, os.Unsetenv(EnvOAuthDomain.Key))
	}

	t.Run("NoEnvVarsSet", func(t *testing.T) {
		defer cleanup()

		cfg := &Config{}
		ConfigureOAuthFromEnv("/redirect/here")(cfg)

		assert.Equal(t, oauth2.Config{}, cfg.oauth)
	})

	t.Run("WithoutDomainSchema", func(t *testing.T) {
		defer cleanup()

		expectedDomain := "domain.com"
		expectedCfg := oauth2.Config{
			ClientID:     "ABCDEFG",
			ClientSecret: "beep-boop",
			RedirectURL:  fmt.Sprintf("https://%s/redirect/here", expectedDomain),
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
			},
			Endpoint: google.Endpoint,
		}

		require.NoError(t, os.Setenv(EnvOAuthClientID.Key, expectedCfg.ClientID))
		require.NoError(t, os.Setenv(EnvOAuthClientSecret.Key, expectedCfg.ClientSecret))
		require.NoError(t, os.Setenv(EnvOAuthDomain.Key, expectedDomain))

		cfg := &Config{}
		ConfigureOAuthFromEnv("/redirect/here")(cfg)

		assert.Equal(t, expectedCfg, cfg.oauth)
	})

	t.Run("Enabled", func(t *testing.T) {
		defer cleanup()

		expectedDomain := "http://domain.com"
		expectedCfg := oauth2.Config{
			ClientID:     "ABCDEFG",
			ClientSecret: "beep-boop",
			RedirectURL:  fmt.Sprintf("%s/redirect/here", expectedDomain),
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
			},
			Endpoint: google.Endpoint,
		}

		require.NoError(t, os.Setenv(EnvOAuthClientID.Key, expectedCfg.ClientID))
		require.NoError(t, os.Setenv(EnvOAuthClientSecret.Key, expectedCfg.ClientSecret))
		require.NoError(t, os.Setenv(EnvOAuthDomain.Key, expectedDomain))

		cfg := &Config{}
		ConfigureOAuthFromEnv("/redirect/here")(cfg)

		assert.Equal(t, expectedCfg, cfg.oauth)
	})
}
