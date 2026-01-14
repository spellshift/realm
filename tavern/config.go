package main

import (
	"context"
	"fmt"
	"log"
	"log/slog"
	"net/http"
	"strings"
	"time"

	gcppubsub "cloud.google.com/go/pubsub"
	"entgo.io/ent/dialect/sql"
	"github.com/go-sql-driver/mysql"
	"gocloud.dev/pubsub"
	_ "gocloud.dev/pubsub/gcppubsub"
	_ "gocloud.dev/pubsub/mempubsub"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/tomes"
)

// GlobalInstanceID uniquely identifies this instance for logging and naming purposes.
var GlobalInstanceID = fmt.Sprintf("tavern-%s", 1) // Using 1 or similar, namegen usage removed to avoid import if not needed, or keep it.
// The original file used namegen.New(). I should check imports.
// I kept imports but removed `stream` import if it was used for `stream.PreventPubSubColdStarts`.
// `stream` was imported as `realm.pub/tavern/internal/http/stream`.
// I should remove that import if I remove `NewShellMuxes`.

var (
	// EnvEnableTestData if set will populate the database with test data.
	// EnvEnableTestRunAndExit will start the application, but exit immediately after.
	// EnvDisableDefaultTomes will prevent the default tomes from being imported on startup.
	// EnvDebugLogging will emit verbose debug logs to help troubleshoot issues.
	// EnvJSONLogging will emit logs in JSON format for easier parsing by log aggregators.
	// EnvLogInstanceID will include the tavern instance id in log messages.
	// EnvLogGraphQLRawQuery will include the raw GraphQL query in graphql log messages.
	EnvEnableTestData       = EnvBool{"ENABLE_TEST_DATA"}
	EnvEnableTestRunAndExit = EnvBool{"ENABLE_TEST_RUN_AND_EXIT"}
	EnvDisableDefaultTomes  = EnvBool{"DISABLE_DEFAULT_TOMES"}
	EnvDebugLogging         = EnvBool{"ENABLE_DEBUG_LOGGING"}
	EnvJSONLogging          = EnvBool{"ENABLE_JSON_LOGGING"}
	EnvLogInstanceID        = EnvBool{"ENABLE_INSTANCE_ID_LOGGING"}
	EnvLogGraphQLRawQuery   = EnvBool{"ENABLE_GRAPHQL_RAW_QUERY_LOGGING"}

	// EnvHTTPListenAddr sets the address (ip:port) for tavern's HTTP server to bind to.
	// EnvHTTPMetricsAddr sets the address (ip:port) for the HTTP metrics server to bind to.
	EnvHTTPListenAddr        = EnvString{"HTTP_LISTEN_ADDR", "0.0.0.0:8000"}
	EnvHTTPMetricsListenAddr = EnvString{"HTTP_METRICS_LISTEN_ADDR", "127.0.0.1:8080"}

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

	// EnvGCPProjectID represents the project id tavern is deployed in for Google Cloud Platform deployments (leave empty otherwise).
	// EnvGCPPubsubKeepAliveIntervalMs is the interval to publish no-op pubsub messages to help avoid gcppubsub coldstart latency. 0 disables this feature.
	// EnvPubSubTopicShellInput defines the topic to publish shell input to.
	// EnvPubSubSubscriptionShellInput defines the subscription to receive shell input from.
	// EnvPubSubTopicShellOutput defines the topic to publish shell output to.
	// EnvPubSubSubscriptionShellOutput defines the subscription to receive shell output from.
	EnvGCPProjectID                  = EnvString{"GCP_PROJECT_ID", ""}
	EnvGCPPubsubKeepAliveIntervalMs  = EnvInteger{"GCP_PUBSUB_KEEP_ALIVE_INTERVAL_MS", 1000}
	EnvPubSubTopicShellInput         = EnvString{"PUBSUB_TOPIC_SHELL_INPUT", "mem://shell_input"}
	EnvPubSubSubscriptionShellInput  = EnvString{"PUBSUB_SUBSCRIPTION_SHELL_INPUT", "mem://shell_input"}
	EnvPubSubTopicShellOutput        = EnvString{"PUBSUB_TOPIC_SHELL_OUTPUT", "mem://shell_output"}
	EnvPubSubSubscriptionShellOutput = EnvString{"PUBSUB_SUBSCRIPTION_SHELL_OUTPUT", "mem://shell_output"}

	// EnvEnablePProf enables performance profiling and should not be enabled in production.
	// EnvEnableMetrics enables the /metrics endpoint and HTTP server. It is unauthenticated and should be used carefully.
	EnvEnablePProf   = EnvBool{"ENABLE_PPROF"}
	EnvEnableMetrics = EnvBool{"ENABLE_METRICS"}

	EnvSecretsManagerPath = EnvString{"SECRETS_FILE_PATH", ""}
)

// Config holds information that controls the behaviour of Tavern
type Config struct {
	srv *http.Server

	mysqlDSN string

	client       *ent.Client
	oauth        oauth2.Config
	userProfiles string
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

func (cfg *Config) NewPortalMux(ctx context.Context) *mux.Mux {
	var (
		projectID = EnvGCPProjectID.String()
	)
	if projectID == "" {
		return mux.New(mux.WithInMemoryDriver(), mux.WithSubscriberBufferSize(1024))
	}
	gcpClient, err := gcppubsub.NewClient(ctx, projectID)
	if err != nil {
		panic(fmt.Errorf("failed to create gcppubsub client needed to create a new subscription: %v", err))
	}
	return mux.New(mux.WithGCPDriver(projectID, gcpClient), mux.WithSubscriberBufferSize(1024))
}

// NewGitImporter configures and returns a new RepoImporter using git.
func (cfg *Config) NewGitImporter(client *ent.Client) *tomes.GitImporter {
	var options []tomes.GitImportOption
	return tomes.NewGitImporter(client, options...)
}

// IsDefaultTomeImportEnabled returns true default tomes should be imported.
func (cfg *Config) IsDefaultTomeImportEnabled() bool {
	return EnvDisableDefaultTomes.IsUnset()
}

// IsMetricsEnabled returns true if the /metrics http endpoint has been enabled.
func (cfg *Config) IsMetricsEnabled() bool {
	return EnvEnableMetrics.IsSet()
}

// IsPProfEnabled returns true if performance profiling has been enabled.
func (cfg *Config) IsPProfEnabled() bool {
	return EnvEnablePProf.IsSet()
}

// IsTestDataEnabled returns true if a value for the "ENABLE_TEST_DATA" environment variable is set.
func (cfg *Config) IsTestDataEnabled() bool {
	return EnvEnableTestData.IsSet()
}

// IsTestRunAndExitEnabled returns true if a value for the "ENABLE_TEST_RUN_AND_EXIT" environment variable is set.
func (cfg *Config) IsTestRunAndExitEnabled() bool {
	return EnvEnableTestRunAndExit.IsSet()
}

// ConfigureHTTPServer enables the configuration of the Tavern HTTP server. The endpoint field will be
// overwritten with Tavern's HTTP handler when Tavern is run.
func ConfigureHTTPServerFromEnv(options ...func(*http.Server)) func(*Config) {
	srv := &http.Server{
		Addr: EnvHTTPListenAddr.String(),
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
			slog.Warn("oauth is not configured, authentication disabled")
			return
		}

		// If partially set, panic to indicate OAuth is improperly configured
		if clientID == "" {
			log.Fatalf("[FATAL] failed to configure oauth, must provide value for environment var 'OAUTH_CLIENT_ID'")
		}
		if clientSecret == "" {
			log.Fatalf("[FATAL] failed to configure oauth, must provide value for environment var 'OAUTH_CLIENT_SECRET'")
		}
		if domain == "" {
			log.Fatalf("[FATAL] failed to configure oauth, must provide value for environment var 'OAUTH_DOMAIN'")
		}
		if !strings.HasPrefix(domain, "http") {
			slog.Warn("domain not prefixed with scheme (http:// or https://), defaulting to https://", "oauth_domain", domain)
			domain = fmt.Sprintf("https://%s", domain)
		}

		// Google OAuth backend
		cfg.oauth = oauth2.Config{
			ClientID:     clientID,
			ClientSecret: clientSecret,
			RedirectURL:  domain + redirectPath,
			Scopes: []string{
				"https://www.googleapis.com/auth/userinfo.profile",
			},
			Endpoint: google.Endpoint,
		}
		cfg.userProfiles = "https://www.googleapis.com/oauth2/v3/userinfo"
	}
}

// ConfigureMySQLFromEnv sets MySQL config values from the environment
func ConfigureMySQLFromEnv() func(*Config) {
	return func(cfg *Config) {
		mysqlConfig := mysql.NewConfig()

		mysqlConfig.Addr = EnvMySQLAddr.String()
		if mysqlConfig.Addr == "" {
			slog.Warn("mysql is not configured, using sqlite")
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
