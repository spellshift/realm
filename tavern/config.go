package main

import (
	"log"
	"os"

	"github.com/kcarretto/realm/tavern/ent"
	"golang.org/x/oauth2"

	"github.com/go-sql-driver/mysql"
)

// Config holds information that controls the behaviour of Tavern
type Config struct {
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
		mysql  = "file:ent?mode=memory&cache=shared&_fk=1"
		driver = "sqlite3"
	)
	if cfg != nil && cfg.mysql != "" {
		mysql = cfg.mysql
		driver = "mysql"
	}

	return ent.Open(
		driver,
		mysql,
		options...,
	)
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
			log.Printf("WARNING: OAuth is not configured, authentication disabled")
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

		cfg.oauth = oauth2.Config{
			ClientID:     clientID,
			ClientSecret: clientSecret,
			RedirectURL:  domain + redirectPath,
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
			},
		}
	}
}

// ConfigureMySQLFromEnv sets MySQL config values from the environment
func ConfigureMySQLFromEnv() func(*Config) {
	return func(cfg *Config) {
		var (
			addr   = ""
			net    = "tcp"
			user   = "root"
			passwd = ""
			dbName = "tavern"
		)
		if envAddr := os.Getenv("MYSQL_ADDR"); envAddr != "" {
			addr = envAddr
		} else {
			log.Printf("no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite")
			return
		}
		if envNet := os.Getenv("MYSQL_NET"); envNet != "" {
			net = envNet
		}
		if envUser := os.Getenv("MYSQL_USER"); envUser != "" {
			user = envUser
		}
		if envPasswd := os.Getenv("MYSQL_PASSWD"); envPasswd != "" {
			passwd = envPasswd
		}
		if envDB := os.Getenv("MYSQL_DB"); envDB != "" {
			dbName = envDB
		}

		mysqlConfig := mysql.Config{
			Addr:      addr,
			Net:       net,
			User:      user,
			Passwd:    passwd,
			DBName:    dbName,
			ParseTime: true,
		}
		cfg.mysql = mysqlConfig.FormatDSN()
	}
}
