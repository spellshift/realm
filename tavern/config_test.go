package main

import (
	"context"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"realm.pub/tavern/internal/ent/migrate"
	"realm.pub/tavern/internal/ent/tag"
)

// TestConfigureMySQLFromEnv ensures environment variables set the proper config values.
func TestConfigureMySQLFromEnv(t *testing.T) {
	cleanup := func() {
		require.NoError(t, os.Unsetenv(EnvMySQLAddr.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLNet.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLUser.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLPasswd.Key))
		require.NoError(t, os.Unsetenv(EnvMySQLDB.Key))
		require.NoError(t, os.Unsetenv(EnvDBMaxIdleConns.Key))
		require.NoError(t, os.Unsetenv(EnvDBMaxOpenConns.Key))
		require.NoError(t, os.Unsetenv(EnvDBMaxConnLifetime.Key))
	}

	t.Run("SQLLite", func(t *testing.T) {
		defer cleanup()

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)

	})

	t.Run("Defaults", func(t *testing.T) {
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

	t.Run("SQLLite", func(t *testing.T) {
		defer cleanup()
		require.NoError(t, os.Setenv(EnvDBMaxIdleConns.Key, "1337"))
		require.NoError(t, os.Setenv(EnvDBMaxOpenConns.Key, "420"))
		require.NoError(t, os.Setenv(EnvDBMaxConnLifetime.Key, "20"))

		cfg := &Config{}
		ConfigureMySQLFromEnv()(cfg)

		client, err := cfg.Connect()
		require.NoError(t, err)
		require.NotNil(t, client)

		require.NoError(t, client.Schema.Create(
			context.Background(),
			migrate.WithGlobalUniqueID(true),
		))

		// Create an ent to assert we're not impacted by a "no such table:" bug, which can happen if DBLimits are not properly applied
		_, err = client.Tag.Create().SetName("Test").SetKind(tag.KindGroup).Save(context.Background())
		require.NoError(t, err)
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

func TestConfigurePubSubFromEnv(t *testing.T) {
	cleanup := func() {
		require.NoError(t, os.Unsetenv(EnvPubSubSubscriberMaxMessagesBuffered.Key))
	}
	defer cleanup()

	t.Run("Default", func(t *testing.T) {
		assert.Equal(t, 32768, EnvPubSubSubscriberMaxMessagesBuffered.Int())
	})

	t.Run("Set", func(t *testing.T) {
		require.NoError(t, os.Setenv(EnvPubSubSubscriberMaxMessagesBuffered.Key, "9999"))
		assert.Equal(t, 9999, EnvPubSubSubscriberMaxMessagesBuffered.Int())
	})
}
