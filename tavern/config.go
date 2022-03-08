package main

import (
	"log"
	"os"

	"github.com/kcarretto/realm/tavern/ent"

	"github.com/go-sql-driver/mysql"
)

// Config holds information that controls the behaviour of Tavern
type Config struct {
	mysql  string
	client *ent.Client
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
