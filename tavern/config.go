package main

import (
	"context"
	"fmt"
	"log"
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
	"realm.pub/tavern/internal/http/stream"
	"realm.pub/tavern/internal/namegen"
	"realm.pub/tavern/tomes"
)

// GlobalInstanceID uniquely identifies this instance for logging and naming purposes.
var GlobalInstanceID = namegen.NewComplex()

var (
	// EnvEnableTestData if set will populate the database with test data.
	// EnvEnableTestRunAndExit will start the application, but exit immediately after.
	// EnvDisableDefaultTomes will prevent the default tomes from being imported on startup.
	// EnvDebugLogging will emit verbose debug logs to help troubleshoot issues.
	EnvEnableTestData       = EnvString{"ENABLE_TEST_DATA", ""}
	EnvEnableTestRunAndExit = EnvString{"ENABLE_TEST_RUN_AND_EXIT", ""}
	EnvDisableDefaultTomes  = EnvString{"DISABLE_DEFAULT_TOMES", ""}
	EnvDebugLogging         = EnvString{"ENABLE_DEBUG_LOGGING", ""}

	// EnvHTTPListenAddr sets the address (ip:port) for tavern's HTTP server to bind to.
	// EnvHTTPMetricsAddr sets the address (ip:port) for the HTTP metrics server to bind to.
	EnvHTTPListenAddr        = EnvString{"HTTP_LISTEN_ADDR", "0.0.0.0:80"}
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

	// EnvPubSubTopicShellInput defines the topic to publish shell input to.
	// EnvPubSubSubscriptionShellInput defines the subscription to receive shell input from.
	// EnvPubSubTopicShellOutput defines the topic to publish shell output to.
	// EnvPubSubSubscriptionShellOutput defines the subscription to receive shell output from.
	EnvPubSubTopicShellInput         = EnvString{"PUBSUB_TOPIC_SHELL_INPUT", "mem://shell_input"}
	EnvPubSubSubscriptionShellInput  = EnvString{"PUBSUB_SUBSCRIPTION_SHELL_INPUT", "mem://shell_input"}
	EnvPubSubTopicShellOutput        = EnvString{"PUBSUB_TOPIC_SHELL_OUTPUT", "mem://shell_output"}
	EnvPubSubSubscriptionShellOutput = EnvString{"PUBSUB_SUBSCRIPTION_SHELL_OUTPUT", "mem://shell_output"}

	// EnvEnablePProf enables performance profiling and should not be enabled in production.
	// EnvEnableMetrics enables the /metrics endpoint and HTTP server. It is unauthenticated and should be used carefully.
	EnvEnablePProf   = EnvString{"ENABLE_PPROF", ""}
	EnvEnableMetrics = EnvString{"ENABLE_METRICS", ""}
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

// NewShellMuxes configures two stream.Mux instances for shell i/o.
// The wsMux will be used by websockets to subscribe to shell output and publish new input.
// The grpcMux will be used by gRPC to subscribe to shell input and publish new output.
func (cfg *Config) NewShellMuxes(ctx context.Context) (wsMux *stream.Mux, grpcMux *stream.Mux) {
	var (
		projectID        string
		topicShellInput  = EnvPubSubTopicShellInput.String()
		topicShellOutput = EnvPubSubTopicShellOutput.String()
		subShellInput    = EnvPubSubSubscriptionShellInput.String()
		subShellOutput   = EnvPubSubSubscriptionShellOutput.String()
	)

	pubOutput, err := pubsub.OpenTopic(ctx, topicShellOutput)
	if err != nil {
		log.Fatalf("[FATAL] Failed to connect to pubsub topic (%q): %v", topicShellOutput, err)
	}

	// For GCP, messages for a "Subscription" are load-balanced across all of the "Subscribers" to that same "Subscription"
	// This means we must make a new "Subscription" in GCP for each instance of tavern to ensure they all receive the
	// appropriate input/output from shells. For more information, see the information here:
	// https://cloud.google.com/pubsub/docs/pubsub-basics#choose_a_publish_and_subscribe_pattern
	if strings.HasPrefix(subShellInput, "gcppubsub://") && strings.HasPrefix(subShellOutput, "gcppubsub://") {
		client, err := gcppubsub.NewClient(ctx, projectID)
		if err != nil {
			panic(fmt.Errorf("failed to create gcppubsub client needed to create a new subscription"))
		}
		defer client.Close()

		subShellOutput = fmt.Sprintf("%s--%s", strings.TrimPrefix(subShellOutput, "gcppubsub://"), GlobalInstanceID)

		createGCPSubscription := func(ctx context.Context, subName EnvString) string {
			name := fmt.Sprintf("%s--%s", strings.TrimPrefix(subName.String(), "gcppubsub://"), GlobalInstanceID)
			sub, err := client.CreateSubscription(ctx, name, gcppubsub.SubscriptionConfig{
				AckDeadline:      10 * time.Second,
				ExpirationPolicy: 24 * time.Hour, // Automatically delete unused subscriptions after 1 day
			})
			if err != nil {
				panic(fmt.Errorf("failed to create gcppubsub subscription, to disable creation do not use the 'gcppubsub://' prefix for the environment variable %q", EnvPubSubSubscriptionShellInput.Key))
			}
			exists, err := sub.Exists(ctx)
			if err != nil {
				panic(fmt.Errorf("failed to check if gcppubsub subscription was succesfully created: %w", err))
			}
			if !exists {
				panic(fmt.Errorf("failed to create gcppubsub subscription, it does not exist! name=%q", name))
			}
			return name
		}
		subShellInput = fmt.Sprintf("gcpubsub://", createGCPSubscription(ctx, EnvPubSubSubscriptionShellInput))
		subShellOutput = fmt.Sprintf("gcpubsub://", createGCPSubscription(ctx, EnvPubSubSubscriptionShellOutput))
	}

	subOutput, err := pubsub.OpenSubscription(ctx, subShellOutput)
	if err != nil {
		log.Fatalf("[FATAL] Failed to connect to pubsub subscription (%q): %v", subShellOutput, err)
	}

	pubInput, err := pubsub.OpenTopic(ctx, topicShellInput)
	if err != nil {
		log.Fatalf("[FATAL] Failed to connect to pubsub topic (%q): %v", topicShellInput, err)
	}

	subInput, err := pubsub.OpenSubscription(ctx, subShellInput)
	if err != nil {
		log.Fatalf("[FATAL] Failed to connect to pubsub subscription (%q): %v", subShellInput, err)
	}

	wsMux = stream.NewMux(pubInput, subOutput)
	grpcMux = stream.NewMux(pubOutput, subInput)
	return
}

// NewGitImporter configures and returns a new RepoImporter using git.
func (cfg *Config) NewGitImporter(client *ent.Client) *tomes.GitImporter {
	var options []tomes.GitImportOption
	return tomes.NewGitImporter(client, options...)
}

// IsDefaultTomeImportEnabled returns true default tomes should be imported.
func (cfg *Config) IsDefaultTomeImportEnabled() bool {
	return EnvDisableDefaultTomes.String() == ""
}

// IsMetricsEnabled returns true if the /metrics http endpoint has been enabled.
func (cfg *Config) IsMetricsEnabled() bool {
	return EnvEnableMetrics.String() != ""
}

// IsPProfEnabled returns true if performance profiling has been enabled.
func (cfg *Config) IsPProfEnabled() bool {
	return EnvEnablePProf.String() != ""
}

// IsTestDataEnabled returns true if a value for the "ENABLE_TEST_DATA" environment variable is set.
func (cfg *Config) IsTestDataEnabled() bool {
	return EnvEnableTestData.String() != ""
}

// IsTestRunAndExitEnabled returns true if a value for the "ENABLE_TEST_RUN_AND_EXIT" environment variable is set.
func (cfg *Config) IsTestRunAndExitEnabled() bool {
	return EnvEnableTestRunAndExit.String() != ""
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
