package main

import (
	"context"
	"fmt"
	"log"
	"log/slog"
	"net/http"
	"strings"
	"time"

	gcppubsub "cloud.google.com/go/pubsub/v2"
	"entgo.io/ent/dialect/sql"
	"github.com/go-sql-driver/mysql"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/namegen"
	"realm.pub/tavern/internal/portals/mux"
	xpubsub "realm.pub/tavern/internal/portals/pubsub"
	"realm.pub/tavern/tomes"
)

// GlobalInstanceID uniquely identifies this instance for logging and naming purposes.
var GlobalInstanceID = fmt.Sprintf("tavern-%s", namegen.New())

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

	// EnvSchedulerURI selects the scheduler backend via a URI scheme.
	// Examples:
	//   mem://                                                (in-memory, default)
	//   gcp://projects/{project}/locations/{location}         (GCP Cloud Scheduler)
	EnvSchedulerURI = EnvString{"SCHEDULER_URI", "mem://"}

	// EnvGCPProjectID represents the project id tavern is deployed in for Google Cloud Platform deployments (leave empty otherwise).
	EnvGCPProjectID = EnvString{"GCP_PROJECT_ID", ""}

	EnvPubSubSubscriberMaxMessagesBuffered = EnvInteger{"PUBSUB_SUBSCRIBER_MAX_MESSAGES_BUFFERED", 15625}

	// PubSub Configuration Variables
	EnvGCPPublishDelayThresholdMs                  = EnvInteger{"PUBSUB_GCP_PUBLISH_DELAY_THRESHOLD_MS", 10}
	EnvGCPPublishCountThreshold                    = EnvInteger{"PUBSUB_GCP_PUBLISH_COUNT_THRESHOLD", 100}
	EnvGCPPublishByteThreshold                     = EnvInteger{"PUBSUB_GCP_PUBLISH_BYTE_THRESHOLD", 1e6}
	EnvGCPPublishNumGoroutines                     = EnvInteger{"PUBSUB_GCP_PUBLISH_NUM_GOROUTINES", 25}
	EnvGCPPublishTimeoutMs                         = EnvInteger{"PUBSUB_GCP_PUBLISH_TIMEOUT_MS", 1000}
	EnvGCPPublishFlowControlMaxOutstandingMessages = EnvInteger{"PUBSUB_GCP_PUBLISH_FLOW_CONTROL_MAX_OUTSTANDING_MESSAGES", 1000}
	EnvGCPPublishFlowControlMaxOutstandingBytes    = EnvInteger{"PUBSUB_GCP_PUBLISH_FLOW_CONTROL_MAX_OUTSTANDING_BYTES", -1}
	EnvGCPPublishFlowControlLimitExceededBehavior  = EnvString{"PUBSUB_GCP_PUBLISH_FLOW_CONTROL_LIMIT_EXCEEDED_BEHAVIOR", "ignore"}

	EnvGCPReceiveMaxExtensionMs             = EnvInteger{"PUBSUB_GCP_RECEIVE_MAX_EXTENSION_MS", 0}
	EnvGCPReceiveMaxDurationPerAckExtension = EnvInteger{"PUBSUB_GCP_RECEIVE_MAX_DURATION_PER_ACK_EXTENSION_MS", 0}
	EnvGCPReceiveMinDurationPerAckExtension = EnvInteger{"PUBSUB_GCP_RECEIVE_MIN_DURATION_PER_ACK_EXTENSION_MS", 0}
	EnvGCPReceiveMaxOutstandingMessages     = EnvInteger{"PUBSUB_GCP_RECEIVE_MAX_OUTSTANDING_MESSAGES", 1000}
	EnvGCPReceiveMaxOutstandingBytes        = EnvInteger{"PUBSUB_GCP_RECEIVE_MAX_OUTSTANDING_BYTES", -1}
	EnvGCPReceiveNumGoroutines              = EnvInteger{"PUBSUB_GCP_RECEIVE_NUM_GOROUTINES", 10}

	EnvGCPSubscriptionTTLHours  = EnvInteger{"PUBSUB_GCP_SUBSCRIPTION_TTL_HOURS", 24}
	EnvGCPPublishReadyTimeoutMs = EnvInteger{"PUBSUB_GCP_PUBLISH_READY_TIMEOUT_MS", 0}

	// EnvEnablePProf enables performance profiling and should not be enabled in production.
	// EnvEnableMetrics enables the /metrics endpoint and HTTP server. It is unauthenticated and should be used carefully.
	// EnvEnableAIMCP enables the AI MCP (Model Context Protocol) server endpoint.
	EnvEnablePProf   = EnvBool{"ENABLE_PPROF"}
	EnvEnableMetrics = EnvBool{"ENABLE_METRICS"}
	EnvEnableAIMCP   = EnvBool{"ENABLE_AI_MCP"}

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
	client := ent.NewClient(append(options, ent.Driver(drv))...)
	client.Host.Use(ent.HookDeriveHostEvents())
	client.Task.Use(ent.HookDeriveQuestEvents())
	client.Event.Use(ent.HookDeriveNotifications())
	client.User.Use(ent.HookDeriveUserRequestEvents())
	return client, nil
}

func (cfg *Config) NewPortalMux(ctx context.Context) *mux.Mux {
	var (
		projectID     = EnvGCPProjectID.String()
		subBufferSize = EnvPubSubSubscriberMaxMessagesBuffered.Int()
	)

	// Prepare PubSub Settings
	pubSettings := gcppubsub.PublishSettings{
		DelayThreshold: time.Duration(EnvGCPPublishDelayThresholdMs.Int()) * time.Millisecond,
		CountThreshold: EnvGCPPublishCountThreshold.Int(),
		ByteThreshold:  EnvGCPPublishByteThreshold.Int(),
		NumGoroutines:  EnvGCPPublishNumGoroutines.Int(),
		Timeout:        time.Duration(EnvGCPPublishTimeoutMs.Int()) * time.Millisecond,
		FlowControlSettings: gcppubsub.FlowControlSettings{
			MaxOutstandingMessages: EnvGCPPublishFlowControlMaxOutstandingMessages.Int(),
			MaxOutstandingBytes:    EnvGCPPublishFlowControlMaxOutstandingBytes.Int(),
		},
	}
	switch strings.ToLower(EnvGCPPublishFlowControlLimitExceededBehavior.String()) {
	case "block":
		pubSettings.FlowControlSettings.LimitExceededBehavior = gcppubsub.FlowControlBlock
	case "signalerror":
		pubSettings.FlowControlSettings.LimitExceededBehavior = gcppubsub.FlowControlSignalError
	default: // ignore
		pubSettings.FlowControlSettings.LimitExceededBehavior = gcppubsub.FlowControlIgnore
	}

	recvSettings := gcppubsub.ReceiveSettings{
		MaxExtension:               time.Duration(EnvGCPReceiveMaxExtensionMs.Int()) * time.Millisecond,
		MaxDurationPerAckExtension: time.Duration(EnvGCPReceiveMaxDurationPerAckExtension.Int()) * time.Millisecond,
		MinDurationPerAckExtension: time.Duration(EnvGCPReceiveMinDurationPerAckExtension.Int()) * time.Millisecond,
		MaxOutstandingMessages:     EnvGCPReceiveMaxOutstandingMessages.Int(),
		MaxOutstandingBytes:        EnvGCPReceiveMaxOutstandingBytes.Int(),
		NumGoroutines:              EnvGCPReceiveNumGoroutines.Int(),
	}

	ttl := time.Duration(EnvGCPSubscriptionTTLHours.Int()) * time.Hour
	readyTimeout := time.Duration(EnvGCPPublishReadyTimeoutMs.Int()) * time.Millisecond

	var client *xpubsub.Client

	if projectID == "" {
		client = xpubsub.NewClient(
			xpubsub.WithInMemoryDriver(
				xpubsub.WithPublishSettings(pubSettings),
				xpubsub.WithReceiveSettings(recvSettings),
				xpubsub.WithExpirationPolicy(ttl),
				xpubsub.WithPublishReadyTimeout(readyTimeout),
			),
		)
	} else {
		gcpClient, err := gcppubsub.NewClient(ctx, projectID)
		if err != nil {
			panic(fmt.Errorf("failed to create gcppubsub client needed to create a new subscription: %v", err))
		}
		client = xpubsub.NewClient(
			xpubsub.WithGCPDriver(
				GlobalInstanceID,
				gcpClient,
				xpubsub.WithPublishSettings(pubSettings),
				xpubsub.WithReceiveSettings(recvSettings),
				xpubsub.WithExpirationPolicy(ttl),
				xpubsub.WithPublishReadyTimeout(readyTimeout),
			),
		)
	}

	return mux.New(mux.WithPubSubClient(client), mux.WithSubscriberBufferSize(subBufferSize))
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

// IsMCPEnabled returns true if the AI MCP endpoint has been enabled.
func (cfg *Config) IsMCPEnabled() bool {
	return EnvEnableAIMCP.IsSet()
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
