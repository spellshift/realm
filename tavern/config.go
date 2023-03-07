package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/kcarretto/realm/tavern/ent"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"

	"entgo.io/ent/dialect/sql"
	"github.com/go-sql-driver/mysql"
)

// Config holds information that controls the behaviour of Tavern
type Config struct {
	srv *http.Server

	mysql  string
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
	if cfg != nil && cfg.mysql != "" {
		mysqlDSN = cfg.mysql
		driver = "mysql"
	}

	drv, err := sql.Open(driver, mysqlDSN)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}
	// Get the underlying sql.DB object of the driver.
	db := drv.DB()
	db.SetMaxIdleConns(10) // TODO: Move to environment variable
	db.SetMaxOpenConns(100)
	db.SetConnMaxLifetime(time.Hour)
	return ent.NewClient(append(options, ent.Driver(drv))...), nil

	// return ent.Open(
	// 	driver,
	// 	mysql,
	// 	options...,
	// )
}

// IsTestDataEnabled returns true if a value for the "ENABLE_TEST_DATA" environment variable is set.
func (cfg *Config) IsTestDataEnabled() bool {
	return os.Getenv("ENABLE_TEST_DATA") != ""
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
			clientID     = os.Getenv("OAUTH_CLIENT_ID")
			clientSecret = os.Getenv("OAUTH_CLIENT_SECRET")
			domain       = os.Getenv("OAUTH_DOMAIN")
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
		mysqlConfig := mysql.Config{
			Net:       "tcp",
			User:      "root",
			DBName:    "tavern",
			ParseTime: true,
		}

		if envAddr := os.Getenv("MYSQL_ADDR"); envAddr != "" {
			mysqlConfig.Addr = envAddr
		} else {
			log.Printf("[WARN] MySQL is not configured, using SQLite")
			return
		}
		if envNet := os.Getenv("MYSQL_NET"); envNet != "" {
			mysqlConfig.Net = envNet
		}
		if envUser := os.Getenv("MYSQL_USER"); envUser != "" {
			mysqlConfig.User = envUser
		}
		if envPasswd := os.Getenv("MYSQL_PASSWD"); envPasswd != "" {
			mysqlConfig.Passwd = envPasswd
		}
		if envDB := os.Getenv("MYSQL_DB"); envDB != "" {
			mysqlConfig.DBName = envDB
		}

		cfg.mysql = mysqlConfig.FormatDSN()
	}
}

// ConfigureMySQLFromClient sets the provided Ent client as the main interface for DB access.
func ConfigureMySQLFromClient(client *ent.Client) func(*Config) {
	return func(cfg *Config) {
		cfg.client = client
	}
}
