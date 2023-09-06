package main

import (
	"fmt"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/kcarretto/realm/tavern/internal/ent"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"

	"entgo.io/ent/dialect/sql"
	"github.com/go-sql-driver/mysql"
)

var (
	// EnvEnableTestData if set will populate the database with test data.
	EnvEnableTestData = EnvString{"ENABLE_TEST_DATA", ""}

	// EnvOAuthClientID set to configure OAuth Client ID.
	// EnvOAuthClientSecret set to configure OAuth Client Secret.
	// EnvOAuthDomain set to configure OAuth domain for consent flow redirect.
	EnvOAuthClientID     = EnvString{"OAUTH_CLIENT_ID", ""}
	EnvOAuthClientSecret = EnvString{"OAUTH_CLIENT_SECRET", ""}
	EnvOAuthDomain       = EnvString{"OAUTH_DOMAIN", ""}

	// EnvMySQLAddr defines the MySQL address to connect to, if unset SQLLite is used.
	// EnvMySQLNet defines the network used to connect to MySQL (e.g. unix).
	// EnvMySQLUser defines the MySQL user to authenticate as.
	// EnvMySQLPasswd defines the password for the MySQL user to authenticate with.
	// EnvMySQLDB defines the name of the MySQL database to use.
	EnvMySQLAddr   = EnvString{"MYSQL_ADDR", ""}
	EnvMySQLNet    = EnvString{"MYSQL_NET", "tcp"}
	EnvMySQLUser   = EnvString{"MYSQL_USER", "root"}
	EnvMySQLPasswd = EnvString{"MYSQL_PASSWD", ""}
	EnvMySQLDB     = EnvString{"MYSQL_DB", "tavern"}

	// EnvDBMaxIdleConns defines the maximum number of idle db connections to allow.
	// EnvDBMaxOpenConns defines the maximum number of open db connections to allow.
	// EnvDBMaxConnLifetime defines the maximum lifetime of a db connection.
	EnvDBMaxIdleConns    = EnvInteger{"DB_MAX_IDLE_CONNS", 10}
	EnvDBMaxOpenConns    = EnvInteger{"DB_MAX_OPEN_CONNS", 100}
	EnvDBMaxConnLifetime = EnvInteger{"DB_MAX_CONN_LIFETIME", 3600}
)

// Config holds information that controls the behaviour of Tavern
type Config struct {
	srv *http.Server

	mysqlDSN string

	client *ent.Client
	oauth  oauth2.Config
}

// Connect to the database using configured drivers and uri
func (cfg *Config) Connect(options ...ent.Option) (*ent.Client, error) {
	if cfg != nil && cfg.client != nil {
		return cfg.client, nil
	}

	var (
		mysqlDSN = "file:ent?mode=memory&cache=shared&_fk=1"
		driver   = "sqlite3"
	)
	if cfg != nil && cfg.mysqlDSN != "" {
		mysqlDSN = cfg.mysqlDSN
		driver = "mysql"
	}

	drv, err := sql.Open(driver, mysqlDSN)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	// Setup DB Pool Config
	var (
		maxIdleConns    = EnvDBMaxIdleConns.Int()
		maxOpenConns    = EnvDBMaxOpenConns.Int()
		maxConnLifetime = time.Duration(EnvDBMaxConnLifetime.Int()) * time.Second
	)
	if maxIdleConns < 0 {
		log.Fatalf("[FATAL] %q must be greater than or equal to 0 if set, got: %d", EnvDBMaxIdleConns.Key, maxIdleConns)
	}
	if maxOpenConns <= 0 {
		log.Fatalf("[FATAL] %q must be greater than 0 if set, got: %d", EnvDBMaxOpenConns.Key, maxOpenConns)
	}
	if maxConnLifetime <= 10*time.Second {
		log.Fatalf("[FATAL] %q must be greater than 10 seconds if set, got: %d", EnvDBMaxConnLifetime.Key, maxConnLifetime)
	}

	// Get the underlying sql.DB object of the driver.
	db := drv.DB()
	db.SetMaxIdleConns(maxIdleConns)
	db.SetMaxOpenConns(maxOpenConns)
	db.SetConnMaxLifetime(maxConnLifetime)
	return ent.NewClient(append(options, ent.Driver(drv))...), nil
}

// IsTestDataEnabled returns true if a value for the "ENABLE_TEST_DATA" environment variable is set.
func (cfg *Config) IsTestDataEnabled() bool {
	return EnvEnableTestData.String() != ""
}

// ConfigureHTTPServer enables the configuration of the Tavern HTTP server. The endpoint field will be
// overwritten with Tavern's HTTP handler when Tavern is run.
func ConfigureHTTPServer(address string, options ...func(*http.Server)) func(*Config) {
	srv := &http.Server{
		Addr: address,
	}
	for _, opt := range options {
		opt(srv)
	}
	return func(cfg *Config) {
		cfg.srv = srv
	}
}

// ConfigureOAuthFromEnv sets OAuth config values from the environment
func ConfigureOAuthFromEnv(redirectPath string) func(*Config) {
	return func(cfg *Config) {
		var (
			clientID     = EnvOAuthClientID.String()
			clientSecret = EnvOAuthClientSecret.String()
			domain       = EnvOAuthDomain.String()
		)

		// If none are set, default to auth disabled
		if clientID == "" && clientSecret == "" && domain == "" {
			log.Printf("[WARN] OAuth is not configured, authentication disabled")
			return
		}

		// If partially set, panic to indicate OAuth is improperly configured
		if clientID == "" {
			log.Fatalf("[FATAL] To configure OAuth, must provide value for environment var 'OAUTH_CLIENT_ID'")
		}
		if clientSecret == "" {
			log.Fatalf("[FATAL] To configure OAuth, must provide value for environment var 'OAUTH_CLIENT_SECRET'")
		}
		if domain == "" {
			log.Fatalf("[FATAL] To configure OAuth, must provide value for environment var 'OAUTH_DOMAIN'")
		}
		if !strings.HasPrefix(domain, "http") {
			log.Printf("[WARN] Domain (%q) not prefixed with scheme (http:// or https://), defaulting to https://", domain)
			domain = fmt.Sprintf("https://%s", domain)
		}

		cfg.oauth = oauth2.Config{
			ClientID:     clientID,
			ClientSecret: clientSecret,
			RedirectURL:  domain + redirectPath,
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
			},
			Endpoint: google.Endpoint,
		}
	}
}

// ConfigureMySQLFromEnv sets MySQL config values from the environment
func ConfigureMySQLFromEnv() func(*Config) {
	return func(cfg *Config) {
		mysqlConfig := mysql.NewConfig()

		mysqlConfig.Addr = EnvMySQLAddr.String()
		if mysqlConfig.Addr == "" {
			log.Printf("[WARN] MySQL is not configured, using SQLite")
			return
		}

		mysqlConfig.ParseTime = true
		mysqlConfig.Net = EnvMySQLNet.String()
		mysqlConfig.User = EnvMySQLUser.String()
		mysqlConfig.Passwd = EnvMySQLPasswd.String()
		mysqlConfig.DBName = EnvMySQLDB.String()

		cfg.mysqlDSN = mysqlConfig.FormatDSN()
	}
}

// ConfigureMySQLFromClient sets the provided Ent client as the main interface for DB access.
func ConfigureMySQLFromClient(client *ent.Client) func(*Config) {
	return func(cfg *Config) {
		cfg.client = client
	}
}
